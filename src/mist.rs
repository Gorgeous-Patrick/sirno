//! Mist projection specs and local projection manifests.
//!
//! A mist spec owns presentation settings for one projected lake workspace.
//! The local manifest records the exact spec and entry fingerprints used for a render.

use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::anchor::{AnchorError, SIRNO_CONTROL_DIR_NAME, entry_fingerprint};
use crate::entry::Entry;
use crate::identifier::{EntryAtom, EntryAtomError};
use crate::structural::{StructuralEdgeDirection, StructuralRenderSettings, StructuralSettings};

/// Directory below `.sirno/` that stores shared mist specs.
pub const MIST_SPEC_DIR_NAME: &str = "mist";
/// Name of the local manifest written inside a misty lake projection.
pub const MIST_MANIFEST_FILE_NAME: &str = "mist.toml";
/// Current mist manifest schema.
pub const MIST_MANIFEST_SCHEMA: u32 = 1;

const DEFAULT_MIST_NAME: &str = "default";
const MIST_FILE_HEADER: &str = "\
# This file is generated and managed by Sirno.
# It records the mist projection state for this workspace.

";

/// Generated content render settings owned by one mist.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MistRenderSettings {
    /// Structural link directions rendered in generated footers.
    pub structural: StructuralRenderSettings,
}

impl MistRenderSettings {
    /// Return true when no render policy is configured.
    pub fn is_empty(&self) -> bool {
        self.structural.is_empty()
    }

    /// Validate structural render directions against registered project relations.
    pub fn validate(&self, structural: &StructuralSettings) -> Result<(), MistError> {
        validate_structural_render_settings(&self.structural, structural)
    }

    /// Apply this mist's render policy to registered structural relations.
    pub fn structural_settings(
        &self, structural: &StructuralSettings,
    ) -> Result<StructuralSettings, MistError> {
        self.validate(structural)?;
        Ok(structural.with_render_settings(&self.structural))
    }
}

/// A shared mist spec stored below `.sirno/mist/`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MistSpec {
    /// Projection render settings.
    pub render: MistRenderSettings,
}

impl MistSpec {
    /// Return the canonical default mist name.
    pub fn default_name() -> EntryAtom {
        EntryAtom::new(DEFAULT_MIST_NAME).expect("default mist name is a valid entry atom")
    }

    /// Resolve one mist spec path next to a project config.
    pub fn path_for_config(config_path: impl AsRef<Path>, name: &EntryAtom) -> PathBuf {
        config_path
            .as_ref()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(SIRNO_CONTROL_DIR_NAME)
            .join(MIST_SPEC_DIR_NAME)
            .join(format!("{name}.toml"))
    }

    /// Load the default mist when it exists.
    ///
    /// An absent default mist means no projection render policy has been configured yet.
    pub fn default_for_config(config_path: impl AsRef<Path>) -> Result<Self, MistError> {
        let path = Self::path_for_config(config_path, &Self::default_name());
        match Self::from_file(&path) {
            | Ok(spec) => Ok(spec),
            | Err(MistError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
                Ok(Self::default())
            }
            | Err(source) => Err(source),
        }
    }

    /// Load one named mist spec.
    pub fn named_for_config(
        config_path: impl AsRef<Path>, name: &EntryAtom,
    ) -> Result<Self, MistError> {
        Self::from_file(Self::path_for_config(config_path, name))
    }

    /// Load a mist spec from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, MistError> {
        let path = path.as_ref();
        trace!("sirno mist spec load begin: path={}", path.display());
        let source = fs::read_to_string(path)
            .map_err(|source| MistError::Read { path: path.to_path_buf(), source })?;
        Self::from_source(path, &source)
    }

    /// Load a mist spec from source text.
    pub fn from_source(path: impl AsRef<Path>, source: &str) -> Result<Self, MistError> {
        let path = path.as_ref();
        let spec: Self = toml::from_str(source)
            .map_err(|source| MistError::Parse { path: path.to_path_buf(), source })?;
        trace!("sirno mist spec load end");
        Ok(spec)
    }

    /// Write this mist spec as complete TOML.
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), MistError> {
        let path = path.as_ref();
        trace!("sirno mist spec write begin: path={}", path.display());
        write_complete_file(path, &toml::to_string_pretty(self).map_err(MistError::Render)?)?;
        trace!("sirno mist spec write end");
        Ok(())
    }
}

