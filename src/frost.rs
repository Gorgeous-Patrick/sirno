//! Sirno Frost facade.
//!
//! Sirno Frost exposes frozen snapshots as typed Sirno entries.
//! The current backend uses `eter` filesystem snapshots as durable storage.
//! That layout is private to this module.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use eter::filesystem::{FilesystemBackend, FilesystemEntryId, FilesystemError, FilesystemWriteTxn};
use eter::{
    EntryFacet, Eter, Eterator, Field, Lifecycle, LiveEntries, Resolution, SnapshotRef, WriteTxn,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::check::{CheckMode, CheckReport};
use crate::entry::{Entry, EntryMetadata, EntryStructuralFields};
use crate::id::{EntryId, EntryIdError};
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

    /// The private Frost backend path.
    pub fn root(&self) -> &Path {
        &self.root
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

    /// Write or replace one entry.
    // sirno:witness:sirno-frost:begin
    pub fn put_entry(&mut self, entry: &Entry) -> Result<SnapshotRef, FrostError> {
        trace!("sirno put_entry begin: id={}", entry.id);
        Self::reject_frozen_entries(std::slice::from_ref(entry))?;
        let fs_id = entry.id.to_filesystem_id()?;
        let facet = StoredEntryFacet::from_entry(entry);
        let snapshot = facet.apply_to(self.backend.write(), &fs_id).commit()?;
        trace!("sirno put_entry end: version={}", snapshot.version());
        Ok(snapshot)
    }
    // sirno:witness:sirno-frost:end

    /// Read one entry at the current snapshot.
    pub fn read_entry(&self, id: &EntryId) -> Result<Option<Entry>, FrostError> {
        self.read_entry_at_snapshot(self.current_snapshot()?, id)
    }

    /// Read one entry at a selected frozen snapshot.
    // sirno:witness:sirno-frost:begin
    pub fn read_entry_at_snapshot(
        &self, at: SnapshotRef, id: &EntryId,
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
            let id = EntryId::try_from(fs_id)?;
            if let Some(entry) = self.read_entry_at_snapshot(at, &id)? {
                entries.push(entry);
            }
        }
        trace!("sirno read_all_entries end: entries={}", entries.len());
        Ok(entries)
    }
    // sirno:witness:sirno-frost:end

    /// Check current entries at the selected boundary.
    pub fn check_current(&self, mode: CheckMode) -> Result<CheckReport, FrostError> {
        let entries = self.read_all_entries()?;
        Ok(mode.check_entries(&entries, &StructuralSettings::default()))
    }

    /// Freeze a public Markdown entry directory into Sirno Frost.
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
        let version = self.commit_entries(&entries)?;
        trace!("sirno commit_entry_directory end: version={}", version.version());
        Ok(version)
    }
    // sirno:witness:sirno-frost:end

    /// Materialize a frozen snapshot into a public Markdown entry directory.
    // sirno:witness:sirno-frost:begin
    pub fn checkout_entry_directory(
        &self, at: SnapshotRef, root: impl Into<PathBuf>, policy: EntryDirectoryWritePolicy,
    ) -> Result<Vec<PathBuf>, FrostError> {
        let root = root.into();
        trace!("sirno checkout_entry_directory begin: at={} root={}", at.version(), root.display());
        let entries = self.read_all_entries_at_snapshot(at)?;
        let paths = EntryDirectory::new(&root).write(&entries, policy)?;
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
        Self::reject_frozen_entries(entries)?;
        let current = self.current_snapshot()?;
        let previous_entries = self
            .read_all_entries_at_snapshot(current)?
            .into_iter()
            .map(|entry| (entry.id.clone(), entry))
            .collect::<BTreeMap<_, _>>();
        let changed_entries = entries
            .iter()
            .filter(|entry| match previous_entries.get(&entry.id) {
                | Some(previous) => previous != *entry,
                | None => true,
            })
            .collect::<Vec<_>>();
        let public_ids = entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        let deleted_ids = previous_entries
            .keys()
            .filter(|id| !public_ids.contains(*id))
            .cloned()
            .collect::<Vec<_>>();

        if changed_entries.is_empty() && deleted_ids.is_empty() {
            return Ok(current);
        }

        let mut txn = self.backend.write();
        for entry in changed_entries {
            let fs_id = entry.id.to_filesystem_id()?;
            txn = StoredEntryFacet::from_entry(entry).apply_to(txn, &fs_id);
        }
        for id in deleted_ids {
            let fs_id = id.to_filesystem_id()?;
            txn = txn.delete::<Lifecycle<EntryLifecycle>>(&fs_id);
        }
        Ok(txn.commit()?)
    }

    fn reject_frozen_entries(entries: &[Entry]) -> Result<(), FrostError> {
        if let Some(entry) = entries.iter().find(|entry| entry.metadata.frozen.is_some()) {
            return Err(FrostError::FrozenEntryCommit(entry.id.clone()));
        }
        Ok(())
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
    }
    // sirno:witness:sirno-frost:end
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredEntryFacet {
    name: Option<String>,
    desc: Option<String>,
    structural: EntryStructuralFields,
    body: Option<String>,
}

