//! Public Markdown entry directory support.
//!
//! This module reads the human-facing Sirno store shape:
//! a flat directory of `*.md` files whose filename stems are entry ids.
//! Path-shaped ids remain outside this first file-surface implementation.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use thiserror::Error;
use tracing::trace;

use crate::check::{CheckMode, CheckReport, CheckSeverity, check_entries};
use crate::entry::{Entry, EntryParseError, EntryRenderError, default_seed_entries};
use crate::id::EntryId;
use crate::links::{
    GeneratedLinkError, GeneratedLinkIndex, GeneratedLinkSettings, apply_generated_links,
    delete_generated_links, generated_links_are_stale, validate_generated_links,
};

/// Check report for a public Markdown entry directory.
#[derive(Debug)]
pub struct EntryDirectoryReport {
    root: PathBuf,
    entries: Vec<Entry>,
    paths_by_id: BTreeMap<EntryId, PathBuf>,
    file_diagnostics: Vec<EntryFileDiagnostic>,
    relation_report: CheckReport,
}

/// Settings for checking a public Markdown entry directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryDirectoryCheckSettings {
    /// Check generated-link footer freshness.
    pub link: bool,
    /// Settings used to render expected generated links.
    pub links: GeneratedLinkSettings,
    /// Store-root-relative paths ignored by Sirno.
    pub ignore: Vec<PathBuf>,
}

impl Default for EntryDirectoryCheckSettings {
    fn default() -> Self {
        Self { link: true, links: GeneratedLinkSettings::default(), ignore: Vec::new() }
    }
}

impl EntryDirectoryCheckSettings {
    /// Return true when a root-relative path is ignored.
    pub fn ignores(&self, relative_path: &Path) -> bool {
        self.ignore.iter().any(|ignored| {
            !ignored.as_os_str().is_empty()
                && (relative_path == ignored || relative_path.starts_with(ignored))
        })
    }
}

impl EntryDirectoryReport {
    /// Directory that was checked.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Parsed entries that were valid enough to participate in relation checks.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// File-level diagnostics from loading the entry directory.
    pub fn file_diagnostics(&self) -> &[EntryFileDiagnostic] {
        &self.file_diagnostics
    }

    /// Relation check report for successfully parsed entries.
    pub fn relation_report(&self) -> &CheckReport {
        &self.relation_report
    }

    /// Return the path associated with a parsed entry id.
    pub fn entry_path(&self, id: &EntryId) -> Option<&Path> {
        self.paths_by_id.get(id).map(PathBuf::as_path)
    }

    /// Returns true when no file or relation diagnostics were produced.
    pub fn is_clean(&self) -> bool {
        self.file_diagnostics.is_empty() && self.relation_report.is_clean()
    }

    /// Returns true when any file or relation diagnostic is an error.
    pub fn has_errors(&self) -> bool {
        self.file_diagnostics.iter().any(|diagnostic| diagnostic.severity == CheckSeverity::Error)
            || self.relation_report.has_errors()
    }
}

/// Diagnostic produced while loading the public Markdown entry surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryFileDiagnostic {
    /// Diagnostic severity.
    pub severity: CheckSeverity,
    /// Path responsible for the diagnostic.
    pub path: PathBuf,
    /// Human-readable diagnostic message.
    pub message: String,
}

/// Result of generating link footers for an entry directory.
#[derive(Debug)]
pub struct GenLinkDirectoryReport {
    root: PathBuf,
    entry_count: usize,
    changed_paths: Vec<PathBuf>,
}

impl GenLinkDirectoryReport {
    /// Directory whose entries were processed.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Number of entries processed.
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// Entry files whose generated-link region changed.
    pub fn changed_paths(&self) -> &[PathBuf] {
        &self.changed_paths
    }
}

impl EntryFileDiagnostic {
    /// Construct a diagnostic for one path.
    pub fn new(
        severity: CheckSeverity, path: impl Into<PathBuf>, message: impl Into<String>,
    ) -> Self {
        Self { severity, path: path.into(), message: message.into() }
    }
}

