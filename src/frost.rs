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

use crate::artifact::{EntryArtifact, EntryArtifactPath, EntryArtifactPathError};
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

struct ArtifactContentField;
impl Field for ArtifactContentField {
    type Content = Vec<u8>;
}

const ARTIFACT_BACKEND_PREFIX: &str = ".artifact-";

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

    /// Return the private backend directory path for one stored entry.
    pub fn entry_storage_path(root: impl AsRef<Path>, id: &EntryId) -> Result<PathBuf, FrostError> {
        Ok(root.as_ref().join(id.to_filesystem_id()?.as_str()))
    }

    /// Return the private backend directory path for one stored artifact.
    pub fn artifact_storage_path(
        root: impl AsRef<Path>, owner: &EntryId, path: &EntryArtifactPath,
    ) -> Result<PathBuf, FrostError> {
        Ok(root.as_ref().join(artifact_filesystem_id(owner, path)?.as_str()))
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
            if artifact_identity_from_filesystem_id(&fs_id)?.is_some() {
                continue;
            }
            let id = EntryId::try_from(fs_id)?;
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
            let Some((owner, path)) = artifact_identity_from_filesystem_id(&fs_id)? else {
                continue;
            };
            let Some(facet) = StoredArtifactFacet::load_from(&self.backend, at, &fs_id)? else {
                continue;
            };
            artifacts.push(facet.into_artifact(owner, path)?);
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

    /// Require a public entry to match the current Frost snapshot.
    ///
    /// Generated-link regions and the `frozen:` marker are public lake state.
    /// They are removed before comparing to Frost storage.
    pub fn ensure_entry_current(&self, entry: &Entry) -> Result<(), FrostError> {
        let entries = Self::entries_without_generated_links(std::slice::from_ref(entry))?;
        let entry = Self::entry_without_frozen_marker(&entries[0]);
        self.ensure_entry_matches_snapshot(self.current_snapshot()?, &entry)
    }

    /// Require a public entry and its artifacts to match the current Frost snapshot.
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
        let version = self.commit_entries_and_artifacts(&entries, report.artifacts())?;
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
        let public_ids = entries.iter().map(|entry| entry.id.clone()).collect::<BTreeSet<_>>();
        let deleted_ids = previous_entries
            .keys()
            .filter(|id| !public_ids.contains(*id))
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

        if changed_entries.is_empty()
            && deleted_ids.is_empty()
            && changed_artifacts.is_empty()
            && deleted_artifacts.is_empty()
        {
            return Ok(current);
        }

        let mut txn = self.backend.write();
        for entry in changed_entries {
            let fs_id = entry.id.to_filesystem_id()?;
            txn = StoredEntryFacet::from_entry(entry).apply_to(txn, &fs_id);
        }
        for artifact in changed_artifacts {
            let fs_id = artifact_filesystem_id(&artifact.owner, &artifact.path)?;
            txn = StoredArtifactFacet::from_artifact(artifact).apply_to(txn, &fs_id);
        }
        for id in deleted_ids {
            let fs_id = id.to_filesystem_id()?;
            txn = txn.delete::<Lifecycle<EntryLifecycle>>(&fs_id);
        }
        for (owner, path) in deleted_artifacts {
            let fs_id = artifact_filesystem_id(&owner, &path)?;
            txn = txn
                .delete::<Lifecycle<EntryLifecycle>>(&fs_id)
                .delete::<ArtifactContentField>(&fs_id);
        }
        Ok(txn.commit()?)
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
            .with_field::<ArtifactContentField>("artifact-content")
    }
    // sirno:witness:sirno-frost:end
}

// sirno:witness:entry-artifact:begin
fn artifact_key(artifact: &EntryArtifact) -> (EntryId, EntryArtifactPath) {
    (artifact.owner.clone(), artifact.path.clone())
}

fn artifacts_by_owner(
    artifacts: impl IntoIterator<Item = EntryArtifact>,
) -> BTreeMap<EntryId, Vec<EntryArtifact>> {
    let mut by_owner = BTreeMap::<EntryId, Vec<EntryArtifact>>::new();
    for artifact in artifacts {
        by_owner.entry(artifact.owner.clone()).or_default().push(artifact);
    }
    for artifacts in by_owner.values_mut() {
        artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    }
    by_owner
}