impl StoredEntryFacet {
    fn from_entry(entry: &Entry) -> Self {
        Self {
            name: Some(entry.metadata.name.clone()),
            desc: Some(entry.metadata.desc.clone()),
            structural: entry.metadata.structural.clone(),
            body: Some(entry.body.clone()),
        }
    }

    fn into_entry(self, id: EntryId) -> Result<Entry, FrostError> {
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

    fn apply_structural<'a>(
        txn: SirnoWriteTxn<'a>, fs_id: &FilesystemEntryId, value: &EntryStructuralFields,
    ) -> SirnoWriteTxn<'a> {
        if value.is_empty() {
            txn.delete::<StructuralField>(fs_id)
        } else {
            txn.set::<StructuralField>(fs_id, value.clone())
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
    /// Public Markdown entry directory operation failed.
    #[error(transparent)]
    EntryDirectory(#[from] EntryDirectoryError),
    /// A filesystem directory cannot be interpreted as a Sirno entry id.
    #[error(transparent)]
    EntryId(#[from] EntryIdError),
    /// Public Markdown entry directory must pass review checks before Frost commit.
    #[error("entry directory must pass review checks before Frost commit: {0}")]
    InvalidEntryDirectory(PathBuf),
    /// Seed initialization would overwrite an existing entry.
    #[error("entry `{0}` already exists")]
    EntryAlreadyExists(EntryId),
    /// A frozen entry cannot be committed to Sirno Frost.
    #[error("entry `{0}` is frozen; run `sirno melt {0}` before Frost commit")]
    FrozenEntryCommit(EntryId),
    /// A frozen entry is missing a required Sirno field.
    #[error("frozen entry `{id}` is missing required field `{field}`")]
    CorruptEntry {
        /// Entry containing the corrupt field state.
        id: EntryId,
        /// Field that could not be resolved.
        field: &'static str,
    },
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
        metadata.push_structural_target("topic", EntryId::new("concept").unwrap());
        let entry = Entry::new(EntryId::new("witness").unwrap(), metadata, "Body.\n");

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
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_public_entry(public.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let version = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let read = frost.read_entry_at_snapshot(version, &entry.id).unwrap();

        assert_eq!(read, Some(entry));
    }

    #[test]
    fn checkout_preserves_structural_field_order_after_frost_round_trip() {
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let checkout = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        entry.metadata.push_structural_target("zeta", EntryId::new("concept").unwrap());
        entry.metadata.push_structural_target("area", EntryId::new("meta").unwrap());
        write_public_entry(public.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let version = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
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
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        let footer = StructuralEdgeIndex::from_entries(std::slice::from_ref(&entry))
            .render_entry(&entry, &StructuralSettings::default());
        entry.body = GeneratedLinkBody::new(&entry.body).apply(&footer).unwrap();
        write_public_entry(public.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let version = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let read = frost.read_entry_at_snapshot(version, &entry.id).unwrap().unwrap();

        assert!(entry.body.contains(crate::BEGIN_LINKS_GUARD));
        assert_eq!(read.body, "Alpha body.\n");
    }

    #[test]
    fn commit_entry_directory_rejects_frozen_entry() {
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        entry.metadata.frozen = Some(FrozenMarker::Present);
        write_public_entry(public.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let error = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap_err();

        assert!(matches!(error, FrostError::FrozenEntryCommit(id) if id == entry.id));
    }

    #[test]
    fn put_entry_rejects_frozen_entry() {
        let temp = tempfile::tempdir().unwrap();
        let mut frost = SirnoFrost::open(temp.path()).unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        entry.metadata.frozen = Some(FrozenMarker::Present);

        let error = frost.put_entry(&entry).unwrap_err();

        assert!(matches!(error, FrostError::FrozenEntryCommit(id) if id == entry.id));
    }

    #[test]
    fn multi_entry_commit_uses_one_snapshot() {
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_public_entry(public.path(), &alpha);
        write_public_entry(public.path(), &beta);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let version = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_eq!(frost.current_snapshot().unwrap(), version);
        assert_entry_snapshot_file(frost_path.path(), &alpha.id, version);
        assert_entry_snapshot_file(frost_path.path(), &beta.id, version);
    }

    #[test]
    fn changed_entry_commit_writes_only_changed_entry_snapshot() {
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_public_entry(public.path(), &alpha);
        write_public_entry(public.path(), &beta);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let mut changed_alpha = alpha.clone();
        changed_alpha.body = "Alpha changed body.\n".to_owned();
        write_public_entry(public.path(), &changed_alpha);
        let second = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_ne!(first, second);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &alpha.id), [first, second]);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &beta.id), [first]);
        assert_eq!(frost.read_entry_at_snapshot(second, &alpha.id).unwrap(), Some(changed_alpha));
        assert_eq!(frost.read_entry_at_snapshot(second, &beta.id).unwrap(), Some(beta));
    }

    #[test]
    fn no_op_commit_returns_current_snapshot() {
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_public_entry(public.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let second = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_eq!(first, second);
        assert_eq!(frost.current_snapshot().unwrap(), first);
    }

    #[test]
    fn removing_public_entry_creates_frost_lifecycle_deletion() {
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_public_entry(public.path(), &alpha);
        write_public_entry(public.path(), &beta);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        fs::remove_file(public.path().join("beta.md")).unwrap();
        let second = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
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
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let checkout = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_public_entry(public.path(), &alpha);
        write_public_entry(public.path(), &beta);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        fs::remove_file(public.path().join("beta.md")).unwrap();
        frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
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
    }

    fn test_entry(id: &str, name: &str) -> Entry {
        let metadata = EntryMetadata::new(name, format!("{name} desc.")).expect("valid metadata");
        Entry::new(EntryId::new(id).expect("valid id"), metadata, format!("{name} body.\n"))
    }

    fn write_public_entry(root: &Path, entry: &Entry) {
        let path = root.join(format!("{}.md", entry.id.as_str()));
        fs::write(path, entry.to_markdown().expect("render entry")).expect("write entry");
    }

    fn assert_entry_snapshot_file(root: &Path, id: &EntryId, snapshot: SnapshotRef) {
        let versions = entry_snapshot_versions(root, id);
        assert_eq!(versions, [snapshot]);
    }

    fn entry_snapshot_versions(root: &Path, id: &EntryId) -> Vec<SnapshotRef> {
        let dir = root.join(id.as_str());
        let mut versions = fs::read_dir(dir)
            .unwrap()
            .map(|entry| {
                let name = entry.unwrap().file_name().to_string_lossy().to_string();
                let version = u64::from_str_radix(&name[..16], 16).unwrap();
                SnapshotRef::new(eter::GcGeneration::INITIAL, Eterator(version))
            })
            .collect::<Vec<_>>();
        versions.sort();
        versions
    }
}
