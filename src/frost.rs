//! Sirno Frost facade.
//!
//! Sirno Frost exposes frozen snapshots as typed Sirno entries.
//! The current backend uses `eter` filesystem snapshots as durable storage.
//! That layout is private to this module.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use eter::filesystem::{FilesystemBackend, FilesystemEntryId, FilesystemError, FilesystemWriteTxn};
use eter::{
    EntryFacet, Eter, Eterator, Field, GcOption, Lifecycle, LiveEntries, Resolution, SnapshotRef,
    WriteTxn,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::artifact::{EntryArtifact, EntryArtifactPath, EntryArtifactPathError};
use crate::check::{CheckMode, CheckReport};
use crate::entry::{Entry, EntryMetadata, EntryStructuralFields};
use crate::identifier::{EntryAddress, EntryAddressError};
use crate::lake::{
    EntryDirectory, EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryWritePolicy,
};
use crate::render::GeneratedLinkBody;
use crate::structural::StructuralSettings;

/// Lifecycle state used by Sirno entries in the `eter` backend.
///
/// Sirno currently distinguishes only entry presence.
/// Deletion or archival policy is left to a later design step.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum EntryLifecycle {
    /// The entry exists at this snapshot.
    Active,
}

type SirnoBackend = FilesystemBackend<EntryLifecycle>;
type SirnoWriteTxn<'a> = FilesystemWriteTxn<'a, EntryLifecycle>;

struct NameField;
impl Field for NameField {
    type Content = String;
}

struct DescField;
impl Field for DescField {
    type Content = String;
}

struct StructuralField;
impl Field for StructuralField {
    type Content = EntryStructuralFields;
}

struct ArtifactManifestField;
impl Field for ArtifactManifestField {
    type Content = Vec<EntryArtifactPath>;
}

/// Sirno Frost facade for Sirno entries.
///
/// Invariant: all entries written through this type are represented through
/// typed metadata fields and a Markdown body in the configured `eter` backend.
#[derive(Debug)]
// sirno:witness:sirno-frost:begin
pub struct SirnoFrost {
    root: PathBuf,
    backend: SirnoBackend,
}
// sirno:witness:sirno-frost:end

/// Result of garbage-collecting the private frost backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrostGcReport {
    /// Snapshot reference before collection.
    pub before: SnapshotRef,
    /// Snapshot reference after collection.
    pub after: SnapshotRef,
    /// Artifact byte files removed from sparse artifact version directories.
    pub artifact_files_removed: usize,
    /// Empty artifact directories removed after byte-file collection.
    pub artifact_directories_removed: usize,
}