/// Check one public Markdown entry directory.
pub fn check_entry_directory(
    root: impl Into<PathBuf>, mode: CheckMode,
) -> Result<EntryDirectoryReport, EntryDirectoryError> {
    check_entry_directory_with_settings(root, mode, &EntryDirectoryCheckSettings::default())
}

/// Check one public Markdown entry directory with explicit settings.
pub fn check_entry_directory_with_settings(
    root: impl Into<PathBuf>, mode: CheckMode, settings: &EntryDirectoryCheckSettings,
) -> Result<EntryDirectoryReport, EntryDirectoryError> {
    let root = root.into();
    trace!("check_entry_directory begin: root={}", root.display());
    let loaded = load_entry_directory(&root, mode, settings)?;
    let relation_report = check_entries(&loaded.entries, mode);
    trace!(
        "check_entry_directory end: entries={} file_diagnostics={} relation_diagnostics={}",
        loaded.entries.len(),
        loaded.file_diagnostics.len(),
        relation_report.diagnostics().len()
    );
    Ok(EntryDirectoryReport {
        root,
        entries: loaded.entries,
        paths_by_id: loaded.paths_by_id,
        file_diagnostics: loaded.file_diagnostics,
        relation_report,
    })
}

/// Initialize a public Markdown entry directory with ordinary seed entries.
///
/// Existing entry files are never overwritten.
pub fn init_entry_directory(root: impl Into<PathBuf>) -> Result<Vec<PathBuf>, EntryDirectoryError> {
    let root = root.into();
    trace!("init_entry_directory begin: root={}", root.display());
    fs::create_dir_all(&root)?;
    let mut paths = Vec::new();
    for entry in default_seed_entries()? {
        let path = write_new_entry_file(&root, &entry)?;
        paths.push(path);
    }
    trace!("init_entry_directory end: entries={}", paths.len());
    Ok(paths)
}

/// Create one public Markdown entry file.
///
/// The entry directory is created if needed.
/// Existing entry files are never overwritten.
pub fn create_entry_file(
    root: impl Into<PathBuf>, entry: &Entry,
) -> Result<PathBuf, EntryDirectoryError> {
    let root = root.into();
    trace!("create_entry_file begin: root={} id={}", root.display(), entry.id);
    fs::create_dir_all(&root)?;
    let path = write_new_entry_file(&root, entry)?;
    trace!("create_entry_file end: path={}", path.display());
    Ok(path)
}

/// Generate Markdown link footers for one public entry directory.
///
/// The directory must pass review-mode checks before any file is written.
pub fn gen_link_entry_directory(
    root: impl Into<PathBuf>, settings: &GeneratedLinkSettings,
) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
    gen_link_entry_directory_with_ignored_paths(root, settings, Vec::<PathBuf>::new())
}

/// Generate Markdown link footers for one public entry directory with ignored paths.
///
/// Ignored paths are relative to the entry directory root.
pub fn gen_link_entry_directory_with_ignored_paths(
    root: impl Into<PathBuf>, settings: &GeneratedLinkSettings,
    ignore: impl IntoIterator<Item = PathBuf>,
) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
    let root = root.into();
    trace!("gen_link_entry_directory begin: root={}", root.display());
    let check_settings = EntryDirectoryCheckSettings {
        link: false,
        links: *settings,
        ignore: ignore.into_iter().collect(),
    };
    let checked = check_entry_directory_with_settings(&root, CheckMode::Review, &check_settings)?;
    if checked.has_errors() {
        return Err(EntryDirectoryError::InvalidEntryDirectory(root));
    }

    let mut changed_paths = Vec::new();
    let index = GeneratedLinkIndex::from_entries(checked.entries());
    for entry in checked.entries() {
        let path = checked
            .entry_path(&entry.id)
            .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
        let source = fs::read_to_string(path)?;
        let footer = index.render_entry(entry, settings);
        let body = apply_generated_links(&entry.body, &footer)?;
        let rendered = Entry::replace_markdown_body(&source, &body)?;
        if rendered != source {
            fs::write(path, rendered).map_err(|source| EntryDirectoryError::WriteFile {
                path: path.to_path_buf(),
                source,
            })?;
            changed_paths.push(path.to_path_buf());
        }
    }

    trace!(
        "gen_link_entry_directory end: entries={} changed={}",
        checked.entries().len(),
        changed_paths.len()
    );
    Ok(GenLinkDirectoryReport { root, entry_count: checked.entries().len(), changed_paths })
}

