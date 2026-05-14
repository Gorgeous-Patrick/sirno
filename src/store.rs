//! Sirno Lake facade.
//!
//! The facade exposes typed Sirno entries.
//! The current backend uses `eter` filesystem snapshots as durable storage.
//! That layout is private to this module.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use eter::filesystem::{FilesystemBackend, FilesystemEntryId, FilesystemError, FilesystemWriteTxn};
use eter::{
    EntryFacet, EntryFacetStoreExt, Eter, Eterator, Field, Lifecycle, LiveEntries, Resolution,
    SnapshotRef, WriteTxn,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::check::{CheckMode, CheckReport, check_entries};
use crate::entry::{Entry, EntryMetadata, WitnessMarker, default_seed_entries};
use crate::files::{
    EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryWritePolicy,
    check_entry_directory_with_settings, write_entry_directory,
};
use crate::id::{EntryId, EntryIdError};
use crate::links::delete_generated_links;

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

struct DescriptionField;
impl Field for DescriptionField {
    type Content = String;
}

struct CategoryField;
impl Field for CategoryField {
    type Content = Vec<EntryId>;
}

struct ClusteeField;
impl Field for ClusteeField {
    type Content = Vec<EntryId>;
}

struct RefinerField;
impl Field for RefinerField {
    type Content = Vec<EntryId>;
}

struct WitnessField;
impl Field for WitnessField {
    type Content = WitnessMarker;
}

/// History store facade for Sirno entries.
///
/// Invariant: all entries written through this type are represented through
/// typed metadata fields and a Markdown body in the configured `eter` backend.
#[derive(Debug)]
// sirno:witness:history-store:begin
pub struct SirnoStore {
    root: PathBuf,
    backend: SirnoBackend,
}
// sirno:witness:history-store:end

impl SirnoStore {
    /// Open or initialize a history store rooted at `root`.
    // sirno:witness:history-store:begin
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        trace!("sirno history store open begin");
        let root = root.into();
        let backend = SirnoBackend::open(&root, sirno_registry())?;
        trace!("sirno history store open end");
        Ok(Self { root, backend })
    }
    // sirno:witness:history-store:end

    /// The root path used by this lake.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Return the current backend snapshot reference.
    // sirno:witness:history-store:begin
    pub fn current_snapshot(&self) -> Result<SnapshotRef, StoreError> {
        Ok(self.backend.current_snapshot()?)
    }
    // sirno:witness:history-store:end

    /// Return the current backend snapshot version coordinate.
    // sirno:witness:history-store:begin
    pub fn current_version(&self) -> Result<Eterator, StoreError> {
        Ok(self.backend.current_version()?)
    }
    // sirno:witness:history-store:end

    /// Pair a version coordinate with the current backend GC generation.
    ///
    /// `eter` rejects stale snapshot references.
    /// Sirno exposes version coordinates at the CLI and resolves them against the current generation.
    // sirno:witness:history-store:begin
    pub fn snapshot_for_version(&self, version: Eterator) -> Result<SnapshotRef, StoreError> {
        Ok(SnapshotRef::new(self.backend.gc_generation()?, version))
    }
    // sirno:witness:history-store:end

    /// Write or replace one entry.
    // sirno:witness:history-store:begin
    pub fn put_entry(&mut self, entry: &Entry) -> Result<SnapshotRef, StoreError> {
        trace!("sirno put_entry begin: id={}", entry.id);
        let fs_id = entry.id.to_filesystem_id()?;
        let facet = StoredEntryFacet::from_entry(entry);
        let snapshot = self.backend.write_facet(&fs_id, &facet)?;
        trace!("sirno put_entry end: version={}", snapshot.version());
        Ok(snapshot)
    }
    // sirno:witness:history-store:end

    /// Read one entry at the current snapshot.
    pub fn read_entry(&self, id: &EntryId) -> Result<Option<Entry>, StoreError> {
        self.read_entry_at_snapshot(self.current_snapshot()?, id)
    }

    /// Read one entry at a selected history snapshot.
    // sirno:witness:history-store:begin
    pub fn read_entry_at_snapshot(
        &self, at: SnapshotRef, id: &EntryId,
    ) -> Result<Option<Entry>, StoreError> {
        trace!("sirno read_entry_at begin: id={id} at={}", at.version());
        let fs_id = id.to_filesystem_id()?;
        let Some(facet) = self.backend.load_facet::<StoredEntryFacet>(at, &fs_id)? else {
            trace!("sirno read_entry_at end: absent");
            return Ok(None);
        };
        let entry = facet.into_entry(id.clone())?;
        trace!("sirno read_entry_at end: present");
        Ok(Some(entry))
    }
    // sirno:witness:history-store:end

    /// Read every active entry at the current snapshot.
    pub fn read_all_entries(&self) -> Result<Vec<Entry>, StoreError> {
        self.read_all_entries_at_snapshot(self.current_snapshot()?)
    }

    /// Read every active entry at a selected history snapshot.
    // sirno:witness:history-store:begin
    pub fn read_all_entries_at_snapshot(&self, at: SnapshotRef) -> Result<Vec<Entry>, StoreError> {
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
    // sirno:witness:history-store:end

    /// Check current entries at the selected boundary.
    pub fn check_current(&self, mode: CheckMode) -> Result<CheckReport, StoreError> {
        let entries = self.read_all_entries()?;
        Ok(check_entries(&entries, mode))
    }

    /// Commit a public Markdown entry directory into this history store.
    ///
    /// The directory must pass review-mode checks before any history row is written.
    /// Generated-link regions are stripped from the committed snapshot.
    // sirno:witness:history-store:begin
    pub fn commit_entry_directory(
        &mut self, root: impl Into<PathBuf>, settings: &EntryDirectoryCheckSettings,
    ) -> Result<SnapshotRef, StoreError> {
        let root = root.into();
        trace!("sirno commit_entry_directory begin: root={}", root.display());
        let report = check_entry_directory_with_settings(&root, CheckMode::Review, settings)?;
        if report.has_errors() {
            return Err(StoreError::InvalidEntryDirectory(root));
        }
        let entries = entries_without_generated_links(report.entries())?;
        let version = self.commit_entries(&entries)?;
        trace!("sirno commit_entry_directory end: version={}", version.version());
        Ok(version)
    }
    // sirno:witness:history-store:end

    /// Materialize a history snapshot into a public Markdown entry directory.
    // sirno:witness:history-store:begin
    pub fn checkout_entry_directory(
        &self, at: SnapshotRef, root: impl Into<PathBuf>, policy: EntryDirectoryWritePolicy,
    ) -> Result<Vec<PathBuf>, StoreError> {
        let root = root.into();
        trace!("sirno checkout_entry_directory begin: at={} root={}", at.version(), root.display());
        let entries = self.read_all_entries_at_snapshot(at)?;
        let paths = write_entry_directory(&root, &entries, policy)?;
        trace!("sirno checkout_entry_directory end: entries={}", paths.len());
        Ok(paths)
    }
    // sirno:witness:history-store:end

    /// Initialize ordinary seed entries.
    ///
    /// The initialized entries are ordinary Sirno entries.
    /// They are created together and are not privileged by later operations.
    // sirno:witness:history-store:begin
    pub fn init_default_entries(&mut self) -> Result<SnapshotRef, StoreError> {
        trace!("sirno init_default_entries begin");
        let entries = default_seed_entries()?;
        for entry in &entries {
            let fs_id = entry.id.to_filesystem_id()?;
            if self.backend.entry_id_in_use(&fs_id)? {
                return Err(StoreError::EntryAlreadyExists(entry.id.clone()));
            }
        }
        let version = self.commit_entries(&entries)?;
        trace!("sirno init_default_entries end: version={}", version.version());
        Ok(version)
    }
    // sirno:witness:history-store:end

    // sirno:witness:history-store:begin
    fn commit_entries(&mut self, entries: &[Entry]) -> Result<SnapshotRef, StoreError> {
        let current = self.current_snapshot()?;
        if self.read_all_entries_at_snapshot(current)? == entries {
            return Ok(current);
        }

        let public_ids = entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        let previous_ids = self
            .backend
            .live_entries(current)?
            .into_iter()
            .map(EntryId::try_from)
            .collect::<Result<BTreeSet<_>, _>>()?;

        let mut txn = self.backend.write();
        for entry in entries {
            let fs_id = entry.id.to_filesystem_id()?;
            txn = StoredEntryFacet::from_entry(entry).apply_to(txn, &fs_id);
        }
        for id in previous_ids.difference(&public_ids) {
            let fs_id = id.to_filesystem_id()?;
            txn = txn.delete::<Lifecycle<EntryLifecycle>>(&fs_id);
        }
        Ok(txn.commit()?)
    }
    // sirno:witness:history-store:end
}

fn entries_without_generated_links(entries: &[Entry]) -> Result<Vec<Entry>, StoreError> {
    entries
        .iter()
        .map(|entry| {
            let body = delete_generated_links(&entry.body).map_err(EntryDirectoryError::from)?;
            let body = strip_trailing_generated_link_divider(&body);
            Ok(Entry::new(entry.id.clone(), entry.metadata.clone(), body))
        })
        .collect()
}

fn strip_trailing_generated_link_divider(body: &str) -> String {
    body.strip_suffix("\n\n---\n")
        .map(|before| format!("{before}\n"))
        .unwrap_or_else(|| body.to_owned())
}

fn sirno_registry() -> eter::filesystem::FilesystemFieldRegistry {
    eter::filesystem::builtins_registry::<EntryLifecycle>()
        .with_field::<NameField>("name")
        .with_field::<DescriptionField>("description")
        .with_field::<CategoryField>("category")
        .with_field::<ClusteeField>("clustee")
        .with_field::<RefinerField>("refiner")
        .with_field::<WitnessField>("witness")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredEntryFacet {
    name: Option<String>,
    description: Option<String>,
    category: Vec<EntryId>,
    clustee: Vec<EntryId>,
    refiner: Vec<EntryId>,
    witness: Option<WitnessMarker>,
    body: Option<String>,
}

impl StoredEntryFacet {
    fn from_entry(entry: &Entry) -> Self {
        Self {
            name: Some(entry.metadata.name.clone()),
            description: Some(entry.metadata.description.clone()),
            category: entry.metadata.category.clone(),
            clustee: entry.metadata.clustee.clone(),
            refiner: entry.metadata.refiner.clone(),
            witness: entry.metadata.witness,
            body: Some(entry.body.clone()),
        }
    }

    fn into_entry(self, id: EntryId) -> Result<Entry, StoreError> {
        let name =
            self.name.ok_or_else(|| StoreError::CorruptEntry { id: id.clone(), field: "name" })?;
        let description = self
            .description
            .ok_or_else(|| StoreError::CorruptEntry { id: id.clone(), field: "description" })?;
        let body =
            self.body.ok_or_else(|| StoreError::CorruptEntry { id: id.clone(), field: "body" })?;
        let mut metadata = EntryMetadata::new(name, description)?;
        metadata.category = self.category;
        metadata.clustee = self.clustee;
        metadata.refiner = self.refiner;
        metadata.witness = self.witness;
        Ok(Entry::new(id, metadata, body))
    }
}

impl EntryFacet<SirnoBackend> for StoredEntryFacet {
    fn load_from(
        store: &SirnoBackend, at: SnapshotRef, id: &FilesystemEntryId,
    ) -> Result<Option<Self>, FilesystemError> {
        if !store.entry_exists(at, id)? {
            return Ok(None);
        }

        Ok(Some(Self {
            name: resolve_optional_text::<NameField>(store, at, id)?,
            description: resolve_optional_text::<DescriptionField>(store, at, id)?,
            category: resolve_optional_list::<CategoryField>(store, at, id)?,
            clustee: resolve_optional_list::<ClusteeField>(store, at, id)?,
            refiner: resolve_optional_list::<RefinerField>(store, at, id)?,
            witness: match store.resolve::<WitnessField>(at, id)? {
                | Resolution::Content(marker) => Some(marker),
                | Resolution::Deleted | Resolution::Absent => None,
            },
            body: match store.resolve_body(at, id)? {
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
            .set::<NameField>(id, required_facet_text(&self.name, "name"))
            .set::<DescriptionField>(id, required_facet_text(&self.description, "description"));

        let txn = apply_optional_list::<CategoryField>(txn, id, &self.category);
        let txn = apply_optional_list::<ClusteeField>(txn, id, &self.clustee);
        let txn = apply_optional_list::<RefinerField>(txn, id, &self.refiner);
        let txn = match self.witness {
            | Some(marker) => txn.set::<WitnessField>(id, marker),
            | None => txn.delete::<WitnessField>(id),
        };
        txn.set_body(id, required_facet_text(&self.body, "body"))
    }
}

fn resolve_optional_text<F: Field<Content = String>>(
    store: &SirnoBackend, at: SnapshotRef, id: &FilesystemEntryId,
) -> Result<Option<String>, FilesystemError> {
    match store.resolve::<F>(at, id)? {
        | Resolution::Content(value) => Ok(Some(value)),
        | Resolution::Deleted | Resolution::Absent => Ok(None),
    }
}

fn resolve_optional_list<F: Field<Content = Vec<EntryId>>>(
    store: &SirnoBackend, at: SnapshotRef, id: &FilesystemEntryId,
) -> Result<Vec<EntryId>, FilesystemError> {
    match store.resolve::<F>(at, id)? {
        | Resolution::Content(value) => Ok(value),
        | Resolution::Deleted | Resolution::Absent => Ok(Vec::new()),
    }
}

fn apply_optional_list<'a, F>(
    txn: SirnoWriteTxn<'a>, fs_id: &FilesystemEntryId, value: &[EntryId],
) -> SirnoWriteTxn<'a>
where
    F: Field<Content = Vec<EntryId>>,
{
    if value.is_empty() { txn.delete::<F>(fs_id) } else { txn.set::<F>(fs_id, value.to_vec()) }
}

fn required_facet_text(value: &Option<String>, field: &'static str) -> String {
    value
        .clone()
        .unwrap_or_else(|| panic!("stored Sirno entry facet is missing required `{field}` field"))
}

/// Error raised by Sirno Lake operations.
#[derive(Debug, Error)]
pub enum StoreError {
    /// The backend reported a filesystem-store error.
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
    /// Public Markdown entry directory must pass review checks before history commit.
    #[error("entry directory must pass review checks before history commit: {0}")]
    InvalidEntryDirectory(PathBuf),
    /// Seed initialization would overwrite an existing entry.
    #[error("entry `{0}` already exists")]
    EntryAlreadyExists(EntryId),
    /// A stored entry is missing a required Sirno field.
    #[error("stored entry `{id}` is missing required field `{field}`")]
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

    use crate::files::EntryDirectoryWritePolicy;
    use crate::links::{GeneratedLinkIndex, GeneratedLinkSettings, apply_generated_links};

    #[test]
    fn init_creates_ordinary_seed_entries() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = SirnoStore::open(temp.path()).unwrap();

        store.init_default_entries().unwrap();
        let entries = store.read_all_entries().unwrap();
        let ids = entries.iter().map(|entry| entry.id.as_str()).collect::<Vec<_>>();

        assert_eq!(ids, ["concept", "meta", "narrative"]);
        assert!(store.check_current(CheckMode::Review).unwrap().is_clean());
    }

    #[test]
    fn put_and_read_entry_round_trips_metadata_and_body() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = SirnoStore::open(temp.path()).unwrap();
        let mut metadata = EntryMetadata::new("Witness", "Repository evidence.").unwrap();
        metadata.category.push(EntryId::new("concept").unwrap());
        metadata.witness = Some(WitnessMarker::Present);
        let entry = Entry::new(EntryId::new("witness").unwrap(), metadata, "Body.\n");

        store.put_entry(&entry).unwrap();
        let read = store.read_entry(&entry.id).unwrap().unwrap();

        assert_eq!(read, entry);
    }

    #[test]
    fn init_refuses_to_overwrite_existing_seed_entries() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = SirnoStore::open(temp.path()).unwrap();

        store.init_default_entries().unwrap();
        let error = store.init_default_entries().unwrap_err();

        assert!(matches!(error, StoreError::EntryAlreadyExists(_)));
    }

    #[test]
    fn commit_entry_directory_round_trips_single_entry() {
        let public = tempfile::tempdir().unwrap();
        let history = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_public_entry(public.path(), &entry);
        let mut store = SirnoStore::open(history.path()).unwrap();

        let version = store
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let read = store.read_entry_at_snapshot(version, &entry.id).unwrap();

        assert_eq!(read, Some(entry));
    }

    #[test]
    fn commit_entry_directory_strips_generated_links_from_history() {
        let public = tempfile::tempdir().unwrap();
        let history = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        let footer = GeneratedLinkIndex::from_entries(std::slice::from_ref(&entry))
            .render_entry(&entry, &GeneratedLinkSettings::default());
        entry.body = apply_generated_links(&entry.body, &footer).unwrap();
        write_public_entry(public.path(), &entry);
        let mut store = SirnoStore::open(history.path()).unwrap();

        let version = store
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let read = store.read_entry_at_snapshot(version, &entry.id).unwrap().unwrap();

        assert!(entry.body.contains(crate::BEGIN_LINKS_GUARD));
        assert_eq!(read.body, "Alpha body.\n");
    }

    #[test]
    fn multi_entry_commit_uses_one_snapshot() {
        let public = tempfile::tempdir().unwrap();
        let history = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_public_entry(public.path(), &alpha);
        write_public_entry(public.path(), &beta);
        let mut store = SirnoStore::open(history.path()).unwrap();

        let version = store
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_eq!(store.current_snapshot().unwrap(), version);
        assert_entry_snapshot_file(history.path(), &alpha.id, version);
        assert_entry_snapshot_file(history.path(), &beta.id, version);
    }

    #[test]
    fn no_op_commit_returns_current_snapshot() {
        let public = tempfile::tempdir().unwrap();
        let history = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_public_entry(public.path(), &entry);
        let mut store = SirnoStore::open(history.path()).unwrap();

        let first = store
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let second = store
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_eq!(first, second);
        assert_eq!(store.current_snapshot().unwrap(), first);
    }

    #[test]
    fn removing_public_entry_creates_history_lifecycle_deletion() {
        let public = tempfile::tempdir().unwrap();
        let history = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_public_entry(public.path(), &alpha);
        write_public_entry(public.path(), &beta);
        let mut store = SirnoStore::open(history.path()).unwrap();

        let first = store
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        fs::remove_file(public.path().join("beta.md")).unwrap();
        let second = store
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_ne!(first, second);
        assert!(store.read_entry_at_snapshot(first, &beta.id).unwrap().is_some());
        assert!(store.read_entry_at_snapshot(second, &alpha.id).unwrap().is_some());
        assert_eq!(store.read_entry_at_snapshot(second, &beta.id).unwrap(), None);
    }

    #[test]
    fn checkout_entry_directory_materializes_historical_state() {
        let public = tempfile::tempdir().unwrap();
        let history = tempfile::tempdir().unwrap();
        let checkout = tempfile::tempdir().unwrap();
        let alpha = test_entry("alpha", "Alpha");
        let beta = test_entry("beta", "Beta");
        write_public_entry(public.path(), &alpha);
        write_public_entry(public.path(), &beta);
        let mut store = SirnoStore::open(history.path()).unwrap();

        let first = store
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        fs::remove_file(public.path().join("beta.md")).unwrap();
        store
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        store
            .checkout_entry_directory(
                first,
                checkout.path(),
                EntryDirectoryWritePolicy::EmptyDirectory,
            )
            .unwrap();

        let checked = check_entry_directory_with_settings(
            checkout.path(),
            CheckMode::Review,
            &EntryDirectoryCheckSettings::default(),
        )
        .unwrap();
        assert_eq!(checked.entries(), &[alpha, beta]);
    }

    fn test_entry(id: &str, name: &str) -> Entry {
        let metadata =
            EntryMetadata::new(name, format!("{name} description.")).expect("valid metadata");
        Entry::new(EntryId::new(id).expect("valid id"), metadata, format!("{name} body.\n"))
    }

    fn write_public_entry(root: &Path, entry: &Entry) {
        let path = root.join(format!("{}.md", entry.id.as_str()));
        fs::write(path, entry.to_markdown().expect("render entry")).expect("write entry");
    }

    fn assert_entry_snapshot_file(root: &Path, id: &EntryId, snapshot: SnapshotRef) {
        let dir = root.join(id.as_str());
        let paths = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].starts_with(&format!("{:016x}-", snapshot.version())));
    }
}
