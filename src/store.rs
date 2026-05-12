//! Sirno store facade.
//!
//! The facade exposes typed Sirno entries.
//! The current backend uses `eter` filesystem snapshots as durable storage.
//! That layout is private to this module.

use std::fs;
use std::path::{Path, PathBuf};

use eter::filesystem::{FilesystemBackend, FilesystemEntryId, FilesystemError};
use eter::{Eter, Eterator, Field, Lifecycle, Resolution, WriteTxn};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::check::{CheckMode, CheckReport, check_entries};
use crate::entry::{Entry, EntryMetadata, WitnessMarker, default_seed_entries};
use crate::id::{EntryId, EntryIdError};

/// Lifecycle state used by Sirno entries in the `eter` backend.
///
/// Sirno currently distinguishes only entry presence.
/// Deletion or archival policy is left to a later design step.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryLifecycle {
    /// The entry exists at this snapshot.
    Active,
}

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

/// Store facade for Sirno entries.
///
/// Invariant: all entries written through this type are represented through
/// typed metadata fields and a Markdown body in the configured `eter` backend.
#[derive(Debug)]
pub struct SirnoStore {
    root: PathBuf,
    backend: FilesystemBackend<EntryLifecycle>,
}

impl SirnoStore {
    /// Open or initialize a store rooted at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        trace!("sirno store open begin");
        let root = root.into();
        let backend = FilesystemBackend::open(&root, sirno_registry())?;
        trace!("sirno store open end");
        Ok(Self { root, backend })
    }

    /// The root path used by this store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Return the current backend snapshot version.
    pub fn current_version(&self) -> Result<Eterator, StoreError> {
        Ok(self.backend.current_version()?)
    }

    /// Write or replace one entry.
    pub fn put_entry(&mut self, entry: &Entry) -> Result<Eterator, StoreError> {
        trace!("sirno put_entry begin: id={}", entry.id);
        let version = self.write_entries(std::slice::from_ref(entry))?;
        trace!("sirno put_entry end: version={}", version.version());
        Ok(version)
    }

    /// Read one entry at the current snapshot.
    pub fn read_entry(&self, id: &EntryId) -> Result<Option<Entry>, StoreError> {
        self.read_entry_at(self.current_version()?, id)
    }

    /// Read every active entry at the current snapshot.
    pub fn read_all_entries(&self) -> Result<Vec<Entry>, StoreError> {
        trace!("sirno read_all_entries begin");
        let at = self.current_version()?;
        let mut entries = Vec::new();
        for id in self.scan_entry_ids()? {
            if let Some(entry) = self.read_entry_at(at, &id)? {
                entries.push(entry);
            }
        }
        trace!("sirno read_all_entries end: entries={}", entries.len());
        Ok(entries)
    }

    /// Check current entries at the selected boundary.
    pub fn check_current(&self, mode: CheckMode) -> Result<CheckReport, StoreError> {
        let entries = self.read_all_entries()?;
        Ok(check_entries(&entries, mode))
    }

    /// Initialize ordinary seed entries.
    ///
    /// The initialized entries are ordinary Sirno entries.
    /// They are created together and are not privileged by later operations.
    pub fn init_default_entries(&mut self) -> Result<Eterator, StoreError> {
        trace!("sirno init_default_entries begin");
        let entries = default_seed_entries()?;
        for entry in &entries {
            let fs_id = entry.id.to_filesystem_id()?;
            if self.backend.entry_id_in_use(&fs_id)? {
                return Err(StoreError::EntryAlreadyExists(entry.id.clone()));
            }
        }
        let version = self.write_entries(&entries)?;
        trace!("sirno init_default_entries end: version={}", version.version());
        Ok(version)
    }

    fn read_entry_at(&self, at: Eterator, id: &EntryId) -> Result<Option<Entry>, StoreError> {
        trace!("sirno read_entry_at begin: id={id} at={}", at.version());
        let fs_id = id.to_filesystem_id()?;
        if !self.backend.entry_exists(at, &fs_id)? {
            trace!("sirno read_entry_at end: absent");
            return Ok(None);
        }

        let metadata = EntryMetadata {
            name: self.resolve_required::<NameField>(at, &fs_id, id, "name")?,
            description: self.resolve_required::<DescriptionField>(
                at,
                &fs_id,
                id,
                "description",
            )?,
            category: self.resolve_optional_list::<CategoryField>(at, &fs_id)?,
            clustee: self.resolve_optional_list::<ClusteeField>(at, &fs_id)?,
            refiner: self.resolve_optional_list::<RefinerField>(at, &fs_id)?,
            witness: match self.backend.resolve::<WitnessField>(at, &fs_id)? {
                | Resolution::Content(marker) => Some(marker),
                | Resolution::Deleted | Resolution::Absent => None,
            },
        };
        let body = match self.backend.resolve_body(at, &fs_id)? {
            | Resolution::Content(body) => body,
            | Resolution::Deleted | Resolution::Absent => {
                return Err(StoreError::CorruptEntry { id: id.clone(), field: "body" });
            }
        };

        trace!("sirno read_entry_at end: present");
        Ok(Some(Entry::new(id.clone(), metadata, body)))
    }

    fn resolve_required<F: Field<Content = String>>(
        &self, at: Eterator, fs_id: &FilesystemEntryId, id: &EntryId, field: &'static str,
    ) -> Result<String, StoreError> {
        match self.backend.resolve::<F>(at, fs_id)? {
            | Resolution::Content(value) => Ok(value),
            | Resolution::Deleted | Resolution::Absent => {
                Err(StoreError::CorruptEntry { id: id.clone(), field })
            }
        }
    }

    fn resolve_optional_list<F: Field<Content = Vec<EntryId>>>(
        &self, at: Eterator, fs_id: &FilesystemEntryId,
    ) -> Result<Vec<EntryId>, StoreError> {
        match self.backend.resolve::<F>(at, fs_id)? {
            | Resolution::Content(value) => Ok(value),
            | Resolution::Deleted | Resolution::Absent => Ok(Vec::new()),
        }
    }

    fn write_entries(&mut self, entries: &[Entry]) -> Result<Eterator, StoreError> {
        let mut txn = self.backend.write();
        for entry in entries {
            let fs_id = entry.id.to_filesystem_id()?;
            txn = txn
                .set::<Lifecycle<EntryLifecycle>>(&fs_id, EntryLifecycle::Active)
                .set::<NameField>(&fs_id, entry.metadata.name.clone())
                .set::<DescriptionField>(&fs_id, entry.metadata.description.clone());

            txn = write_optional_list::<CategoryField>(txn, &fs_id, &entry.metadata.category);
            txn = write_optional_list::<ClusteeField>(txn, &fs_id, &entry.metadata.clustee);
            txn = write_optional_list::<RefinerField>(txn, &fs_id, &entry.metadata.refiner);
            txn = match entry.metadata.witness {
                | Some(marker) => txn.set::<WitnessField>(&fs_id, marker),
                | None => txn.delete::<WitnessField>(&fs_id),
            };
            txn = txn.set_body(&fs_id, entry.body.clone());
        }
        Ok(txn.commit()?)
    }

    fn scan_entry_ids(&self) -> Result<Vec<EntryId>, StoreError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let raw = entry.file_name().to_string_lossy().to_string();
            ids.push(EntryId::new(&raw)?);
        }
        ids.sort();
        Ok(ids)
    }
}

fn write_optional_list<'a, F>(
    txn: eter::filesystem::FilesystemWriteTxn<'a, EntryLifecycle>, fs_id: &FilesystemEntryId,
    value: &[EntryId],
) -> eter::filesystem::FilesystemWriteTxn<'a, EntryLifecycle>
where
    F: Field<Content = Vec<EntryId>>,
{
    if value.is_empty() { txn.delete::<F>(fs_id) } else { txn.set::<F>(fs_id, value.to_vec()) }
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

/// Error raised by Sirno store operations.
#[derive(Debug, Error)]
pub enum StoreError {
    /// The backend reported a filesystem-store error.
    #[error(transparent)]
    Filesystem(#[from] FilesystemError),
    /// Filesystem scanning failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A filesystem directory cannot be interpreted as a Sirno entry id.
    #[error(transparent)]
    EntryId(#[from] EntryIdError),
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
        let mut metadata = EntryMetadata::new("Witness", "A relation.").unwrap();
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
}
