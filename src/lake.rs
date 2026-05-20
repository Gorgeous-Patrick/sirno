//! Public Sirno Lake support.
//!
//! This module reads the human-facing Sirno Lake shape:
//! a flat directory of `*.md` files whose filename stems are entry ids.
//! Lake-owned artifacts live under the reserved `.artifacts` directory.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use thiserror::Error;
use tracing::trace;

use crate::artifact::{
    ARTIFACT_DIRECTORY_NAME, EntryArtifact, EntryArtifactPath, EntryArtifactPathError,
};
use crate::check::{CheckMode, CheckReport, CheckSeverity};
use crate::entry::{
    Entry, EntryParseError, EntryRenderError, FrozenMarker, has_mixed_line_endings,
};
use crate::freeze::FrozenPath;
use crate::id::EntryId;
use crate::render::{GeneratedLinkBody, GeneratedLinkError};
use crate::structural::{StructuralEdgeIndex, StructuralSettings};
use crate::witness::{WitnessCheckSettings, WitnessError};

const READONLY_CHECKOUT_WARNING: &str = "\
> This file is a read-only Sirno Frost checkout.
> Do not edit it by hand.

";

/// Public Markdown entry directory.
///
/// Invariant: `root` is the directory containing one flat `*.md` file per entry id.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:lake:begin
pub struct EntryDirectory {
    root: PathBuf,
}
// sirno:witness:lake:end

/// Check report for a public Markdown entry directory.
#[derive(Debug)]
// sirno:witness:lake:begin
pub struct EntryDirectoryReport {
    root: PathBuf,
    entries: Vec<Entry>,
    artifacts: Vec<EntryArtifact>,
    paths_by_id: BTreeMap<EntryId, PathBuf>,
    file_diagnostics: Vec<EntryFileDiagnostic>,
    structural_report: CheckReport,
}
// sirno:witness:lake:end

/// Settings for checking a public Markdown entry directory.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:lake:begin
pub struct EntryDirectoryCheckSettings {
    /// Check generated footer freshness.
    pub render: bool,
    /// Configured structural fields and generated footer settings.
    pub structural: StructuralSettings,
    /// Lake-root-relative paths ignored by Sirno.
    pub ignore: Vec<PathBuf>,
    /// Repository witness scan settings.
    pub witness: Option<WitnessCheckSettings>,
}
// sirno:witness:lake:end

impl Default for EntryDirectoryCheckSettings {
    fn default() -> Self {
        Self {
            render: true,
            structural: StructuralSettings::default(),
            ignore: Vec::new(),
            witness: None,
        }
    }
}

impl EntryDirectoryCheckSettings {
    /// Return true when a root-relative path is ignored.
    // sirno:witness:lake:begin
    pub fn ignores(&self, relative_path: &Path) -> bool {
        self.ignore.iter().any(|ignored| {
            !ignored.as_os_str().is_empty()
                && (relative_path == ignored || relative_path.starts_with(ignored))
        })
    }
    // sirno:witness:lake:end
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

    /// Lake-owned artifacts attached to parsed entries.
    pub fn artifacts(&self) -> &[EntryArtifact] {
        &self.artifacts
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

/// Diagnostic produced while loading the public Markdown entry lake.
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

/// Result of renaming one entry id in a public entry directory.
#[derive(Debug)]
pub struct EntryRenameReport {
    old_id: EntryId,
    new_id: EntryId,
    changed_paths: Vec<PathBuf>,
}

impl EntryRenameReport {
    /// Entry id before the rename.
    pub fn old_id(&self) -> &EntryId {
        &self.old_id
    }

    /// Entry id after the rename.
    pub fn new_id(&self) -> &EntryId {
        &self.new_id
    }

    /// Entry files changed by the rename.
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
        /// Lake-root-relative paths preserved by checkout.
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

impl EntryDirectory {
    /// Construct an entry directory rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Directory containing public Markdown entry files.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Public Markdown file path for one entry id.
    pub fn entry_path(&self, id: &EntryId) -> PathBuf {
        self.entry_file_path(id)
    }

    /// Public artifact directory path for one entry id.
    pub fn entry_artifact_root_path(&self, id: &EntryId) -> PathBuf {
        self.entry_artifact_directory(id)
    }

    /// Public artifact file path for one entry-owned artifact.
    pub fn entry_artifact_path(&self, id: &EntryId, path: &EntryArtifactPath) -> PathBuf {
        self.entry_artifact_directory(id).join(path.to_path_buf())
    }

    /// Returns true when this directory contains the file for `id`.
    pub fn entry_exists(&self, id: &EntryId) -> Result<bool, EntryDirectoryError> {
        if !self.root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(self.root.clone()));
        }
        if !self.root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
        }

        let path = self.entry_file_path(id);
        match fs::symlink_metadata(path) {
            | Ok(metadata) => Ok(metadata.file_type().is_file()),
            | Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(false),
            | Err(source) => Err(source.into()),
        }
    }

