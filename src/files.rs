//! Public Markdown entry directory support.
//!
//! This module reads the human-facing Sirno store shape:
//! a flat directory of `*.md` files whose filename stems are entry ids.
//! Path-shaped ids remain outside this first flat-file implementation.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use thiserror::Error;
use tracing::trace;

use crate::check::{CheckMode, CheckReport, CheckSeverity, check_entries};
use crate::entry::{Entry, EntryParseError, EntryRenderError, default_seed_entries};
use crate::id::EntryId;
use crate::links::{
    GeneratedLinkError, GeneratedLinkIndex, GeneratedLinkSettings, apply_generated_links,
    delete_generated_links, generated_links_are_stale, validate_generated_links,
};
use crate::witness::{WitnessCheckSettings, WitnessError, scan_witnesses};

const READONLY_CHECKOUT_WARNING: &str = "\
> This file is a read-only Sirno history checkout.
> Do not edit it by hand.

";

/// Check report for a public Markdown entry directory.
#[derive(Debug)]
// sirno:witness:start sirno-store
pub struct EntryDirectoryReport {
    root: PathBuf,
    entries: Vec<Entry>,
    paths_by_id: BTreeMap<EntryId, PathBuf>,
    file_diagnostics: Vec<EntryFileDiagnostic>,
    structural_report: CheckReport,
}
// sirno:witness:end

/// Settings for checking a public Markdown entry directory.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:start sirno-store
pub struct EntryDirectoryCheckSettings {
    /// Check generated-link footer freshness.
    pub link: bool,
    /// Settings used to render expected generated links.
    pub links: GeneratedLinkSettings,
    /// Store-root-relative paths ignored by Sirno.
    pub ignore: Vec<PathBuf>,
    /// Repository witness scan settings.
    pub witness: Option<WitnessCheckSettings>,
}
// sirno:witness:end

impl Default for EntryDirectoryCheckSettings {
    fn default() -> Self {
        Self {
            link: true,
            links: GeneratedLinkSettings::default(),
            ignore: Vec::new(),
            witness: None,
        }
    }
}

impl EntryDirectoryCheckSettings {
    /// Return true when a root-relative path is ignored.
    // sirno:witness:start sirno-store
    pub fn ignores(&self, relative_path: &Path) -> bool {
        self.ignore.iter().any(|ignored| {
            !ignored.as_os_str().is_empty()
                && (relative_path == ignored || relative_path.starts_with(ignored))
        })
    }
    // sirno:witness:end
}

impl EntryDirectoryReport {
    /// Directory that was checked.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Parsed entries that were valid enough to participate in structural checks.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// File-level diagnostics from loading the entry directory.
    pub fn file_diagnostics(&self) -> &[EntryFileDiagnostic] {
        &self.file_diagnostics
    }

    /// Structural check report for successfully parsed entries.
    pub fn structural_report(&self) -> &CheckReport {
        &self.structural_report
    }

    /// Return the path associated with a parsed entry id.
    pub fn entry_path(&self, id: &EntryId) -> Option<&Path> {
        self.paths_by_id.get(id).map(PathBuf::as_path)
    }

    /// Returns true when no file or structural diagnostics were produced.
    pub fn is_clean(&self) -> bool {
        self.file_diagnostics.is_empty() && self.structural_report.is_clean()
    }

    /// Returns true when any file or structural diagnostic is an error.
    pub fn has_errors(&self) -> bool {
        self.file_diagnostics.iter().any(|diagnostic| diagnostic.severity == CheckSeverity::Error)
            || self.structural_report.has_errors()
    }
}

/// Diagnostic produced while loading the public Markdown entry store.
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

