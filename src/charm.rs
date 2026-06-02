//! Charm manifests and spell resolution helpers.
//!
//! Charms are entry-owned artifact bundles.
//! Spells are ready-to-run commands resolved from those bundles.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{Entry, EntryArtifact, EntryArtifactPath, EntryArtifactPathError};

/// Canonical charm manifest filename inside an entry artifact root.
pub const CHARM_MANIFEST_FILE_NAME: &str = "Sirno.charm.toml";

/// Repository-local cache directory for built spells.
pub const SPELL_CACHE_DIRECTORY: &str = ".sirno/spells";

/// One command declared by a charm manifest.
///
/// Invariant: `command` is a non-empty argv vector.
/// Every argv element is non-empty.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharmCommandSpec {
    /// Command argv vector.
    pub command: Vec<String>,
}

/// Build command declaration in a charm manifest.
///
/// Invariant: `command` follows `CharmCommandSpec`.
/// `output`, when present, is a normal relative path.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharmBuildSpec {
    /// Build command argv vector.
    pub command: Vec<String>,
    /// Optional spell output path relative to the spell cache or artifact root.
    #[serde(default)]
    pub output: Option<PathBuf>,
}

/// Spell invocation declaration in a charm manifest.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellSpec {
    /// Spell command argv vector.
    pub command: Vec<String>,
}

/// Charm preparation declarations in a charm manifest.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CharmPreparationSpec {
    /// Optional setup command.
    pub setup: Option<CharmCommandSpec>,
    /// Optional check command.
    pub check: Option<CharmCommandSpec>,
    /// Optional build command.
    pub build: Option<CharmBuildSpec>,
}

/// Parsed `Sirno.charm.toml` manifest.
///
/// Invariant: all declared commands are valid argv vectors.
/// Hook declarations are parsed for compatibility but are not executed by the minimal CLI.
// sirno:witness:charm-manifest:begin
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharmManifest {
    /// Spell invocation declaration.
    pub spell: SpellSpec,
    /// Charm preparation declarations.
    #[serde(default)]
    pub charm: CharmPreparationSpec,
    /// Hook ids for which the charm is eligible.
    #[serde(default)]
    pub hooks: Vec<String>,
    /// Optional input declaration reserved for later hook payload work.
    #[serde(default)]
    pub inputs: Option<toml::Value>,
}

impl CharmManifest {
    /// Parse a charm manifest from artifact bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CharmError> {
        let source = std::str::from_utf8(bytes).map_err(CharmError::ManifestUtf8)?;
        let manifest: Self = toml::from_str(source).map_err(CharmError::ManifestParse)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Return true when this manifest declares a source charm.
    pub fn is_source(&self) -> bool {
        self.charm.build.is_some()
    }

    fn validate(&self) -> Result<(), CharmError> {
        validate_argv("spell.command", &self.spell.command)?;
        if let Some(setup) = &self.charm.setup {
            validate_argv("charm.setup.command", &setup.command)?;
        }
        if let Some(check) = &self.charm.check {
            validate_argv("charm.check.command", &check.command)?;
        }
        if let Some(build) = &self.charm.build {
            validate_argv("charm.build.command", &build.command)?;
            if let Some(output) = &build.output {
                validate_relative_output(output)?;
            }
        }
        Ok(())
    }
}
// sirno:witness:charm-manifest:end

/// One discovered charm with parsed manifest and source entry.
// sirno:witness:charm:begin
#[derive(Clone, Debug)]
pub struct CharmBundle {
    /// Entry that owns the charm.
    pub entry: Entry,
    /// Parsed charm manifest.
    pub manifest: CharmManifest,
    /// Absolute or config-resolved artifact root path.
    pub artifact_root: PathBuf,
    /// Owner artifact bytes by normalized artifact path.
    pub artifacts: BTreeMap<EntryArtifactPath, Vec<u8>>,
}

impl CharmBundle {
    /// Return whether this charm declares a build command.
    pub fn is_source(&self) -> bool {
        self.manifest.is_source()
    }