impl FrostGcReport {
    /// Return true when collection removed stored rows, byte files, or artifact directories.
    pub fn collected(self) -> bool {
        self.before != self.after
            || self.artifact_files_removed > 0
            || self.artifact_directories_removed > 0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ArtifactGcReport {
    files_removed: usize,
    directories_removed: usize,
}

impl SirnoFrost {
    /// Open or initialize Sirno Frost at `root`.
    // sirno:witness:sirno-frost:begin
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, FrostError> {
        trace!("sirno frost open begin");
        let root = root.into();
        let backend = SirnoBackend::open(&root, Self::registry())?;
        trace!("sirno frost open end");
        Ok(Self { root, backend })
    }
    // sirno:witness:sirno-frost:end

    /// The private Sirno Frost backend path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Return the private backend directory path for one stored entry.
    pub fn entry_storage_path(
        root: impl AsRef<Path>, id: &EntryAddress,
    ) -> Result<PathBuf, FrostError> {
        Ok(root.as_ref().join(id.to_filesystem_id()?.as_str()))
    }

    /// Return the private backend directory path for one entry-version artifact tree.
    pub fn entry_artifact_snapshot_path(
        root: impl AsRef<Path>, owner: &EntryAddress, version: Eterator,
    ) -> Result<PathBuf, FrostError> {
        Ok(Self::entry_storage_path(root, owner)?
            .join(artifact_snapshot_directory_name(owner, version)?))
    }

    /// Return the current backend snapshot reference.
    // sirno:witness:sirno-frost:begin
    pub fn current_snapshot(&self) -> Result<SnapshotRef, FrostError> {
        Ok(self.backend.current_snapshot()?)
    }
    // sirno:witness:sirno-frost:end

    /// Return the current backend snapshot version coordinate.
    // sirno:witness:sirno-frost:begin
    pub fn current_version(&self) -> Result<Eterator, FrostError> {
        Ok(self.backend.current_version()?)
    }
    // sirno:witness:sirno-frost:end

    /// Pair a version coordinate with the current backend GC generation.
    ///
    /// `eter` rejects stale snapshot references.
    /// Sirno exposes version coordinates at the CLI and resolves them against the current generation.
    // sirno:witness:sirno-frost:begin
    pub fn snapshot_for_version(&self, version: Eterator) -> Result<SnapshotRef, FrostError> {
        Ok(SnapshotRef::new(self.backend.gc_generation()?, version))
    }
    // sirno:witness:sirno-frost:end

    /// Garbage-collect storage unreachable from the current frost snapshot.
    ///
    /// The filesystem backend does not persist retired snapshots.
    /// Sirno supplies the latest frost snapshot as the explicit live set.
    // sirno:witness:sirno-frost:begin
    pub fn gc_current_snapshot(&mut self) -> Result<FrostGcReport, FrostError> {
        trace!("sirno frost gc begin");
        let before = self.current_snapshot()?;
        let after = if before.eterator == Eterator::EMPTY {
            before
        } else {
            self.backend.gc(GcOption::UseLiveSet(BTreeSet::from([before])))?;
            self.current_snapshot()?
        };
        let artifact_gc = self.gc_artifact_snapshot_directories(after)?;
        if after.eterator == Eterator::EMPTY {
            trace!("sirno frost gc end: empty");
            return Ok(FrostGcReport {
                before,
                after,
                artifact_files_removed: artifact_gc.files_removed,
                artifact_directories_removed: artifact_gc.directories_removed,
            });
        }
        trace!(
            "sirno frost gc end: before={} after={} artifact_files_removed={} artifact_dirs_removed={}",
            before.generation.number(),
            after.generation.number(),
            artifact_gc.files_removed,
            artifact_gc.directories_removed
        );
        Ok(FrostGcReport {
            before,
            after,
            artifact_files_removed: artifact_gc.files_removed,
            artifact_directories_removed: artifact_gc.directories_removed,
        })
    }
    // sirno:witness:sirno-frost:end

    /// Write or replace one entry.
    // sirno:witness:sirno-frost:begin
    pub fn put_entry(&mut self, entry: &Entry) -> Result<SnapshotRef, FrostError> {
        trace!("sirno put_entry begin: id={}", entry.id);
        let current = self.current_snapshot()?;
        let entry_to_write = Self::entry_without_frozen_marker(entry);
        if entry.metadata.frozen.is_some() {
            self.ensure_entry_matches_snapshot(current, &entry_to_write)?;
            trace!("sirno put_entry end: version={}", current.version());
            return Ok(current);
        }
        let fs_id = entry.id.to_filesystem_id()?;
        let facet = StoredEntryFacet::from_entry(&entry_to_write);
        let snapshot = facet.apply_to(self.backend.write(), &fs_id).commit()?;
        trace!("sirno put_entry end: version={}", snapshot.version());
        Ok(snapshot)
    }
    // sirno:witness:sirno-frost:end

    /// Read one entry at the current snapshot.
    pub fn read_entry(&self, id: &EntryAddress) -> Result<Option<Entry>, FrostError> {
        self.read_entry_at_snapshot(self.current_snapshot()?, id)
    }

    /// Read one entry at a selected frozen snapshot.
    // sirno:witness:sirno-frost:begin
    pub fn read_entry_at_snapshot(
        &self, at: SnapshotRef, id: &EntryAddress,
    ) -> Result<Option<Entry>, FrostError> {
        trace!("sirno read_entry_at begin: id={id} at={}", at.version());
        let fs_id = id.to_filesystem_id()?;
        let Some(facet) = StoredEntryFacet::load_from(&self.backend, at, &fs_id)? else {
            trace!("sirno read_entry_at end: absent");
            return Ok(None);
        };
        let entry = facet.into_entry(id.clone())?;
        trace!("sirno read_entry_at end: present");
        Ok(Some(entry))
    }
    // sirno:witness:sirno-frost:end

    /// Read every active entry at the current snapshot.
    pub fn read_all_entries(&self) -> Result<Vec<Entry>, FrostError> {
        self.read_all_entries_at_snapshot(self.current_snapshot()?)
    }

    /// Read every active entry at a selected frozen snapshot.
    // sirno:witness:sirno-frost:begin
    pub fn read_all_entries_at_snapshot(&self, at: SnapshotRef) -> Result<Vec<Entry>, FrostError> {
        trace!("sirno read_all_entries begin: at={}", at.version());
        let mut entries = Vec::new();
        for fs_id in self.backend.live_entries(at)? {
            let id = EntryAddress::try_from(fs_id)?;
            if let Some(entry) = self.read_entry_at_snapshot(at, &id)? {
                entries.push(entry);
            }
        }
        trace!("sirno read_all_entries end: entries={}", entries.len());
        Ok(entries)
    }
    // sirno:witness:sirno-frost:end

    /// Read every active lake-owned artifact at the current snapshot.
    // sirno:witness:entry-artifact:begin
    pub fn read_all_artifacts(&self) -> Result<Vec<EntryArtifact>, FrostError> {
        self.read_all_artifacts_at_snapshot(self.current_snapshot()?)
    }

    /// Read every active lake-owned artifact at a selected frozen snapshot.
    pub fn read_all_artifacts_at_snapshot(
        &self, at: SnapshotRef,
    ) -> Result<Vec<EntryArtifact>, FrostError> {
        trace!("sirno read_all_artifacts begin: at={}", at.version());
        let mut artifacts = Vec::new();
        for fs_id in self.backend.live_entries(at)? {
            let owner = EntryAddress::try_from(fs_id.clone())?;
            let Some(facet) = StoredEntryFacet::load_from(&self.backend, at, &fs_id)? else {
                continue;
            };
            for path in facet.artifact_paths {
                let content = self.read_artifact_content_at_snapshot(at, &owner, &path)?;
                artifacts.push(EntryArtifact::new(owner.clone(), path, content));
            }
        }
        artifacts.sort_by(|left, right| {
            left.owner.cmp(&right.owner).then_with(|| left.path.cmp(&right.path))
        });
        trace!("sirno read_all_artifacts end: artifacts={}", artifacts.len());
        Ok(artifacts)
    }
    // sirno:witness:entry-artifact:end

    /// Check current entries at the selected boundary.
    pub fn check_current(&self, mode: CheckMode) -> Result<CheckReport, FrostError> {
        let entries = self.read_all_entries()?;
        Ok(mode.check_entries(&entries, &StructuralSettings::default()))
    }

    /// Require a lake entry to match the current frost snapshot.
    ///
    /// Generated-link regions and the `frozen:` marker are lake state.
    /// They are removed before comparing to frost storage.
    pub fn ensure_entry_current(&self, entry: &Entry) -> Result<(), FrostError> {
        let entries = Self::entries_without_generated_links(std::slice::from_ref(entry))?;
        let entry = Self::entry_without_frozen_marker(&entries[0]);
        self.ensure_entry_matches_snapshot(self.current_snapshot()?, &entry)
    }

    /// Require a lake entry and its artifacts to match the current frost snapshot.
    // sirno:witness:entry-artifact:begin
    pub fn ensure_entry_bundle_current(
        &self, entry: &Entry, artifacts: &[EntryArtifact],
    ) -> Result<(), FrostError> {
        self.ensure_entry_current(entry)?;
        let snapshot = self.current_snapshot()?;
        let previous = artifacts_by_owner(self.read_all_artifacts_at_snapshot(snapshot)?);
        let current = artifacts_by_owner(artifacts.iter().cloned());
        if previous.get(&entry.id).map(Vec::as_slice).unwrap_or_default()
            != current.get(&entry.id).map(Vec::as_slice).unwrap_or_default()
        {
            return Err(FrostError::FrozenEntryChanged(entry.id.clone()));
        }
        Ok(())
    }
    // sirno:witness:entry-artifact:end

    /// Freeze a lake entry directory into frost.
    ///
    /// The directory must pass review-mode checks before any frozen row is written.
    /// Generated-link regions are stripped from the committed snapshot.
    // sirno:witness:sirno-frost:begin
    pub fn commit_entry_directory(
        &mut self, root: impl Into<PathBuf>, settings: &EntryDirectoryCheckSettings,
    ) -> Result<SnapshotRef, FrostError> {
        let root = root.into();
        trace!("sirno commit_entry_directory begin: root={}", root.display());
        let report = EntryDirectory::new(&root).check_with_settings(CheckMode::Review, settings)?;
        if report.has_errors() {
            return Err(FrostError::InvalidEntryDirectory(root));
        }
        let entries = Self::entries_without_generated_links(report.entries())?;
        let version = self.commit_entries_and_artifacts(&entries, report.artifacts())?;
        trace!("sirno commit_entry_directory end: version={}", version.version());
        Ok(version)
    }
    // sirno:witness:sirno-frost:end

    /// Materialize a frozen snapshot into a lake entry directory.
    // sirno:witness:sirno-frost:begin
    pub fn checkout_entry_directory(
        &self, at: SnapshotRef, root: impl Into<PathBuf>, policy: EntryDirectoryWritePolicy,
    ) -> Result<Vec<PathBuf>, FrostError> {
        let root = root.into();
        trace!("sirno checkout_entry_directory begin: at={} root={}", at.version(), root.display());
        let entries = self.read_all_entries_at_snapshot(at)?;
        let artifacts = self.read_all_artifacts_at_snapshot(at)?;
        let paths =
            EntryDirectory::new(&root).write_with_artifacts(&entries, &artifacts, policy)?;
        trace!("sirno checkout_entry_directory end: entries={}", paths.len());
        Ok(paths)
    }
    // sirno:witness:sirno-frost:end

    /// Initialize ordinary seed entries.
    ///
    /// The initialized entries are ordinary Sirno entries.
    /// They are created together and are not privileged by later operations.
    // sirno:witness:sirno-frost:begin
    pub fn init_default_entries(&mut self) -> Result<SnapshotRef, FrostError> {
        trace!("sirno init_default_entries begin");
        let entries = Entry::default_seed_entries()?;
        for entry in &entries {
            let fs_id = entry.id.to_filesystem_id()?;
            if self.backend.entry_id_in_use(&fs_id)? {
                return Err(FrostError::EntryAlreadyExists(entry.id.clone()));
            }
        }
        let version = self.commit_entries(&entries)?;
        trace!("sirno init_default_entries end: version={}", version.version());
        Ok(version)
    }
    // sirno:witness:sirno-frost:end

    // sirno:witness:sirno-frost:begin
    fn commit_entries(&mut self, entries: &[Entry]) -> Result<SnapshotRef, FrostError> {
        self.commit_entries_and_artifacts(entries, &[])
    }

    fn commit_entries_and_artifacts(
        &mut self, entries: &[Entry], artifacts: &[EntryArtifact],
    ) -> Result<SnapshotRef, FrostError> {
        let current = self.current_snapshot()?;
        let previous_entries = self
            .read_all_entries_at_snapshot(current)?
            .into_iter()
            .map(|entry| (entry.id.clone(), entry))
            .collect::<BTreeMap<_, _>>();
        let previous_artifacts = self
            .read_all_artifacts_at_snapshot(current)?
            .into_iter()
            .map(|artifact| (artifact_key(&artifact), artifact))
            .collect::<BTreeMap<_, _>>();
        let current_artifacts = artifacts
            .iter()
            .cloned()
            .map(|artifact| (artifact_key(&artifact), artifact))
            .collect::<BTreeMap<_, _>>();
        let previous_artifacts_by_owner = artifacts_by_owner(previous_artifacts.values().cloned());
        let current_artifacts_by_owner = artifacts_by_owner(current_artifacts.values().cloned());
        let entries = entries
            .iter()
            .map(|entry| {
                let entry_to_commit = Self::entry_without_frozen_marker(entry);
                if entry.metadata.frozen.is_some()
                    && previous_entries.get(&entry.id) != Some(&entry_to_commit)
                {
                    return Err(FrostError::FrozenEntryChanged(entry.id.clone()));
                }
                if entry.metadata.frozen.is_some()
                    && previous_artifacts_by_owner
                        .get(&entry.id)
                        .map(Vec::as_slice)
                        .unwrap_or_default()
                        != current_artifacts_by_owner
                            .get(&entry.id)
                            .map(Vec::as_slice)
                            .unwrap_or_default()
                {
                    return Err(FrostError::FrozenEntryChanged(entry.id.clone()));
                }
                Ok(entry_to_commit)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let changed_entries = entries
            .iter()
            .filter(|entry| match previous_entries.get(&entry.id) {
                | Some(previous) => previous != *entry,
                | None => true,
            })
            .collect::<Vec<_>>();
        let lake_ids = entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        let deleted_ids = previous_entries
            .keys()
            .filter(|id| !lake_ids.contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        let changed_artifacts = current_artifacts
            .iter()
            .filter(|(key, artifact)| match previous_artifacts.get(key) {
                | Some(previous) => previous != *artifact,
                | None => true,
            })
            .map(|(_, artifact)| artifact)
            .collect::<Vec<_>>();
        let deleted_artifacts = previous_artifacts
            .keys()
            .filter(|key| !current_artifacts.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        let changed_artifact_owners = changed_artifacts
            .iter()
            .map(|artifact| artifact.owner.clone())
            .chain(deleted_artifacts.iter().map(|(owner, _)| owner.clone()))
            .collect::<BTreeSet<_>>();
        let changed_entry_addresses =
            changed_entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        let entries_to_write = entries
            .iter()
            .filter(|entry| {
                changed_entry_addresses.contains(&entry.id)
                    || changed_artifact_owners.contains(&entry.id)
            })
            .collect::<Vec<_>>();

        if changed_entries.is_empty()
            && deleted_ids.is_empty()
            && changed_artifacts.is_empty()
            && deleted_artifacts.is_empty()
        {
            return Ok(current);
        }

        let next = next_version(current);
        let artifact_snapshot_owners =
            entries_to_write.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        self.write_artifact_snapshot_directories(
            next,
            &artifact_snapshot_owners,
            &changed_artifacts,
        )?;

        let mut txn = self.backend.write();
        for entry in entries_to_write {
            let fs_id = entry.id.to_filesystem_id()?;
            let artifacts =
                current_artifacts_by_owner.get(&entry.id).map(Vec::as_slice).unwrap_or_default();
            txn =
                StoredEntryFacet::from_entry_with_artifacts(entry, artifacts).apply_to(txn, &fs_id);
        }
        for id in deleted_ids {
            let fs_id = id.to_filesystem_id()?;
            txn = txn.delete::<Lifecycle<EntryLifecycle>>(&fs_id);
        }
        Ok(txn.commit()?)
    }

    fn read_artifact_content_at_snapshot(
        &self, at: SnapshotRef, owner: &EntryAddress, path: &EntryArtifactPath,
    ) -> Result<Vec<u8>, FrostError> {
        Ok(fs::read(self.artifact_content_path_at_snapshot(at, owner, path)?)?)
    }

    fn artifact_content_path_at_snapshot(
        &self, at: SnapshotRef, owner: &EntryAddress, path: &EntryArtifactPath,
    ) -> Result<PathBuf, FrostError> {
        for (_, directory) in
            self.artifact_snapshot_directories_at(owner, at.eterator)?.into_iter().rev()
        {
            let artifact_path = directory.join(path.to_path_buf());
            if artifact_path.exists() {
                if !artifact_path.is_file() {
                    return Err(FrostError::CorruptArtifact {
                        owner: owner.clone(),
                        path: path.clone(),
                    });
                }
                return Ok(artifact_path);
            }
        }
        Err(FrostError::CorruptArtifact { owner: owner.clone(), path: path.clone() })
    }

    fn artifact_content_paths_at_snapshot(
        &self, at: SnapshotRef,
    ) -> Result<BTreeSet<PathBuf>, FrostError> {
        let mut paths = BTreeSet::new();
        if at.eterator == Eterator::EMPTY {
            return Ok(paths);
        }
        for fs_id in self.backend.live_entries(at)? {
            let owner = EntryAddress::try_from(fs_id.clone())?;
            let Some(facet) = StoredEntryFacet::load_from(&self.backend, at, &fs_id)? else {
                continue;
            };
            for path in facet.artifact_paths {
                paths.insert(self.artifact_content_path_at_snapshot(at, &owner, &path)?);
            }
        }
        Ok(paths)
    }

    fn gc_artifact_snapshot_directories(
        &self, live: SnapshotRef,
    ) -> Result<ArtifactGcReport, FrostError> {
        let keep = self.artifact_content_paths_at_snapshot(live)?;
        let mut report = ArtifactGcReport::default();
        for directory in self.artifact_snapshot_directories()? {
            for file in artifact_snapshot_files(&directory)? {
                if keep.contains(&file) {
                    continue;
                }
                fs::remove_file(file)?;
                report.files_removed += 1;
            }
            report.directories_removed += remove_empty_artifact_directories(&directory)?;
        }
        Ok(report)
    }

    fn artifact_snapshot_directories_at(
        &self, owner: &EntryAddress, at: Eterator,
    ) -> Result<Vec<(Eterator, PathBuf)>, FrostError> {
        let entry_root = Self::entry_storage_path(&self.root, owner)?;
        if !entry_root.exists() {
            return Ok(Vec::new());
        }

        let mut directories = Vec::new();
        for entry in fs::read_dir(&entry_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let version = parse_artifact_snapshot_directory_name(&name, owner)?;
            if version <= at {
                directories.push((version, entry.path()));
            }
        }
        directories.sort_by_key(|(version, _)| *version);
        Ok(directories)
    }

    fn artifact_snapshot_directories(&self) -> Result<Vec<PathBuf>, FrostError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }

        let mut directories = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let fs_id = FilesystemEntryId::new(entry.file_name().to_string_lossy().to_string())?;
            let owner = EntryAddress::try_from(fs_id)?;
            for child in fs::read_dir(entry.path())? {
                let child = child?;
                if !child.file_type()?.is_dir() {
                    continue;
                }
                let name = child.file_name().to_string_lossy().to_string();
                parse_artifact_snapshot_directory_name(&name, &owner)?;
                directories.push(child.path());
            }
        }
        directories.sort();
        Ok(directories)
    }

    fn write_artifact_snapshot_directories(
        &self, version: Eterator, owners: &BTreeSet<EntryAddress>,
        changed_artifacts: &[&EntryArtifact],
    ) -> Result<(), FrostError> {
        for owner in owners {
            let directory = Self::entry_artifact_snapshot_path(&self.root, owner, version)?;
            if directory.exists() {
                fs::remove_dir_all(&directory)?;
            }
        }

        for artifact in changed_artifacts {
            let directory =
                Self::entry_artifact_snapshot_path(&self.root, &artifact.owner, version)?;
            let path = directory.join(artifact.path.to_path_buf());
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, &artifact.content)?;
        }
        Ok(())
    }

    fn ensure_entry_matches_snapshot(
        &self, snapshot: SnapshotRef, entry: &Entry,
    ) -> Result<(), FrostError> {
        if self.read_entry_at_snapshot(snapshot, &entry.id)?.as_ref() != Some(entry) {
            return Err(FrostError::FrozenEntryChanged(entry.id.clone()));
        }
        Ok(())
    }

    fn entry_without_frozen_marker(entry: &Entry) -> Entry {
        let mut metadata = entry.metadata.clone();
        metadata.frozen = None;
        Entry::new(entry.id.clone(), metadata, entry.body.clone())
    }

    fn entries_without_generated_links(entries: &[Entry]) -> Result<Vec<Entry>, FrostError> {
        entries
            .iter()
            .map(|entry| {
                let body = GeneratedLinkBody::new(&entry.body)
                    .delete()
                    .map_err(EntryDirectoryError::from)?;
                let body = Self::strip_trailing_generated_link_divider(&body);
                Ok(Entry::new(entry.id.clone(), entry.metadata.clone(), body))
            })
            .collect()
    }

    fn strip_trailing_generated_link_divider(body: &str) -> String {
        body.strip_suffix("\n\n---\n")
            .map(|before| format!("{before}\n"))
            .unwrap_or_else(|| body.to_owned())
    }

    fn registry() -> eter::filesystem::FilesystemFieldRegistry {
        eter::filesystem::builtins_registry::<EntryLifecycle>()
            .with_field::<NameField>("name")
            .with_field::<DescField>("desc")
            .with_field::<StructuralField>("structural")
            .with_field::<ArtifactManifestField>("artifacts")
    }
    // sirno:witness:sirno-frost:end
}

// sirno:witness:entry-artifact:begin
fn artifact_key(artifact: &EntryArtifact) -> (EntryAddress, EntryArtifactPath) {
    (artifact.owner.clone(), artifact.path.clone())
}

fn artifacts_by_owner(
    artifacts: impl IntoIterator<Item = EntryArtifact>,
) -> BTreeMap<EntryAddress, Vec<EntryArtifact>> {
    let mut by_owner = BTreeMap::<EntryAddress, Vec<EntryArtifact>>::new();
    for artifact in artifacts {
        by_owner.entry(artifact.owner.clone()).or_default().push(artifact);
    }
    for artifacts in by_owner.values_mut() {
        artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    }
    by_owner
}

fn next_version(current: SnapshotRef) -> Eterator {
    Eterator(
        current.version().checked_add(1).unwrap_or_else(|| panic!("frost version space exhausted")),
    )
}

fn artifact_snapshot_directory_name(
    owner: &EntryAddress, version: Eterator,
) -> Result<String, FrostError> {
    Ok(format!("{:016x}-{}", version.version(), owner.to_filesystem_id()?.as_str()))
}

fn parse_artifact_snapshot_directory_name(
    name: &str, owner: &EntryAddress,
) -> Result<Eterator, FrostError> {
    let expected_suffix = format!("-{}", owner.to_filesystem_id()?.as_str());
    let Some(hex) = name.strip_suffix(&expected_suffix) else {
        return Err(FrostError::CorruptArtifactSnapshotDirectory(name.to_owned()));
    };
    if hex.len() != 16 {
        return Err(FrostError::CorruptArtifactSnapshotDirectory(name.to_owned()));
    }
    let version = u64::from_str_radix(hex, 16)
        .map_err(|_| FrostError::CorruptArtifactSnapshotDirectory(name.to_owned()))?;
    Ok(Eterator(version))
}

fn artifact_snapshot_files(directory: &Path) -> Result<Vec<PathBuf>, FrostError> {
    let mut files = Vec::new();
    collect_artifact_snapshot_files(directory, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_artifact_snapshot_files(
    directory: &Path, files: &mut Vec<PathBuf>,
) -> Result<(), FrostError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_artifact_snapshot_files(&entry.path(), files)?;
        } else if file_type.is_file() {
            files.push(entry.path());
        } else {
            return Err(FrostError::CorruptArtifactSnapshotDirectory(
                entry.path().display().to_string(),
            ));
        }
    }
    Ok(())
}

fn remove_empty_artifact_directories(directory: &Path) -> Result<usize, FrostError> {
    let mut removed = 0;
    let mut child_directories = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            child_directories.push(entry.path());
        }
    }
    for child in child_directories {
        removed += remove_empty_artifact_directories(&child)?;
    }
    if fs::read_dir(directory)?.next().transpose()?.is_none() {
        fs::remove_dir(directory)?;
        removed += 1;
    }
    Ok(removed)
}
// sirno:witness:entry-artifact:end

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredEntryFacet {
    name: Option<String>,
    desc: Option<String>,
    structural: EntryStructuralFields,
    artifact_paths: Vec<EntryArtifactPath>,
    body: Option<String>,
}