/// Conflict policy for writing a complete public entry directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryDirectoryWritePolicy {
    /// Require the target directory to be absent or empty before writing entries.
    EmptyDirectory,
    /// Replace managed Markdown entries while preserving ignored paths.
    ReplaceDirectory {
        /// Store-root-relative paths preserved by checkout.
        ignore: Vec<PathBuf>,
    },
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
// sirno:witness:start sirno-store
pub fn check_entry_directory_with_settings(
    root: impl Into<PathBuf>, mode: CheckMode, settings: &EntryDirectoryCheckSettings,
) -> Result<EntryDirectoryReport, EntryDirectoryError> {
    let root = root.into();
    trace!("check_entry_directory begin: root={}", root.display());
    let loaded = load_entry_directory(&root, mode, settings)?;
    let structural_report = check_entries(&loaded.entries, mode);
    trace!(
        "check_entry_directory end: entries={} file_diagnostics={} structural_diagnostics={}",
        loaded.entries.len(),
        loaded.file_diagnostics.len(),
        structural_report.diagnostics().len()
    );
    Ok(EntryDirectoryReport {
        root,
        entries: loaded.entries,
        paths_by_id: loaded.paths_by_id,
        file_diagnostics: loaded.file_diagnostics,
        structural_report,
    })
}
// sirno:witness:end

/// Initialize a public Markdown entry directory with ordinary seed entries.
///
/// Existing entry files are never overwritten.
// sirno:witness:start sirno-store
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
// sirno:witness:end

/// Create one public Markdown entry file.
///
/// The entry directory is created if needed.
/// Existing entry files are never overwritten.
// sirno:witness:start sirno-store
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
// sirno:witness:end

/// Write a complete public Markdown entry directory.
///
/// The write policy controls how existing target contents are handled.
// sirno:witness:start sirno-store
pub fn write_entry_directory(
    root: impl Into<PathBuf>, entries: &[Entry], policy: EntryDirectoryWritePolicy,
) -> Result<Vec<PathBuf>, EntryDirectoryError> {
    let root = root.into();
    trace!("write_entry_directory begin: root={} entries={}", root.display(), entries.len());
    prepare_entry_directory_target(&root, policy)?;
    let mut paths = Vec::new();
    for entry in entries {
        paths.push(write_new_entry_file(&root, entry)?);
    }
    trace!("write_entry_directory end: entries={}", paths.len());
    Ok(paths)
}
// sirno:witness:end

/// Mark the public Markdown entry directory as read-only.
///
/// Ignored paths are left untouched.
pub fn set_entry_directory_readonly(
    root: impl AsRef<Path>, settings: &EntryDirectoryCheckSettings,
) -> Result<(), EntryDirectoryError> {
    set_entry_directory_writability(root.as_ref(), settings, false)
}

/// Mark the public Markdown entry directory as writable.
///
/// Ignored paths are left untouched.
pub fn set_entry_directory_writable(
    root: impl AsRef<Path>, settings: &EntryDirectoryCheckSettings,
) -> Result<(), EntryDirectoryError> {
    set_entry_directory_writability(root.as_ref(), settings, true)
}

/// Add read-only checkout warnings to rendered entry files.
///
/// The warning is written as a Markdown blockquote at the beginning of the body.
pub fn add_readonly_checkout_warnings(paths: &[PathBuf]) -> Result<(), EntryDirectoryError> {
    for path in paths {
        let source = fs::read_to_string(path)?;
        let source = add_readonly_checkout_warning(path, &source)?;
        fs::write(path, source)
            .map_err(|source| EntryDirectoryError::WriteFile { path: path.clone(), source })?;
    }
    Ok(())
}

/// Generate Markdown link footers for one public entry directory.
///
/// The directory must pass review-mode checks before any file is written.
pub fn gen_link_entry_directory(
    root: impl Into<PathBuf>, settings: &GeneratedLinkSettings,
) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
    gen_link_entry_directory_with_ignored_paths(root, settings, Vec::<PathBuf>::new())
}

/// Check which generated Markdown link footers would change in one public entry directory.
///
/// No file is written.
pub fn check_gen_link_entry_directory(
    root: impl Into<PathBuf>, settings: &GeneratedLinkSettings,
) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
    check_gen_link_entry_directory_with_ignored_paths(root, settings, Vec::<PathBuf>::new())
}

/// Generate Markdown link footers for one public entry directory with ignored paths.
///
/// Ignored paths are relative to the entry directory root.
pub fn gen_link_entry_directory_with_ignored_paths(
    root: impl Into<PathBuf>, settings: &GeneratedLinkSettings,
    ignore: impl IntoIterator<Item = PathBuf>,
) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
    process_gen_link_entry_directory(root, settings, ignore, GenLinkOperation::Write)
}