    /// Read one public Markdown entry file source by id.
    pub fn read_entry_source(&self, id: &EntryId) -> Result<String, EntryDirectoryError> {
        if !self.root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(self.root.clone()));
        }
        if !self.root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
        }

        let path = self.entry_file_path(id);
        match fs::symlink_metadata(&path) {
            | Ok(metadata) if metadata.file_type().is_file() => {}
            | Ok(_) => return Err(EntryDirectoryError::EntryNotFound(id.clone())),
            | Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Err(EntryDirectoryError::EntryNotFound(id.clone()));
            }
            | Err(source) => return Err(source.into()),
        }

        Ok(fs::read_to_string(path)?)
    }

    /// Read one public Markdown entry file by id.
    pub fn read_entry(&self, id: &EntryId) -> Result<Entry, EntryDirectoryError> {
        let source = self.read_entry_source(id)?;
        Ok(Entry::from_markdown(id.clone(), &source)?)
    }

    /// Read lake-owned artifacts for one entry id.
    // sirno:witness:entry-artifact:begin
    pub fn read_entry_artifacts(
        &self, id: &EntryId,
    ) -> Result<Vec<EntryArtifact>, EntryDirectoryError> {
        if !self.root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(self.root.clone()));
        }
        if !self.root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
        }

        let owner_root = self.entry_artifact_directory(id);
        if !owner_root.exists() {
            return Ok(Vec::new());
        }
        if !owner_root.is_dir() {
            return Err(EntryDirectoryError::CheckoutConflict(owner_root));
        }

        let mut artifacts = Vec::new();
        for path in sorted_recursive_paths(&owner_root)? {
            let file_type = fs::symlink_metadata(&path)?.file_type();
            if file_type.is_dir() {
                continue;
            }
            if !file_type.is_file() {
                return Err(EntryDirectoryError::CheckoutConflict(path));
            }
            let relative_path = path.strip_prefix(&owner_root).map_err(|source| {
                EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: owner_root.clone(),
                    source,
                }
            })?;
            let artifact_path = EntryArtifactPath::new(relative_path)?;
            artifacts.push(EntryArtifact::new(id.clone(), artifact_path, fs::read(path)?));
        }
        artifacts.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(artifacts)
    }
    // sirno:witness:entry-artifact:end

    /// Copy one filesystem file into an entry's artifact tree.
    pub fn add_entry_artifact(
        &self, id: &EntryId, source: &Path, artifact_path: &EntryArtifactPath,
    ) -> Result<PathBuf, EntryDirectoryError> {
        self.ensure_entry_artifacts_mutable(id)?;
        match fs::symlink_metadata(source) {
            | Ok(metadata) if metadata.file_type().is_file() => {}
            | Ok(_) => {
                return Err(EntryDirectoryError::ArtifactSourceNotFile(source.to_path_buf()));
            }
            | Err(source) => return Err(source.into()),
        }

        let path = self.entry_artifact_path(id, artifact_path);
        if path.exists() {
            return Err(EntryDirectoryError::ArtifactAlreadyExists {
                owner: id.clone(),
                path: artifact_path.clone(),
            });
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, &path)?;
        Ok(path)
    }

    /// Rename one entry-owned artifact path.
    pub fn rename_entry_artifact(
        &self, id: &EntryId, old_path: &EntryArtifactPath, new_path: &EntryArtifactPath,
    ) -> Result<PathBuf, EntryDirectoryError> {
        self.ensure_entry_artifacts_mutable(id)?;
        if old_path == new_path {
            return Err(EntryDirectoryError::ArtifactRenameSamePath {
                owner: id.clone(),
                path: old_path.clone(),
            });
        }

        let source = self.entry_artifact_path(id, old_path);
        match fs::symlink_metadata(&source) {
            | Ok(metadata) if metadata.file_type().is_file() => {}
            | Ok(_) => {
                return Err(EntryDirectoryError::ArtifactNotFound {
                    owner: id.clone(),
                    path: old_path.clone(),
                });
            }
            | Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Err(EntryDirectoryError::ArtifactNotFound {
                    owner: id.clone(),
                    path: old_path.clone(),
                });
            }
            | Err(source) => return Err(source.into()),
        }
        let destination = self.entry_artifact_path(id, new_path);
        if destination.exists() {
            return Err(EntryDirectoryError::ArtifactAlreadyExists {
                owner: id.clone(),
                path: new_path.clone(),
            });
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&source, &destination).map_err(|source_error| {
            EntryDirectoryError::RenameFile {
                source_path: source.clone(),
                destination_path: destination.clone(),
                source: source_error,
            }
        })?;
        self.remove_empty_artifact_parents(id, old_path)?;
        Ok(destination)
    }

    /// Remove one entry-owned artifact file.
    pub fn remove_entry_artifact(
        &self, id: &EntryId, artifact_path: &EntryArtifactPath,
    ) -> Result<PathBuf, EntryDirectoryError> {
        self.ensure_entry_artifacts_mutable(id)?;
        let path = self.entry_artifact_path(id, artifact_path);
        match fs::symlink_metadata(&path) {
            | Ok(metadata) if metadata.file_type().is_file() => {}
            | Ok(_) => {
                return Err(EntryDirectoryError::ArtifactNotFound {
                    owner: id.clone(),
                    path: artifact_path.clone(),
                });
            }
            | Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Err(EntryDirectoryError::ArtifactNotFound {
                    owner: id.clone(),
                    path: artifact_path.clone(),
                });
            }
            | Err(source) => return Err(source.into()),
        }
        set_path_writable(&path)?;
        fs::remove_file(&path)?;
        self.remove_empty_artifact_parents(id, artifact_path)?;
        Ok(path)
    }

    /// Check this public Markdown entry directory.
    pub fn check(&self, mode: CheckMode) -> Result<EntryDirectoryReport, EntryDirectoryError> {
        self.check_with_settings(mode, &EntryDirectoryCheckSettings::default())
    }

    /// Check this public Markdown entry directory with explicit settings.
    // sirno:witness:lake:begin
    pub fn check_with_settings(
        &self, mode: CheckMode, settings: &EntryDirectoryCheckSettings,
    ) -> Result<EntryDirectoryReport, EntryDirectoryError> {
        trace!("check_entry_directory begin: root={}", self.root.display());
        let loaded = LoadedEntryDirectory::load(&self.root, mode, settings)?;
        let structural_report = mode.check_entries(&loaded.entries, &settings.structural);
        trace!(
            "check_entry_directory end: entries={} file_diagnostics={} structural_diagnostics={}",
            loaded.entries.len(),
            loaded.file_diagnostics.len(),
            structural_report.diagnostics().len()
        );
        Ok(EntryDirectoryReport {
            root: self.root.clone(),
            entries: loaded.entries,
            artifacts: loaded.artifacts,
            paths_by_id: loaded.paths_by_id,
            file_diagnostics: loaded.file_diagnostics,
            structural_report,
        })
    }
    // sirno:witness:lake:end

    /// Initialize this directory with ordinary seed entries.
    ///
    /// Existing entry files are never overwritten.
    // sirno:witness:lake:begin
    pub fn init(&self) -> Result<Vec<PathBuf>, EntryDirectoryError> {
        trace!("init_entry_directory begin: root={}", self.root.display());
        fs::create_dir_all(&self.root)?;
        let mut paths = Vec::new();
        for entry in Entry::default_seed_entries()? {
            let path = self.write_new_entry_file(&entry)?;
            paths.push(path);
        }
        trace!("init_entry_directory end: entries={}", paths.len());
        Ok(paths)
    }
    // sirno:witness:lake:end

    /// Create one public Markdown entry file in this directory.
    ///
    /// The entry directory is created if needed.
    /// Existing entry files are never overwritten.
    // sirno:witness:lake:begin
    pub fn create_entry(&self, entry: &Entry) -> Result<PathBuf, EntryDirectoryError> {
        trace!("create_entry_file begin: root={} id={}", self.root.display(), entry.id);
        fs::create_dir_all(&self.root)?;
        let path = self.write_new_entry_file(entry)?;
        trace!("create_entry_file end: path={}", path.display());
        Ok(path)
    }
    // sirno:witness:lake:end

    /// Mark one public Markdown entry as frozen and read-only.
    ///
    /// The entry metadata gains the canonical `frozen:` marker.
    /// Local file protection is applied after the marker is written.
    pub fn freeze_entry(&self, id: &EntryId) -> Result<PathBuf, EntryDirectoryError> {
        self.set_entry_frozen(id, true)
    }

    /// Mark one public Markdown entry as melted and writable.
    ///
    /// The canonical `frozen:` marker is removed from entry metadata.
    /// The file is left writable so normal editing can resume.
    pub fn melt_entry(&self, id: &EntryId) -> Result<PathBuf, EntryDirectoryError> {
        self.set_entry_frozen(id, false)
    }

    /// Rename one entry id and every structural metadata reference that names it.
    ///
    /// When the old id is a configured structural field, entry metadata keys are renamed too.
    /// Existing generated-link regions are refreshed after metadata changes.
    /// Prose outside generated-link regions remains user-owned.
    pub fn rename_entry(
        &self, old_id: &EntryId, new_id: &EntryId, settings: &EntryDirectoryCheckSettings,
    ) -> Result<EntryRenameReport, EntryDirectoryError> {
        trace!(
            "rename_entry begin: root={} old_id={} new_id={}",
            self.root.display(),
            old_id,
            new_id
        );
        if old_id == new_id {
            return Err(EntryDirectoryError::RenameSameId(old_id.clone()));
        }

        let mut check_settings = settings.clone();
        check_settings.render = false;
        let checked = self.check_with_settings(CheckMode::Review, &check_settings)?;
        if checked.has_errors() {
            return Err(EntryDirectoryError::InvalidEntryDirectory(self.root.clone()));
        }

        if checked.entry_path(old_id).is_none() {
            return Err(EntryDirectoryError::EntryNotFound(old_id.clone()));
        }
        if checked
            .entries()
            .iter()
            .any(|entry| &entry.id == old_id && entry.metadata.frozen.is_some())
        {
            return Err(EntryDirectoryError::FrozenEntryProtected(old_id.clone()));
        }
        let new_path = self.entry_file_path(new_id);
        match fs::symlink_metadata(&new_path) {
            | Ok(_) => {
                return Err(EntryDirectoryError::EntryAlreadyExists {
                    id: new_id.clone(),
                    path: new_path,
                });
            }
            | Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
            | Err(source) => return Err(source.into()),
        }

        let mut renamed_structural = settings.structural.clone();
        let rename_structural_field = renamed_structural.rename_field(old_id, new_id);

        let mut entries = Vec::<(EntryId, Entry, bool)>::new();
        for entry in checked.entries() {
            let original_id = entry.id.clone();
            let mut entry = entry.clone();
            if &entry.id == old_id {
                entry.id = new_id.clone();
            }
            let mut content_changed = entry.metadata.rename_structural_target(old_id, new_id);
            if rename_structural_field {
                content_changed |= entry.metadata.rename_structural_field(old_id, new_id);
            }
            entries.push((original_id, entry, content_changed));
        }

        let indexed_entries = entries.iter().map(|(_, entry, _)| entry.clone()).collect::<Vec<_>>();
        let link_index = StructuralEdgeIndex::from_entries(&indexed_entries);
        let mut changed_paths = Vec::new();

        for (original_id, mut entry, mut content_changed) in entries {
            if content_changed && entry.metadata.frozen.is_some() {
                return Err(EntryDirectoryError::FrozenEntryProtected(original_id));
            }
            let source_path = checked
                .entry_path(&original_id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryPath(original_id.clone()))?;
            let destination_path =
                if &original_id == old_id { new_path.as_path() } else { source_path };
            let footer = link_index.render_entry(&entry, &renamed_structural);
            let body = GeneratedLinkBody::new(&entry.body);
            if body.is_stale(&footer)? {
                entry.body = body.apply(&footer)?;
                content_changed = true;
            }
            if content_changed && entry.metadata.frozen.is_some() {
                return Err(EntryDirectoryError::FrozenEntryProtected(original_id.clone()));
            }

            if &original_id == old_id {
                set_path_writable(source_path)?;
                fs::rename(source_path, destination_path).map_err(|source| {
                    EntryDirectoryError::RenameFile {
                        source_path: source_path.to_path_buf(),
                        destination_path: destination_path.to_path_buf(),
                        source,
                    }
                })?;
                if content_changed {
                    let rendered = entry.to_markdown()?;
                    set_path_writable(destination_path)?;
                    fs::write(destination_path, rendered).map_err(|source| {
                        EntryDirectoryError::WriteFile {
                            path: destination_path.to_path_buf(),
                            source,
                        }
                    })?;
                }
                if entry.metadata.frozen.is_some() {
                    freeze_path_best_effort(destination_path)?;
                }
                changed_paths.push(destination_path.to_path_buf());
                continue;
            }

            if content_changed {
                let rendered = entry.to_markdown()?;
                set_path_writable(source_path)?;
                fs::write(source_path, rendered).map_err(|source| {
                    EntryDirectoryError::WriteFile { path: source_path.to_path_buf(), source }
                })?;
                if entry.metadata.frozen.is_some() {
                    freeze_path_best_effort(source_path)?;
                }
                changed_paths.push(source_path.to_path_buf());
            }
        }

        let old_artifacts = self.entry_artifact_directory(old_id);
        if old_artifacts.exists() {
            if !old_artifacts.is_dir() {
                return Err(EntryDirectoryError::CheckoutConflict(old_artifacts));
            }
            let new_artifacts = self.entry_artifact_directory(new_id);
            if new_artifacts.exists() {
                return Err(EntryDirectoryError::EntryAlreadyExists {
                    id: new_id.clone(),
                    path: new_artifacts,
                });
            }
            set_path_writable(&self.artifact_root())?;
            fs::rename(&old_artifacts, &new_artifacts).map_err(|source| {
                EntryDirectoryError::RenameFile {
                    source_path: old_artifacts.clone(),
                    destination_path: new_artifacts.clone(),
                    source,
                }
            })?;
            changed_paths.push(new_artifacts);
        }

        changed_paths.sort();
        changed_paths.dedup();
        trace!("rename_entry end: changed={}", changed_paths.len());
        Ok(EntryRenameReport { old_id: old_id.clone(), new_id: new_id.clone(), changed_paths })
    }

    /// Write a complete public Markdown entry directory.
    ///
    /// The write policy controls how existing target contents are handled.
    // sirno:witness:lake:begin
    pub fn write(
        &self, entries: &[Entry], policy: EntryDirectoryWritePolicy,
    ) -> Result<Vec<PathBuf>, EntryDirectoryError> {
        self.write_with_artifacts(entries, &[], policy)
    }

    /// Write a complete public entry directory with lake-owned artifacts.
    ///
    /// The write policy controls how existing target contents are handled.
    pub fn write_with_artifacts(
        &self, entries: &[Entry], artifacts: &[EntryArtifact], policy: EntryDirectoryWritePolicy,
    ) -> Result<Vec<PathBuf>, EntryDirectoryError> {
        trace!(
            "write_entry_directory begin: root={} entries={}",
            self.root.display(),
            entries.len()
        );
        self.prepare_target(policy)?;
        let mut paths = Vec::new();
        for entry in entries {
            paths.push(self.write_new_entry_file(entry)?);
        }
        paths.extend(self.write_entry_artifacts(entries, artifacts)?);
        trace!("write_entry_directory end: entries={}", paths.len());
        Ok(paths)
    }
    // sirno:witness:lake:end

    /// Mark this directory as read-only.
    ///
    /// Sirno removes ordinary write permission from managed paths.
    /// It also applies the best-effort immutable guard used by frozen entries.
    /// Ignored paths are left untouched.
    pub fn set_readonly(
        &self, settings: &EntryDirectoryCheckSettings,
    ) -> Result<(), EntryDirectoryError> {
        self.set_writability(settings, false)
    }

    /// Mark this directory as writable.
    ///
    /// Sirno clears its best-effort immutable guard before restoring ordinary write permission.
    /// Ignored paths are left untouched.
    pub fn set_writable(
        &self, settings: &EntryDirectoryCheckSettings,
    ) -> Result<(), EntryDirectoryError> {
        self.set_writability(settings, true)
    }

    /// Add read-only checkout warnings to rendered entry files.
    ///
    /// The warning is written as a Markdown blockquote at the beginning of the body.
    pub fn add_readonly_checkout_warnings(
        &self, paths: &[PathBuf],
    ) -> Result<(), EntryDirectoryError> {
        trace!("add_readonly_checkout_warnings begin: root={}", self.root.display());
        for path in paths {
            let source = fs::read_to_string(path)?;
            let source = add_readonly_checkout_warning(path, &source)?;
            fs::write(path, source)
                .map_err(|source| EntryDirectoryError::WriteFile { path: path.clone(), source })?;
        }
        Ok(())
    }

    /// Generate Markdown link footers for this public entry directory.
    ///
    /// The directory must pass review-mode checks before any file is written.
    pub fn generate_links(
        &self, settings: &StructuralSettings,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        self.generate_links_with_ignored_paths(settings, Vec::<PathBuf>::new())
    }

    /// Check which generated Markdown link footers would change in this directory.
    ///
    /// No file is written.
    pub fn check_generated_links(
        &self, settings: &StructuralSettings,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        self.check_generated_links_with_ignored_paths(settings, Vec::<PathBuf>::new())
    }

    /// Generate Markdown link footers for this directory with ignored paths.
    ///
    /// Ignored paths are relative to the entry directory root.
    pub fn generate_links_with_ignored_paths(
        &self, settings: &StructuralSettings, ignore: impl IntoIterator<Item = PathBuf>,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        self.process_generated_links(settings, ignore, GenLinkOperation::Write)
    }

    /// Check which generated Markdown link footers would change with ignored paths.
    ///
    /// Ignored paths are relative to the entry directory root.
    /// No file is written.
    pub fn check_generated_links_with_ignored_paths(
        &self, settings: &StructuralSettings, ignore: impl IntoIterator<Item = PathBuf>,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        self.process_generated_links(settings, ignore, GenLinkOperation::Check)
    }

    fn process_generated_links(
        &self, settings: &StructuralSettings, ignore: impl IntoIterator<Item = PathBuf>,
        operation: GenLinkOperation,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        trace!(
            "gen_link_entry_directory begin: root={} operation={}",
            self.root.display(),
            operation.label()
        );
        let check_settings = EntryDirectoryCheckSettings {
            render: false,
            structural: settings.clone(),
            ignore: ignore.into_iter().collect(),
            witness: None,
        };
        let checked = self.check_with_settings(CheckMode::Review, &check_settings)?;
        if checked.has_errors() {
            return Err(EntryDirectoryError::InvalidEntryDirectory(self.root.clone()));
        }

        let mut changed_paths = Vec::new();
        let index = StructuralEdgeIndex::from_entries(checked.entries());
        for entry in checked.entries() {
            let path = checked
                .entry_path(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
            let source = fs::read_to_string(path)?;
            let footer = index.render_entry(entry, settings);
            let body = GeneratedLinkBody::new(&entry.body).apply(&footer)?;
            let rendered = Entry::replace_markdown_body(&source, &body)?;
            if rendered != source {
                if operation.writes() {
                    if entry.metadata.frozen.is_some() {
                        return Err(EntryDirectoryError::FrozenEntryProtected(entry.id.clone()));
                    }
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
        Ok(GenLinkDirectoryReport {
            root: self.root.clone(),
            entry_count: checked.entries().len(),
            changed_paths,
        })
    }

    /// Delete generated Markdown link footers from this public entry directory.
    ///
    /// The directory must parse cleanly before any file is written.
    pub fn delete_generated_links(&self) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        self.delete_generated_links_with_ignored_paths(Vec::<PathBuf>::new())
    }

    /// Delete generated Markdown link footers with ignored paths.
    ///
    /// Ignored paths are relative to the entry directory root.
    pub fn delete_generated_links_with_ignored_paths(
        &self, ignore: impl IntoIterator<Item = PathBuf>,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        trace!("delete_gen_link_entry_directory begin: root={}", self.root.display());
        let check_settings = EntryDirectoryCheckSettings {
            render: false,
            structural: StructuralSettings::default(),
            ignore: ignore.into_iter().collect(),
            witness: None,
        };
        let checked = self.check_with_settings(CheckMode::Edit, &check_settings)?;
        if checked.has_errors() {
            return Err(EntryDirectoryError::InvalidEntryDirectory(self.root.clone()));
        }

        let mut changed_paths = Vec::new();
        for entry in checked.entries() {
            let path = checked
                .entry_path(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
            let source = fs::read_to_string(path)?;
            let body = GeneratedLinkBody::new(&entry.body).delete()?;
            let rendered = Entry::replace_markdown_body(&source, &body)?;
            if rendered != source {
                if entry.metadata.frozen.is_some() {
                    return Err(EntryDirectoryError::FrozenEntryProtected(entry.id.clone()));
                }
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
        Ok(GenLinkDirectoryReport {
            root: self.root.clone(),
            entry_count: checked.entries().len(),
            changed_paths,
        })
    }

    fn write_new_entry_file(&self, entry: &Entry) -> Result<PathBuf, EntryDirectoryError> {
        let path = self.entry_file_path(&entry.id);
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

    // sirno:witness:entry-artifact:begin
    fn write_entry_artifacts(
        &self, entries: &[Entry], artifacts: &[EntryArtifact],
    ) -> Result<Vec<PathBuf>, EntryDirectoryError> {
        if artifacts.is_empty() {
            return Ok(Vec::new());
        }

        let entry_ids = entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        let mut seen = BTreeSet::<(EntryId, EntryArtifactPath)>::new();
        let mut paths = Vec::new();
        for artifact in artifacts {
            if !entry_ids.contains(&artifact.owner) {
                return Err(EntryDirectoryError::EntryNotFound(artifact.owner.clone()));
            }
            if !seen.insert((artifact.owner.clone(), artifact.path.clone())) {
                return Err(EntryDirectoryError::DuplicateArtifact {
                    owner: artifact.owner.clone(),
                    path: artifact.path.clone(),
                });
            }

            let path =
                self.entry_artifact_directory(&artifact.owner).join(artifact.path.to_path_buf());
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file =
                OpenOptions::new().write(true).create_new(true).open(&path).map_err(|source| {
                    EntryDirectoryError::CreateFile { path: path.clone(), source }
                })?;
            file.write_all(&artifact.content)
                .map_err(|source| EntryDirectoryError::WriteFile { path: path.clone(), source })?;
            paths.push(path);
        }
        Ok(paths)
    }
    // sirno:witness:entry-artifact:end

    fn set_entry_frozen(&self, id: &EntryId, frozen: bool) -> Result<PathBuf, EntryDirectoryError> {
        if !self.root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(self.root.clone()));
        }
        if !self.root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
        }

        let path = self.entry_file_path(id);
        let source = fs::read_to_string(&path)?;
        let mut entry = Entry::from_markdown(id.clone(), &source)?;
        entry.metadata.frozen = frozen.then_some(FrozenMarker::Present);
        let rendered = entry.to_markdown()?;
        if !frozen {
            melt_path_best_effort(&path)?;
        }
        if rendered != source {
            set_path_writable(&path)?;
            fs::write(&path, rendered)
                .map_err(|source| EntryDirectoryError::WriteFile { path: path.clone(), source })?;
        }

        if frozen {
            freeze_path_best_effort(&path)?;
            self.set_entry_artifact_writability(id, false)?;
        } else {
            set_path_writable(&path)?;
            self.set_entry_artifact_writability(id, true)?;
        }
        Ok(path)
    }

    fn prepare_target(&self, policy: EntryDirectoryWritePolicy) -> Result<(), EntryDirectoryError> {
        match policy {
            | EntryDirectoryWritePolicy::EmptyDirectory => {
                if self.root.exists() {
                    if !self.root.is_dir() {
                        return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
                    }
                    if fs::read_dir(&self.root)?.next().is_some() {
                        return Err(EntryDirectoryError::DirectoryNotEmpty(self.root.clone()));
                    }
                } else {
                    fs::create_dir_all(&self.root)?;
                }
            }
            | EntryDirectoryWritePolicy::ReplaceDirectory { ignore } => {
                if self.root.exists() {
                    if !self.root.is_dir() {
                        return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
                    }
                    melt_path_best_effort(&self.root)?;
                    let settings = EntryDirectoryCheckSettings {
                        ignore,
                        witness: None,
                        ..EntryDirectoryCheckSettings::default()
                    };
                    self.remove_managed_entry_files(&settings)?;
                } else {
                    fs::create_dir_all(&self.root)?;
                }
            }
        }
        Ok(())
    }

    fn remove_managed_entry_files(
        &self, settings: &EntryDirectoryCheckSettings,
    ) -> Result<(), EntryDirectoryError> {
        for path in sorted_directory_paths(&self.root)? {
            let relative_path =
                path.strip_prefix(&self.root).map_err(|source| EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: self.root.clone(),
                    source,
                })?;
            if settings.ignores(relative_path) {
                continue;
            }

            let file_type = fs::symlink_metadata(&path)?.file_type();
            if relative_path == Path::new(ARTIFACT_DIRECTORY_NAME) && file_type.is_dir() {
                melt_tree_best_effort(&path)?;
                fs::remove_dir_all(&path)?;
                continue;
            }
            if file_type.is_file()
                && path.extension().and_then(|extension| extension.to_str()) == Some("md")
                && Self::is_managed_entry_file(&path)?
            {
                melt_path_best_effort(&path)?;
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

    fn is_frozen_entry_file(path: &Path) -> Result<bool, EntryDirectoryError> {
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            return Ok(false);
        };
        let Ok(id) = EntryId::new(stem) else {
            return Ok(false);
        };
        let source = fs::read_to_string(path)?;
        Ok(Entry::from_markdown(id, &source)
            .map(|entry| entry.metadata.frozen.is_some())
            .unwrap_or(false))
    }

    fn set_writability(
        &self, settings: &EntryDirectoryCheckSettings, writable: bool,
    ) -> Result<(), EntryDirectoryError> {
        if !self.root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(self.root.clone()));
        }
        if !self.root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
        }

        if writable {
            melt_path_best_effort(&self.root)?;
        }
        self.set_child_writability(&self.root, settings, writable)?;
        if !writable {
            freeze_path_best_effort(&self.root)?;
        }
        Ok(())
    }

    fn set_child_writability(
        &self, directory: &Path, settings: &EntryDirectoryCheckSettings, writable: bool,
    ) -> Result<(), EntryDirectoryError> {
        for path in sorted_directory_paths(directory)? {
            let relative_path =
                path.strip_prefix(&self.root).map_err(|source| EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: self.root.clone(),
                    source,
                })?;
            if settings.ignores(relative_path) {
                continue;
            }

            let file_type = fs::symlink_metadata(&path)?.file_type();
            if writable {
                if self.is_frozen_managed_path(&path)? {
                    continue;
                }
                melt_path_best_effort(&path)?;
            }
            if file_type.is_dir() {
                self.set_child_writability(&path, settings, writable)?;
            }
            if !writable {
                freeze_path_best_effort(&path)?;
            }
        }
        Ok(())
    }

    fn entry_file_path(&self, id: &EntryId) -> PathBuf {
        self.root.join(format!("{}.md", id.as_str()))
    }

    fn artifact_root(&self) -> PathBuf {
        self.root.join(ARTIFACT_DIRECTORY_NAME)
    }

    fn entry_artifact_directory(&self, id: &EntryId) -> PathBuf {
        self.artifact_root().join(id.as_str())
    }

    fn is_entry_file_path(&self, path: &Path) -> bool {
        path.parent() == Some(self.root.as_path())
            && path.extension().and_then(|extension| extension.to_str()) == Some("md")
    }

    fn is_frozen_managed_path(&self, path: &Path) -> Result<bool, EntryDirectoryError> {
        if self.is_entry_file_path(path) && Self::is_frozen_entry_file(path)? {
            return Ok(true);
        }

        let artifact_root = self.artifact_root();
        if let Ok(relative) = path.strip_prefix(&artifact_root) {
            let Some(owner) = relative.components().next().and_then(|component| match component {
                | Component::Normal(owner) => owner.to_str(),
                | _ => None,
            }) else {
                return Ok(false);
            };
            let Ok(owner) = EntryId::new(owner) else {
                return Ok(false);
            };
            let owner_entry = self.entry_file_path(&owner);
            if !owner_entry.exists() {
                return Ok(false);
            }
            return Self::is_frozen_entry_file(&owner_entry);
        }

        Ok(false)
    }

    fn set_entry_artifact_writability(
        &self, id: &EntryId, writable: bool,
    ) -> Result<(), EntryDirectoryError> {
        let owner_root = self.entry_artifact_directory(id);
        if !owner_root.exists() {
            return Ok(());
        }
        if !owner_root.is_dir() {
            return Err(EntryDirectoryError::CheckoutConflict(owner_root));
        }

        let paths = sorted_recursive_paths(&owner_root)?;
        if writable {
            set_path_writable(&self.artifact_root())?;
            set_path_writable(&owner_root)?;
            for path in paths {
                melt_path_best_effort(&path)?;
            }
            return Ok(());
        }

        for path in paths.iter().rev() {
            freeze_path_best_effort(path)?;
        }
        freeze_path_best_effort(&owner_root)
    }

    fn ensure_entry_artifacts_mutable(&self, id: &EntryId) -> Result<(), EntryDirectoryError> {
        let entry = self.read_entry(id)?;
        if entry.metadata.frozen.is_some() {
            return Err(EntryDirectoryError::FrozenEntryProtected(id.clone()));
        }
        Ok(())
    }

    fn remove_empty_artifact_parents(
        &self, id: &EntryId, artifact_path: &EntryArtifactPath,
    ) -> Result<(), EntryDirectoryError> {
        let owner_root = self.entry_artifact_directory(id);
        let Some(mut directory) =
            self.entry_artifact_path(id, artifact_path).parent().map(Path::to_path_buf)
        else {
            return Ok(());
        };

        while directory.starts_with(&owner_root) {
            if fs::read_dir(&directory)?.next().is_some() {
                break;
            }
            fs::remove_dir(&directory)?;
            if directory == owner_root {
                break;
            }
            let Some(parent) = directory.parent().map(Path::to_path_buf) else {
                break;
            };
            directory = parent;
        }
        Ok(())
    }
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

#[derive(Debug)]
struct LoadedEntryDirectory {
    entries: Vec<Entry>,
    artifacts: Vec<EntryArtifact>,
    paths_by_id: BTreeMap<EntryId, PathBuf>,
    file_diagnostics: Vec<EntryFileDiagnostic>,
}

impl LoadedEntryDirectory {
    fn load(
        root: &Path, mode: CheckMode, settings: &EntryDirectoryCheckSettings,
    ) -> Result<Self, EntryDirectoryError> {
        if !root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(root.to_path_buf()));
        }
        if !root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(root.to_path_buf()));
        }

        let non_entry_severity = mode.severity();
        let mut entries = Vec::new();
        let mut paths_by_id = BTreeMap::<EntryId, PathBuf>::new();
        let mut seen_ids = BTreeSet::<EntryId>::new();
        let mut file_diagnostics = Vec::new();
        let mut artifact_root = None;

        for path in sorted_directory_paths(root)? {
            let relative_path =
                path.strip_prefix(root).map_err(|source| EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: root.to_path_buf(),
                    source,
                })?;
            if settings.ignores(relative_path) {
                continue;
            }

            if relative_path == Path::new(ARTIFACT_DIRECTORY_NAME) {
                artifact_root = Some(path);
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
            if has_mixed_line_endings(&source) {
                file_diagnostics.push(EntryFileDiagnostic::new(
                    CheckSeverity::Warning,
                    &path,
                    "entry file uses mixed LF and CRLF line endings",
                ));
            }
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
        let mut loaded = Self { entries, artifacts: Vec::new(), paths_by_id, file_diagnostics };
        loaded.load_artifacts(root, artifact_root.as_deref(), mode)?;
        loaded.add_generated_link_diagnostics(mode, settings)?;
        loaded.add_witness_diagnostics(mode, settings)?;
        Ok(loaded)
    }

    // sirno:witness:entry-artifact:begin
    fn load_artifacts(
        &mut self, root: &Path, artifact_root: Option<&Path>, mode: CheckMode,
    ) -> Result<(), EntryDirectoryError> {
        let Some(artifact_root) = artifact_root else {
            return Ok(());
        };
        let severity = mode.severity();
        let file_type = fs::symlink_metadata(artifact_root)?.file_type();
        if !file_type.is_dir() {
            self.file_diagnostics.push(EntryFileDiagnostic::new(
                severity,
                artifact_root,
                "entry artifact storage must be a directory",
            ));
            return Ok(());
        }

        let ids = self.entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        for owner_path in sorted_directory_paths(artifact_root)? {
            let owner_type = fs::symlink_metadata(&owner_path)?.file_type();
            if !owner_type.is_dir() {
                self.file_diagnostics.push(EntryFileDiagnostic::new(
                    severity,
                    &owner_path,
                    "entry artifact storage contains an unsupported filesystem item",
                ));
                continue;
            }

            let Some(owner_name) = owner_path.file_name().and_then(|name| name.to_str()) else {
                self.file_diagnostics.push(EntryFileDiagnostic::new(
                    CheckSeverity::Error,
                    &owner_path,
                    "entry artifact directory name must be valid UTF-8",
                ));
                continue;
            };
            let owner = match EntryId::new(owner_name) {
                | Ok(owner) => owner,
                | Err(source) => {
                    self.file_diagnostics.push(EntryFileDiagnostic::new(
                        CheckSeverity::Error,
                        &owner_path,
                        format!("entry artifact directory name is not a valid entry id: {source}"),
                    ));
                    continue;
                }
            };
            if !ids.contains(&owner) {
                self.file_diagnostics.push(EntryFileDiagnostic::new(
                    severity,
                    &owner_path,
                    format!("entry artifact directory references missing entry `{owner}`"),
                ));
                continue;
            }

            self.load_entry_artifacts(root, &owner_path, &owner, severity)?;
        }
        self.artifacts.sort_by(|left, right| {
            left.owner.cmp(&right.owner).then_with(|| left.path.cmp(&right.path))
        });
        Ok(())
    }

    fn load_entry_artifacts(
        &mut self, root: &Path, owner_root: &Path, owner: &EntryId, severity: CheckSeverity,
    ) -> Result<(), EntryDirectoryError> {
        for path in sorted_recursive_paths(owner_root)? {
            let file_type = fs::symlink_metadata(&path)?.file_type();
            if file_type.is_dir() {
                continue;
            }
            if !file_type.is_file() {
                self.file_diagnostics.push(EntryFileDiagnostic::new(
                    severity,
                    &path,
                    "entry artifact tree contains an unsupported filesystem item",
                ));
                continue;
            }

            let relative_path =
                path.strip_prefix(owner_root).map_err(|source| EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: root.to_path_buf(),
                    source,
                })?;
            let artifact_path = match EntryArtifactPath::new(relative_path) {
                | Ok(path) => path,
                | Err(source) => {
                    self.file_diagnostics.push(EntryFileDiagnostic::new(
                        CheckSeverity::Error,
                        &path,
                        format!("invalid entry artifact path: {source}"),
                    ));
                    continue;
                }
            };
            let content = fs::read(&path)?;
            self.artifacts.push(EntryArtifact::new(owner.clone(), artifact_path, content));
        }
        Ok(())
    }
    // sirno:witness:entry-artifact:end

    fn add_generated_link_diagnostics(
        &mut self, mode: CheckMode, settings: &EntryDirectoryCheckSettings,
    ) -> Result<(), EntryDirectoryError> {
        let index = StructuralEdgeIndex::from_entries(&self.entries);
        for entry in &self.entries {
            let path = self
                .paths_by_id
                .get(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
            let body = GeneratedLinkBody::new(&entry.body);
            match body.validate() {
                | Ok(()) if settings.render => {
                    let expected = index.render_entry(entry, &settings.structural);
                    if body.is_stale(&expected)? {
                        self.file_diagnostics.push(EntryFileDiagnostic::new(
                            mode.severity(),
                            path,
                            "generated links are stale; run `sirno render`",
                        ));
                    }
                }
                | Ok(()) => {}
                | Err(source) => {
                    self.file_diagnostics.push(EntryFileDiagnostic::new(
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
        &mut self, mode: CheckMode, settings: &EntryDirectoryCheckSettings,
    ) -> Result<(), EntryDirectoryError> {
        let Some(witness) = &settings.witness else {
            return Ok(());
        };
        if witness.is_empty() {
            return Ok(());
        }

        let index = witness.scan()?;
        let ids = self.entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        let severity = mode.severity();

        for witness_id in index.entry_ids() {
            if ids.contains(witness_id) {
                continue;
            }
            for record in index.records_for(witness_id) {
                self.file_diagnostics.push(EntryFileDiagnostic::new(
                    severity,
                    &record.path,
                    format!("repository witness block references missing entry `{witness_id}`"),
                ));
            }
        }
        // sirno:witness:structural-check:begin
        for delimiter in index.orphan_delimiters() {
            self.file_diagnostics.push(EntryFileDiagnostic::new(
                severity,
                delimiter.path(),
                delimiter.diagnostic_message(),
            ));
        }
        // sirno:witness:structural-check:end

        Ok(())
    }
}

fn sorted_directory_paths(root: &Path) -> Result<Vec<PathBuf>, EntryDirectoryError> {
    let mut paths = fs::read_dir(root)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    Ok(paths)
}

fn sorted_recursive_paths(root: &Path) -> Result<Vec<PathBuf>, EntryDirectoryError> {
    let mut paths = Vec::new();
    collect_sorted_recursive_paths(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_sorted_recursive_paths(
    root: &Path, paths: &mut Vec<PathBuf>,
) -> Result<(), EntryDirectoryError> {
    for path in sorted_directory_paths(root)? {
        if fs::symlink_metadata(&path)?.file_type().is_dir() {
            collect_sorted_recursive_paths(&path, paths)?;
        }
        paths.push(path);
    }
    Ok(())
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

fn freeze_path_best_effort(path: &Path) -> Result<(), EntryDirectoryError> {
    set_path_writable_flag(path, false)?;
    if let Err(source) = FrozenPath::new(path).freeze() {
        trace!("immutable freeze unavailable: path={} error={source}", path.display());
    }
    Ok(())
}

fn melt_path_best_effort(path: &Path) -> Result<(), EntryDirectoryError> {
    if let Err(source) = FrozenPath::new(path).melt() {
        trace!("immutable melt unavailable: path={} error={source}", path.display());
    }
    set_path_writable(path)
}

fn melt_tree_best_effort(root: &Path) -> Result<(), EntryDirectoryError> {
    for path in sorted_recursive_paths(root)?.iter().rev() {
        melt_path_best_effort(path)?;
    }
    melt_path_best_effort(root)
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
    /// Entry rename source and destination ids must differ.
    #[error("entry rename source and destination are both `{0}`")]
    RenameSameId(EntryId),
    /// The entry selected for rename does not exist.
    #[error("entry `{0}` does not exist")]
    EntryNotFound(EntryId),
    /// The destination entry id already exists.
    #[error("entry `{id}` already exists at {path}")]
    EntryAlreadyExists {
        /// Existing destination id.
        id: EntryId,
        /// Existing destination path.
        path: PathBuf,
    },
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
    /// An artifact path could not be represented.
    #[error(transparent)]
    ArtifactPath(#[from] EntryArtifactPathError),
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
    /// A command attempted to change an entry marked as frozen.
    #[error("entry `{0}` is frozen; run `sirno melt {0}` before changing it")]
    FrozenEntryProtected(EntryId),
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
    /// An entry file could not be renamed.
    #[error("failed to rename entry file {source_path} to {destination_path}")]
    RenameFile {
        /// Existing entry path.
        source_path: PathBuf,
        /// New entry path.
        destination_path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Two artifact records name the same entry-owned path.
    #[error("entry `{owner}` has duplicate artifact `{path}`")]
    DuplicateArtifact {
        /// Entry that owns the duplicate artifact.
        owner: EntryId,
        /// Duplicate owner-relative artifact path.
        path: EntryArtifactPath,
    },
    /// Artifact source must be a regular file.
    #[error("artifact source is not a file: {0}")]
    ArtifactSourceNotFile(PathBuf),
    /// One artifact path was expected but is absent.
    #[error("entry `{owner}` has no artifact `{path}`")]
    ArtifactNotFound {
        /// Entry that owns the missing artifact.
        owner: EntryId,
        /// Owner-relative artifact path.
        path: EntryArtifactPath,
    },
    /// One artifact path already exists.
    #[error("entry `{owner}` already has artifact `{path}`")]
    ArtifactAlreadyExists {
        /// Entry that owns the existing artifact.
        owner: EntryId,
        /// Owner-relative artifact path.
        path: EntryArtifactPath,
    },
    /// Artifact rename source and destination paths must differ.
    #[error("entry `{owner}` artifact rename source and destination are both `{path}`")]
    ArtifactRenameSamePath {
        /// Entry that owns the artifact.
        owner: EntryId,
        /// Repeated owner-relative artifact path.
        path: EntryArtifactPath,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EntryMetadata, RepoMember, StructuralFieldSettings, WitnessCheckSettings, WitnessSettings,
    };

    const FIELD_KIND: &str = "kind";
    const FIELD_AREA: &str = "area";
    const FIELD_PARENT: &str = "parent";

    fn write_entry(root: &Path, name: &str, body: &str) {
        fs::write(root.join(name), body).unwrap();
    }

    fn write_structural_field_entries(root: &Path, fields: &[&str]) {
        for field in fields {
            write_entry(
                root,
                &format!("{field}.md"),
                &format!(
                    "\
---
name: {field}
desc: A structural field.
---

Body.
"
                ),
            );
        }
    }

    fn entry_directory(root: impl Into<PathBuf>) -> EntryDirectory {
        EntryDirectory::new(root)
    }

    fn witness_settings(root: &Path) -> EntryDirectoryCheckSettings {
        EntryDirectoryCheckSettings {
            witness: Some(WitnessCheckSettings::new(
                root,
                [RepoMember::new("src").unwrap()],
                WitnessSettings::standard(),
            )),
            ..EntryDirectoryCheckSettings::default()
        }
    }

    fn structural_settings(
        fields: impl IntoIterator<Item = (&'static str, StructuralFieldSettings)>,
    ) -> StructuralSettings {
        StructuralSettings::from_fields(fields)
    }

    fn all_test_fields_linked() -> StructuralSettings {
        structural_settings([
            (FIELD_KIND, render_settings(true, true, false)),
            (FIELD_AREA, render_settings(true, true, false)),
            (FIELD_PARENT, render_settings(true, true, false)),
        ])
    }

    fn render_settings(to: bool, from: bool, clique: bool) -> StructuralFieldSettings {
        StructuralFieldSettings::render_only(to, from, clique)
    }

    // sirno:witness:witness-fixture-isolation:begin
    fn witness_begin(id: &str) -> String {
        format!("{}{}{}{}", "// sirno", ":witness:", id, ":begin")
    }

    fn witness_end(id: &str) -> String {
        format!("{}{}{}{}", "// sirno", ":witness:", id, ":end")
    }

    fn witness_block(id: &str) -> String {
        let opening = witness_begin(id);
        let closing = witness_end(id);
        format!("{opening}\nbody\n{closing}\n")
    }
    // sirno:witness:witness-fixture-isolation:end

    #[test]
    fn checks_clean_markdown_entry_directory() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "meta.md",
            "\
---
name: Meta
desc: A metadata entry.
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
desc: A named idea.
---

Body.
",
        );

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(report.is_clean());
        assert_eq!(report.entries().len(), 2);
        assert!(report.entry_path(&EntryId::new("concept").unwrap()).is_some());
    }

    #[test]
    fn check_loads_entry_artifacts_from_reserved_directory() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "concept.md",
            "\
---
name: Concept
desc: A named idea.
---

Body.
",
        );
        let artifact_dir = temp.path().join(ARTIFACT_DIRECTORY_NAME).join("concept").join("images");
        fs::create_dir_all(&artifact_dir).unwrap();
        fs::write(artifact_dir.join("logo.bin"), [0, 1, 2, 3]).unwrap();

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(report.is_clean());
        assert_eq!(report.artifacts().len(), 1);
        assert_eq!(report.artifacts()[0].owner, EntryId::new("concept").unwrap());
        assert_eq!(report.artifacts()[0].path.as_str(), "images/logo.bin");
        assert_eq!(report.artifacts()[0].content, vec![0, 1, 2, 3]);
    }

    #[test]
    fn check_reports_artifacts_for_missing_entry() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join(ARTIFACT_DIRECTORY_NAME).join("ghost")).unwrap();

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(report.has_errors());
        assert!(report.file_diagnostics()[0].message.contains("missing entry `ghost`"));
    }

    #[test]
    fn entry_exists_checks_entry_file_presence() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "concept.md",
            "\
---
name: Concept
desc: A named idea.
---

Body.
",
        );

        assert!(
            entry_directory(temp.path()).entry_exists(&EntryId::new("concept").unwrap()).unwrap()
        );
        assert!(
            !entry_directory(temp.path()).entry_exists(&EntryId::new("missing").unwrap()).unwrap()
        );
    }

    #[test]
    fn reports_parse_error_with_file_path() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(temp.path(), "bad.md", "no frontmatter\n");

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(report.has_errors());
        assert_eq!(report.file_diagnostics().len(), 1);
        assert_eq!(report.file_diagnostics()[0].path, temp.path().join("bad.md"));
        assert!(report.file_diagnostics()[0].message.contains("failed to parse entry"));
    }

    #[test]
    fn reports_mixed_line_endings_as_warning() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "meta.md",
            "---\r\nname: Meta\ndesc: A metadata entry.\r\n---\r\n\r\nBody.\n",
        );

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(!report.is_clean());
        assert!(!report.has_errors());
        assert_eq!(report.entries().len(), 1);
        assert_eq!(report.file_diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(report.file_diagnostics()[0].message.contains("mixed LF and CRLF"));
    }

    #[test]
    fn reports_non_markdown_file_as_review_error() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("note.txt"), "text").unwrap();

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert_eq!(report.file_diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
    }

    #[test]
    fn reports_non_markdown_file_as_edit_warning() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("note.txt"), "text").unwrap();

        let report = entry_directory(temp.path()).check(CheckMode::Edit).unwrap();

        assert_eq!(report.file_diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }

    #[test]
    fn ignores_configured_lake_paths() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join(".obsidian")).unwrap();
        fs::write(temp.path().join("note.txt"), "text").unwrap();
        write_entry(
            temp.path(),
            "meta.md",
            "\
---
name: Meta
desc: A metadata entry.
---

Body.
",
        );

        let report = entry_directory(temp.path())
            .check_with_settings(
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
desc: A named idea.
kind:
  - meta
---
",
        );
        write_structural_field_entries(temp.path(), &[FIELD_KIND]);

        let report = entry_directory(temp.path())
            .check_with_settings(
                CheckMode::Review,
                &EntryDirectoryCheckSettings {
                    structural: structural_settings([(
                        FIELD_KIND,
                        StructuralFieldSettings::default(),
                    )]),
                    ..EntryDirectoryCheckSettings::default()
                },
            )
            .unwrap();

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
desc: A witnessed entry.
---