    /// Return the direct or source kind label.
    pub fn kind_label(&self) -> &'static str {
        if self.is_source() { "source" } else { "direct" }
    }

    /// Return a stable fingerprint for this charm state.
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"sirno-charm-v1\n");
        hasher.update(self.entry.to_markdown().expect("entry markdown rendering is infallible"));
        hasher.update(b"\n");
        for (path, content) in &self.artifacts {
            hasher.update(path.as_str().as_bytes());
            hasher.update(b"\0");
            hasher.update(content.len().to_string().as_bytes());
            hasher.update(b"\0");
            hasher.update(content);
            hasher.update(b"\n");
        }
        hex_digest(hasher.finalize().as_slice())
    }
}
// sirno:witness:charm:end

/// Return the manifest artifact path.
pub fn manifest_artifact_path() -> EntryArtifactPath {
    EntryArtifactPath::new(CHARM_MANIFEST_FILE_NAME).expect("manifest filename is a valid path")
}

/// Build an artifact map from loaded entry artifacts.
pub fn artifact_map(artifacts: Vec<EntryArtifact>) -> BTreeMap<EntryArtifactPath, Vec<u8>> {
    artifacts.into_iter().map(|artifact| (artifact.path, artifact.content)).collect()
}

fn validate_argv(field: &'static str, argv: &[String]) -> Result<(), CharmError> {
    if argv.is_empty() {
        return Err(CharmError::EmptyCommand(field));
    }
    if argv.iter().any(|arg| arg.is_empty()) {
        return Err(CharmError::EmptyCommandArgument(field));
    }
    Ok(())
}

fn validate_relative_output(path: &Path) -> Result<(), CharmError> {
    if path.as_os_str().is_empty() {
        return Err(CharmError::InvalidBuildOutput(path.to_path_buf()));
    }
    if path.is_absolute()
        || path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CharmError::InvalidBuildOutput(path.to_path_buf()));
    }
    Ok(())
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

/// Error raised by charm discovery, parsing, or resolution.
#[derive(Debug, Error)]
pub enum CharmError {
    /// A manifest artifact is missing.
    #[error("entry `{0}` has no {CHARM_MANIFEST_FILE_NAME} artifact")]
    MissingManifest(crate::EntryAddress),
    /// A manifest is not valid UTF-8.
    #[error("{CHARM_MANIFEST_FILE_NAME} must be UTF-8")]
    ManifestUtf8(#[source] std::str::Utf8Error),
    /// A manifest is not valid TOML.
    #[error("failed to parse {CHARM_MANIFEST_FILE_NAME}: {0}")]
    ManifestParse(#[source] toml::de::Error),
    /// A manifest command vector is empty.
    #[error("{0} must contain at least one argv element")]
    EmptyCommand(&'static str),
    /// A manifest command vector contains an empty argument.
    #[error("{0} must not contain empty argv elements")]
    EmptyCommandArgument(&'static str),
    /// A build output path is not normal and relative.
    #[error("charm.build.output must be a normal relative path: {0}")]
    InvalidBuildOutput(PathBuf),
    /// A manifest artifact path could not be represented.
    #[error(transparent)]
    ArtifactPath(#[from] EntryArtifactPathError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_direct_manifest() {
        let manifest = CharmManifest::from_bytes(
            br#"
hooks = ["example"]

[spell]
command = ["sh", "script.sh"]

[charm.check]
command = ["sh", "-n", "script.sh"]
"#,
        )
        .unwrap();

        assert!(!manifest.is_source());
        assert_eq!(manifest.spell.command, vec!["sh", "script.sh"]);
        assert_eq!(manifest.hooks, vec!["example"]);
    }

    #[test]
    fn parses_source_manifest() {
        let manifest = CharmManifest::from_bytes(
            br#"
[spell]
command = ["tool"]

[charm.build]
command = ["cargo", "build"]
output = "target/debug/tool"
"#,
        )
        .unwrap();

        assert!(manifest.is_source());
        assert_eq!(manifest.charm.build.unwrap().output, Some(PathBuf::from("target/debug/tool")));
    }

    #[test]
    fn rejects_empty_spell_command() {
        let error = CharmManifest::from_bytes(
            br#"
[spell]
command = []
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("spell.command"));
    }
}