fn artifact_filesystem_id(
    owner: &EntryId, path: &EntryArtifactPath,
) -> Result<FilesystemEntryId, FrostError> {
    FilesystemEntryId::new(format!(
        "{ARTIFACT_BACKEND_PREFIX}{}-{}",
        hex_encode(owner.as_str().as_bytes()),
        hex_encode(path.as_str().as_bytes())
    ))
    .map_err(FrostError::from)
}

fn artifact_identity_from_filesystem_id(
    id: &FilesystemEntryId,
) -> Result<Option<(EntryId, EntryArtifactPath)>, FrostError> {
    let Some(encoded) = id.as_str().strip_prefix(ARTIFACT_BACKEND_PREFIX) else {
        return Ok(None);
    };
    let Some((owner, path)) = encoded.split_once('-') else {
        return Err(FrostError::CorruptArtifactId(id.as_str().to_owned()));
    };
    let owner = String::from_utf8(hex_decode(owner)?)
        .map_err(|_| FrostError::CorruptArtifactId(id.as_str().to_owned()))?;
    let path = String::from_utf8(hex_decode(path)?)
        .map_err(|_| FrostError::CorruptArtifactId(id.as_str().to_owned()))?;
    Ok(Some((EntryId::new(owner)?, EntryArtifactPath::new(path)?)))
}
// sirno:witness:entry-artifact:end

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode(encoded: &str) -> Result<Vec<u8>, FrostError> {
    if !encoded.len().is_multiple_of(2) {
        return Err(FrostError::CorruptArtifactId(encoded.to_owned()));
    }
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    let chars = encoded.as_bytes();
    for pair in chars.chunks_exact(2) {
        let high = hex_value(pair[0]).ok_or_else(|| {
            FrostError::CorruptArtifactId(String::from_utf8_lossy(pair).to_string())
        })?;
        let low = hex_value(pair[1]).ok_or_else(|| {
            FrostError::CorruptArtifactId(String::from_utf8_lossy(pair).to_string())
        })?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        | b'0'..=b'9' => Some(byte - b'0'),
        | b'a'..=b'f' => Some(byte - b'a' + 10),
        | _ => None,
    }
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

// sirno:witness:entry-artifact:begin
#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredArtifactFacet {
    content: Option<Vec<u8>>,
}

impl StoredArtifactFacet {
    fn from_artifact(artifact: &EntryArtifact) -> Self {
        Self { content: Some(artifact.content.clone()) }
    }

    fn into_artifact(
        self, owner: EntryId, path: EntryArtifactPath,
    ) -> Result<EntryArtifact, FrostError> {
        let content = self.content.ok_or_else(|| FrostError::CorruptArtifact {
            owner: owner.clone(),
            path: path.clone(),
        })?;
        Ok(EntryArtifact::new(owner, path, content))
    }

    fn load_from(
        backend: &SirnoBackend, at: SnapshotRef, id: &FilesystemEntryId,
    ) -> Result<Option<Self>, FilesystemError> {
        if !backend.entry_exists(at, id)? {
            return Ok(None);
        }

        Ok(Some(Self {
            content: match backend.resolve::<ArtifactContentField>(at, id)? {
                | Resolution::Content(content) => Some(content),
                | Resolution::Deleted | Resolution::Absent => None,
            },
        }))
    }

