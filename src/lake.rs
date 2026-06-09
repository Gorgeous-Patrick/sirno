//! Sirno Lake support.
//!
//! This module reads the human-facing Sirno Lake shape:
//! Markdown entry files whose lake-relative paths become entry addresses.
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
    DESC_FIELD, Entry, EntryParseError, EntryRenderError, FrozenMarker, NAME_FIELD, RawEntry,
    has_mixed_line_endings,
};
use crate::freeze::FrozenPath;
use crate::identifier::EntryAtom;
use crate::identifier::{EntryAddress, EntryAddressError};
use crate::meta::{MetaRegistry, MetaRegistryError};
use crate::render::{GeneratedLinkBody, GeneratedLinkError};
use crate::structural::{StructuralEdgeIndex, StructuralRenderSettings, StructuralSettings};
use crate::witness::{WitnessCheckSettings, WitnessError};

const READONLY_CHECKOUT_WARNING: &str = "\
> This file is a read-only Sirno managed checkout.
> Do not edit it by hand.

";

/// Sirno Lake entry directory.
///
/// Invariant: `root` is the directory containing Sirno Lake Markdown entry files.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:reservoir:begin
pub struct EntryDirectory {
    root: PathBuf,
}
// sirno:witness:reservoir:end

/// Check report for a Sirno Lake entry directory.
#[derive(Debug)]
// sirno:witness:reservoir:begin
pub struct EntryDirectoryReport {
    root: PathBuf,
    entries: Vec<Entry>,
    artifacts: Vec<EntryArtifact>,
    paths_by_address: BTreeMap<EntryAddress, PathBuf>,
    file_diagnostics: Vec<EntryFileDiagnostic>,
    structural_report: CheckReport,
    meta: MetaRegistry,
    structural: StructuralSettings,
}
// sirno:witness:reservoir:end

/// Settings for checking a Sirno Lake entry directory.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:reservoir:begin
pub struct EntryDirectoryCheckSettings {
    /// Check generated footer freshness.
    pub render: bool,
    /// Generated-footer render policy for discovered structural relations.
    pub structural_render: StructuralRenderSettings,
    /// Path for the generated meta registry lockfile.
    pub meta_path: Option<PathBuf>,
    /// Lake-root-relative paths ignored by Sirno.
    pub ignore: Vec<PathBuf>,
    /// Repository witness scan settings.
    pub witness: Option<WitnessCheckSettings>,
}
// sirno:witness:reservoir:end

impl Default for EntryDirectoryCheckSettings {
    fn default() -> Self {
        Self {
            render: true,
            structural_render: StructuralRenderSettings::default(),
            meta_path: None,
            ignore: Vec::new(),
            witness: None,
        }
    }
}

impl EntryDirectoryCheckSettings {
    /// Return true when a root-relative path is ignored.
    // sirno:witness:reservoir:begin
    pub fn ignores(&self, relative_path: &Path) -> bool {
        self.ignore.iter().any(|ignored| {
            !ignored.as_os_str().is_empty()
                && (relative_path == ignored || relative_path.starts_with(ignored))
        })
    }
    // sirno:witness:reservoir:end
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

    /// Generated meta registry discovered while loading this report.
    pub fn meta(&self) -> &MetaRegistry {
        &self.meta
    }

    /// Discovered structural relation settings used by this report.
    pub fn structural(&self) -> &StructuralSettings {
        &self.structural
    }

    /// Return the path associated with a parsed entry address.
    pub fn entry_file_path(&self, id: &EntryAddress) -> Option<&Path> {
        self.paths_by_address.get(id).map(PathBuf::as_path)
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

/// Diagnostic produced while loading the Sirno Lake.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:diagnostics:begin
pub struct EntryFileDiagnostic {
    /// Diagnostic severity.
    pub severity: CheckSeverity,
    /// Stable diagnostic code.
    pub code: &'static str,
    /// Path responsible for the diagnostic.
    pub path: PathBuf,
    /// One-based line number when the diagnostic has a source position.
    pub line: Option<usize>,
    /// One-based column number when the diagnostic has a source position.
    pub column: Option<usize>,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Repair hint for human and agent-facing output.
    pub help: Option<String>,
}
// sirno:witness:diagnostics:end

/// Result of generating link footers for an entry directory.
#[derive(Debug)]
pub struct GenLinkDirectoryReport {
    root: PathBuf,
    entry_count: usize,
    changed_entry_count: usize,
    changed_paths: Vec<PathBuf>,
}

impl GenLinkDirectoryReport {
    /// Build a generated-link report from an already processed entry set.
    pub(crate) fn new(
        root: impl Into<PathBuf>, entry_count: usize, changed_entry_count: usize,
        changed_paths: Vec<PathBuf>,
    ) -> Self {
        Self { root: root.into(), entry_count, changed_entry_count, changed_paths }
    }

    /// Directory whose entries were processed.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Number of entries processed.
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// Number of entries whose generated-link region or projection changed.
    pub fn changed_entry_count(&self) -> usize {
        self.changed_entry_count
    }

    /// Entry files whose generated-link region changed.
    pub fn changed_paths(&self) -> &[PathBuf] {
        &self.changed_paths
    }
}

/// Result of renaming one entry address in a Sirno Lake entry directory.
#[derive(Debug)]
pub struct EntryRenameReport {
    old_id: EntryAddress,
    new_id: EntryAddress,
    changed_paths: Vec<PathBuf>,
}

/// Result of clearing or repairing Sirno local filesystem protection.
#[derive(Debug)]
pub struct EntryProtectionReport {
    root: PathBuf,
    paths: Vec<PathBuf>,
}

// sirno:witness:glacier:begin
/// Result of replacing one glacier.
#[derive(Debug)]
pub struct GlacierReport {
    root: PathBuf,
    domain: EntryAtom,
    changed_paths: Vec<PathBuf>,
}
// sirno:witness:glacier:end

impl EntryRenameReport {
    /// Entry address before the rename.
    pub fn old_id(&self) -> &EntryAddress {
        &self.old_id
    }

    /// Entry address after the rename.
    pub fn new_id(&self) -> &EntryAddress {
        &self.new_id
    }

    /// Entry files changed by the rename.
    pub fn changed_paths(&self) -> &[PathBuf] {
        &self.changed_paths
    }
}

impl EntryProtectionReport {
    /// Lake directory whose local protection was inspected.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Paths selected by the local protection operation.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

impl GlacierReport {
    /// Lake directory that was updated.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Glacier domain that was crystallized.
    pub fn domain(&self) -> &EntryAtom {
        &self.domain
    }

    /// Paths written or removed by crystallization.
    pub fn changed_paths(&self) -> &[PathBuf] {
        &self.changed_paths
    }
}

/// Conflict policy for writing a complete Sirno Lake entry directory.
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
    // sirno:witness:diagnostics:begin
    /// Construct a diagnostic for one path.
    pub fn new(
        severity: CheckSeverity, path: impl Into<PathBuf>, message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: "lake.file",
            path: path.into(),
            line: None,
            column: None,
            message: message.into(),
            help: None,
        }
    }

    /// Return this diagnostic with a stable code.
    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = code;
        self
    }

    /// Return this diagnostic with a source position.
    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Return this diagnostic with a repair hint.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
    // sirno:witness:diagnostics:end
}

// sirno:witness:diagnostics:begin
fn entry_parse_diagnostic(
    path: &Path, source: &EntryParseError, registry: Option<&MetaRegistry>,
) -> EntryFileDiagnostic {
    if let Some(diagnostic) = undefined_intrinsic_diagnostic(path, source, registry) {
        return diagnostic;
    }
    let mut diagnostic = EntryFileDiagnostic::new(
        CheckSeverity::Error,
        path,
        format!("failed to parse entry: {source}"),
    )
    .with_code(source.code());
    if let Some((line, column)) = source.position() {
        diagnostic = diagnostic.with_position(line + 1, column);
    }
    if let Some(help) = source.help() {
        diagnostic = diagnostic.with_help(help);
    }
    diagnostic
}

fn undefined_intrinsic_diagnostic(
    path: &Path, source: &EntryParseError, registry: Option<&MetaRegistry>,
) -> Option<EntryFileDiagnostic> {
    let EntryParseError::FieldMustBeList(field) = source else {
        return None;
    };
    if field != NAME_FIELD && field != DESC_FIELD {
        return None;
    }
    if registry.is_some_and(|registry| registry.contains_intrinsic_field(field)) {
        return None;
    }
    Some(
        EntryFileDiagnostic::new(
            CheckSeverity::Error,
            path,
            format!(
                "metadata field `{field}` is scalar, but this lake does not define `{field}` as an intrinsic field"
            ),
        )
        .with_code("entry.metadata.intrinsic.undefined")
        .with_help(format!(
            "Create entry `{field}` with `meta.type: \"intrinsic\"`, or restore the seed entry."
        )),
    )
}
// sirno:witness:diagnostics:end

impl EntryDirectory {
    /// Construct an entry directory rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Directory containing Sirno Lake Markdown entry files.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Sirno Lake artifact directory path for one entry address.
    pub fn entry_artifact_root_path(&self, id: &EntryAddress) -> PathBuf {
        self.entry_artifact_directory(id)
    }

    /// Sirno Lake artifact file path for one entry-owned artifact.
    pub fn entry_artifact_path(&self, id: &EntryAddress, path: &EntryArtifactPath) -> PathBuf {
        self.entry_artifact_directory(id).join(path.to_path_buf())
    }

    /// Returns true when this directory contains the file for `id`.
    pub fn entry_exists(&self, id: &EntryAddress) -> Result<bool, EntryDirectoryError> {
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

    /// Read one Sirno Lake Markdown entry file source by path.
    pub fn read_entry_source(&self, id: &EntryAddress) -> Result<String, EntryDirectoryError> {
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

    /// Read every projected Markdown entry through an external metadata registry.
    ///
    /// This is used by mist projections.
    /// The projection is an interface over selected entries,
    /// while the reservoir supplies the lake-wide metadata vocabulary.
    pub fn read_entries_with_registry(
        &self, meta: &MetaRegistry, ignore: impl IntoIterator<Item = PathBuf>,
    ) -> Result<Vec<Entry>, EntryDirectoryError> {
        if !self.root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(self.root.clone()));
        }
        if !self.root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
        }