/// Delete generated Markdown link footers from one public entry directory.
///
/// The directory must parse cleanly before any file is written.
pub fn delete_gen_link_entry_directory(
    root: impl Into<PathBuf>,
) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
    delete_gen_link_entry_directory_with_ignored_paths(root, Vec::<PathBuf>::new())
}

/// Delete generated Markdown link footers from one public entry directory with ignored paths.
///
/// Ignored paths are relative to the entry directory root.
pub fn delete_gen_link_entry_directory_with_ignored_paths(
    root: impl Into<PathBuf>, ignore: impl IntoIterator<Item = PathBuf>,
) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
    let root = root.into();
    trace!("delete_gen_link_entry_directory begin: root={}", root.display());
    let check_settings = EntryDirectoryCheckSettings {
        link: false,
        links: GeneratedLinkSettings::default(),
        ignore: ignore.into_iter().collect(),
    };
    let checked = check_entry_directory_with_settings(&root, CheckMode::Edit, &check_settings)?;
    if checked.has_errors() {
        return Err(EntryDirectoryError::InvalidEntryDirectory(root));
    }

    let mut changed_paths = Vec::new();
    for entry in checked.entries() {
        let path = checked
            .entry_path(&entry.id)
            .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
        let source = fs::read_to_string(path)?;
        let body = delete_generated_links(&entry.body)?;
        let rendered = Entry::replace_markdown_body(&source, &body)?;
        if rendered != source {
            fs::write(path, rendered).map_err(|source| EntryDirectoryError::WriteFile {
                path: path.to_path_buf(),
                source,
            })?;
            changed_paths.push(path.to_path_buf());
        }
    }

    trace!(
        "delete_gen_link_entry_directory end: entries={} changed={}",
        checked.entries().len(),
        changed_paths.len()
    );
    Ok(GenLinkDirectoryReport { root, entry_count: checked.entries().len(), changed_paths })
}

#[derive(Debug)]
struct LoadedEntryDirectory {
    entries: Vec<Entry>,
    paths_by_id: BTreeMap<EntryId, PathBuf>,
    file_diagnostics: Vec<EntryFileDiagnostic>,
}

