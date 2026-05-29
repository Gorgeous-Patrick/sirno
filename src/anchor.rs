//! Git-tracked accepted baselines for Sirno Lakes.
//!
//! Anchor records the reviewed shape of a lake in a small TOML file.
//! Tide compares the current waterline against that file to derive review work.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::trace;

use crate::artifact::EntryArtifact;
use crate::entry::{Entry, EntryRenderError};
use crate::identifier::{EntryAddress, EntryAddressError};
use crate::lake::EntryDirectoryReport;
use crate::render::{GeneratedLinkBody, GeneratedLinkError};
use crate::structural::{StructuralEdgeDirection, StructuralSettings};

/// Directory that holds project-local Sirno control files.
pub const SIRNO_CONTROL_DIR_NAME: &str = ".sirno";
/// Canonical accepted-baseline filename.
pub const ANCHOR_FILE_NAME: &str = "anchor.toml";
/// Current anchor schema.
pub const ANCHOR_SCHEMA: u32 = 1;

const ANCHOR_FILE_HEADER: &str = "\
# This file is generated and managed by Sirno.
# It records the accepted Sirno Lake baseline for Git.

";

const FINGERPRINT_PREFIX: &str = "sha256:";

/// Anchor file keyed by entry address text.
pub type AnchorEntryMap = IndexMap<String, AnchorEntry>;

/// Accepted lake baseline stored in `.sirno/anchor.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:anchor:begin
pub struct AnchorFile {
    /// Anchor schema version.
    pub schema: u32,
    /// Lake path this anchor describes, relative to `Sirno.toml` when possible.
    pub lake: PathBuf,
    /// Accepted entries keyed by entry address.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub entries: AnchorEntryMap,
}
// sirno:witness:anchor:end

/// Accepted state for one live entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorEntry {
    /// Canonical entry fingerprint.
    pub fingerprint: String,
    /// Canonical artifact-tree fingerprint when the entry owns artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_fingerprint: Option<String>,
    /// Structural link fields needed for baseline-side Tide traversal.
    #[serde(default, flatten, skip_serializing_if = "IndexMap::is_empty")]
    pub structural: IndexMap<String, Vec<EntryAddress>>,
}

impl AnchorEntry {
    /// Construct a validated anchor entry record.
    pub fn new(
        fingerprint: String, artifact_fingerprint: Option<String>,
        structural: IndexMap<String, Vec<EntryAddress>>,
    ) -> Self {
        Self { fingerprint, artifact_fingerprint, structural }
    }
}

impl AnchorFile {
    /// Resolve the anchor file path next to the config file.
    pub fn path_for_config(config_path: impl AsRef<Path>) -> PathBuf {
        config_path
            .as_ref()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(SIRNO_CONTROL_DIR_NAME)
            .join(ANCHOR_FILE_NAME)
    }

    /// Build an anchor from a checked lake report.
    pub fn from_report(
        lake: impl Into<PathBuf>, report: &EntryDirectoryReport, settings: &StructuralSettings,
    ) -> Result<Self, AnchorError> {
        let mut artifacts_by_owner = BTreeMap::<EntryAddress, Vec<&EntryArtifact>>::new();
        for artifact in report.artifacts() {
            artifacts_by_owner.entry(artifact.owner.clone()).or_default().push(artifact);
        }

        let mut entries = AnchorEntryMap::new();
        for entry in report.entries() {
            let structural = structural_fields_for_anchor(entry, settings);
            let artifact_fingerprint = artifacts_by_owner
                .get(&entry.id)
                .map(|artifacts| artifact_tree_fingerprint(artifacts.iter().copied()));
            entries.insert(
                entry.id.to_string(),
                AnchorEntry::new(entry_fingerprint(entry)?, artifact_fingerprint, structural),
            );
        }

        let anchor = Self { schema: ANCHOR_SCHEMA, lake: lake.into(), entries };
        anchor.validate()?;
        Ok(anchor)
    }