/// Entry fingerprint recorded in one projection manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MistManifestEntry {
    /// Entry address.
    pub id: String,
    /// Canonical entry fingerprint.
    pub fingerprint: String,
}

/// Local projection manifest written to `.sirno/mist.toml` inside a misty lake.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MistManifest {
    /// Manifest schema version.
    pub schema: u32,
    /// Mist name that produced the projection.
    pub mist: EntryAtom,
    /// Mist spec path used for the projection.
    pub spec: PathBuf,
    /// Render settings used for the projection.
    pub render: MistRenderSettings,
    /// Source entries and fingerprints used for rendering.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<MistManifestEntry>,
}

impl MistManifest {
    /// Resolve the manifest path inside one projected lake workspace.
    pub fn path_for_projection(lake: impl AsRef<Path>) -> PathBuf {
        lake.as_ref().join(SIRNO_CONTROL_DIR_NAME).join(MIST_MANIFEST_FILE_NAME)
    }

    /// Build a projection manifest from checked entries.
    pub fn from_entries(
        mist: EntryAtom, spec: PathBuf, render: MistRenderSettings, entries: &[Entry],
    ) -> Result<Self, MistError> {
        let entries = entries
            .iter()
            .map(|entry| {
                Ok(MistManifestEntry {
                    id: entry.id.to_string(),
                    fingerprint: entry_fingerprint(entry)?,
                })
            })
            .collect::<Result<Vec<_>, MistError>>()?;
        Ok(Self { schema: MIST_MANIFEST_SCHEMA, mist, spec, render, entries })
    }

    /// Write this manifest only when the on-disk content would change.
    pub fn write_if_changed(&self, path: impl AsRef<Path>) -> Result<bool, MistError> {
        let path = path.as_ref();
        let source = self.to_toml()?;
        match fs::read_to_string(path) {
            | Ok(current) if current == source => return Ok(false),
            | Ok(_) => {}
            | Err(source) if source.kind() == ErrorKind::NotFound => {}
            | Err(source) => {
                return Err(MistError::Read { path: path.to_path_buf(), source });
            }
        }
        write_complete_file(path, &source)?;
        Ok(true)
    }

    /// Remove a projection manifest when it exists.
    pub fn remove_if_exists(path: impl AsRef<Path>) -> Result<bool, MistError> {
        let path = path.as_ref();
        match fs::remove_file(path) {
            | Ok(()) => Ok(true),
            | Err(source) if source.kind() == ErrorKind::NotFound => Ok(false),
            | Err(source) => Err(MistError::Remove { path: path.to_path_buf(), source }),
        }
    }

    fn to_toml(&self) -> Result<String, MistError> {
        let mut source = String::from(MIST_FILE_HEADER);
        source.push_str(&toml::to_string_pretty(self).map_err(MistError::Render)?);
        Ok(source)
    }
}

fn validate_structural_render_settings(
    render: &StructuralRenderSettings, structural: &StructuralSettings,
) -> Result<(), MistError> {
    for (field, directions) in render.fields() {
        if !structural.contains_field(field) {
            return Err(MistError::RenderStructuralField(field.to_owned()));
        }
        let mut seen = Vec::new();
        for direction in directions {
            if seen.contains(direction) {
                return Err(MistError::DuplicateRenderStructuralDirection {
                    field: field.to_owned(),
                    direction: direction.to_string(),
                });
            }
            seen.push(*direction);
        }
    }
    Ok(())
}

fn write_complete_file(path: &Path, source: &str) -> Result<(), MistError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|source| MistError::CreateDirectory { path: parent.to_path_buf(), source })?;
    }
    let temporary_path = temporary_path(path);
    let mut file =
        OpenOptions::new().write(true).create_new(true).open(&temporary_path).map_err(
            |source| MistError::CreateTemporary { path: temporary_path.clone(), source },
        )?;
    if let Err(source) = file.write_all(source.as_bytes()) {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err(MistError::WriteTemporary { path: temporary_path, source });
    }
    if let Err(source) = file.sync_all() {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err(MistError::WriteTemporary { path: temporary_path, source });
    }
    drop(file);
    if let Err(source) = fs::rename(&temporary_path, path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(MistError::Replace { path: path.to_path_buf(), temporary_path, source });
    }
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().unwrap_or_else(|| OsStr::new(MIST_MANIFEST_FILE_NAME));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let mut temporary_name = OsString::from(".");
    temporary_name.push(file_name);
    temporary_name.push(format!(".{}.{}.tmp", std::process::id(), nonce));
    parent.join(temporary_name)
}