fn load_entry_directory(
    root: &Path, mode: CheckMode, settings: &EntryDirectoryCheckSettings,
) -> Result<LoadedEntryDirectory, EntryDirectoryError> {
    if !root.exists() {
        return Err(EntryDirectoryError::MissingDirectory(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(EntryDirectoryError::NotDirectory(root.to_path_buf()));
    }

    let non_entry_severity = match mode {
        | CheckMode::Edit => CheckSeverity::Warning,
        | CheckMode::Review => CheckSeverity::Error,
    };
    let mut entries = Vec::new();
    let mut paths_by_id = BTreeMap::<EntryId, PathBuf>::new();
    let mut seen_ids = BTreeSet::<EntryId>::new();
    let mut file_diagnostics = Vec::new();

    for path in sorted_directory_paths(root)? {
        let relative_path = path.strip_prefix(root).map_err(|source| {
            EntryDirectoryError::StripRoot { path: path.clone(), root: root.to_path_buf(), source }
        })?;
        if settings.ignores(relative_path) {
            continue;
        }

        let file_type = fs::symlink_metadata(&path)?.file_type();
        if file_type.is_dir() {
            file_diagnostics.push(EntryFileDiagnostic::new(
                non_entry_severity,
                &path,
                "entry directory contains unsupported subdirectory",
            ));
            continue;
        }
        if !file_type.is_file() {
            file_diagnostics.push(EntryFileDiagnostic::new(
                non_entry_severity,
                &path,
                "entry directory contains unsupported filesystem item",
            ));
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            file_diagnostics.push(EntryFileDiagnostic::new(
                non_entry_severity,
                &path,
                "entry directory contains non-Markdown file",
            ));
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            file_diagnostics.push(EntryFileDiagnostic::new(
                CheckSeverity::Error,
                &path,
                "entry file stem must be valid UTF-8",
            ));
            continue;
        };

        let id = match EntryId::new(stem) {
            | Ok(id) => id,
            | Err(source) => {
                file_diagnostics.push(EntryFileDiagnostic::new(
                    CheckSeverity::Error,
                    &path,
                    format!("entry file stem is not a valid id: {source}"),
                ));
                continue;
            }
        };

        if seen_ids.contains(&id) {
            let first_path = paths_by_id
                .get(&id)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_owned());
            file_diagnostics.push(EntryFileDiagnostic::new(
                CheckSeverity::Error,
                &path,
                format!("entry id `{id}` also appears at {first_path}"),
            ));
            continue;
        }

        let source = fs::read_to_string(&path)?;
        let entry = match Entry::from_markdown(id.clone(), &source) {
            | Ok(entry) => entry,
            | Err(source) => {
                file_diagnostics.push(EntryFileDiagnostic::new(
                    CheckSeverity::Error,
                    &path,
                    format!("failed to parse entry: {source}"),
                ));
                continue;
            }
        };
        seen_ids.insert(id.clone());
        paths_by_id.insert(id, path);
        entries.push(entry);
    }

    entries.sort_by(|left, right| left.id.cmp(&right.id));
    add_generated_link_diagnostics(&entries, &paths_by_id, mode, settings, &mut file_diagnostics)?;
    Ok(LoadedEntryDirectory { entries, paths_by_id, file_diagnostics })
}

fn add_generated_link_diagnostics(
    entries: &[Entry], paths_by_id: &BTreeMap<EntryId, PathBuf>, mode: CheckMode,
    settings: &EntryDirectoryCheckSettings, file_diagnostics: &mut Vec<EntryFileDiagnostic>,
) -> Result<(), EntryDirectoryError> {
    let index = GeneratedLinkIndex::from_entries(entries);
    for entry in entries {
        let path = paths_by_id
            .get(&entry.id)
            .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
        match validate_generated_links(&entry.body) {
            | Ok(()) if settings.link => {
                let expected = index.render_entry(entry, &settings.links);
                if generated_links_are_stale(&entry.body, &expected)? {
                    file_diagnostics.push(EntryFileDiagnostic::new(
                        mode_severity(mode),
                        path,
                        "generated links are stale; run `sirno gen-link`",
                    ));
                }
            }
            | Ok(()) => {}
            | Err(source) => {
                file_diagnostics.push(EntryFileDiagnostic::new(
                    CheckSeverity::Error,
                    path,
                    format!("malformed generated links: {source}"),
                ));
            }
        }
    }
    Ok(())
}

fn mode_severity(mode: CheckMode) -> CheckSeverity {
    match mode {
        | CheckMode::Edit => CheckSeverity::Warning,
        | CheckMode::Review => CheckSeverity::Error,
    }
}

fn sorted_directory_paths(root: &Path) -> Result<Vec<PathBuf>, EntryDirectoryError> {
    let mut paths = fs::read_dir(root)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    Ok(paths)
}

fn write_new_entry_file(root: &Path, entry: &Entry) -> Result<PathBuf, EntryDirectoryError> {
    let path = root.join(format!("{}.md", entry.id.as_str()));
    let source = entry.to_markdown()?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|source| EntryDirectoryError::CreateFile { path: path.clone(), source })?;
    file.write_all(source.as_bytes())
        .map_err(|source| EntryDirectoryError::WriteFile { path: path.clone(), source })?;
    Ok(path)
}