impl StoredEntryFacet {
    fn from_entry(entry: &Entry) -> Self {
        Self::from_entry_with_artifacts(entry, &[])
    }

    fn from_entry_with_artifacts(entry: &Entry, artifacts: &[EntryArtifact]) -> Self {
        Self {
            name: Some(entry.metadata.name.clone()),
            desc: Some(entry.metadata.desc.clone()),
            structural: entry.metadata.structural.clone(),
            artifact_paths: artifacts.iter().map(|artifact| artifact.path.clone()).collect(),
            body: Some(entry.body.clone()),
        }
    }

    fn into_entry(self, id: EntryAddress) -> Result<Entry, FrostError> {
        let name =
            self.name.ok_or_else(|| FrostError::CorruptEntry { id: id.clone(), field: "name" })?;
        let desc =
            self.desc.ok_or_else(|| FrostError::CorruptEntry { id: id.clone(), field: "desc" })?;
        let body =
            self.body.ok_or_else(|| FrostError::CorruptEntry { id: id.clone(), field: "body" })?;
        let mut metadata = EntryMetadata::new(name, desc)?;
        metadata.structural = self.structural;
        Ok(Entry::new(id, metadata, body))
    }

    fn resolve_optional_text<F: Field<Content = String>>(
        backend: &SirnoBackend, at: SnapshotRef, id: &FilesystemEntryId,
    ) -> Result<Option<String>, FilesystemError> {
        match backend.resolve::<F>(at, id)? {
            | Resolution::Content(value) => Ok(Some(value)),
            | Resolution::Deleted | Resolution::Absent => Ok(None),
        }
    }