Body.
",
        );
        fs::write(src.join("lib.rs"), witness_block("witnessed")).unwrap();

        let report = entry_directory(&docs)
            .check_with_settings(CheckMode::Review, &witness_settings(temp.path()))
            .unwrap();

        assert!(report.is_clean());
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
desc: A concept.
---

Body.
",
        );
        fs::write(src.join("lib.rs"), witness_block("ghost-entry")).unwrap();

        let report = entry_directory(&docs)
            .check_with_settings(CheckMode::Review, &witness_settings(temp.path()))
            .unwrap();

        assert!(report.has_errors());
        assert!(report.file_diagnostics()[0].message.contains("missing entry `ghost-entry`"));
    }

    #[test]
    fn check_reports_orphan_witness_begin_delimiter() {
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
desc: A concept.
---

Body.
",
        );
        fs::write(src.join("lib.rs"), format!("{}\nbody\n", witness_begin("concept"))).unwrap();

        let report = entry_directory(&docs)
            .check_with_settings(CheckMode::Review, &witness_settings(temp.path()))
            .unwrap();

        assert!(report.has_errors());
        assert_eq!(report.file_diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.file_diagnostics()[0].message.contains("opening delimiter"));
        assert!(report.file_diagnostics()[0].message.contains("no closing delimiter"));
    }

    #[test]
    fn check_reports_orphan_witness_end_delimiter_as_edit_warning() {
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
desc: A concept.
---

Body.
",
        );
        fs::write(src.join("lib.rs"), format!("body\n{}\n", witness_end("concept"))).unwrap();

        let report = entry_directory(&docs)
            .check_with_settings(CheckMode::Edit, &witness_settings(temp.path()))
            .unwrap();

        assert!(!report.has_errors());
        assert_eq!(report.file_diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(report.file_diagnostics()[0].message.contains("closing delimiter"));
        assert!(report.file_diagnostics()[0].message.contains("no opening delimiter"));
    }

    #[test]
    fn missing_directory_is_a_load_error() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing");

        let error = entry_directory(&missing).check(CheckMode::Review).unwrap_err();

        assert!(matches!(error, EntryDirectoryError::MissingDirectory(_)));
    }

    #[test]
    fn initializes_seed_entry_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");

        let paths = entry_directory(&root).init().unwrap();
        let report = entry_directory(&root).check(CheckMode::Review).unwrap();

        assert_eq!(paths.len(), 4);
        assert!(root.join("concept.md").exists());
        assert!(report.is_clean());
    }

    #[test]
    fn init_refuses_to_overwrite_entry_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");

        entry_directory(&root).init().unwrap();
        let error = entry_directory(&root).init().unwrap_err();

        assert!(matches!(error, EntryDirectoryError::CreateFile { .. }));
    }

    #[test]
    fn create_entry_file_writes_one_entry() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        let mut metadata = EntryMetadata::new("Local Idea", "A local design idea.").unwrap();
        metadata.push_structural_target(FIELD_KIND, EntryId::new("meta").unwrap());
        let entry = Entry::new(EntryId::new("local-idea").unwrap(), metadata, "");

        let path = entry_directory(&root).create_entry(&entry).unwrap();
        let source = fs::read_to_string(&path).unwrap();

        assert_eq!(path, root.join("local-idea.md"));
        assert!(source.contains("name: Local Idea\n"));
        assert!(source.contains("kind:\n  - meta\n"));
    }

    #[test]
    fn create_entry_file_refuses_to_overwrite() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        let metadata = EntryMetadata::new("Local Idea", "A local design idea.").unwrap();
        let entry = Entry::new(EntryId::new("local-idea").unwrap(), metadata, "");

        entry_directory(&root).create_entry(&entry).unwrap();
        let error = entry_directory(&root).create_entry(&entry).unwrap_err();

        assert!(matches!(error, EntryDirectoryError::CreateFile { .. }));
    }

    #[test]
    fn rename_entry_updates_file_structural_targets_and_generated_links() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        fs::create_dir(&root).unwrap();
        write_entry(
            &root,
            "concept.md",
            "\
---
name: Concept
desc: A named idea.
---

Body.
",
        );
        write_structural_field_entries(&root, &[FIELD_KIND, FIELD_AREA]);
        write_entry(
            &root,
            "old-entry.md",
            "\
---
name: Source
desc: Source entry.
kind:
  - concept
---

Body.
",
        );
        write_entry(
            &root,
            "reader.md",
            "\
---
name: Reader
desc: Reader entry.
area:
  - old-entry
---

Body.
",
        );
        let settings = EntryDirectoryCheckSettings {
            structural: structural_settings([
                (FIELD_KIND, StructuralFieldSettings::default()),
                (FIELD_AREA, render_settings(true, true, false)),
            ]),
            ..EntryDirectoryCheckSettings::default()
        };
        let directory = entry_directory(&root);
        directory.generate_links(&settings.structural).unwrap();
        let artifact_dir = root.join(ARTIFACT_DIRECTORY_NAME).join("old-entry");
        fs::create_dir_all(&artifact_dir).unwrap();
        fs::write(artifact_dir.join("note.txt"), "artifact").unwrap();

        let report = directory
            .rename_entry(
                &EntryId::new("old-entry").unwrap(),
                &EntryId::new("new-entry").unwrap(),
                &settings,
            )
            .unwrap();
        let checked = directory.check_with_settings(CheckMode::Review, &settings).unwrap();
        let reader_source = fs::read_to_string(root.join("reader.md")).unwrap();
        let renamed_source = fs::read_to_string(root.join("new-entry.md")).unwrap();

        assert_eq!(report.old_id(), &EntryId::new("old-entry").unwrap());
        assert_eq!(report.new_id(), &EntryId::new("new-entry").unwrap());
        assert!(report.changed_paths().contains(&root.join("new-entry.md")));
        assert!(!root.join("old-entry.md").exists());
        assert!(root.join("new-entry.md").exists());
        assert!(!root.join(ARTIFACT_DIRECTORY_NAME).join("old-entry").exists());
        assert_eq!(
            fs::read_to_string(
                root.join(ARTIFACT_DIRECTORY_NAME).join("new-entry").join("note.txt")
            )
            .unwrap(),
            "artifact"
        );
        assert!(reader_source.contains("area:\n  - new-entry\n"));
        assert!(!reader_source.contains("old-entry"));
        assert!(renamed_source.contains("[reader](reader.md)"));
        assert!(checked.is_clean());
    }

    #[test]
    fn rename_entry_updates_configured_structural_field_names() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        fs::create_dir(&root).unwrap();
        write_entry(
            &root,
            "concept.md",
            "\
---
name: Concept
desc: A named idea.
---

Body.
",
        );
        write_entry(
            &root,
            "refines.md",
            "\
---
name: Refines
desc: A structural field.
---

Body.
",
        );
        write_entry(
            &root,
            "reader.md",
            "\
---
name: Reader
desc: Reader entry.
refines:
  - concept
---

Body.
",
        );
        let settings = EntryDirectoryCheckSettings {
            structural: structural_settings([("refines", render_settings(true, true, false))]),
            ..EntryDirectoryCheckSettings::default()
        };
        let directory = entry_directory(&root);
        directory.generate_links(&settings.structural).unwrap();

        directory
            .rename_entry(
                &EntryId::new("refines").unwrap(),
                &EntryId::new("prerequisite").unwrap(),
                &settings,
            )
            .unwrap();
        let reader_source = fs::read_to_string(root.join("reader.md")).unwrap();
        let concept_source = fs::read_to_string(root.join("concept.md")).unwrap();

        assert!(!root.join("refines.md").exists());
        assert!(root.join("prerequisite.md").exists());
        assert!(reader_source.contains("prerequisite:\n  - concept\n"));
        assert!(!reader_source.contains("refines:"));
        assert!(concept_source.contains("- prerequisite (from):\n  - [reader](reader.md)"));
    }

    #[test]
    fn rename_entry_refuses_existing_destination() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "old-entry.md",
            "\
---
name: Old
desc: Old entry.
---