/// Error raised before an entry directory can be checked.
#[derive(Debug, Error)]
pub enum EntryDirectoryError {
    /// The requested entry directory does not exist.
    #[error("entry directory does not exist: {0}")]
    MissingDirectory(PathBuf),
    /// The requested entry directory path is not a directory.
    #[error("entry path is not a directory: {0}")]
    NotDirectory(PathBuf),
    /// Reading the directory or one of its files failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A discovered path could not be made relative to the entry directory.
    #[error("entry path {path} is not inside entry directory {root}")]
    StripRoot {
        /// Path discovered while loading the entry directory.
        path: PathBuf,
        /// Entry directory root.
        root: PathBuf,
        /// Underlying path-prefix error.
        #[source]
        source: std::path::StripPrefixError,
    },
    /// An entry could not be parsed or constructed.
    #[error(transparent)]
    EntryParse(#[from] EntryParseError),
    /// A seed entry could not be rendered.
    #[error(transparent)]
    Render(#[from] EntryRenderError),
    /// Generated-link footer boundaries were malformed while writing.
    #[error(transparent)]
    GeneratedLink(#[from] GeneratedLinkError),
    /// Generated-link operations require a clean enough entry directory.
    #[error("entry directory must pass checks before changing generated links: {0}")]
    InvalidEntryDirectory(PathBuf),
    /// A parsed entry had no file path in the directory report.
    #[error("entry `{0}` has no source file path")]
    MissingEntryPath(EntryId),
    /// An entry file could not be created.
    #[error("failed to create entry file {path}")]
    CreateFile {
        /// Path that could not be created.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// An entry file could not be written.
    #[error("failed to write entry file {path}")]
    WriteFile {
        /// Path that could not be written.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntryMetadata;

    fn write_entry(root: &Path, name: &str, body: &str) {
        fs::write(root.join(name), body).unwrap();
    }

    #[test]
    fn checks_clean_markdown_entry_directory() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "meta.md",
            "\
---
name: Meta
description: A category for categories.
---

Body.
",
        );
        write_entry(
            temp.path(),
            "concept.md",
            "\
---
name: Concept
description: A named idea.
category:
  - meta
---

Body.
",
        );

        let report = check_entry_directory(temp.path(), CheckMode::Review).unwrap();

        assert!(report.is_clean());
        assert_eq!(report.entries().len(), 2);
        assert!(report.entry_path(&EntryId::new("concept").unwrap()).is_some());
    }

    #[test]
    fn reports_parse_error_with_file_path() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(temp.path(), "bad.md", "no frontmatter\n");

        let report = check_entry_directory(temp.path(), CheckMode::Review).unwrap();

        assert!(report.has_errors());
        assert_eq!(report.file_diagnostics().len(), 1);
        assert_eq!(report.file_diagnostics()[0].path, temp.path().join("bad.md"));
        assert!(report.file_diagnostics()[0].message.contains("failed to parse entry"));
    }

    #[test]
    fn reports_non_markdown_file_as_review_error() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("note.txt"), "text").unwrap();

        let report = check_entry_directory(temp.path(), CheckMode::Review).unwrap();

        assert_eq!(report.file_diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
    }

    #[test]
    fn reports_non_markdown_file_as_edit_warning() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("note.txt"), "text").unwrap();

        let report = check_entry_directory(temp.path(), CheckMode::Edit).unwrap();

        assert_eq!(report.file_diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }

    #[test]
    fn ignores_configured_store_paths() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join(".obsidian")).unwrap();
        fs::write(temp.path().join("note.txt"), "text").unwrap();
        write_entry(
            temp.path(),
            "meta.md",
            "\
---
name: Meta
description: A category for categories.
---

Body.
",
        );

        let report = check_entry_directory_with_settings(
            temp.path(),
            CheckMode::Review,
            &EntryDirectoryCheckSettings {
                ignore: vec![PathBuf::from(".obsidian"), PathBuf::from("note.txt")],
                ..EntryDirectoryCheckSettings::default()
            },
        )
        .unwrap();

        assert!(report.is_clean());
        assert_eq!(report.entries().len(), 1);
    }

    #[test]
    fn reports_relation_diagnostics_from_loaded_entries() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "concept.md",
            "\
---
name: Concept
description: A named idea.
category:
  - meta