        let settings = EntryDirectoryCheckSettings {
            render: false,
            structural_render: StructuralRenderSettings::default(),
            meta_path: None,
            ignore: ignore.into_iter().collect(),
            witness: None,
        };
        let mut artifact_root = None;
        let mut diagnostics = Vec::new();
        let entry_paths = collect_entry_file_paths(
            &self.root,
            &self.root,
            &settings,
            CheckSeverity::Error,
            &mut artifact_root,
            &mut diagnostics,
        )?;
        if diagnostics.iter().any(|diagnostic| diagnostic.severity == CheckSeverity::Error) {
            return Err(EntryDirectoryError::InvalidEntryDirectory(self.root.clone()));
        }

        let mut raw_entries = Vec::new();
        for path in entry_paths {
            let relative_path =
                path.strip_prefix(&self.root).map_err(|source| EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: self.root.clone(),
                    source,
                })?;
            let id = EntryAddress::from_lake_relative_path(relative_path)?;
            let source = fs::read_to_string(&path)?;
            raw_entries.push(RawEntry::from_markdown(id, &source)?);
        }

        raw_entries.sort_by(|left, right| left.id.cmp(&right.id));
        let registry_scopes =
            EntryRegistryScopes::from_registry_and_raw_entries(meta, &raw_entries);
        let mut entries = Vec::new();
        for raw_entry in raw_entries {
            let registry = registry_scopes.registry_for(&raw_entry);
            let entry = raw_entry.into_entry(registry)?;
            GeneratedLinkBody::new(&entry.body).validate()?;
            entries.push(entry);
        }

        entries.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(entries)
    }

    /// Read one Sirno Lake Markdown entry file by path.
    pub fn read_entry(&self, id: &EntryAddress) -> Result<Entry, EntryDirectoryError> {
        let settings =
            EntryDirectoryCheckSettings { render: false, ..EntryDirectoryCheckSettings::default() };
        let report = self.check_with_settings(CheckMode::Edit, &settings)?;
        report
            .entries()
            .iter()
            .find(|entry| &entry.id == id)
            .cloned()
            .ok_or_else(|| EntryDirectoryError::EntryNotFound(id.clone()))
    }

    /// Replace one existing Sirno Lake Markdown entry source.
    ///
    /// The entry address controls the target path.
    /// Frozen entries cannot be changed through this low-level replacement.
    pub fn replace_entry_source(
        &self, id: &EntryAddress, source: &str,
    ) -> Result<PathBuf, EntryDirectoryError> {
        if !self.root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(self.root.clone()));
        }
        if !self.root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
        }

        let path = self.entry_file_path(id);
        let current = self.read_entry_source(id)?;
        if current == source {
            return Ok(path);
        }
        let raw_entry = RawEntry::from_markdown(id.clone(), &current)?;
        if raw_entry.entry_meta()?.frozen.is_some() {
            return Err(EntryDirectoryError::FrozenEntryProtected(id.clone()));
        }
        set_path_writable(&path)?;
        fs::write(&path, source)
            .map_err(|source| EntryDirectoryError::WriteFile { path: path.clone(), source })?;
        Ok(path)
    }

    /// Write one Sirno Lake Markdown entry source, creating it when absent.
    ///
    /// Existing frozen entries cannot be changed.
    pub fn write_entry_source(
        &self, id: &EntryAddress, source: &str,
    ) -> Result<PathBuf, EntryDirectoryError> {
        let _ = RawEntry::from_markdown(id.clone(), source)?;
        match self.entry_exists(id) {
            | Ok(true) => self.replace_entry_source(id, source),
            | Ok(false) => {
                let path = self.entry_file_path(id);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut file =
                    OpenOptions::new().write(true).create_new(true).open(&path).map_err(
                        |source| EntryDirectoryError::CreateFile { path: path.clone(), source },
                    )?;
                file.write_all(source.as_bytes()).map_err(|source| {
                    EntryDirectoryError::WriteFile { path: path.clone(), source }
                })?;
                Ok(path)
            }
            | Err(EntryDirectoryError::MissingDirectory(_)) => {
                fs::create_dir_all(&self.root)?;
                self.write_entry_source(id, source)
            }
            | Err(error) => Err(error),
        }
    }

    /// Read lake-owned artifacts for one entry address.
    // sirno:witness:entry-artifact:begin
    pub fn read_entry_artifacts(
        &self, id: &EntryAddress,
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
        &self, id: &EntryAddress, source: &Path, artifact_path: &EntryArtifactPath,
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
        &self, id: &EntryAddress, old_path: &EntryArtifactPath, new_path: &EntryArtifactPath,
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
        &self, id: &EntryAddress, artifact_path: &EntryArtifactPath,
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

    /// Check this Sirno Lake entry directory.
    pub fn check(&self, mode: CheckMode) -> Result<EntryDirectoryReport, EntryDirectoryError> {
        self.check_with_settings(mode, &EntryDirectoryCheckSettings::default())
    }

    /// Check this Sirno Lake entry directory with explicit settings.
    // sirno:witness:reservoir:begin
    pub fn check_with_settings(
        &self, mode: CheckMode, settings: &EntryDirectoryCheckSettings,
    ) -> Result<EntryDirectoryReport, EntryDirectoryError> {
        trace!("check_entry_directory begin: root={}", self.root.display());
        let loaded = LoadedEntryDirectory::load(&self.root, mode, settings)?;
        let structural_report = mode.check_entries(&loaded.entries, &loaded.meta);
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
            paths_by_address: loaded.paths_by_address,
            file_diagnostics: loaded.file_diagnostics,
            structural_report,
            meta: loaded.meta,
            structural: loaded.structural,
        })
    }
    // sirno:witness:reservoir:end

    /// Initialize this directory with ordinary seed entries.
    ///
    /// Existing entry files are never overwritten.
    // sirno:witness:reservoir:begin
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
    // sirno:witness:reservoir:end

    /// Create one Sirno Lake Markdown entry file in this directory.
    ///
    /// The entry directory is created if needed.
    /// Existing entry files are never overwritten.
    // sirno:witness:reservoir:begin
    pub fn create_entry(&self, entry: &Entry) -> Result<PathBuf, EntryDirectoryError> {
        trace!("create_entry_file begin: root={} id={}", self.root.display(), entry.id);
        fs::create_dir_all(&self.root)?;
        let path = self.write_new_entry_file(entry)?;
        trace!("create_entry_file end: path={}", path.display());
        Ok(path)
    }
    // sirno:witness:reservoir:end

    /// Mark one Sirno Lake Markdown entry as frozen and read-only.
    ///
    /// The entry metadata gains the canonical `reviewed` frozen reason.
    /// Local file protection is applied after the reason is written.
    pub fn freeze_entry(&self, id: &EntryAddress) -> Result<PathBuf, EntryDirectoryError> {
        self.set_entry_frozen(id, true)
    }

    /// Mark one Sirno Lake Markdown entry as melted and writable.
    ///
    /// The canonical `reviewed` frozen reason is removed from entry metadata.
    /// The file is left writable so normal editing can resume.
    pub fn melt_entry(&self, id: &EntryAddress) -> Result<PathBuf, EntryDirectoryError> {
        self.set_entry_frozen(id, false)
    }

    /// Rename one entry address and every structural link reference that names it.
    ///
    /// Existing generated-link regions are refreshed after metadata changes.
    /// Prose outside generated-link regions remains user-owned.
    pub fn rename_entry(
        &self, old_id: &EntryAddress, new_id: &EntryAddress, settings: &EntryDirectoryCheckSettings,
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

        if checked.entry_file_path(old_id).is_none() {
            return Err(EntryDirectoryError::EntryNotFound(old_id.clone()));
        }
        if checked
            .entries()
            .iter()
            .any(|entry| &entry.id == old_id && entry.metadata.meta.frozen.is_some())
        {
            return Err(EntryDirectoryError::FrozenEntryProtected(old_id.clone()));
        }
        let new_path = self.entry_file_path(new_id);
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)?;
        }
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

        let renames_structural_relation = checked
            .entries()
            .iter()
            .any(|entry| &entry.id == old_id && entry.metadata.meta.is_structural_relation());
        let mut entries = Vec::<(EntryAddress, Entry, bool)>::new();
        for entry in checked.entries() {
            let original_id = entry.id.clone();
            let mut entry = entry.clone();
            if &entry.id == old_id {
                entry.id = new_id.clone();
            }
            let mut content_changed = entry.metadata.rename_structural_target(old_id, new_id);
            if renames_structural_relation {
                content_changed |= entry.metadata.rename_structural_field(old_id, new_id);
            }
            entries.push((original_id, entry, content_changed));
        }

        let indexed_entries = entries.iter().map(|(_, entry, _)| entry.clone()).collect::<Vec<_>>();
        let structural = StructuralSettings::from_entries(&indexed_entries)
            .with_render_settings(&settings.structural_render);
        let link_index = StructuralEdgeIndex::from_entries(&indexed_entries);
        let mut changed_paths = Vec::new();

        for (original_id, mut entry, mut content_changed) in entries {
            if content_changed && entry.metadata.meta.frozen.is_some() {
                return Err(EntryDirectoryError::FrozenEntryProtected(original_id));
            }
            let source_path = checked
                .entry_file_path(&original_id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryFilePath(original_id.clone()))?;
            let destination_path =
                if &original_id == old_id { new_path.as_path() } else { source_path };
            let footer = link_index.render_entry(&entry, &structural);
            let body = GeneratedLinkBody::new(&entry.body);
            if body.is_stale(&footer)? {
                entry.body = body.apply(&footer)?;
                content_changed = true;
            }
            if content_changed && entry.metadata.meta.frozen.is_some() {
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
                if entry.metadata.meta.frozen.is_some() {
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
                if entry.metadata.meta.frozen.is_some() {
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

    /// Write a complete Sirno Lake entry directory.
    ///
    /// The write policy controls how existing target contents are handled.
    // sirno:witness:reservoir:begin
    pub fn write(
        &self, entries: &[Entry], policy: EntryDirectoryWritePolicy,
    ) -> Result<Vec<PathBuf>, EntryDirectoryError> {
        self.write_with_artifacts(entries, &[], policy)
    }

    /// Write a complete Sirno Lake entry directory with lake-owned artifacts.
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
    // sirno:witness:reservoir:end

    /// Replace one glacier.
    ///
    /// Existing files under the glacier domain must already be managed by crystallization.
    // sirno:witness:glacier:begin
    pub fn replace_glacier(
        &self, domain: &EntryAtom, entries: &[Entry], artifacts: &[EntryArtifact],
        settings: &EntryDirectoryCheckSettings,
    ) -> Result<GlacierReport, EntryDirectoryError> {
        trace!("replace_glacier begin: root={} domain={}", self.root.display(), domain);
        fs::create_dir_all(&self.root)?;
        for entry in entries {
            if !entry.id.starts_with_domain(domain) {
                return Err(EntryDirectoryError::GlacierEntryOutsideDomain {
                    domain: domain.clone(),
                    id: entry.id.clone(),
                });
            }
            if !entry.metadata.meta.frozen.as_ref().is_some_and(|marker| marker.is_managed()) {
                return Err(EntryDirectoryError::GlacierEntryNotManaged(entry.id.clone()));
            }
        }

        self.ensure_glacier_replaceable(domain, settings)?;

        let mut changed_paths = Vec::new();
        changed_paths.extend(self.remove_glacier_entries(domain, settings)?);
        changed_paths.extend(self.remove_glacier_artifacts(domain)?);
        for entry in entries {
            changed_paths.push(self.write_new_entry_file(entry)?);
        }
        changed_paths.extend(self.write_entry_artifacts(entries, artifacts)?);

        changed_paths.sort();
        changed_paths.dedup();
        trace!("replace_glacier end: domain={} changed={}", domain, changed_paths.len());
        Ok(GlacierReport { root: self.root.clone(), domain: domain.clone(), changed_paths })
    }
    // sirno:witness:glacier:end

    /// Require all existing paths in a glacier domain to be crystallization-managed.
    pub fn ensure_glacier_replaceable(
        &self, domain: &EntryAtom, settings: &EntryDirectoryCheckSettings,
    ) -> Result<(), EntryDirectoryError> {
        self.ensure_glacier_entries_replaceable(domain, settings)?;
        self.ensure_glacier_artifacts_replaceable(domain)?;
        Ok(())
    }

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

    /// Clear every Sirno local protection guard in this directory.
    ///
    /// Frozen metadata and checkout lock state remain unchanged.
    /// Ignored paths are left untouched.
    pub fn clear_local_protection(
        &self, settings: &EntryDirectoryCheckSettings, dry_run: bool,
    ) -> Result<EntryProtectionReport, EntryDirectoryError> {
        self.require_existing_directory()?;
        let paths = self.local_protection_paths(settings)?;
        if !dry_run {
            for path in &paths {
                melt_path_best_effort(path)?;
            }
        }
        Ok(EntryProtectionReport { root: self.root.clone(), paths })
    }

    /// Reapply local protection from frozen metadata and checkout state.
    ///
    /// `protect_checkout` selects the whole lake for an immutable managed checkout.
    /// Otherwise, only entries carrying `frozen` reasons and their artifact trees are protected.
    /// Ignored paths are left untouched.
    pub fn fix_local_protection(
        &self, settings: &EntryDirectoryCheckSettings, protect_checkout: bool, dry_run: bool,
    ) -> Result<EntryProtectionReport, EntryDirectoryError> {
        self.require_existing_directory()?;
        let paths = if protect_checkout {
            self.local_protection_paths(settings)?
        } else {
            self.frozen_entry_protection_paths(settings)?
        };
        if !dry_run {
            if protect_checkout {
                self.set_readonly(settings)?;
            } else {
                self.fix_frozen_entry_protection(settings)?;
            }
        }
        Ok(EntryProtectionReport { root: self.root.clone(), paths })
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

    /// Generate Markdown link footers for this Sirno Lake entry directory.
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

    /// Generate Markdown link footers using directory check settings.
    pub fn generate_links_with_check_settings(
        &self, settings: &EntryDirectoryCheckSettings,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        self.process_generated_links_with_check_settings(settings, GenLinkOperation::Write)
    }

    /// Generate Markdown link footers while allowing managed glacier entries to change.
    pub fn generate_links_for_crystallization(
        &self, settings: &EntryDirectoryCheckSettings,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        self.process_generated_links_with_check_settings(settings, GenLinkOperation::WriteManaged)
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

    /// Check generated Markdown link footers using directory check settings.
    ///
    /// No file is written.
    pub fn check_generated_links_with_check_settings(
        &self, settings: &EntryDirectoryCheckSettings,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        self.process_generated_links_with_check_settings(settings, GenLinkOperation::Check)
    }

    fn process_generated_links(
        &self, settings: &StructuralSettings, ignore: impl IntoIterator<Item = PathBuf>,
        operation: GenLinkOperation,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        let check_settings = EntryDirectoryCheckSettings {
            render: false,
            structural_render: StructuralRenderSettings::default(),
            meta_path: None,
            ignore: ignore.into_iter().collect(),
            witness: None,
        };
        self.process_generated_links_inner(&check_settings, settings, operation)
    }

    fn process_generated_links_with_check_settings(
        &self, settings: &EntryDirectoryCheckSettings, operation: GenLinkOperation,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        let check_settings = EntryDirectoryCheckSettings { render: false, ..settings.clone() };
        let checked = self.check_with_settings(CheckMode::Review, &check_settings)?;
        let structural = checked.structural().clone();
        self.process_checked_generated_links(checked, &structural, operation)
    }

    fn process_generated_links_inner(
        &self, check_settings: &EntryDirectoryCheckSettings, structural: &StructuralSettings,
        operation: GenLinkOperation,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        trace!(
            "gen_link_entry_directory begin: root={} operation={}",
            self.root.display(),
            operation.label()
        );
        let checked = self.check_with_settings(CheckMode::Review, check_settings)?;
        self.process_checked_generated_links(checked, structural, operation)
    }

    fn process_checked_generated_links(
        &self, checked: EntryDirectoryReport, structural: &StructuralSettings,
        operation: GenLinkOperation,
    ) -> Result<GenLinkDirectoryReport, EntryDirectoryError> {
        if checked.has_errors() {
            return Err(EntryDirectoryError::InvalidEntryDirectory(self.root.clone()));
        }

        let mut changed_paths = Vec::new();
        let index = StructuralEdgeIndex::from_entries(checked.entries());
        for entry in checked.entries() {
            let path = checked
                .entry_file_path(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryFilePath(entry.id.clone()))?;
            let source = fs::read_to_string(path)?;
            let footer = index.render_entry(entry, structural);
            let body = GeneratedLinkBody::new(&entry.body).apply(&footer)?;
            let rendered = Entry::replace_markdown_body(&source, &body)?;
            if rendered != source {
                if operation.writes() {
                    if !operation.allows_entry_write(entry) {
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
            changed_entry_count: changed_paths.len(),
            changed_paths,
        })
    }

    /// Delete generated Markdown link footers from this Sirno Lake entry directory.
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
            structural_render: StructuralRenderSettings::default(),
            meta_path: None,
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
                .entry_file_path(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryFilePath(entry.id.clone()))?;
            let source = fs::read_to_string(path)?;
            let body = GeneratedLinkBody::new(&entry.body).delete()?;
            let rendered = Entry::replace_markdown_body(&source, &body)?;
            if rendered != source {
                if entry.metadata.meta.frozen.is_some() {
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
            changed_entry_count: changed_paths.len(),
            changed_paths,
        })
    }

    fn write_new_entry_file(&self, entry: &Entry) -> Result<PathBuf, EntryDirectoryError> {
        let path = self.entry_file_path(&entry.id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
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

        let entry_addresses = entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        let mut seen = BTreeSet::<(EntryAddress, EntryArtifactPath)>::new();
        let mut paths = Vec::new();
        for artifact in artifacts {
            if !entry_addresses.contains(&artifact.owner) {
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

    fn set_entry_frozen(
        &self, id: &EntryAddress, frozen: bool,
    ) -> Result<PathBuf, EntryDirectoryError> {
        if !self.root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(self.root.clone()));
        }
        if !self.root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
        }

        let path = self.entry_file_path(id);
        let source = fs::read_to_string(&path)?;
        let mut entry = self.read_entry(id)?;
        if frozen {
            match &mut entry.metadata.meta.frozen {
                | Some(marker) => marker.insert_reviewed(),
                | None => entry.metadata.meta.frozen = Some(FrozenMarker::reviewed()),
            }
        } else if let Some(marker) = &mut entry.metadata.meta.frozen
            && !marker.remove_reviewed()
        {
            entry.metadata.meta.frozen = None;
        }
        let still_frozen = entry.metadata.meta.frozen.is_some();
        let rendered = entry.to_markdown()?;
        if !frozen {
            melt_path_best_effort(&path)?;
        }
        if rendered != source {
            set_path_writable(&path)?;
            fs::write(&path, rendered)
                .map_err(|source| EntryDirectoryError::WriteFile { path: path.clone(), source })?;
        }

        if frozen || still_frozen {
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
                        meta_path: None,
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
        self.remove_managed_entry_files_in(&self.root, settings)
    }

    fn remove_glacier_entries(
        &self, domain: &EntryAtom, settings: &EntryDirectoryCheckSettings,
    ) -> Result<Vec<PathBuf>, EntryDirectoryError> {
        let domain_root = self.root.join(domain.as_str());
        if !domain_root.exists() {
            return Ok(Vec::new());
        }
        if !domain_root.is_dir() {
            return Err(EntryDirectoryError::CheckoutConflict(domain_root));
        }

        let mut changed = Vec::new();
        for path in sorted_recursive_paths(&domain_root)?.into_iter().rev() {
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_dir() {
                if fs::read_dir(&path)?.next().is_none() {
                    fs::remove_dir(&path)?;
                    changed.push(path);
                }
                continue;
            }
            if settings.ignores(path.strip_prefix(&self.root).map_err(|source| {
                EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: self.root.clone(),
                    source,
                }
            })?) {
                continue;
            }
            if !metadata.file_type().is_file()
                || path.extension().and_then(|extension| extension.to_str()) != Some("md")
            {
                return Err(EntryDirectoryError::CheckoutConflict(path));
            }
            if !Self::is_managed_frozen_entry_file(&path)? {
                return Err(EntryDirectoryError::UnmanagedGlacierPath(path));
            }
            melt_path_best_effort(&path)?;
            fs::remove_file(&path)?;
            changed.push(path);
        }
        if fs::read_dir(&domain_root)?.next().is_none() {
            fs::remove_dir(&domain_root)?;
            changed.push(domain_root);
        }
        Ok(changed)
    }

    fn ensure_glacier_entries_replaceable(
        &self, domain: &EntryAtom, settings: &EntryDirectoryCheckSettings,
    ) -> Result<(), EntryDirectoryError> {
        let domain_root = self.root.join(domain.as_str());
        if !domain_root.exists() {
            return Ok(());
        }
        if !domain_root.is_dir() {
            return Err(EntryDirectoryError::CheckoutConflict(domain_root));
        }

        for path in sorted_recursive_paths(&domain_root)? {
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_dir() {
                continue;
            }
            if settings.ignores(path.strip_prefix(&self.root).map_err(|source| {
                EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: self.root.clone(),
                    source,
                }
            })?) {
                continue;
            }
            if !metadata.file_type().is_file()
                || path.extension().and_then(|extension| extension.to_str()) != Some("md")
            {
                return Err(EntryDirectoryError::CheckoutConflict(path));
            }
            if !Self::is_managed_frozen_entry_file(&path)? {
                return Err(EntryDirectoryError::UnmanagedGlacierPath(path));
            }
        }
        Ok(())
    }

    fn remove_glacier_artifacts(
        &self, domain: &EntryAtom,
    ) -> Result<Vec<PathBuf>, EntryDirectoryError> {
        let artifact_root = self.artifact_root();
        if !artifact_root.exists() {
            return Ok(Vec::new());
        }
        if !artifact_root.is_dir() {
            return Err(EntryDirectoryError::CheckoutConflict(artifact_root));
        }

        let mut changed = Vec::new();
        for owner_root in sorted_directory_paths(&artifact_root)? {
            let Some(owner_name) = owner_root.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Ok(owner) = EntryAddress::new(owner_name) else {
                continue;
            };
            if !owner.starts_with_domain(domain) {
                continue;
            }
            let owner_entry = self.entry_file_path(&owner);
            if owner_entry.exists() && !Self::is_managed_frozen_entry_file(&owner_entry)? {
                return Err(EntryDirectoryError::UnmanagedGlacierPath(owner_root));
            }
            melt_tree_best_effort(&owner_root)?;
            fs::remove_dir_all(&owner_root)?;
            changed.push(owner_root);
        }
        Ok(changed)
    }

    fn ensure_glacier_artifacts_replaceable(
        &self, domain: &EntryAtom,
    ) -> Result<(), EntryDirectoryError> {
        let artifact_root = self.artifact_root();
        if !artifact_root.exists() {
            return Ok(());
        }
        if !artifact_root.is_dir() {
            return Err(EntryDirectoryError::CheckoutConflict(artifact_root));
        }

        for owner_root in sorted_directory_paths(&artifact_root)? {
            let Some(owner_name) = owner_root.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Ok(owner) = EntryAddress::new(owner_name) else {
                continue;
            };
            if !owner.starts_with_domain(domain) {
                continue;
            }
            let owner_entry = self.entry_file_path(&owner);
            if owner_entry.exists() && !Self::is_managed_frozen_entry_file(&owner_entry)? {
                return Err(EntryDirectoryError::UnmanagedGlacierPath(owner_root));
            }
        }
        Ok(())
    }

    fn remove_managed_entry_files_in(
        &self, directory: &Path, settings: &EntryDirectoryCheckSettings,
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
            if relative_path == Path::new(ARTIFACT_DIRECTORY_NAME) && file_type.is_dir() {
                melt_tree_best_effort(&path)?;
                fs::remove_dir_all(&path)?;
                continue;
            }
            if is_reserved_builtin_root(relative_path) {
                return Err(EntryDirectoryError::CheckoutConflict(path));
            }
            if file_type.is_dir() {
                self.remove_managed_entry_files_in(&path, settings)?;
                if fs::read_dir(&path)?.next().is_none() {
                    fs::remove_dir(&path)?;
                }
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
        let Ok(id) = EntryAddress::new(stem) else {
            return Ok(false);
        };
        let source = fs::read_to_string(path)?;
        Ok(RawEntry::from_markdown(id, &source).is_ok())
    }

    fn is_frozen_entry_file(path: &Path) -> Result<bool, EntryDirectoryError> {
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            return Ok(false);
        };
        let Ok(id) = EntryAddress::new(stem) else {
            return Ok(false);
        };
        let source = fs::read_to_string(path)?;
        Ok(RawEntry::from_markdown(id, &source)
            .and_then(|entry| entry.entry_meta())
            .map(|meta| meta.frozen.is_some())
            .unwrap_or(false))
    }

    fn is_managed_frozen_entry_file(path: &Path) -> Result<bool, EntryDirectoryError> {
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            return Ok(false);
        };
        let Ok(id) = EntryAddress::new(stem) else {
            return Ok(false);
        };
        let source = fs::read_to_string(path)?;
        Ok(RawEntry::from_markdown(id, &source)
            .and_then(|entry| entry.entry_meta())
            .map(|meta| meta.frozen.as_ref().is_some_and(|marker| marker.is_managed()))
            .unwrap_or(false))
    }

    fn require_existing_directory(&self) -> Result<(), EntryDirectoryError> {
        if !self.root.exists() {
            return Err(EntryDirectoryError::MissingDirectory(self.root.clone()));
        }
        if !self.root.is_dir() {
            return Err(EntryDirectoryError::NotDirectory(self.root.clone()));
        }
        Ok(())
    }

    fn local_protection_paths(
        &self, settings: &EntryDirectoryCheckSettings,
    ) -> Result<Vec<PathBuf>, EntryDirectoryError> {
        let mut paths = vec![self.root.clone()];
        for path in sorted_recursive_paths(&self.root)? {
            let relative_path =
                path.strip_prefix(&self.root).map_err(|source| EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: self.root.clone(),
                    source,
                })?;
            if settings.ignores(relative_path) {
                continue;
            }
            paths.push(path);
        }
        paths.sort();
        paths.dedup();
        Ok(paths)
    }

    fn frozen_entry_protection_paths(
        &self, settings: &EntryDirectoryCheckSettings,
    ) -> Result<Vec<PathBuf>, EntryDirectoryError> {
        let checked = self.check_with_settings(CheckMode::Edit, settings)?;
        if checked.has_errors() {
            return Err(EntryDirectoryError::InvalidEntryDirectory(self.root.clone()));
        }

        let mut paths = Vec::new();
        for entry in checked.entries().iter().filter(|entry| entry.metadata.meta.frozen.is_some()) {
            let path = checked
                .entry_file_path(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryFilePath(entry.id.clone()))?;
            paths.push(path.to_path_buf());
            self.push_entry_artifact_tree_paths(&entry.id, &mut paths)?;
        }
        paths.sort();
        paths.dedup();
        Ok(paths)
    }

    fn push_entry_artifact_tree_paths(
        &self, id: &EntryAddress, paths: &mut Vec<PathBuf>,
    ) -> Result<(), EntryDirectoryError> {
        let owner_root = self.entry_artifact_directory(id);
        if !owner_root.exists() {
            return Ok(());
        }
        if !owner_root.is_dir() {
            return Err(EntryDirectoryError::CheckoutConflict(owner_root));
        }

        paths.push(owner_root.clone());
        paths.extend(sorted_recursive_paths(&owner_root)?);
        Ok(())
    }

    fn fix_frozen_entry_protection(
        &self, settings: &EntryDirectoryCheckSettings,
    ) -> Result<(), EntryDirectoryError> {
        let checked = self.check_with_settings(CheckMode::Edit, settings)?;
        if checked.has_errors() {
            return Err(EntryDirectoryError::InvalidEntryDirectory(self.root.clone()));
        }

        for entry in checked.entries().iter().filter(|entry| entry.metadata.meta.frozen.is_some()) {
            let path = checked
                .entry_file_path(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryFilePath(entry.id.clone()))?;
            freeze_path_best_effort(path)?;
            self.set_entry_artifact_writability(&entry.id, false)?;
        }
        Ok(())
    }

    fn set_writability(
        &self, settings: &EntryDirectoryCheckSettings, writable: bool,
    ) -> Result<(), EntryDirectoryError> {
        self.require_existing_directory()?;

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

    pub fn entry_file_path(&self, id: &EntryAddress) -> PathBuf {
        self.root.join(id.to_lake_relative_path())
    }

    fn artifact_root(&self) -> PathBuf {
        self.root.join(ARTIFACT_DIRECTORY_NAME)
    }

    fn entry_artifact_directory(&self, id: &EntryAddress) -> PathBuf {
        self.artifact_root().join(id.as_str())
    }

    fn is_entry_file_path(&self, path: &Path) -> bool {
        path.starts_with(&self.root)
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
            let Ok(owner) = EntryAddress::new(owner) else {
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
        &self, id: &EntryAddress, writable: bool,
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

    fn ensure_entry_artifacts_mutable(&self, id: &EntryAddress) -> Result<(), EntryDirectoryError> {
        let entry = self.read_entry(id)?;
        if entry.metadata.meta.frozen.is_some() {
            return Err(EntryDirectoryError::FrozenEntryProtected(id.clone()));
        }
        Ok(())
    }

    fn remove_empty_artifact_parents(
        &self, id: &EntryAddress, artifact_path: &EntryArtifactPath,
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
    WriteManaged,
}

impl GenLinkOperation {
    fn label(self) -> &'static str {
        match self {
            | Self::Check => "check",
            | Self::Write => "write",
            | Self::WriteManaged => "write-managed",
        }
    }

    fn writes(self) -> bool {
        matches!(self, Self::Write | Self::WriteManaged)
    }

    fn allows_entry_write(self, entry: &Entry) -> bool {
        match &entry.metadata.meta.frozen {
            | None => true,
            | Some(marker) => self == Self::WriteManaged && marker.is_managed(),
        }
    }
}

#[derive(Debug)]
struct LoadedEntryDirectory {
    entries: Vec<Entry>,
    artifacts: Vec<EntryArtifact>,
    paths_by_address: BTreeMap<EntryAddress, PathBuf>,
    file_diagnostics: Vec<EntryFileDiagnostic>,
    meta: MetaRegistry,
    structural: StructuralSettings,
}

#[derive(Debug)]
struct EntryRegistryScopes {
    local: MetaRegistry,
    managed_by_domain: BTreeMap<EntryAtom, MetaRegistry>,
}

impl EntryRegistryScopes {
    fn from_raw_entries(raw_entries: &[RawEntry]) -> Self {
        let meta = MetaRegistry::from_raw_entries(raw_entries);
        Self::from_registry_and_raw_entries(&meta, raw_entries)
    }

    fn from_registry_and_raw_entries(meta: &MetaRegistry, raw_entries: &[RawEntry]) -> Self {
        let mut domains = BTreeSet::<EntryAtom>::new();
        for entry in raw_entries {
            if let Some(domain) = managed_glacier_domain(entry) {
                domains.insert(domain);
            }
        }
        let local = meta.without_domains(domains.iter());
        let managed_by_domain = domains
            .into_iter()
            .map(|domain| {
                let registry = meta.only_domain(&domain);
                (domain, registry)
            })
            .collect();
        Self { local, managed_by_domain }
    }

    fn registry_for(&self, entry: &RawEntry) -> &MetaRegistry {
        let Some(domain) = managed_glacier_domain(entry) else {
            return &self.local;
        };
        self.managed_by_domain
            .get(&domain)
            .expect("managed glacier domain registry was built from the same raw entries")
    }
}

fn managed_glacier_domain(entry: &RawEntry) -> Option<EntryAtom> {
    let meta = entry.entry_meta().ok()?;
    if !meta.frozen.as_ref().is_some_and(|marker| marker.is_managed()) {
        return None;
    }
    let domain = entry.id.as_str().split_once('.')?.0;
    Some(EntryAtom::new(domain).expect("entry address segment is a valid atom"))
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
        let mut raw_entries = Vec::new();
        let mut raw_paths_by_address = BTreeMap::<EntryAddress, PathBuf>::new();
        let mut paths_by_address = BTreeMap::<EntryAddress, PathBuf>::new();
        let mut seen_ids = BTreeSet::<EntryAddress>::new();
        let mut file_diagnostics = Vec::new();
        let mut artifact_root = None;

        let entry_paths = collect_entry_file_paths(
            root,
            root,
            settings,
            non_entry_severity,
            &mut artifact_root,
            &mut file_diagnostics,
        )?;

        for path in entry_paths {
            let relative_path =
                path.strip_prefix(root).map_err(|source| EntryDirectoryError::StripRoot {
                    path: path.clone(),
                    root: root.to_path_buf(),
                    source,
                })?;
            let id = match EntryAddress::from_lake_relative_path(relative_path) {
                | Ok(id) => id,
                | Err(source) => {
                    file_diagnostics.push(
                        EntryFileDiagnostic::new(
                            CheckSeverity::Error,
                            &path,
                            format!("entry file path is not a valid entry address: {source}"),
                        )
                        .with_code("lake.entry.path.invalid")
                        .with_help("Rename the file to a valid lake-relative Markdown entry path."),
                    );
                    continue;
                }
            };

            if seen_ids.contains(&id) {
                let first_path = raw_paths_by_address
                    .get(&id)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<unknown>".to_owned());
                file_diagnostics.push(
                    EntryFileDiagnostic::new(
                        CheckSeverity::Error,
                        &path,
                        format!("entry address `{id}` also appears at {first_path}"),
                    )
                    .with_code("lake.entry.duplicate")
                    .with_help("Keep one Markdown file for each entry address."),
                );
                continue;
            }

            let source = fs::read_to_string(&path)?;
            if has_mixed_line_endings(&source) {
                file_diagnostics.push(
                    EntryFileDiagnostic::new(
                        CheckSeverity::Warning,
                        &path,
                        "entry file uses mixed LF and CRLF line endings",
                    )
                    .with_code("lake.entry.line-ending.mixed")
                    .with_help("Rewrite the file with one line-ending style."),
                );
            }
            let raw_entry = match RawEntry::from_markdown(id.clone(), &source) {
                | Ok(entry) => entry,
                | Err(source) => {
                    file_diagnostics.push(entry_parse_diagnostic(&path, &source, None));
                    continue;
                }
            };
            seen_ids.insert(id.clone());
            raw_paths_by_address.insert(id, path.clone());
            raw_entries.push(raw_entry);
        }

        raw_entries.sort_by(|left, right| left.id.cmp(&right.id));
        let meta = MetaRegistry::from_raw_entries(&raw_entries);
        if let Some(path) = &settings.meta_path {
            meta.write(path)?;
        }
        let registry_scopes = EntryRegistryScopes::from_raw_entries(&raw_entries);
        let mut entries = Vec::new();
        for raw_entry in raw_entries {
            let id = raw_entry.id.clone();
            let path = raw_paths_by_address
                .get(&id)
                .expect("raw entry path was recorded before typed parsing")
                .clone();
            let registry = registry_scopes.registry_for(&raw_entry);
            let entry = match raw_entry.into_entry(registry) {
                | Ok(entry) => entry,
                | Err(source) => {
                    file_diagnostics.push(entry_parse_diagnostic(&path, &source, Some(registry)));
                    continue;
                }
            };
            paths_by_address.insert(id, path);
            entries.push(entry);
        }
        entries.sort_by(|left, right| left.id.cmp(&right.id));
        let structural = meta.structural().with_render_settings(&settings.structural_render);
        let mut loaded = Self {
            entries,
            artifacts: Vec::new(),
            paths_by_address,
            file_diagnostics,
            meta,
            structural,
        };
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
            self.file_diagnostics.push(
                EntryFileDiagnostic::new(
                    severity,
                    artifact_root,
                    "entry artifact storage must be a directory",
                )
                .with_code("lake.artifact.root.not-directory"),
            );
            return Ok(());
        }

        let ids = self.entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        for owner_path in sorted_directory_paths(artifact_root)? {
            let owner_type = fs::symlink_metadata(&owner_path)?.file_type();
            if !owner_type.is_dir() {
                self.file_diagnostics.push(
                    EntryFileDiagnostic::new(
                        severity,
                        &owner_path,
                        "entry artifact storage contains an unsupported filesystem item",
                    )
                    .with_code("lake.artifact.owner.unsupported"),
                );
                continue;
            }

            let Some(owner_name) = owner_path.file_name().and_then(|name| name.to_str()) else {
                self.file_diagnostics.push(
                    EntryFileDiagnostic::new(
                        CheckSeverity::Error,
                        &owner_path,
                        "entry artifact directory name must be valid UTF-8",
                    )
                    .with_code("lake.artifact.owner.utf8"),
                );
                continue;
            };
            let owner = match EntryAddress::new(owner_name) {
                | Ok(owner) => owner,
                | Err(source) => {
                    self.file_diagnostics.push(EntryFileDiagnostic::new(
                        CheckSeverity::Error,
                        &owner_path,
                        format!(
                            "entry artifact directory name is not a valid entry address: {source}"
                        ),
                    )
                    .with_code("lake.artifact.owner.invalid"));
                    continue;
                }
            };
            if !ids.contains(&owner) {
                self.file_diagnostics.push(
                    EntryFileDiagnostic::new(
                        severity,
                        &owner_path,
                        format!("entry artifact directory references missing entry `{owner}`"),
                    )
                    .with_code("lake.artifact.owner.missing")
                    .with_help(format!(
                        "Create entry `{owner}` or remove this artifact directory."
                    )),
                );
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
        &mut self, root: &Path, owner_root: &Path, owner: &EntryAddress, severity: CheckSeverity,
    ) -> Result<(), EntryDirectoryError> {
        for path in sorted_recursive_paths(owner_root)? {
            let file_type = fs::symlink_metadata(&path)?.file_type();
            if file_type.is_dir() {
                continue;
            }
            if !file_type.is_file() {
                self.file_diagnostics.push(
                    EntryFileDiagnostic::new(
                        severity,
                        &path,
                        "entry artifact tree contains an unsupported filesystem item",
                    )
                    .with_code("lake.artifact.path.unsupported"),
                );
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
                    self.file_diagnostics.push(
                        EntryFileDiagnostic::new(
                            CheckSeverity::Error,
                            &path,
                            format!("invalid entry artifact path: {source}"),
                        )
                        .with_code("lake.artifact.path.invalid"),
                    );
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
                .paths_by_address
                .get(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryFilePath(entry.id.clone()))?;
            let body = GeneratedLinkBody::new(&entry.body);
            match body.validate() {
                | Ok(()) if settings.render => {
                    let expected = index.render_entry(entry, &self.structural);
                    if body.is_stale(&expected)? {
                        self.file_diagnostics.push(
                            EntryFileDiagnostic::new(
                                mode.severity(),
                                path,
                                "generated links are stale; run `sirno mist render`",
                            )
                            .with_code("lake.generated-links.stale")
                            .with_help("Run `sirno mist render` to refresh generated links."),
                        );
                    }
                }
                | Ok(()) => {}
                | Err(source) => {
                    self.file_diagnostics.push(
                        EntryFileDiagnostic::new(
                            CheckSeverity::Error,
                            path,
                            format!("malformed generated links: {source}"),
                        )
                        .with_code("lake.generated-links.malformed")
                        .with_help("Repair the generated-link guard block before rendering."),
                    );
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

        for witness_path in index.entry_addresses() {
            if ids.contains(witness_path) {
                continue;
            }
            for record in index.records_for(witness_path) {
                self.file_diagnostics.push(
                    EntryFileDiagnostic::new(
                        severity,
                        &record.path,
                        format!(
                            "repository witness block references missing entry `{witness_path}`"
                        ),
                    )
                    .with_code("lake.witness.entry.missing")
                    .with_position(record.opening.start_line, record.opening.start_column)
                    .with_help(format!(
                        "Create entry `{witness_path}` or update the witness marker."
                    )),
                );
            }
        }
        // sirno:witness:structural-check:begin
        for delimiter in index.orphan_delimiters() {
            self.file_diagnostics.push(
                EntryFileDiagnostic::new(
                    severity,
                    delimiter.path(),
                    delimiter.diagnostic_message(),
                )
                .with_code("lake.witness.delimiter.orphan"),
            );
        }
        // sirno:witness:structural-check:end

        Ok(())
    }
}

// sirno:witness:local-lakelet:begin
fn collect_entry_file_paths(
    root: &Path, directory: &Path, settings: &EntryDirectoryCheckSettings, severity: CheckSeverity,
    artifact_root: &mut Option<PathBuf>, diagnostics: &mut Vec<EntryFileDiagnostic>,
) -> Result<Vec<PathBuf>, EntryDirectoryError> {
    let mut entries = Vec::new();
    for path in sorted_directory_paths(directory)? {
        let relative_path = path.strip_prefix(root).map_err(|source| {
            EntryDirectoryError::StripRoot { path: path.clone(), root: root.to_path_buf(), source }
        })?;
        if settings.ignores(relative_path) {
            continue;
        }

        let file_type = fs::symlink_metadata(&path)?.file_type();
        if relative_path == Path::new(ARTIFACT_DIRECTORY_NAME) {
            *artifact_root = Some(path);
            continue;
        }
        if is_reserved_builtin_root(relative_path) {
            diagnostics.push(
                EntryFileDiagnostic::new(
                    severity,
                    &path,
                    "entry directory contains reserved built-in path",
                )
                .with_code("lake.path.reserved"),
            );
            continue;
        }
        if file_type.is_dir() {
            entries.extend(collect_entry_file_paths(
                root,
                &path,
                settings,
                severity,
                artifact_root,
                diagnostics,
            )?);
            continue;
        }
        if !file_type.is_file() {
            diagnostics.push(
                EntryFileDiagnostic::new(
                    severity,
                    &path,
                    "entry directory contains unsupported filesystem item",
                )
                .with_code("lake.path.unsupported"),
            );
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            diagnostics.push(
                EntryFileDiagnostic::new(
                    severity,
                    &path,
                    "entry directory contains non-Markdown file",
                )
                .with_code("lake.path.non-markdown"),
            );
            continue;
        }
        entries.push(path);
    }
    Ok(entries)
}
// sirno:witness:local-lakelet:end

fn is_reserved_builtin_root(relative_path: &Path) -> bool {
    relative_path.components().next().is_some_and(|component| match component {
        | Component::Normal(name) => name.to_str().is_some_and(|name| name.starts_with('.')),
        | _ => false,
    })
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
    let id = EntryAddress::new(stem)
        .map_err(|_| EntryDirectoryError::CheckoutConflict(path.to_path_buf()))?;
    let entry = RawEntry::from_markdown(id, source)?;
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
    #[error("entry address is not a directory: {0}")]
    NotDirectory(PathBuf),
    /// The target entry directory must be empty for this write policy.
    #[error("entry directory must be empty before checkout: {0}")]
    DirectoryNotEmpty(PathBuf),
    /// Checkout would overwrite a path that is not a managed entry file.
    #[error("checkout conflict at unmanaged path: {0}")]
    CheckoutConflict(PathBuf),
    /// Entry rename source and destination paths must differ.
    #[error("entry rename source and destination are both `{0}`")]
    RenameSameId(EntryAddress),
    /// The entry selected for rename does not exist.
    #[error("entry `{0}` does not exist")]
    EntryNotFound(EntryAddress),
    /// The destination entry address already exists.
    #[error("entry `{id}` already exists at {path}")]
    EntryAlreadyExists {
        /// Existing destination path.
        id: EntryAddress,
        /// Existing destination path.
        path: PathBuf,
    },
    /// Reading the directory or one of its files failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A discovered path could not be made relative to the entry directory.
    #[error("entry address {path} is not inside entry directory {root}")]
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
    /// The generated meta registry lockfile could not be written.
    #[error(transparent)]
    MetaRegistry(#[from] MetaRegistryError),
    /// An entry address could not be represented.
    #[error(transparent)]
    EntryAddress(#[from] EntryAddressError),
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
    FrozenEntryProtected(EntryAddress),
    /// Crystallization tried to write an entry outside the selected glacier domain.
    #[error("glacier entry `{id}` is outside glacier domain `{domain}`")]
    GlacierEntryOutsideDomain {
        /// Selected glacier domain.
        domain: EntryAtom,
        /// Entry address outside the domain.
        id: EntryAddress,
    },
    /// Crystallization tried to write an entry without managed protection.
    #[error("glacier entry `{0}` must carry frozen reason `managed`")]
    GlacierEntryNotManaged(EntryAddress),
    /// A path inside a glacier is not managed by crystallization.
    #[error("glacier contains unmanaged path: {0}")]
    UnmanagedGlacierPath(PathBuf),
    /// A parsed entry had no file path in the directory report.
    #[error("entry `{0}` has no source file path")]
    MissingEntryFilePath(EntryAddress),
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
        /// Existing entry address.
        source_path: PathBuf,
        /// New entry address.
        destination_path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Two artifact records name the same entry-owned path.
    #[error("entry `{owner}` has duplicate artifact `{path}`")]
    DuplicateArtifact {
        /// Entry that owns the duplicate artifact.
        owner: EntryAddress,
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
        owner: EntryAddress,
        /// Owner-relative artifact path.
        path: EntryArtifactPath,
    },
    /// One artifact path already exists.
    #[error("entry `{owner}` already has artifact `{path}`")]
    ArtifactAlreadyExists {
        /// Entry that owns the existing artifact.
        owner: EntryAddress,
        /// Owner-relative artifact path.
        path: EntryArtifactPath,
    },
    /// Artifact rename source and destination paths must differ.
    #[error("entry `{owner}` artifact rename source and destination are both `{path}`")]
    ArtifactRenameSamePath {
        /// Entry that owns the artifact.
        owner: EntryAddress,
        /// Repeated owner-relative artifact path.
        path: EntryArtifactPath,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structural::StructuralEdgeDirection;
    use crate::{
        EntryMetadata, MetaFile, RepoMember, StructuralFieldSettings, WitnessCheckSettings,
        WitnessSettings,
    };

    const FIELD_KIND: &str = "kind";
    const FIELD_AREA: &str = "area";
    const FIELD_PARENT: &str = "parent";

    fn write_entry(root: &Path, name: &str, body: &str) {
        ensure_intrinsic_field_entries(root);
        write_entry_without_intrinsics(root, name, body);
    }

    fn write_entry_without_intrinsics(root: &Path, name: &str, body: &str) {
        let path = root.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn ensure_intrinsic_field_entries(root: &Path) {
        fs::create_dir_all(root).unwrap();
        let name_path = root.join("name.md");
        if !name_path.exists() {
            fs::write(
                &name_path,
                "\
---
name: Name
desc: The required plain-string title field for entries.
meta.type: \"intrinsic\"
---

Body.
",
            )
            .unwrap();
        }
        let desc_path = root.join("desc.md");
        if !desc_path.exists() {
            fs::write(
                &desc_path,
                "\
---
name: Description
desc: The required plain-string summary field for entries.
meta.type: \"intrinsic\"
---

Body.
",
            )
            .unwrap();
        }
    }

    fn write_structural_field_entries(root: &Path, fields: &[&str]) {
        write_structural_field_entries_with_meta(root, fields, true);
    }

    fn write_plain_structural_field_entries(root: &Path, fields: &[&str]) {
        write_structural_field_entries_with_meta(root, fields, false);
    }

    fn write_structural_field_entries_with_meta(root: &Path, fields: &[&str], meta: bool) {
        for field in fields {
            let meta = if meta {
                "meta.type: \"structural\"\nmeta.ripple.lake: []\nmeta.ripple.anchor: []\n"
            } else {
                ""
            };
            write_entry(
                root,
                &format!("{field}.md"),
                &format!(
                    "\
---
name: {field}
desc: A structural link relation.
{meta}\
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

    fn structural_render_settings(
        fields: impl IntoIterator<Item = (&'static str, StructuralFieldSettings)>,
    ) -> StructuralRenderSettings {
        StructuralRenderSettings::from_fields(fields.into_iter().map(|(field, settings)| {
            let mut directions = Vec::new();
            if settings.to.render {
                directions.push(StructuralEdgeDirection::To);
            }
            if settings.from.render {
                directions.push(StructuralEdgeDirection::From);
            }
            if settings.clique.render {
                directions.push(StructuralEdgeDirection::Clique);
            }
            (field, directions)
        }))
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

    // sirno:witness:witness-lookup:begin
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
    // sirno:witness:witness-lookup:end

    #[test]
    fn replaces_only_managed_glacier_entries() {
        let temp = tempfile::tempdir().unwrap();
        let directory = entry_directory(temp.path());
        let domain = EntryAtom::new("core").unwrap();
        let mut metadata = EntryMetadata::new("Design", "Managed upstream entry.").unwrap();
        metadata.meta.frozen = Some(FrozenMarker::managed());
        let entry = Entry::new(EntryAddress::new("core.design").unwrap(), metadata, "Body.\n");
        let artifact = EntryArtifact::new(
            EntryAddress::new("core.design").unwrap(),
            EntryArtifactPath::new(Path::new("logo.bin")).unwrap(),
            b"logo".to_vec(),
        );

        let report = directory
            .replace_glacier(
                &domain,
                &[entry],
                &[artifact],
                &EntryDirectoryCheckSettings::default(),
            )
            .unwrap();

        assert!(report.changed_paths().iter().any(|path| path.ends_with("core/design.md")));
        assert!(temp.path().join("core/design.md").exists());
        assert!(temp.path().join(".artifacts/core.design/logo.bin").exists());

        write_entry(
            temp.path(),
            "core/local.md",
            "\
---
name: Local
desc: Unmanaged local entry.
---

Body.
",
        );
        let error = directory
            .replace_glacier(&domain, &[], &[], &EntryDirectoryCheckSettings::default())
            .unwrap_err();

        assert!(
            matches!(error, EntryDirectoryError::UnmanagedGlacierPath(path) if path.ends_with("core/local.md"))
        );
    }

    #[test]
    fn melt_preserves_managed_frozen_reason() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "core/name.md",
            "\
---
core.name: Name
core.desc: The required title field.
meta:
  frozen:
    - managed
meta.type: \"intrinsic\"
---

Body.
",
        );
        write_entry(
            temp.path(),
            "core/desc.md",
            "\
---
core.name: Description
core.desc: The required summary field.
meta:
  frozen:
    - managed
meta.type: \"intrinsic\"
---

Body.
",
        );
        write_entry(
            temp.path(),
            "core/design.md",
            "\
---
core.name: Design
core.desc: Managed upstream entry.
meta:
  frozen:
    - reviewed
    - managed
---

Body.
",
        );
        let directory = entry_directory(temp.path());
        directory.melt_entry(&EntryAddress::new("core.design").unwrap()).unwrap();

        let source = fs::read_to_string(temp.path().join("core/design.md")).unwrap();
        assert!(!source.contains("reviewed"));
        assert!(source.contains("  - managed"));
        let entry = directory.read_entry(&EntryAddress::new("core.design").unwrap()).unwrap();
        assert!(entry.metadata.meta.frozen.as_ref().is_some_and(|marker| marker.is_managed()));
    }

    #[test]
    fn read_entries_with_registry_applies_scoped_meta_to_glacier_projection() {
        let temp = tempfile::tempdir().unwrap();
        let reservoir = temp.path().join("reservoir");
        let projection = temp.path().join("projection");
        let local_source = "\
---
name: Local
desc: Local projected entry.
---

Body.
";
        let core_name_source = "\
---
core.name: Name
core.desc: The required title field.
meta:
  frozen:
    - managed
meta.type: \"intrinsic\"
---

Body.
";
        let core_desc_source = "\
---
core.name: Description
core.desc: The required summary field.
meta:
  frozen:
    - managed
meta.type: \"intrinsic\"
---

Body.
";
        let core_design_source = "\
---
core.name: Design
core.desc: Managed upstream entry.
meta:
  frozen:
    - managed
---

Body.
";
        write_entry(&reservoir, "local.md", local_source);
        write_entry(&reservoir, "core/name.md", core_name_source);
        write_entry(&reservoir, "core/desc.md", core_desc_source);
        write_entry(&reservoir, "core/design.md", core_design_source);
        let report = entry_directory(reservoir).check(CheckMode::Review).unwrap();
        assert!(report.is_clean(), "{report:?}");

        write_entry_without_intrinsics(&projection, "local.md", local_source);
        write_entry_without_intrinsics(&projection, "core/name.md", core_name_source);
        write_entry_without_intrinsics(&projection, "core/desc.md", core_desc_source);
        write_entry_without_intrinsics(&projection, "core/design.md", core_design_source);

        let entries = entry_directory(projection)
            .read_entries_with_registry(report.meta(), Vec::<PathBuf>::new())
            .unwrap();

        assert_eq!(entries.len(), 4);
        let local = entries.iter().find(|entry| entry.id.as_str() == "local").unwrap();
        assert_eq!(local.metadata.name(), "Local");
        assert_eq!(local.metadata.desc(), "Local projected entry.");
        let design = entries.iter().find(|entry| entry.id.as_str() == "core.design").unwrap();
        assert_eq!(design.metadata.name(), "Design");
        assert_eq!(design.metadata.desc(), "Managed upstream entry.");
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
        assert_eq!(report.entries().len(), 4);
        assert!(report.entry_file_path(&EntryAddress::new("concept").unwrap()).is_some());
    }

    #[test]
    fn check_loads_nested_entry_addresses() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "core/design.md",
            "\
---
name: Design
desc: Core design.
---

Body.
",
        );

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(report.is_clean());
        assert!(report.entries().iter().any(|entry| entry.id.as_str() == "core.design"));
        assert_eq!(
            report.entry_file_path(&EntryAddress::new("core.design").unwrap()).unwrap(),
            temp.path().join("core/design.md")
        );
    }

    #[test]
    fn check_rejects_dotted_markdown_filename() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "core.design.md",
            "\
---
name: Design
desc: Core design.
---

Body.
",
        );

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(report.has_errors());
        assert!(report.file_diagnostics()[0].message.contains("filename must not contain dots"));
        assert!(report.entries().iter().all(|entry| entry.id.as_str() != "core.design"));
    }

    #[test]
    fn check_treats_unknown_leading_dot_root_as_reserved_builtin() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            ".custom/design.md",
            "\
---
name: Design
desc: Core design.
---

Body.
",
        );

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(report.has_errors());
        assert!(report.file_diagnostics()[0].message.contains("reserved built-in path"));
        assert!(report.entries().iter().all(|entry| entry.id.as_str() != ".custom.design"));
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
        assert_eq!(report.artifacts()[0].owner, EntryAddress::new("concept").unwrap());
        assert_eq!(report.artifacts()[0].path.as_str(), "images/logo.bin");
        assert_eq!(report.artifacts()[0].content, vec![0, 1, 2, 3]);
    }

    #[test]
    fn check_loads_dotted_entry_artifacts_from_reserved_directory() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "core/design.md",
            "\
---
name: Design
desc: Core design.
---

Body.
",
        );
        let artifact_dir =
            temp.path().join(ARTIFACT_DIRECTORY_NAME).join("core.design").join("images");
        fs::create_dir_all(&artifact_dir).unwrap();
        fs::write(artifact_dir.join("logo.bin"), [0, 1, 2, 3]).unwrap();

        let report = entry_directory(temp.path()).check(CheckMode::Review).unwrap();

        assert!(report.is_clean());
        assert_eq!(report.artifacts()[0].owner, EntryAddress::new("core.design").unwrap());
        assert_eq!(report.artifacts()[0].path.as_str(), "images/logo.bin");
    }

    #[test]
    fn artifact_operations_accept_dotted_entry_addresses() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "core/design.md",
            "\
---
name: Design
desc: Core design.
---

Body.
",
        );
        let source = temp.path().join("logo.bin");
        fs::write(&source, [0, 1, 2, 3]).unwrap();
        let directory = entry_directory(temp.path());
        let id = EntryAddress::new("core.design").unwrap();

        let added = directory
            .add_entry_artifact(&id, &source, &EntryArtifactPath::new("images/logo.bin").unwrap())
            .unwrap();
        let renamed = directory
            .rename_entry_artifact(
                &id,
                &EntryArtifactPath::new("images/logo.bin").unwrap(),
                &EntryArtifactPath::new("assets/logo.bin").unwrap(),
            )
            .unwrap();
        let artifacts = directory.read_entry_artifacts(&id).unwrap();
        let removed = directory
            .remove_entry_artifact(&id, &EntryArtifactPath::new("assets/logo.bin").unwrap())
            .unwrap();

        assert_eq!(added, temp.path().join(".artifacts/core.design/images/logo.bin"));
        assert_eq!(renamed, temp.path().join(".artifacts/core.design/assets/logo.bin"));
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].owner, id);
        assert_eq!(artifacts[0].path.as_str(), "assets/logo.bin");
        assert_eq!(artifacts[0].content, vec![0, 1, 2, 3]);
        assert_eq!(removed, temp.path().join(".artifacts/core.design/assets/logo.bin"));
        assert!(!temp.path().join(".artifacts/core.design").exists());
    }

    #[test]
    fn artifact_operations_protect_frozen_dotted_entry_addresses() {
        let temp = tempfile::tempdir().unwrap();
        write_entry(
            temp.path(),
            "core/design.md",
            "\
---
name: Design
desc: Core design.
---

Body.
",
        );
        let source = temp.path().join("logo.bin");
        fs::write(&source, [0, 1, 2, 3]).unwrap();
        let directory = entry_directory(temp.path());
        let id = EntryAddress::new("core.design").unwrap();
        directory.freeze_entry(&id).unwrap();

        let error = directory
            .add_entry_artifact(&id, &source, &EntryArtifactPath::new("images/logo.bin").unwrap())
            .unwrap_err();

        assert!(matches!(error, EntryDirectoryError::FrozenEntryProtected(path) if path == id));
        directory.clear_local_protection(&EntryDirectoryCheckSettings::default(), false).unwrap();
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
            entry_directory(temp.path())
                .entry_exists(&EntryAddress::new("concept").unwrap())
                .unwrap()
        );
        assert!(
            !entry_directory(temp.path())
                .entry_exists(&EntryAddress::new("missing").unwrap())
                .unwrap()
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
        assert_eq!(report.entries().len(), 3);
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
        assert_eq!(report.entries().len(), 3);
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
            .check_with_settings(CheckMode::Review, &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert!(report.has_errors());
        assert_eq!(report.structural_report().diagnostics().len(), 1);
    }

    #[test]
    fn check_writes_generated_meta_registry_lockfile() {
        let temp = tempfile::tempdir().unwrap();
        let meta_path = temp.path().join(".sirno/meta.toml");
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
        write_structural_field_entries(temp.path(), &[FIELD_KIND]);

        let report = entry_directory(temp.path())
            .check_with_settings(
                CheckMode::Review,
                &EntryDirectoryCheckSettings {
                    render: false,
                    meta_path: Some(meta_path.clone()),
                    ..EntryDirectoryCheckSettings::default()
                },
            )
            .unwrap();

        assert!(report.is_clean());
        let file: MetaFile = toml::from_str(&fs::read_to_string(meta_path).unwrap()).unwrap();
        assert_eq!(file.intrinsics[0].field, "desc");
        assert_eq!(file.intrinsics[1].field, "name");
        assert_eq!(file.structural[0].field, FIELD_KIND);
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

        assert_eq!(paths.len(), 6);
        assert!(root.join("concept.md").exists());
        assert!(root.join("name.md").exists());
        assert!(root.join("desc.md").exists());
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
        metadata.push_structural_target(FIELD_KIND, EntryAddress::new("meta").unwrap());
        let entry = Entry::new(EntryAddress::new("local-idea").unwrap(), metadata, "");

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
        let entry = Entry::new(EntryAddress::new("local-idea").unwrap(), metadata, "");

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
            structural_render: structural_render_settings([
                (FIELD_KIND, StructuralFieldSettings::default()),
                (FIELD_AREA, render_settings(true, true, false)),
            ]),
            ..EntryDirectoryCheckSettings::default()
        };
        let directory = entry_directory(&root);
        directory.generate_links_with_check_settings(&settings).unwrap();
        let artifact_dir = root.join(ARTIFACT_DIRECTORY_NAME).join("old-entry");
        fs::create_dir_all(&artifact_dir).unwrap();
        fs::write(artifact_dir.join("note.txt"), "artifact").unwrap();

        let report = directory
            .rename_entry(
                &EntryAddress::new("old-entry").unwrap(),
                &EntryAddress::new("new-entry").unwrap(),
                &settings,
            )
            .unwrap();
        let checked = directory.check_with_settings(CheckMode::Review, &settings).unwrap();
        let reader_source = fs::read_to_string(root.join("reader.md")).unwrap();
        let renamed_source = fs::read_to_string(root.join("new-entry.md")).unwrap();

        assert_eq!(report.old_id(), &EntryAddress::new("old-entry").unwrap());
        assert_eq!(report.new_id(), &EntryAddress::new("new-entry").unwrap());
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
    fn rename_structural_relation_entry_renames_field_names() {
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
        write_structural_field_entries(&root, &["refines"]);
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
            structural_render: structural_render_settings([
                ("refines", render_settings(true, true, false)),
                ("prerequisite", render_settings(true, true, false)),
            ]),
            ..EntryDirectoryCheckSettings::default()
        };
        let directory = entry_directory(&root);
        directory.generate_links_with_check_settings(&settings).unwrap();

        directory
            .rename_entry(
                &EntryAddress::new("refines").unwrap(),
                &EntryAddress::new("prerequisite").unwrap(),
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
                &EntryAddress::new("old-entry").unwrap(),
                &EntryAddress::new("new-entry").unwrap(),
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
                &EntryAddress::new("old-entry").unwrap(),
                &EntryAddress::new("new-entry").unwrap(),
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

        let path = entry_directory(temp.path())
            .freeze_entry(&EntryAddress::new("alpha").unwrap())
            .unwrap();
        let source = fs::read_to_string(&path).unwrap();

        assert!(source.contains("meta:\n  frozen:\n    - reviewed\n"));
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
        let path = directory.freeze_entry(&EntryAddress::new("alpha").unwrap()).unwrap();

        directory.set_writable(&settings).unwrap();

        assert_path_readonly(&path);
        directory.melt_entry(&EntryAddress::new("alpha").unwrap()).unwrap();
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
meta:
  frozen:
    - reviewed
---

Body.
",
        );
        let path = entry_directory(temp.path())
            .freeze_entry(&EntryAddress::new("alpha").unwrap())
            .unwrap();

        entry_directory(temp.path()).melt_entry(&EntryAddress::new("alpha").unwrap()).unwrap();
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
meta:
  frozen:
    - reviewed
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
        let entry = Entry::new(EntryAddress::new("new").unwrap(), metadata, "Body.\n");

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
        let entry = Entry::new(EntryAddress::new("new").unwrap(), metadata, "Body.\n");
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
        let entry = Entry::new(EntryAddress::new("new").unwrap(), metadata, "Body.\n");

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
    fn clear_local_protection_keeps_frozen_marker() {
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
        let directory = entry_directory(temp.path());
        let path = directory.freeze_entry(&EntryAddress::new("alpha").unwrap()).unwrap();

        let report = directory
            .clear_local_protection(&EntryDirectoryCheckSettings::default(), false)
            .unwrap();
        let source = fs::read_to_string(&path).unwrap();

        assert!(source.contains("meta:\n  frozen:\n    - reviewed\n"));
        assert_path_writable(&path);
        assert!(report.paths().contains(&path));
    }

    #[test]
    fn fix_local_protection_reapplies_frozen_entry_permissions() {
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
        let directory = entry_directory(temp.path());
        let id = EntryAddress::new("alpha").unwrap();
        let path = directory.freeze_entry(&id).unwrap();
        directory.clear_local_protection(&EntryDirectoryCheckSettings::default(), false).unwrap();

        let report = directory
            .fix_local_protection(&EntryDirectoryCheckSettings::default(), false, false)
            .unwrap();

        assert_path_readonly(&path);
        assert!(report.paths().contains(&path));
        directory.clear_local_protection(&EntryDirectoryCheckSettings::default(), false).unwrap();
    }

    #[test]
    fn fix_local_protection_can_repair_readonly_checkout_state() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        entry_directory(&root).init().unwrap();
        let settings = EntryDirectoryCheckSettings::default();
        let entry_path = root.join("concept.md");

        let report = entry_directory(&root).fix_local_protection(&settings, true, false).unwrap();

        assert_path_readonly(&root);
        assert_path_readonly(&entry_path);
        assert!(report.paths().contains(&root));
        assert!(report.paths().contains(&entry_path));
        entry_directory(&root).clear_local_protection(&settings, false).unwrap();
    }

    #[test]
    fn readonly_checkout_warning_is_visible_body_quote() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        let metadata = EntryMetadata::new("New", "New entry.").unwrap();
        let entry = Entry::new(EntryAddress::new("new").unwrap(), metadata, "Body.\n");
        let mut entries = Entry::default_seed_entries().unwrap();
        entries.push(entry.clone());
        let paths = entry_directory(&root)
            .write(&entries, EntryDirectoryWritePolicy::EmptyDirectory)
            .unwrap();

        entry_directory(&root).add_readonly_checkout_warnings(&paths).unwrap();
        let source = fs::read_to_string(root.join("new.md")).unwrap();
        let checked = entry_directory(&root).check(CheckMode::Review).unwrap();

        assert!(source.contains(
            "\n---\n\n> This file is a read-only Sirno managed checkout.\n\
             > Do not edit it by hand.\n\nBody.\n"
        ));
        let checked_entry =
            checked.entries().iter().find(|entry| entry.id.as_str() == "new").unwrap();
        assert_eq!(checked_entry.metadata, entry.metadata);
        assert!(checked_entry.body.starts_with(READONLY_CHECKOUT_WARNING));
        assert!(checked_entry.body.ends_with("Body.\n"));
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

        assert_eq!(report.entry_count(), 9);
        assert_eq!(report.changed_paths().len(), 9);
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

        assert_eq!(report.entry_count(), 6);
        assert_eq!(report.changed_paths().len(), 6);
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

        assert_eq!(report.entry_count(), 6);
        assert_eq!(report.changed_paths().len(), 6);
        assert!(!concept.contains(crate::render::BEGIN_LINKS_GUARD));
    }

    #[test]
    fn delete_gen_link_is_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        entry_directory(&root).init().unwrap();

        let report = entry_directory(&root).delete_generated_links().unwrap();

        assert_eq!(report.entry_count(), 6);
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
        write_plain_structural_field_entries(&root, &[FIELD_KIND, FIELD_AREA, FIELD_PARENT]);

        let report = entry_directory(&root)
            .check_with_settings(
                CheckMode::Review,
                &EntryDirectoryCheckSettings {
                    render: true,
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
        write_plain_structural_field_entries(&root, &[FIELD_KIND, FIELD_AREA, FIELD_PARENT]);

        let report = entry_directory(&root)
            .check_with_settings(
                CheckMode::Edit,
                &EntryDirectoryCheckSettings {
                    render: true,
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
        write_plain_structural_field_entries(&root, &[FIELD_KIND, FIELD_AREA, FIELD_PARENT]);

        let report = entry_directory(&root)
            .check_with_settings(
                CheckMode::Review,
                &EntryDirectoryCheckSettings {
                    render: false,
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
        assert!(
            report
                .file_diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.message.contains("malformed generated links"))
        );
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