Body.
",
        );
        write_entry(
            temp.path(),
            "new-entry.md",
            "\
---
name: New
desc: New entry.
---

Body.
",
        );

        let error = entry_directory(temp.path())
            .rename_entry(
                &EntryId::new("old-entry").unwrap(),
                &EntryId::new("new-entry").unwrap(),
                &EntryDirectoryCheckSettings::default(),
            )
            .unwrap_err();

        assert!(matches!(error, EntryDirectoryError::EntryAlreadyExists { .. }));
    }

    #[test]
    fn rename_entry_leaves_unreferenced_entries_untouched() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "old-entry.md",
            "\
---
name: Old
desc: Old entry.
---

Body.
",
        );
        let untouched = "\
---
desc: Untouched entry.
name: Untouched
---

Body.
";
        write_entry(temp.path(), "untouched.md", untouched);

        entry_directory(temp.path())
            .rename_entry(
                &EntryId::new("old-entry").unwrap(),
                &EntryId::new("new-entry").unwrap(),
                &EntryDirectoryCheckSettings::default(),
            )
            .unwrap();

        assert_eq!(fs::read_to_string(temp.path().join("untouched.md")).unwrap(), untouched);
    }

    #[test]
    fn freeze_entry_file_writes_marker_and_removes_write_permission() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "alpha.md",
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        );

        let path =
            entry_directory(temp.path()).freeze_entry(&EntryId::new("alpha").unwrap()).unwrap();
        let source = fs::read_to_string(&path).unwrap();

        assert!(source.contains("frozen:\n"));
        assert_path_readonly(&path);
    }

    #[test]
    fn set_writable_preserves_frozen_entry_permission() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "alpha.md",
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        );
        let settings = EntryDirectoryCheckSettings::default();
        let directory = entry_directory(temp.path());
        let path = directory.freeze_entry(&EntryId::new("alpha").unwrap()).unwrap();

        directory.set_writable(&settings).unwrap();

        assert_path_readonly(&path);
        directory.melt_entry(&EntryId::new("alpha").unwrap()).unwrap();
    }

    #[test]
    fn melt_entry_file_removes_marker_and_restores_write_permission() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "alpha.md",
            "\
---
name: Alpha
desc: Alpha entry.
frozen:
---