---
",
        );

        let report = check_entry_directory(temp.path(), CheckMode::Review).unwrap();

        assert!(report.has_errors());
        assert_eq!(report.relation_report().diagnostics().len(), 1);
    }

    #[test]
    fn missing_directory_is_a_load_error() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing");

        let error = check_entry_directory(&missing, CheckMode::Review).unwrap_err();

        assert!(matches!(error, EntryDirectoryError::MissingDirectory(_)));
    }

    #[test]
    fn initializes_seed_entry_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");

        let paths = init_entry_directory(&root).unwrap();
        let report = check_entry_directory(&root, CheckMode::Review).unwrap();

        assert_eq!(paths.len(), 3);
        assert!(root.join("concept.md").exists());
        assert!(report.is_clean());
    }

    #[test]
    fn init_refuses_to_overwrite_entry_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");

        init_entry_directory(&root).unwrap();
        let error = init_entry_directory(&root).unwrap_err();

        assert!(matches!(error, EntryDirectoryError::CreateFile { .. }));
    }

    #[test]
    fn create_entry_file_writes_one_entry() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        let mut metadata = EntryMetadata::new("Local Idea", "A local design idea.").unwrap();
        metadata.category.push(EntryId::new("meta").unwrap());
        let entry = Entry::new(EntryId::new("local-idea").unwrap(), metadata, "");

        let path = create_entry_file(&root, &entry).unwrap();
        let source = fs::read_to_string(&path).unwrap();

        assert_eq!(path, root.join("local-idea.md"));
        assert!(source.contains("name: Local Idea\n"));
        assert!(source.contains("category:\n  - meta\n"));
    }

    #[test]
    fn create_entry_file_refuses_to_overwrite() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        let metadata = EntryMetadata::new("Local Idea", "A local design idea.").unwrap();
        let entry = Entry::new(EntryId::new("local-idea").unwrap(), metadata, "");

        create_entry_file(&root, &entry).unwrap();
        let error = create_entry_file(&root, &entry).unwrap_err();

        assert!(matches!(error, EntryDirectoryError::CreateFile { .. }));
    }

    #[test]
    fn gen_link_writes_generated_footers() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();
        let settings =
            GeneratedLinkSettings { category: true, clustee: true, clique: false, refiner: true };

        let report = gen_link_entry_directory(&root, &settings).unwrap();
        let concept = fs::read_to_string(root.join("concept.md")).unwrap();

        assert_eq!(report.entry_count(), 3);
        assert_eq!(report.changed_paths().len(), 3);
        assert!(concept.contains(crate::links::BEGIN_LINKS_GUARD));
        assert!(concept.contains("\n---\n\n> **Sirno generated links begin."));
        assert!(concept.contains("- [meta](meta.md)"));
        assert!(!concept.contains("## Sirno Links"));
        assert!(!concept.contains("category: [meta](meta.md)"));
    }

    #[test]
    fn gen_link_expands_cliques_with_store_context() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "core.md",
            "\
---
name: Core
description: A clique closure.
---