    /// Load an anchor from a specific file path.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, AnchorError> {
        let path = path.as_ref();
        trace!("sirno anchor load begin: path={}", path.display());
        let source = fs::read_to_string(path)
            .map_err(|source| AnchorError::Read { path: path.to_path_buf(), source })?;
        let anchor: Self = toml::from_str(&source)
            .map_err(|source| AnchorError::Parse { path: path.to_path_buf(), source })?;
        anchor.validate()?;
        trace!("sirno anchor load end");
        Ok(anchor)
    }

    /// Load an anchor when the file exists.
    pub fn from_file_if_exists(path: impl AsRef<Path>) -> Result<Option<Self>, AnchorError> {
        match Self::from_file(path) {
            | Ok(anchor) => Ok(Some(anchor)),
            | Err(AnchorError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
                Ok(None)
            }
            | Err(source) => Err(source),
        }
    }

    /// Write this anchor file atomically.
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), AnchorError> {
        let path = path.as_ref();
        trace!("sirno anchor write begin: path={}", path.display());
        let source = self.to_toml()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| AnchorError::CreateDirectory {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let temporary_path = Self::temporary_path(path);
        let mut file =
            OpenOptions::new().write(true).create_new(true).open(&temporary_path).map_err(
                |source| AnchorError::CreateTemporary { path: temporary_path.clone(), source },
            )?;
        if let Err(source) = file.write_all(source.as_bytes()) {
            drop(file);
            let _ = fs::remove_file(&temporary_path);
            return Err(AnchorError::WriteTemporary { path: temporary_path, source });
        }
        if let Err(source) = file.sync_all() {
            drop(file);
            let _ = fs::remove_file(&temporary_path);
            return Err(AnchorError::WriteTemporary { path: temporary_path, source });
        }
        drop(file);
        if let Err(source) = fs::rename(&temporary_path, path) {
            let _ = fs::remove_file(&temporary_path);
            return Err(AnchorError::Replace { path: path.to_path_buf(), temporary_path, source });
        }
        trace!("sirno anchor write end");
        Ok(())
    }

    /// Validate the anchor shape and fingerprint syntax.
    pub fn validate(&self) -> Result<(), AnchorError> {
        if self.schema != ANCHOR_SCHEMA {
            return Err(AnchorError::UnsupportedSchema { found: self.schema });
        }

        for (id, entry) in &self.entries {
            EntryAddress::new(id.as_str())
                .map_err(|source| AnchorError::EntryAddress { id: id.clone(), source })?;
            validate_fingerprint(format!("entries.{id}.fingerprint"), &entry.fingerprint)?;
            if let Some(fingerprint) = &entry.artifact_fingerprint {
                validate_fingerprint(format!("entries.{id}.artifact_fingerprint"), fingerprint)?;
            }
            for field in entry.structural.keys() {
                validate_structural_field(field)?;
            }
        }

        Ok(())
    }

    fn to_toml(&self) -> Result<String, AnchorError> {
        self.validate()?;
        let mut source = String::from(ANCHOR_FILE_HEADER);
        source.push_str(&toml::to_string_pretty(self).map_err(AnchorError::Render)?);
        Ok(source)
    }

    fn temporary_path(path: &Path) -> PathBuf {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = path.file_name().unwrap_or_else(|| OsStr::new(ANCHOR_FILE_NAME));
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let mut temporary_name = OsString::from(".");
        temporary_name.push(file_name);
        temporary_name.push(format!(".{}.{}.tmp", std::process::id(), nonce));
        parent.join(temporary_name)
    }
}

/// Fingerprint one entry using Anchor schema 1 canonical rendering.
pub fn entry_fingerprint(entry: &Entry) -> Result<String, AnchorError> {
    Ok(sha256_fingerprint(canonical_entry_source(entry)?.as_bytes()))
}

/// Render one entry in the canonical form used by Anchor schema 1.
pub fn canonical_entry_source(entry: &Entry) -> Result<String, AnchorError> {
    let body = GeneratedLinkBody::new(&entry.body).delete()?;
    let body = normalize_line_endings(&strip_trailing_generated_link_divider(&body));
    let entry = Entry::new(entry.id.clone(), entry.metadata.clone(), body);
    Ok(entry.to_markdown()?)
}

/// Fingerprint one owner artifact tree.
pub fn artifact_tree_fingerprint<'a>(
    artifacts: impl IntoIterator<Item = &'a EntryArtifact>,
) -> String {
    let mut artifacts = artifacts.into_iter().collect::<Vec<_>>();
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));

    let mut hasher = Sha256::new();
    hasher.update(b"sirno-artifact-tree-v1\n");
    for artifact in artifacts {
        hasher.update(artifact.path.as_str().as_bytes());
        hasher.update(b"\0");
        hasher.update(artifact.content.len().to_string().as_bytes());
        hasher.update(b"\0");
        hasher.update(&artifact.content);
        hasher.update(b"\n");
    }
    format!("{FINGERPRINT_PREFIX}{}", hex_digest(hasher.finalize().as_slice()))
}

fn structural_fields_for_anchor(
    entry: &Entry, settings: &StructuralSettings,
) -> IndexMap<String, Vec<EntryAddress>> {
    settings
        .fields()
        .filter(|(_, field_settings)| {
            StructuralEdgeDirection::ORDER
                .iter()
                .any(|direction| field_settings.edge(*direction).ripple.anchor)
        })
        .filter_map(|(field, _)| {
            entry
                .metadata
                .structural_field(field)
                .map(|targets| (field.to_owned(), targets.to_vec()))
        })
        .collect()
}

fn strip_trailing_generated_link_divider(body: &str) -> String {
    body.strip_suffix("\n\n---\n")
        .map(|before| format!("{before}\n"))
        .unwrap_or_else(|| body.to_owned())
}

fn normalize_line_endings(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

fn sha256_fingerprint(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"sirno-entry-v1\n");
    hasher.update(bytes);
    format!("{FINGERPRINT_PREFIX}{}", hex_digest(hasher.finalize().as_slice()))
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn validate_fingerprint(field: String, value: &str) -> Result<(), AnchorError> {
    let Some(digest) = value.strip_prefix(FINGERPRINT_PREFIX) else {
        return Err(AnchorError::InvalidFingerprint { field, value: value.to_owned() });
    };
    if digest.len() != 64 || !digest.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(AnchorError::InvalidFingerprint { field, value: value.to_owned() });
    }
    Ok(())
}