Body.
",
        );
        let path =
            entry_directory(temp.path()).freeze_entry(&EntryId::new("alpha").unwrap()).unwrap();

        entry_directory(temp.path()).melt_entry(&EntryId::new("alpha").unwrap()).unwrap();
        let source = fs::read_to_string(&path).unwrap();

        assert!(!source.contains("frozen:\n"));
        assert_path_writable(&path);
    }

    #[test]
    fn generate_links_refuses_to_change_frozen_entry() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "alpha.md",
            "\
---
name: Alpha
desc: Alpha entry.
frozen:
kind:
  - beta
---

Body.
",
        );
        write_entry(
            temp.path(),
            "beta.md",
            "\
---
name: Beta
desc: Beta entry.
---

Body.
",
        );
        write_structural_field_entries(temp.path(), &[FIELD_KIND]);

        let error = entry_directory(temp.path())
            .generate_links(&structural_settings([(
                FIELD_KIND,
                render_settings(true, false, false),
            )]))
            .unwrap_err();

        assert!(
            matches!(error, EntryDirectoryError::FrozenEntryProtected(id) if id.as_str() == "alpha")
        );
    }

    #[test]
    fn replace_entry_directory_preserves_ignored_paths() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        fs::create_dir_all(root.join(".obsidian")).unwrap();
        fs::write(root.join(".obsidian/state.json"), "{}").unwrap();
        fs::write(root.join("old.md"), "---\nname: Old\ndesc: Old.\n---\n").unwrap();
        let metadata = EntryMetadata::new("New", "New entry.").unwrap();
        let entry = Entry::new(EntryId::new("new").unwrap(), metadata, "Body.\n");

        entry_directory(&root)
            .write(
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
    fn replace_entry_directory_replaces_readonly_artifact_tree() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        fs::create_dir_all(root.join(ARTIFACT_DIRECTORY_NAME).join("old")).unwrap();
        fs::write(root.join("old.md"), "---\nname: Old\ndesc: Old.\n---\n").unwrap();
        fs::write(root.join(ARTIFACT_DIRECTORY_NAME).join("old").join("note.txt"), "old").unwrap();
        let directory = entry_directory(&root);
        directory.set_readonly(&EntryDirectoryCheckSettings::default()).unwrap();
        let metadata = EntryMetadata::new("New", "New entry.").unwrap();
        let entry = Entry::new(EntryId::new("new").unwrap(), metadata, "Body.\n");
        let artifact = EntryArtifact::new(
            entry.id.clone(),
            EntryArtifactPath::new("note.txt").unwrap(),
            b"new",
        );

        directory
            .write_with_artifacts(
                std::slice::from_ref(&entry),
                &[artifact],
                EntryDirectoryWritePolicy::ReplaceDirectory { ignore: Vec::new() },
            )
            .unwrap();

        assert!(!root.join("old.md").exists());
        assert!(!root.join(ARTIFACT_DIRECTORY_NAME).join("old").exists());
        assert_eq!(
            fs::read(root.join(ARTIFACT_DIRECTORY_NAME).join("new").join("note.txt")).unwrap(),
            b"new"
        );
    }

    #[test]
    fn replace_entry_directory_rejects_stray_markdown() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("2026-05-12.md"), "").unwrap();
        let metadata = EntryMetadata::new("New", "New entry.").unwrap();
        let entry = Entry::new(EntryId::new("new").unwrap(), metadata, "Body.\n");

        let error = entry_directory(&root)
            .write(&[entry], EntryDirectoryWritePolicy::ReplaceDirectory { ignore: Vec::new() })
            .unwrap_err();

        assert!(matches!(error, EntryDirectoryError::CheckoutConflict(_)));
        assert!(root.join("2026-05-12.md").exists());
    }

    #[test]
    fn readonly_entry_directory_can_be_made_writable_again() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        entry_directory(&root).init().unwrap();
        let settings = EntryDirectoryCheckSettings::default();
        let entry_path = root.join("concept.md");

        entry_directory(&root).set_readonly(&settings).unwrap();
        let entry_was_immutable = FrozenPath::new(&entry_path).is_frozen().unwrap_or(false);
        let root_was_immutable = FrozenPath::new(&root).is_frozen().unwrap_or(false);
        assert!(fs::metadata(&entry_path).unwrap().permissions().readonly());
        assert!(fs::metadata(&root).unwrap().permissions().readonly());

        entry_directory(&root).set_writable(&settings).unwrap();
        assert!(!fs::metadata(&entry_path).unwrap().permissions().readonly());
        assert!(!fs::metadata(&root).unwrap().permissions().readonly());
        if entry_was_immutable {
            assert!(!FrozenPath::new(&entry_path).is_frozen().unwrap());
        }
        if root_was_immutable {
            assert!(!FrozenPath::new(&root).is_frozen().unwrap());
        }
    }

    #[test]
    fn readonly_checkout_warning_is_visible_body_quote() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        let metadata = EntryMetadata::new("New", "New entry.").unwrap();
        let entry = Entry::new(EntryId::new("new").unwrap(), metadata, "Body.\n");
        let paths = entry_directory(&root)
            .write(std::slice::from_ref(&entry), EntryDirectoryWritePolicy::EmptyDirectory)
            .unwrap();

        entry_directory(&root).add_readonly_checkout_warnings(&paths).unwrap();
        let source = fs::read_to_string(root.join("new.md")).unwrap();
        let checked = entry_directory(&root).check(CheckMode::Review).unwrap();

        assert!(source.contains(
            "\n---\n\n> This file is a read-only Sirno Frost checkout.\n\
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
        entry_directory(&root).init().unwrap();
        write_structural_field_entries(&root, &[FIELD_KIND, FIELD_AREA, FIELD_PARENT]);
        let settings = all_test_fields_linked();

        let report = entry_directory(&root).generate_links(&settings).unwrap();
        let concept = fs::read_to_string(root.join("concept.md")).unwrap();

        assert_eq!(report.entry_count(), 7);
        assert_eq!(report.changed_paths().len(), 7);
        assert!(concept.contains(crate::render::BEGIN_LINKS_GUARD));
        assert!(concept.contains("\n---\n\n> **Sirno generated links begin."));
        assert!(concept.contains("- kind (to): (none)"));
        assert!(!concept.contains("## Sirno Links"));
        assert!(!concept.contains("kind: [meta](meta.md)"));
    }

    #[test]
    fn gen_link_expands_cliques_with_lake_context() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "core.md",
            "\
---
name: Core
desc: A review neighborhood.
---

Body.
",
        );
        write_structural_field_entries(temp.path(), &[FIELD_AREA]);
        write_entry(
            temp.path(),
            "left.md",
            "\
---
name: Left
desc: A neighborhood member.
area:
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
desc: A neighborhood member.
area:
  - core
---

Body.
",
        );
        let settings = structural_settings([(FIELD_AREA, render_settings(true, true, true))]);

        entry_directory(temp.path()).generate_links(&settings).unwrap();
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
        entry_directory(&root).init().unwrap();
        let settings = StructuralSettings::default();

        entry_directory(&root).generate_links(&settings).unwrap();
        let report = entry_directory(&root).generate_links(&settings).unwrap();

        assert!(report.changed_paths().is_empty());
    }

    #[test]
    fn check_gen_link_reports_changes_without_writing() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        entry_directory(&root).init().unwrap();
        let settings = StructuralSettings::default();

        let report = entry_directory(&root).check_generated_links(&settings).unwrap();
        let concept = fs::read_to_string(root.join("concept.md")).unwrap();

        assert_eq!(report.entry_count(), 4);
        assert_eq!(report.changed_paths().len(), 4);
        assert!(!concept.contains(crate::render::BEGIN_LINKS_GUARD));

        entry_directory(&root).generate_links(&settings).unwrap();
        let report = entry_directory(&root).check_generated_links(&settings).unwrap();

        assert!(report.changed_paths().is_empty());
    }

    #[test]
    fn delete_gen_link_removes_generated_footers() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        entry_directory(&root).init().unwrap();
        entry_directory(&root).generate_links(&StructuralSettings::default()).unwrap();

        let report = entry_directory(&root).delete_generated_links().unwrap();
        let concept = fs::read_to_string(root.join("concept.md")).unwrap();

        assert_eq!(report.entry_count(), 4);
        assert_eq!(report.changed_paths().len(), 4);
        assert!(!concept.contains(crate::render::BEGIN_LINKS_GUARD));
    }

    #[test]
    fn delete_gen_link_is_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        entry_directory(&root).init().unwrap();

        let report = entry_directory(&root).delete_generated_links().unwrap();

        assert_eq!(report.entry_count(), 4);
        assert!(report.changed_paths().is_empty());
    }

    #[test]
    fn check_reports_stale_generated_links_as_review_error() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        entry_directory(&root).init().unwrap();
        write_structural_field_entries(&root, &[FIELD_KIND, FIELD_AREA, FIELD_PARENT]);
        let old_settings = all_test_fields_linked();
        entry_directory(&root).generate_links(&old_settings).unwrap();

        let report = entry_directory(&root)
            .check_with_settings(
                CheckMode::Review,
                &EntryDirectoryCheckSettings {
                    render: true,
                    structural: StructuralSettings::default(),
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
        entry_directory(&root).init().unwrap();
        write_structural_field_entries(&root, &[FIELD_KIND, FIELD_AREA, FIELD_PARENT]);
        let old_settings = all_test_fields_linked();
        entry_directory(&root).generate_links(&old_settings).unwrap();

        let report = entry_directory(&root)
            .check_with_settings(
                CheckMode::Edit,
                &EntryDirectoryCheckSettings {
                    render: true,
                    structural: StructuralSettings::default(),
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
        entry_directory(&root).init().unwrap();
        write_structural_field_entries(&root, &[FIELD_KIND, FIELD_AREA, FIELD_PARENT]);
        let old_settings = all_test_fields_linked();
        entry_directory(&root).generate_links(&old_settings).unwrap();

        let report = entry_directory(&root)
            .check_with_settings(
                CheckMode::Review,
                &EntryDirectoryCheckSettings {
                    render: false,
                    structural: StructuralSettings::default(),
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
desc: A named idea.
---

Body.
> **Sirno generated links begin. Do not edit this section.**
",
        );

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(report.has_errors());
        assert!(report.file_diagnostics()[0].message.contains("malformed generated links"));
    }

    #[test]
    fn gen_link_refuses_dirty_entry_directory() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(temp.path(), "bad.md", "no frontmatter\n");

        let error = entry_directory(temp.path())
            .generate_links(&StructuralSettings::default())
            .unwrap_err();

        assert!(matches!(error, EntryDirectoryError::InvalidEntryDirectory(_)));
    }

    fn assert_path_readonly(path: &Path) {
        let permissions = fs::metadata(path).unwrap().permissions();
        #[cfg(unix)]
        assert_eq!(permissions.mode() & 0o222, 0);
        #[cfg(not(unix))]
        assert!(permissions.readonly());
    }

    fn assert_path_writable(path: &Path) {
        let permissions = fs::metadata(path).unwrap().permissions();
        #[cfg(unix)]
        assert_ne!(permissions.mode() & 0o222, 0);
        #[cfg(not(unix))]
        assert!(!permissions.readonly());
    }
}