Body.
",
        );
        write_entry(
            temp.path(),
            "left.md",
            "\
---
name: Left
description: A clique member.
clustee:
  - core
---

Body.
",
        );
        write_entry(
            temp.path(),
            "right.md",
            "\
---
name: Right
description: A clique member.
clustee:
  - core
---

Body.
",
        );
        let settings =
            GeneratedLinkSettings { category: false, clustee: true, clique: true, refiner: false };

        gen_link_entry_directory(temp.path(), &settings).unwrap();
        let core = fs::read_to_string(temp.path().join("core.md")).unwrap();
        let left = fs::read_to_string(temp.path().join("left.md")).unwrap();

        assert!(core.contains("- [left](left.md)"));
        assert!(core.contains("- [right](right.md)"));
        assert!(!core.contains("[core](core.md)"));
        assert!(left.contains("- [core](core.md)"));
        assert!(left.contains("- [right](right.md)"));
        assert!(!left.contains("[left](left.md)"));
    }

    #[test]
    fn gen_link_is_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();
        let settings = GeneratedLinkSettings::default();

        gen_link_entry_directory(&root, &settings).unwrap();
        let report = gen_link_entry_directory(&root, &settings).unwrap();

        assert!(report.changed_paths().is_empty());
    }

    #[test]
    fn delete_gen_link_removes_generated_footers() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();
        gen_link_entry_directory(&root, &GeneratedLinkSettings::default()).unwrap();

        let report = delete_gen_link_entry_directory(&root).unwrap();
        let concept = fs::read_to_string(root.join("concept.md")).unwrap();

        assert_eq!(report.entry_count(), 3);
        assert_eq!(report.changed_paths().len(), 3);
        assert!(!concept.contains(crate::links::BEGIN_LINKS_GUARD));
    }

    #[test]
    fn delete_gen_link_is_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();

        let report = delete_gen_link_entry_directory(&root).unwrap();

        assert_eq!(report.entry_count(), 3);
        assert!(report.changed_paths().is_empty());
    }

    #[test]
    fn check_reports_stale_generated_links_as_review_error() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();
        let old_settings =
            GeneratedLinkSettings { category: true, clustee: true, clique: false, refiner: true };
        gen_link_entry_directory(&root, &old_settings).unwrap();

        let report = check_entry_directory_with_settings(
            &root,
            CheckMode::Review,
            &EntryDirectoryCheckSettings {
                link: true,
                links: GeneratedLinkSettings::default(),
                ..EntryDirectoryCheckSettings::default()
            },
        )
        .unwrap();

        assert!(report.has_errors());
        assert_eq!(report.file_diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.file_diagnostics()[0].message.contains("generated links are stale"));
    }

    #[test]
    fn check_reports_stale_generated_links_as_edit_warning() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();
        let old_settings =
            GeneratedLinkSettings { category: true, clustee: true, clique: false, refiner: true };
        gen_link_entry_directory(&root, &old_settings).unwrap();

        let report = check_entry_directory_with_settings(
            &root,
            CheckMode::Edit,
            &EntryDirectoryCheckSettings {
                link: true,
                links: GeneratedLinkSettings::default(),
                ..EntryDirectoryCheckSettings::default()
            },
        )
        .unwrap();

        assert!(!report.has_errors());
        assert_eq!(report.file_diagnostics()[0].severity, CheckSeverity::Warning);
    }

    #[test]
    fn check_can_skip_stale_generated_links() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();
        let old_settings =
            GeneratedLinkSettings { category: true, clustee: true, clique: false, refiner: true };
        gen_link_entry_directory(&root, &old_settings).unwrap();

        let report = check_entry_directory_with_settings(
            &root,
            CheckMode::Review,
            &EntryDirectoryCheckSettings {
                link: false,
                links: GeneratedLinkSettings::default(),
                ..EntryDirectoryCheckSettings::default()
            },
        )
        .unwrap();

        assert!(report.is_clean());
    }

    #[test]
    fn check_reports_malformed_generated_link_boundaries() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "concept.md",
            "\
---
name: Concept
description: A named idea.
---

Body.
> **Sirno generated links begin. Do not edit this section.**
",
        );

        let report = check_entry_directory(temp.path(), CheckMode::Review).unwrap();

        assert!(report.has_errors());
        assert!(report.file_diagnostics()[0].message.contains("malformed generated links"));
    }

    #[test]
    fn gen_link_refuses_dirty_entry_directory() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(temp.path(), "bad.md", "no frontmatter\n");

        let error =
            gen_link_entry_directory(temp.path(), &GeneratedLinkSettings::default()).unwrap_err();

        assert!(matches!(error, EntryDirectoryError::InvalidEntryDirectory(_)));
    }
}