    fn resolve_optional_structural(
        backend: &SirnoBackend, at: SnapshotRef, id: &FilesystemEntryId,
    ) -> Result<EntryStructuralFields, FilesystemError> {
        match backend.resolve::<StructuralField>(at, id)? {
            | Resolution::Content(value) => Ok(value),
            | Resolution::Deleted | Resolution::Absent => Ok(EntryStructuralFields::new()),
        }
    }

    fn resolve_artifact_paths(
        backend: &SirnoBackend, at: SnapshotRef, id: &FilesystemEntryId,
    ) -> Result<Vec<EntryArtifactPath>, FilesystemError> {
        match backend.resolve::<ArtifactManifestField>(at, id)? {
            | Resolution::Content(paths) => Ok(paths),
            | Resolution::Deleted | Resolution::Absent => Ok(Vec::new()),
        }
    }

    fn apply_structural<'a>(
        txn: SirnoWriteTxn<'a>, fs_id: &FilesystemEntryId, value: &EntryStructuralFields,
    ) -> SirnoWriteTxn<'a> {
        if value.is_empty() {
            txn.delete::<StructuralField>(fs_id)
        } else {
            txn.set::<StructuralField>(fs_id, value.clone())
        }
    }

    fn apply_artifacts<'a>(
        txn: SirnoWriteTxn<'a>, fs_id: &FilesystemEntryId, value: &[EntryArtifactPath],
    ) -> SirnoWriteTxn<'a> {
        if value.is_empty() {
            txn.delete::<ArtifactManifestField>(fs_id)
        } else {
            txn.set::<ArtifactManifestField>(fs_id, value.to_vec())
        }
    }

    fn required_text(value: &Option<String>, field: &'static str) -> String {
        value.clone().unwrap_or_else(|| {
            panic!("Sirno Frost entry facet is missing required `{field}` field")
        })
    }
}