/// Error raised by mist spec and manifest operations.
#[derive(Debug, Error)]
pub enum MistError {
    /// The mist name is not a valid entry atom.
    #[error("mist name is invalid")]
    Name(#[from] EntryAtomError),
    /// The mist file could not be read.
    #[error("failed to read mist file {path}")]
    Read {
        /// Mist file path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The mist file could not be parsed as TOML.
    #[error("failed to parse mist file {path}: {source}")]
    Parse {
        /// Mist file path.
        path: PathBuf,
        /// Underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },
    /// The mist file could not be rendered.
    #[error("failed to render mist file")]
    Render(#[source] toml::ser::Error),
    /// A rendered structural relation is not configured.
    #[error("render.structural `{0}` must name a configured link relation")]
    RenderStructuralField(String),
    /// A rendered structural direction is listed more than once.
    #[error("render.structural `{field}` repeats direction `{direction}`")]
    DuplicateRenderStructuralDirection {
        /// Link relation name.
        field: String,
        /// Repeated direction label.
        direction: String,
    },
    /// A canonical entry fingerprint could not be produced.
    #[error("failed to fingerprint mist source entry")]
    Fingerprint(#[from] AnchorError),
    /// A mist directory could not be created.
    #[error("failed to create mist directory {path}")]
    CreateDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A temporary mist file could not be created.
    #[error("failed to create temporary mist file {path}")]
    CreateTemporary {
        /// Temporary path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A temporary mist file could not be written.
    #[error("failed to write temporary mist file {path}")]
    WriteTemporary {
        /// Temporary path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A mist file could not be atomically replaced.
    #[error("failed to replace mist file {path} with {temporary_path}")]
    Replace {
        /// Final path.
        path: PathBuf,
        /// Temporary path.
        temporary_path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A mist manifest could not be removed.
    #[error("failed to remove mist manifest {path}")]
    Remove {
        /// Manifest path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// Ordered structural render map used in mist specs.
pub type MistStructuralRenderMap = IndexMap<String, Vec<StructuralEdgeDirection>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mist_render_settings() {
        let spec = MistSpec::from_source(
            "default.toml",
            r#"
[render.structural]
kind = ["to"]
area = ["to", "from", "clique"]
"#,
        )
        .unwrap();

        assert_eq!(
            spec.render.structural,
            StructuralRenderSettings::from_fields([
                ("kind", [StructuralEdgeDirection::To].as_slice().iter().copied()),
                (
                    "area",
                    [
                        StructuralEdgeDirection::To,
                        StructuralEdgeDirection::From,
                        StructuralEdgeDirection::Clique,
                    ]
                    .as_slice()
                    .iter()
                    .copied(),
                ),
            ])
        );
    }

    #[test]
    fn applies_mist_render_settings_to_project_relations() {
        let structural = StructuralSettings::from_relations([
            ("kind", crate::EntryAddress::new("kind").unwrap()),
            ("area", crate::EntryAddress::new("area").unwrap()),
        ]);
        let spec = MistSpec::from_source(
            "default.toml",
            r#"
[render.structural]
kind = ["to"]
area = ["clique"]
"#,
        )
        .unwrap();

        let effective = spec.render.structural_settings(&structural).unwrap();
        let fields = effective.fields().collect::<Vec<_>>();

        assert!(fields[0].1.to.render);
        assert!(!fields[0].1.from.render);
        assert!(fields[1].1.clique.render);
    }

    #[test]
    fn rejects_render_settings_for_unconfigured_relation() {
        let structural = StructuralSettings::from_relations([(
            "kind",
            crate::EntryAddress::new("kind").unwrap(),
        )]);
        let spec = MistSpec::from_source(
            "default.toml",
            r#"
[render.structural]
area = ["to"]
"#,
        )
        .unwrap();

        let error = spec.render.structural_settings(&structural).unwrap_err();

        assert!(error.to_string().contains("render.structural `area`"));
    }
}