    fn apply_to<'a>(&self, txn: SirnoWriteTxn<'a>, id: &FilesystemEntryId) -> SirnoWriteTxn<'a>
    where
        SirnoBackend: 'a,
    {
        txn.set::<Lifecycle<EntryLifecycle>>(id, EntryLifecycle::Active)
            .set::<ArtifactContentField>(
                id,
                self.content
                    .clone()
                    .unwrap_or_else(|| panic!("Sirno Frost artifact facet is missing content")),
            )
    }
}
// sirno:witness:entry-artifact:end

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
    /// A stored artifact path cannot be interpreted as a lake artifact path.
    #[error(transparent)]
    ArtifactPath(#[from] EntryArtifactPathError),
    /// Public Markdown entry directory must pass review checks before Frost commit.
    #[error("entry directory must pass review checks before Frost commit: {0}")]
    InvalidEntryDirectory(PathBuf),
    /// Seed initialization would overwrite an existing entry.
    #[error("entry `{0}` already exists")]
    EntryAlreadyExists(EntryId),
    /// A frozen entry differs from the current Frost snapshot.
    #[error(
        "entry `{0}` is frozen but does not match the current Frost snapshot; run `sirno melt {0}` before changing it"
    )]
    FrozenEntryChanged(EntryId),
    /// A frozen entry is missing a required Sirno field.
    #[error("frozen entry `{id}` is missing required field `{field}`")]
    CorruptEntry {
        /// Entry containing the corrupt field state.
        id: EntryId,
        /// Field that could not be resolved.
        field: &'static str,
    },
    /// A frozen artifact is missing its stored byte content.
    #[error("frozen artifact `{owner}/{path}` is missing content")]
    CorruptArtifact {
        /// Entry that owns the corrupt artifact.
        owner: EntryId,
        /// Owner-relative artifact path.
        path: EntryArtifactPath,
    },
    /// A Frost artifact backend id cannot be decoded.
    #[error("frozen artifact id is corrupt: {0}")]
    CorruptArtifactId(String),
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

    // sirno:witness:entry-artifact:begin
    #[test]
    fn commit_entry_directory_versions_artifacts_independently() {
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_public_entry(public.path(), &entry);
        write_public_artifact(public.path(), &entry.id, "images/logo.bin", b"old");
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let first = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        write_public_artifact(public.path(), &entry.id, "images/logo.bin", b"new");
        let second = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let artifacts = frost.read_all_artifacts_at_snapshot(second).unwrap();

        assert_ne!(first, second);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &entry.id), [first]);
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].owner, entry.id);
        assert_eq!(artifacts[0].path.as_str(), "images/logo.bin");
        assert_eq!(artifacts[0].content, b"new");
    }
    // sirno:witness:entry-artifact:end

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
    fn commit_entry_directory_allows_current_frozen_entry() {
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
        let mut frozen_alpha = alpha.clone();
        frozen_alpha.metadata.frozen = Some(FrozenMarker::Present);
        let mut changed_beta = beta.clone();
        changed_beta.body = "Beta changed body.\n".to_owned();
        write_public_entry(public.path(), &frozen_alpha);
        write_public_entry(public.path(), &changed_beta);
        let second = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();

        assert_ne!(first, second);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &alpha.id), [first]);
        assert_eq!(entry_snapshot_versions(frost_path.path(), &beta.id), [first, second]);
        assert_eq!(frost.read_entry_at_snapshot(second, &alpha.id).unwrap(), Some(alpha));
        assert_eq!(frost.read_entry_at_snapshot(second, &beta.id).unwrap(), Some(changed_beta));
    }

    #[test]
    fn commit_entry_directory_rejects_changed_frozen_entry() {
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let entry = test_entry("alpha", "Alpha");
        write_public_entry(public.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();
        frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap();
        let mut changed_entry = entry.clone();
        changed_entry.metadata.frozen = Some(FrozenMarker::Present);
        changed_entry.body = "Changed body.\n".to_owned();
        write_public_entry(public.path(), &changed_entry);

        let error = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
            .unwrap_err();

        assert!(matches!(error, FrostError::FrozenEntryChanged(id) if id == entry.id));
    }

    #[test]
    fn commit_entry_directory_rejects_new_frozen_entry() {
        let public = tempfile::tempdir().unwrap();
        let frost_path = tempfile::tempdir().unwrap();
        let mut entry = test_entry("alpha", "Alpha");
        entry.metadata.frozen = Some(FrozenMarker::Present);
        write_public_entry(public.path(), &entry);
        let mut frost = SirnoFrost::open(frost_path.path()).unwrap();

        let error = frost
            .commit_entry_directory(public.path(), &EntryDirectoryCheckSettings::default())
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
        write_public_artifact(public.path(), &alpha.id, "notes.txt", b"artifact");
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
        assert_eq!(
            fs::read(checkout.path().join(".artifacts").join("alpha").join("notes.txt")).unwrap(),
            b"artifact"
        );
    }

    fn test_entry(id: &str, name: &str) -> Entry {
        let metadata = EntryMetadata::new(name, format!("{name} desc.")).expect("valid metadata");
        Entry::new(EntryId::new(id).expect("valid id"), metadata, format!("{name} body.\n"))
    }

    fn write_public_entry(root: &Path, entry: &Entry) {
        let path = root.join(format!("{}.md", entry.id.as_str()));
        fs::write(path, entry.to_markdown().expect("render entry")).expect("write entry");
    }

    fn write_public_artifact(root: &Path, owner: &EntryId, path: &str, content: &[u8]) {
        let path = root.join(".artifacts").join(owner.as_str()).join(path);
        fs::create_dir_all(path.parent().expect("artifact has parent")).unwrap();
        fs::write(path, content).unwrap();
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