impl EntryFacet<SirnoBackend> for StoredEntryFacet {
    fn load_from(
        backend: &SirnoBackend, at: SnapshotRef, id: &FilesystemEntryId,
    ) -> Result<Option<Self>, FilesystemError> {
        if !backend.entry_exists(at, id)? {
            return Ok(None);
        }

        Ok(Some(Self {
            name: Self::resolve_optional_text::<NameField>(backend, at, id)?,
            desc: Self::resolve_optional_text::<DescField>(backend, at, id)?,
            structural: Self::resolve_optional_structural(backend, at, id)?,
            artifact_paths: Self::resolve_artifact_paths(backend, at, id)?,
            body: match backend.resolve_body(at, id)? {
                | Resolution::Content(body) => Some(body),
                | Resolution::Deleted | Resolution::Absent => None,
            },
        }))
    }

    fn apply_to<'a>(&self, txn: SirnoWriteTxn<'a>, id: &FilesystemEntryId) -> SirnoWriteTxn<'a>
    where
        SirnoBackend: 'a,
    {
        let txn = txn
            .set::<Lifecycle<EntryLifecycle>>(id, EntryLifecycle::Active)
            .set::<NameField>(id, Self::required_text(&self.name, "name"))
            .set::<DescField>(id, Self::required_text(&self.desc, "desc"));

        let txn = Self::apply_structural(txn, id, &self.structural);
        let txn = Self::apply_artifacts(txn, id, &self.artifact_paths);
        txn.set_body(id, Self::required_text(&self.body, "body"))
    }
}