fn validate_structural_field(field: &str) -> Result<(), AnchorError> {
    if field.is_empty()
        || field.contains('\n')
        || field.contains('\r')
        || field.contains(',')
        || field == "fingerprint"
        || field == "artifact_fingerprint"
    {
        return Err(AnchorError::InvalidStructuralField(field.to_owned()));
    }
    Ok(())
}

/// Error raised by Anchor operations.
#[derive(Debug, Error)]
pub enum AnchorError {
    /// The anchor file could not be read.
    #[error("failed to read anchor file {path}")]
    Read {
        /// Anchor path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The anchor file could not be parsed as TOML.
    #[error("failed to parse anchor file {path}")]
    Parse {
        /// Anchor path.
        path: PathBuf,
        /// TOML parse error.
        #[source]
        source: toml::de::Error,
    },
    /// The anchor file could not be rendered as TOML.
    #[error("failed to render anchor file")]
    Render(#[source] toml::ser::Error),
    /// The anchor schema is not supported by this binary.
    #[error("unsupported anchor schema {found}; expected {ANCHOR_SCHEMA}")]
    UnsupportedSchema {
        /// Schema found in the file.
        found: u32,
    },
    /// An anchor entry key is not a valid entry address.
    #[error("invalid anchor entry address `{id}`")]
    EntryAddress {
        /// Invalid entry id.
        id: String,
        /// Entry address parse error.
        #[source]
        source: EntryAddressError,
    },
    /// A fingerprint does not use canonical `sha256:<hex>` syntax.
    #[error("invalid anchor fingerprint in {field}: {value}")]
    InvalidFingerprint {
        /// Field path inside the anchor file.
        field: String,
        /// Invalid fingerprint value.
        value: String,
    },
    /// A flattened structural field cannot be represented safely.
    #[error("invalid anchor structural field `{0}`")]
    InvalidStructuralField(String),
    /// The anchor directory could not be created.
    #[error("failed to create anchor directory {path}")]
    CreateDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A temporary anchor file could not be created.
    #[error("failed to create temporary anchor file {path}")]
    CreateTemporary {
        /// Temporary path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A temporary anchor file could not be written.
    #[error("failed to write temporary anchor file {path}")]
    WriteTemporary {
        /// Temporary path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The temporary anchor file could not replace the old file.
    #[error("failed to replace anchor file {path} from {temporary_path}")]
    Replace {
        /// Final path.
        path: PathBuf,
        /// Temporary path.
        temporary_path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Generated footer boundaries were malformed during canonicalization.
    #[error(transparent)]
    GeneratedLink(#[from] GeneratedLinkError),
    /// Entry rendering failed during canonicalization.
    #[error(transparent)]
    EntryRender(#[from] EntryRenderError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EntryMetadata, StructuralEdgeSettings, StructuralFieldSettings, StructuralRippleSettings,
        StructuralSettings,
    };

    fn entry(raw_id: &str, body: impl Into<String>) -> Entry {
        Entry::new(
            EntryAddress::new(raw_id).unwrap(),
            EntryMetadata::new(raw_id, "desc").unwrap(),
            body,
        )
    }

    #[test]
    fn entry_fingerprint_ignores_generated_footer() {
        let plain = entry("alpha", "Body.\n");
        let footer = format!(
            "{}\n\ntext\n\n{}\n",
            crate::render::BEGIN_LINKS_GUARD,
            crate::render::END_LINKS_GUARD
        );
        let rendered = entry("alpha", format!("Body.\n\n---\n{footer}"));

        assert_eq!(entry_fingerprint(&plain).unwrap(), entry_fingerprint(&rendered).unwrap());
    }

    #[test]
    fn anchor_from_report_records_structural_fields() {
        let temp = tempfile::tempdir().unwrap();
        let entry_path = temp.path().join("alpha.md");
        fs::write(
            entry_path,
            "\
---
name: Alpha
desc: Alpha entry.
belongs:
  - beta
---

Body.
",
        )
        .unwrap();
        let settings = StructuralSettings::from_fields([(
            "belongs",
            StructuralFieldSettings::new(
                StructuralEdgeSettings::new(false, StructuralRippleSettings::new(false, true)),
                StructuralEdgeSettings::default(),
                StructuralEdgeSettings::default(),
            ),
        )]);
        let report = crate::EntryDirectory::new(temp.path())
            .check_with_settings(
                crate::CheckMode::Edit,
                &crate::EntryDirectoryCheckSettings {
                    structural: settings.clone(),
                    render: false,
                    structural_inhabitance: false,
                    ignore: Vec::new(),
                    witness: None,
                },
            )
            .unwrap();

        let anchor = AnchorFile::from_report("docs", &report, &settings).unwrap();

        assert_eq!(
            anchor.entries["alpha"].structural["belongs"],
            vec![EntryAddress::new("beta").unwrap()]
        );
    }
}