/// Check which generated Markdown link footers would change with ignored paths.
///
/// Ignored paths are relative to the entry directory root.
/// No file is written.
pub fn check_gen_link_entry_directory_with_ignored_paths(
    root: impl Into<PathBuf>, settings: &GeneratedLinkSettings,
    ignore: impl IntoIterator<Item = PathBuf>,
) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
    process_gen_link_entry_directory(root, settings, ignore, GenLinkOperation::Check)
}

fn process_gen_link_entry_directory(
    root: impl Into<PathBuf>, settings: &GeneratedLinkSettings,
    ignore: impl IntoIterator<Item = PathBuf>, operation: GenLinkOperation,
) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
    let root = root.into();
    trace!(
        "gen_link_entry_directory begin: root={} operation={}",
        root.display(),
        operation.label()
    );
    let check_settings = EntryDirectoryCheckSettings {
        link: false,
        links: *settings,
        ignore: ignore.into_iter().collect(),
        witness: None,
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
            if operation.writes() {
                fs::write(path, rendered).map_err(|source| EntryDirectoryError::WriteFile {
                    path: path.to_path_buf(),
                    source,
                })?;
            }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenLinkOperation {
    Check,
    Write,
}

impl GenLinkOperation {
    fn label(self) -> &'static str {
        match self {
            | Self::Check => "check",
            | Self::Write => "write",
        }
    }

    fn writes(self) -> bool {
        matches!(self, Self::Write)
    }
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
        witness: None,
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
    add_witness_diagnostics(&entries, &paths_by_id, mode, settings, &mut file_diagnostics)?;
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

fn add_witness_diagnostics(
    entries: &[Entry], paths_by_id: &BTreeMap<EntryId, PathBuf>, mode: CheckMode,
    settings: &EntryDirectoryCheckSettings, file_diagnostics: &mut Vec<EntryFileDiagnostic>,
) -> Result<(), EntryDirectoryError> {
    let Some(witness) = &settings.witness else {
        return Ok(());
    };
    if witness.is_empty() {
        return Ok(());
    }

    let index = scan_witnesses(witness)?;
    let ids = entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
    let severity = mode_severity(mode);

    for entry in entries {
        if entry.metadata.witness.is_some() && !index.contains_entry(&entry.id) {
            let path = paths_by_id
                .get(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
            file_diagnostics.push(EntryFileDiagnostic::new(
                severity,
                path,
                format!(
                    "entry `{}` declares `witness:` but no repository witness block was found",
                    entry.id
                ),
            ));
        }
    }

    for witness_id in index.entry_ids() {
        if ids.contains(witness_id) {
            continue;
        }
        for record in index.records_for(witness_id) {
            file_diagnostics.push(EntryFileDiagnostic::new(
                severity,
                &record.path,
                format!("repository witness block references missing entry `{witness_id}`"),
            ));
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

fn add_readonly_checkout_warning(path: &Path, source: &str) -> Result<String, EntryDirectoryError> {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return Err(EntryDirectoryError::CheckoutConflict(path.to_path_buf()));
    };
    let id = EntryId::new(stem)
        .map_err(|_| EntryDirectoryError::CheckoutConflict(path.to_path_buf()))?;
    let entry = Entry::from_markdown(id, source)?;
    if entry.body.starts_with(READONLY_CHECKOUT_WARNING) {
        return Ok(source.to_owned());
    }
    let body = format!("{READONLY_CHECKOUT_WARNING}{}", entry.body);
    Ok(Entry::replace_markdown_body(source, &body)?)
}

fn prepare_entry_directory_target(
    root: &Path, policy: EntryDirectoryWritePolicy,
) -> Result<(), EntryDirectoryError> {
    match policy {
        | EntryDirectoryWritePolicy::EmptyDirectory => {
            if root.exists() {
                if !root.is_dir() {
                    return Err(EntryDirectoryError::NotDirectory(root.to_path_buf()));
                }
                if fs::read_dir(root)?.next().is_some() {
                    return Err(EntryDirectoryError::DirectoryNotEmpty(root.to_path_buf()));
                }
            } else {
                fs::create_dir_all(root)?;
            }
        }
        | EntryDirectoryWritePolicy::ReplaceDirectory { ignore } => {
            if root.exists() {
                if !root.is_dir() {
                    return Err(EntryDirectoryError::NotDirectory(root.to_path_buf()));
                }
                set_path_writable(root)?;
                let settings = EntryDirectoryCheckSettings {
                    ignore,
                    witness: None,
                    ..EntryDirectoryCheckSettings::default()
                };
                remove_managed_entry_files(root, &settings)?;
            } else {
                fs::create_dir_all(root)?;
            }
        }
    }
    Ok(())
}

fn remove_managed_entry_files(
    root: &Path, settings: &EntryDirectoryCheckSettings,
) -> Result<(), EntryDirectoryError> {
    for path in sorted_directory_paths(root)? {
        let relative_path = path.strip_prefix(root).map_err(|source| {
            EntryDirectoryError::StripRoot { path: path.clone(), root: root.to_path_buf(), source }
        })?;
        if settings.ignores(relative_path) {
            continue;
        }

        let file_type = fs::symlink_metadata(&path)?.file_type();
        if file_type.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("md")
            && is_managed_entry_file(&path)?
        {
            set_path_writable(&path)?;
            fs::remove_file(&path)?;
            continue;
        }

        return Err(EntryDirectoryError::CheckoutConflict(path));
    }
    Ok(())
}

fn is_managed_entry_file(path: &Path) -> Result<bool, EntryDirectoryError> {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return Ok(false);
    };
    let Ok(id) = EntryId::new(stem) else {
        return Ok(false);
    };
    let source = fs::read_to_string(path)?;
    Ok(Entry::from_markdown(id, &source).is_ok())
}

fn set_entry_directory_writability(
    root: &Path, settings: &EntryDirectoryCheckSettings, writable: bool,
) -> Result<(), EntryDirectoryError> {
    if !root.exists() {
        return Err(EntryDirectoryError::MissingDirectory(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(EntryDirectoryError::NotDirectory(root.to_path_buf()));
    }

    if writable {
        set_path_writable(root)?;
    }
    set_child_writability(root, root, settings, writable)?;
    if !writable {
        set_path_readonly(root)?;
    }
    Ok(())
}

fn set_child_writability(
    root: &Path, directory: &Path, settings: &EntryDirectoryCheckSettings, writable: bool,
) -> Result<(), EntryDirectoryError> {
    for path in sorted_directory_paths(directory)? {
        let relative_path = path.strip_prefix(root).map_err(|source| {
            EntryDirectoryError::StripRoot { path: path.clone(), root: root.to_path_buf(), source }
        })?;
        if settings.ignores(relative_path) {
            continue;
        }

        let file_type = fs::symlink_metadata(&path)?.file_type();
        if writable {
            set_path_writable(&path)?;
        }
        if file_type.is_dir() {
            set_child_writability(root, &path, settings, writable)?;
        }
        if !writable {
            set_path_readonly(&path)?;
        }
    }
    Ok(())
}

fn set_path_readonly(path: &Path) -> Result<(), EntryDirectoryError> {
    set_path_writable_flag(path, false)
}

fn set_path_writable(path: &Path) -> Result<(), EntryDirectoryError> {
    set_path_writable_flag(path, true)
}

fn set_path_writable_flag(path: &Path, writable: bool) -> Result<(), EntryDirectoryError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    let mut permissions = metadata.permissions();
    set_permissions_writable(&mut permissions, metadata.file_type().is_dir(), writable);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(unix)]
fn set_permissions_writable(permissions: &mut fs::Permissions, is_directory: bool, writable: bool) {
    let mode = permissions.mode();
    let next = if writable {
        if is_directory { mode | 0o700 } else { mode | 0o600 }
    } else {
        mode & !0o222
    };
    permissions.set_mode(next);
}

#[cfg(not(unix))]
fn set_permissions_writable(
    permissions: &mut fs::Permissions, _is_directory: bool, writable: bool,
) {
    permissions.set_readonly(!writable);
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
    /// The target entry directory must be empty for this write policy.
    #[error("entry directory must be empty before checkout: {0}")]
    DirectoryNotEmpty(PathBuf),
    /// Checkout would overwrite a path that is not a managed entry file.
    #[error("checkout conflict at unmanaged path: {0}")]
    CheckoutConflict(PathBuf),
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
    /// Repository witness lookup failed.
    #[error(transparent)]
    Witness(#[from] WitnessError),
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
    use crate::{CodeMember, EntryMetadata, WitnessCheckSettings};

    fn write_entry(root: &Path, name: &str, body: &str) {
        fs::write(root.join(name), body).unwrap();
    }

    fn witness_settings(root: &Path) -> EntryDirectoryCheckSettings {
        EntryDirectoryCheckSettings {
            witness: Some(WitnessCheckSettings::new(root, [CodeMember::new("src").unwrap()])),
            ..EntryDirectoryCheckSettings::default()
        }
    }

    fn witness_block(id: &str) -> String {
        format!("// sirno:witness:start {id}\nbody\n// sirno:witness:end\n")
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
    fn reports_structural_diagnostics_from_loaded_entries() {
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
        assert_eq!(report.structural_report().diagnostics().len(), 1);
    }

    #[test]
    fn check_accepts_witness_block_found_by_mosaika() {
        let temp = tempfile::tempdir().unwrap();
        let docs = temp.path().join("docs");
        let src = temp.path().join("src");
        fs::create_dir_all(&docs).unwrap();
        fs::create_dir_all(&src).unwrap();
        write_entry(
            &docs,
            "witnessed.md",
            "\
---
name: Witnessed
description: A witnessed entry.
witness:
---

Body.
",
        );
        fs::write(src.join("lib.rs"), witness_block("witnessed")).unwrap();

        let report = check_entry_directory_with_settings(
            &docs,
            CheckMode::Review,
            &witness_settings(temp.path()),
        )
        .unwrap();

        assert!(report.is_clean());
    }

    #[test]
    fn check_reports_missing_witness_block() {
        let temp = tempfile::tempdir().unwrap();
        let docs = temp.path().join("docs");
        let src = temp.path().join("src");
        fs::create_dir_all(&docs).unwrap();
        fs::create_dir_all(&src).unwrap();
        write_entry(
            &docs,
            "witnessed.md",
            "\
---
name: Witnessed
description: A witnessed entry.
witness:
---

Body.
",
        );
        fs::write(src.join("lib.rs"), "fn main() {}\n").unwrap();

        let report = check_entry_directory_with_settings(
            &docs,
            CheckMode::Review,
            &witness_settings(temp.path()),
        )
        .unwrap();

        assert!(report.has_errors());
        assert!(report.file_diagnostics()[0].message.contains("no repository witness block"));
    }

    #[test]
    fn check_reports_witness_block_for_missing_entry() {
        let temp = tempfile::tempdir().unwrap();
        let docs = temp.path().join("docs");
        let src = temp.path().join("src");
        fs::create_dir_all(&docs).unwrap();
        fs::create_dir_all(&src).unwrap();
        write_entry(
            &docs,
            "concept.md",
            "\
---
name: Concept
description: A concept.
---

Body.
",
        );
        fs::write(src.join("lib.rs"), witness_block("ghost-entry")).unwrap();

        let report = check_entry_directory_with_settings(
            &docs,
            CheckMode::Review,
            &witness_settings(temp.path()),
        )
        .unwrap();

        assert!(report.has_errors());
        assert!(report.file_diagnostics()[0].message.contains("missing entry `ghost-entry`"));
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
    fn replace_entry_directory_preserves_ignored_paths() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        fs::create_dir_all(root.join(".obsidian")).unwrap();
        fs::write(root.join(".obsidian/state.json"), "{}").unwrap();
        fs::write(root.join("old.md"), "---\nname: Old\ndescription: Old.\n---\n").unwrap();
        let metadata = EntryMetadata::new("New", "New entry.").unwrap();
        let entry = Entry::new(EntryId::new("new").unwrap(), metadata, "Body.\n");

        write_entry_directory(
            &root,
            std::slice::from_ref(&entry),
            EntryDirectoryWritePolicy::ReplaceDirectory {
                ignore: vec![PathBuf::from(".obsidian")],
            },
        )
        .unwrap();

        assert!(!root.join("old.md").exists());
        assert!(root.join("new.md").exists());
        assert!(root.join(".obsidian/state.json").exists());
    }

    #[test]
    fn replace_entry_directory_rejects_stray_markdown() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("2026-05-12.md"), "").unwrap();
        let metadata = EntryMetadata::new("New", "New entry.").unwrap();
        let entry = Entry::new(EntryId::new("new").unwrap(), metadata, "Body.\n");

        let error = write_entry_directory(
            &root,
            &[entry],
            EntryDirectoryWritePolicy::ReplaceDirectory { ignore: Vec::new() },
        )
        .unwrap_err();

        assert!(matches!(error, EntryDirectoryError::CheckoutConflict(_)));
        assert!(root.join("2026-05-12.md").exists());
    }

    #[test]
    fn readonly_entry_directory_can_be_made_writable_again() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();
        let settings = EntryDirectoryCheckSettings::default();

        set_entry_directory_readonly(&root, &settings).unwrap();
        assert!(fs::metadata(root.join("concept.md")).unwrap().permissions().readonly());
        assert!(fs::metadata(&root).unwrap().permissions().readonly());

        set_entry_directory_writable(&root, &settings).unwrap();
        assert!(!fs::metadata(root.join("concept.md")).unwrap().permissions().readonly());
        assert!(!fs::metadata(&root).unwrap().permissions().readonly());
    }

    #[test]
    fn readonly_checkout_warning_is_visible_body_quote() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        let metadata = EntryMetadata::new("New", "New entry.").unwrap();
        let entry = Entry::new(EntryId::new("new").unwrap(), metadata, "Body.\n");
        let paths = write_entry_directory(
            &root,
            std::slice::from_ref(&entry),
            EntryDirectoryWritePolicy::EmptyDirectory,
        )
        .unwrap();

        add_readonly_checkout_warnings(&paths).unwrap();
        let source = fs::read_to_string(root.join("new.md")).unwrap();
        let checked = check_entry_directory(&root, CheckMode::Review).unwrap();

        assert!(source.contains(
            "\n---\n\n> This file is a read-only Sirno history checkout.\n\
             > Do not edit it by hand.\n\nBody.\n"
        ));
        assert_eq!(checked.entries()[0].metadata, entry.metadata);
        assert!(checked.entries()[0].body.starts_with(READONLY_CHECKOUT_WARNING));
        assert!(checked.entries()[0].body.ends_with("Body.\n"));
    }

    #[test]
    fn gen_link_writes_generated_footers() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();
        let settings = GeneratedLinkSettings {
            category: true.into(),
            clustee: true.into(),
            clique: false,
            refiner: true.into(),
        };

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
        let settings = GeneratedLinkSettings {
            category: false.into(),
            clustee: true.into(),
            clique: true,
            refiner: false.into(),
        };

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
    fn check_gen_link_reports_changes_without_writing() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        init_entry_directory(&root).unwrap();
        let settings = GeneratedLinkSettings::default();

        let report = check_gen_link_entry_directory(&root, &settings).unwrap();
        let concept = fs::read_to_string(root.join("concept.md")).unwrap();

        assert_eq!(report.entry_count(), 3);
        assert_eq!(report.changed_paths().len(), 3);
        assert!(!concept.contains(crate::links::BEGIN_LINKS_GUARD));

        gen_link_entry_directory(&root, &settings).unwrap();
        let report = check_gen_link_entry_directory(&root, &settings).unwrap();

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
        let old_settings = GeneratedLinkSettings {
            category: true.into(),
            clustee: true.into(),
            clique: false,
            refiner: true.into(),
        };
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
        let old_settings = GeneratedLinkSettings {
            category: true.into(),
            clustee: true.into(),
            clique: false,
            refiner: true.into(),
        };
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
        let old_settings = GeneratedLinkSettings {
            category: true.into(),
            clustee: true.into(),
            clique: false,
            refiner: true.into(),
        };
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