/// Error raised by Sirno Frost operations.
#[derive(Debug, Error)]
pub enum FrostError {
    /// The backend reported a filesystem backend error.
    #[error(transparent)]
    Filesystem(#[from] FilesystemError),
    /// Filesystem scanning failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Sirno Lake entry directory operation failed.
    #[error(transparent)]
    EntryDirectory(#[from] EntryDirectoryError),
    /// A filesystem directory cannot be interpreted as a Sirno entry address.
    #[error(transparent)]
    EntryAddress(#[from] EntryAddressError),
    /// A stored artifact path cannot be interpreted as a lake artifact path.
    #[error(transparent)]
    ArtifactPath(#[from] EntryArtifactPathError),
    /// Lake entry directory must pass review checks before frost commit.
    #[error("entry directory must pass review checks before frost commit: {0}")]
    InvalidEntryDirectory(PathBuf),
    /// Seed initialization would overwrite an existing entry.
    #[error("entry `{0}` already exists")]
    EntryAlreadyExists(EntryAddress),
    /// A frozen entry differs from the current frost snapshot.
    #[error(
        "entry `{0}` is frozen but does not match the current frost snapshot; run `sirno melt {0}` before changing it"
    )]
    FrozenEntryChanged(EntryAddress),
    /// A frozen entry is missing a required Sirno field.
    #[error("frozen entry `{id}` is missing required field `{field}`")]
    CorruptEntry {
        /// Entry containing the corrupt field state.
        id: EntryAddress,
        /// Field that could not be resolved.
        field: &'static str,
    },
    /// A frozen artifact is missing its stored byte content.
    #[error("frozen artifact `{owner}/{path}` is missing content")]
    CorruptArtifact {
        /// Entry that owns the corrupt artifact.
        owner: EntryAddress,
        /// Owner-relative artifact path.
        path: EntryArtifactPath,
    },
    /// A Sirno Frost artifact snapshot directory cannot be decoded.
    #[error("frozen artifact snapshot directory is corrupt: {0}")]
    CorruptArtifactSnapshotDirectory(String),
    /// Seed metadata could not be constructed.
    #[error(transparent)]
    EntryMetadata(#[from] crate::entry::EntryParseError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use crate::entry::FrozenMarker;
    use crate::lake::EntryDirectoryWritePolicy;
    use crate::render::GeneratedLinkBody;
    use crate::structural::{StructuralEdgeIndex, StructuralSettings};

    #[test]
    fn init_creates_ordinary_seed_entries() {
        let temp = tempfile::tempdir().unwrap();
        let mut frost = SirnoFrost::open(temp.path()).unwrap();

        frost.init_default_entries().unwrap();
        let entries = frost.read_all_entries().unwrap();
        let ids = entries.iter().map(|entry| entry.id.as_str()).collect::<Vec<_>>();

        assert_eq!(ids, ["category", "concept", "meta", "narrative"]);
        assert!(frost.check_current(CheckMode::Review).unwrap().is_clean());
    }

    #[test]
    fn put_and_read_entry_round_trips_metadata_and_body() {
        let temp = tempfile::tempdir().unwrap();
        let mut frost = SirnoFrost::open(temp.path()).unwrap();
        let mut metadata = EntryMetadata::new("Witness", "Repository evidence.").unwrap();
        metadata.push_structural_target("topic", EntryAddress::new("concept").unwrap());
        let entry = Entry::new(EntryAddress::new("witness").unwrap(), metadata, "Body.\n");

        frost.put_entry(&entry).unwrap();
        let read = frost.read_entry(&entry.id).unwrap().unwrap();

        assert_eq!(read, entry);
    }

    #[test]
    fn init_refuses_to_overwrite_existing_seed_entries() {
        let temp = tempfile::tempdir().unwrap();
        let mut frost = SirnoFrost::open(temp.path()).unwrap();

        frost.init_default_entries().unwrap();
        let error = frost.init_default_entries().unwrap_err();

        assert!(matches!(error, FrostError::EntryAlreadyExists(_)));
    }

    #[test]
    fn commit_entry_directory_round_trips_single_entry() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_lake_entry(lake.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let version = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let read = frost.read_entry_at_snapshot(version, &entry.id).unwrap();

        assert_eq!(read, Some(entry));
    }

    // sirno:witness:entry-artifact:begin
    #[test]
    fn commit_entry_directory_stores_artifacts_in_entry_version_directories() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_lake_entry(lake.path(), &entry);
        write_lake_artifact(lake.path(), &entry.id, "images/logo.bin", b"old");
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        write_lake_artifact(lake.path(), &entry.id, "images/logo.bin", b"new");
        let second = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let artifacts = frost.read_all_artifacts_at_snapshot(second).unwrap();

        assert_ne!(first, second);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &entry.id), [first, second]);
        let first_source =
            fs::read_to_string(entry_snapshot_file_path(frost_path.path(), &entry.id, first))
                .unwrap();
        assert!(first_source.contains("artifacts:"));
        assert!(first_source.contains("images/logo.bin"));
        assert_eq!(
            fs::read(artifact_snapshot_path(
                frost_path.path(),
                &entry.id,
                first,
                "images/logo.bin"
            ))
            .unwrap(),
            b"old"
        );
        assert_eq!(
            fs::read(artifact_snapshot_path(
                frost_path.path(),
                &entry.id,
                second,
                "images/logo.bin"
            ))
            .unwrap(),
            b"new"
        );
        assert!(
            fs::read_dir(frost_path.path()).unwrap().all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".artifact-"))
        );
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].owner, entry.id);
        assert_eq!(artifacts[0].path.as_str(), "images/logo.bin");
        assert_eq!(artifacts[0].content, b"new");
    }

    #[test]
    fn unchanged_artifact_content_is_inherited_from_older_entry_version_directory() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        write_lake_entry(lake.path(), &entry);
        write_lake_artifact(lake.path(), &entry.id, "images/logo.bin", b"old");
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        entry.body = "Alpha changed body.\n".to_owned();
        write_lake_entry(lake.path(), &entry);
        let second = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let artifacts = frost.read_all_artifacts_at_snapshot(second).unwrap();

        assert_ne!(first, second);
        assert!(
            !SirnoFrost::entry_artifact_snapshot_path(
                frost_path.path(),
                &entry.id,
                second.eterator
            )
            .unwrap()
            .exists()
        );
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].content, b"old");
    }

    #[test]
    fn gc_current_snapshot_keeps_latest_entries_and_inherited_artifacts() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        write_lake_entry(lake.path(), &entry);
        write_lake_artifact(lake.path(), &entry.id, "images/logo.bin", b"old");
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        entry.body = "Alpha changed body.\n".to_owned();
        write_lake_entry(lake.path(), &entry);
        let second = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let report = frost.gc_current_snapshot().unwrap();
        let versions = entry_snapshot_versions(frost_path.path(), &entry.id)
            .into_iter()
            .map(SnapshotRef::version)
            .collect::<Vec<_>>();
        let artifacts = frost.read_all_artifacts_at_snapshot(report.after).unwrap();

        assert!(report.collected());
        assert_eq!(report.before, second);
        assert!(report.after.generation > second.generation);
        assert_eq!(report.after.version(), second.version());
        assert_eq!(report.artifact_files_removed, 0);
        assert_eq!(report.artifact_directories_removed, 0);
        assert_eq!(versions, [second.version()]);
        assert_eq!(frost.read_entry_at_snapshot(report.after, &entry.id).unwrap(), Some(entry));
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].content, b"old");
        assert!(
            artifact_snapshot_path(
                frost_path.path(),
                &artifacts[0].owner,
                first,
                "images/logo.bin"
            )
            .exists()
        );
    }

    #[test]
    fn gc_current_snapshot_removes_superseded_artifact_bytes() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_lake_entry(lake.path(), &entry);
        write_lake_artifact(lake.path(), &entry.id, "images/logo.bin", b"old");
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        write_lake_artifact(lake.path(), &entry.id, "images/logo.bin", b"new");
        let second = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let first_artifact =
            artifact_snapshot_path(frost_path.path(), &entry.id, first, "images/logo.bin");
        let second_artifact =
            artifact_snapshot_path(frost_path.path(), &entry.id, second, "images/logo.bin");

        let report = frost.gc_current_snapshot().unwrap();
        let artifacts = frost.read_all_artifacts_at_snapshot(report.after).unwrap();

        assert!(report.collected());
        assert_eq!(report.artifact_files_removed, 1);
        assert_eq!(report.artifact_directories_removed, 2);
        assert!(!first_artifact.exists());
        assert!(!first_artifact.parent().unwrap().parent().unwrap().exists());
        assert!(second_artifact.exists());
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].content, b"new");
    }

    #[test]
    fn artifact_deletion_is_recorded_by_entry_manifest() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_lake_entry(lake.path(), &entry);
        write_lake_artifact(lake.path(), &entry.id, "images/logo.bin", b"old");
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        fs::remove_file(lake.path().join(".artifacts").join("alpha").join("images/logo.bin"))
            .unwrap();
        let second = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let artifact_path =
            artifact_snapshot_path(frost_path.path(), &entry.id, first, "images/logo.bin");

        assert_ne!(first, second);
        assert_eq!(frost.read_all_artifacts_at_snapshot(first).unwrap().len(), 1);
        assert!(frost.read_all_artifacts_at_snapshot(second).unwrap().is_empty());

        let report = frost.gc_current_snapshot().unwrap();
        assert_eq!(report.artifact_files_removed, 1);
        assert_eq!(report.artifact_directories_removed, 2);
        assert!(!artifact_path.exists());
    }
    // sirno:witness:entry-artifact:end

    #[test]
    fn checkout_preserves_structural_field_order_after_frost_round_trip() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let checkout = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        entry.metadata.push_structural_target("zeta", EntryAddress::new("concept").unwrap());
        entry.metadata.push_structural_target("area", EntryAddress::new("meta").unwrap());
        write_lake_entry(lake.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let version = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        frost
            .checkout_entry_directory(
                version,
                checkout.path(),
                EntryDirectoryWritePolicy::EmptyDirectory,
            )
            .unwrap();
        let source = fs::read_to_string(checkout.path().join("alpha.md")).unwrap();
        let read = Entry::from_markdown(entry.id.clone(), &source).unwrap();
        let fields = read.metadata.structural_fields().map(|(field, _)| field).collect::<Vec<_>>();

        assert_eq!(fields, ["zeta", "area"]);
        assert!(source.find("zeta:\n").unwrap() < source.find("area:\n").unwrap());
    }

    #[test]
    fn commit_entry_directory_strips_generated_links_from_frost() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        let footer = StructuralEdgeIndex::from_entries(std::slice::from_ref(&entry))
            .render_entry(&entry, &StructuralSettings::default());
        entry.body = GeneratedLinkBody::new(&entry.body).apply(&footer).unwrap();
        write_lake_entry(lake.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let version = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let read = frost.read_entry_at_snapshot(version, &entry.id).unwrap().unwrap();

        assert!(entry.body.contains(crate::BEGIN_LINKS_GUARD));
        assert_eq!(read.body, "Alpha body.\n");
    }

    #[test]
    fn commit_entry_directory_allows_current_frozen_entry() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_lake_entry(lake.path(), &alpha);
        write_lake_entry(lake.path(), &beta);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let mut frozen_alpha = alpha.clone();
        frozen_alpha.metadata.frozen = Some(FrozenMarker::Present);
        let mut changed_beta = beta.clone();
        changed_beta.body = "Beta changed body.\n".to_owned();
        write_lake_entry(lake.path(), &frozen_alpha);
        write_lake_entry(lake.path(), &changed_beta);
        let second = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_ne!(first, second);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &alpha.id), [first]);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &beta.id), [first, second]);
        assert_eq!(frost.read_entry_at_snapshot(second, &alpha.id).unwrap(), Some(alpha));
        assert_eq!(frost.read_entry_at_snapshot(second, &beta.id).unwrap(), Some(changed_beta));
    }

    #[test]
    fn commit_entry_directory_rejects_changed_frozen_entry() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_lake_entry(lake.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();
        frost.commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default()).unwrap();
        let mut changed_entry = entry.clone();
        changed_entry.metadata.frozen = Some(FrozenMarker::Present);
        changed_entry.body = "Changed body.\n".to_owned();
        write_lake_entry(lake.path(), &changed_entry);

        let error = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap_err();

        assert!(matches!(error, FrostError::FrozenEntryChanged(id) if id == entry.id));
    }

    #[test]
    fn commit_entry_directory_rejects_new_frozen_entry() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        entry.metadata.frozen = Some(FrozenMarker::Present);
        write_lake_entry(lake.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let error = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap_err();

        assert!(matches!(error, FrostError::FrozenEntryChanged(id) if id == entry.id));
    }

    #[test]
    fn put_entry_allows_current_frozen_entry() {
        let temp = tempfile::tempdir().unwrap();
        let mut frost = SirnoFrost::open(temp.path()).unwrap();
        let entry = test_entry("alpha", "Alpha");

        let first = frost.put_entry(&entry).unwrap();
        let mut frozen_entry = entry.clone();
        frozen_entry.metadata.frozen = Some(FrozenMarker::Present);
        let second = frost.put_entry(&frozen_entry).unwrap();

        assert_eq!(first, second);
        assert_eq!(frost.read_entry(&entry.id).unwrap(), Some(entry));
    }

    #[test]
    fn put_entry_rejects_changed_frozen_entry() {
        let temp = tempfile::tempdir().unwrap();
        let mut frost = SirnoFrost::open(temp.path()).unwrap();
        let entry = test_entry("alpha", "Alpha");
        frost.put_entry(&entry).unwrap();
        let mut changed_entry = entry.clone();
        changed_entry.metadata.frozen = Some(FrozenMarker::Present);
        changed_entry.body = "Changed body.\n".to_owned();

        let error = frost.put_entry(&changed_entry).unwrap_err();

        assert!(matches!(error, FrostError::FrozenEntryChanged(id) if id == entry.id));
    }

    #[test]
    fn multi_entry_commit_uses_one_snapshot() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_lake_entry(lake.path(), &alpha);
        write_lake_entry(lake.path(), &beta);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let version = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_eq!(frost.current_snapshot().unwrap(), version);
        assert_entry_snapshot_file(frost_path.path(), &alpha.id, version);
        assert_entry_snapshot_file(frost_path.path(), &beta.id, version);
    }

    #[test]
    fn changed_entry_commit_writes_only_changed_entry_snapshot() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_lake_entry(lake.path(), &alpha);
        write_lake_entry(lake.path(), &beta);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let mut changed_alpha = alpha.clone();
        changed_alpha.body = "Alpha changed body.\n".to_owned();
        write_lake_entry(lake.path(), &changed_alpha);
        let second = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_ne!(first, second);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &alpha.id), [first, second]);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &beta.id), [first]);
        assert_eq!(frost.read_entry_at_snapshot(second, &alpha.id).unwrap(), Some(changed_alpha));
        assert_eq!(frost.read_entry_at_snapshot(second, &beta.id).unwrap(), Some(beta));
    }

    #[test]
    fn no_op_commit_returns_current_snapshot() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_lake_entry(lake.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let second = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_eq!(first, second);
        assert_eq!(frost.current_snapshot().unwrap(), first);
    }

    #[test]
    fn removing_lake_entry_creates_frost_lifecycle_deletion() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_lake_entry(lake.path(), &alpha);
        write_lake_entry(lake.path(), &beta);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        fs::remove_file(lake.path().join("beta.md")).unwrap();
        let second = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_ne!(first, second);
        assert!(frost.read_entry_at_snapshot(first, &beta.id).unwrap().is_some());
        assert!(frost.read_entry_at_snapshot(second, &alpha.id).unwrap().is_some());
        assert_eq!(frost.read_entry_at_snapshot(second, &beta.id).unwrap(), None);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &alpha.id), [first]);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &beta.id), [first, second]);
    }

    #[test]
    fn checkout_entry_directory_materializes_frozen_state() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let checkout = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_lake_entry(lake.path(), &alpha);
        write_lake_entry(lake.path(), &beta);
        write_lake_artifact(lake.path(), &alpha.id, "notes.txt", b"artifact");
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        fs::remove_file(lake.path().join("beta.md")).unwrap();
        frost.commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default()).unwrap();
        frost
            .checkout_entry_directory(
                first,
                checkout.path(),
                EntryDirectoryWritePolicy::EmptyDirectory,
            )
            .unwrap();

        let checked = EntryDirectory::new(checkout.path())
            .check_with_settings(CheckMode::Review, &EntryDirectoryCheckSettings::default())
            .unwrap();
        assert_eq!(checked.entries(), &[alpha, beta]);
        assert_eq!(
            fs::read(checkout.path().join(".artifacts").join("alpha").join("notes.txt")).unwrap(),
            b"artifact"
        );
    }

    #[test]
    fn checkout_entry_directory_materializes_nested_entries_with_artifacts() {
        let lake = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let checkout = tempfile::tempdir().unwrap();
        let entry = test_entry("core.design", "Design");
        write_lake_entry(lake.path(), &entry);
        write_lake_artifact(lake.path(), &entry.id, "images/logo.bin", b"artifact");
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let version = frost
            .commit_entry_directory(lake.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        frost
            .checkout_entry_directory(
                version,
                checkout.path(),
                EntryDirectoryWritePolicy::EmptyDirectory,
            )
            .unwrap();

        let checked = EntryDirectory::new(checkout.path())
            .check_with_settings(CheckMode::Review, &EntryDirectoryCheckSettings::default())
            .unwrap();
        assert_eq!(checked.entries(), &[entry]);
        assert!(checkout.path().join("core/design.md").exists());
        assert_eq!(
            fs::read(
                checkout.path().join(".artifacts").join("core.design").join("images/logo.bin")
            )
            .unwrap(),
            b"artifact"
        );
    }

    fn test_entry(id: &str, name: &str) -> Entry {
        let metadata = EntryMetadata::new(name, format!("{name} desc.")).expect("valid metadata");
        Entry::new(EntryAddress::new(id).expect("valid id"), metadata, format!("{name} body.\n"))
    }

    fn write_lake_entry(root: &Path, entry: &Entry) {
        let path = root.join(entry.id.to_lake_relative_path());
        fs::create_dir_all(path.parent().expect("entry has parent")).unwrap();
        fs::write(path, entry.to_markdown().expect("render entry")).expect("write entry");
    }

    fn write_lake_artifact(root: &Path, owner: &EntryAddress, path: &str, content: &[u8]) {
        let path = root.join(".artifacts").join(owner.as_str()).join(path);
        fs::create_dir_all(path.parent().expect("artifact has parent")).unwrap();
        fs::write(path, content).unwrap();
    }

    fn assert_entry_snapshot_file(root: &Path, id: &EntryAddress, snapshot: SnapshotRef) {
        let versions = entry_snapshot_versions(root, id);
        assert_eq!(versions, [snapshot]);
    }

    fn entry_snapshot_versions(root: &Path, id: &EntryAddress) -> Vec<SnapshotRef> {
        let dir = root.join(id.as_str());
        let mut versions = fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                if !entry.file_type().unwrap().is_file() {
                    return None;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".md") {
                    return None;
                }
                let version = u64::from_str_radix(&name[..16], 16).unwrap();
                Some(SnapshotRef::new(eter::GcGeneration::INITIAL, Eterator(version)))
            })
            .collect::<Vec<_>>();
        versions.sort();
        versions
    }

    fn artifact_snapshot_path(
        root: &Path, id: &EntryAddress, snapshot: SnapshotRef, artifact: &str,
    ) -> PathBuf {
        SirnoFrost::entry_artifact_snapshot_path(root, id, snapshot.eterator)
            .unwrap()
            .join(artifact)
    }

    fn entry_snapshot_file_path(root: &Path, id: &EntryAddress, snapshot: SnapshotRef) -> PathBuf {
        root.join(id.as_str()).join(format!("{:016x}-{}.md", snapshot.version(), id.as_str()))
    }
}
