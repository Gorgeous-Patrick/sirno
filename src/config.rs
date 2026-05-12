//! Project configuration for a Sirno-managed repository.
//!
//! A repository is Sirno-managed when it contains `Sirno.toml`.
//! The config names the monograph and the public Markdown entry store.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::links::GeneratedLinkSettings;

/// Canonical Sirno project config filename.
pub const CONFIG_FILE_NAME: &str = "Sirno.toml";

/// Sirno project configuration.
///
/// Invariant: `mono` points to the configured monograph path.
/// `store` points to the configured public Markdown entry store path.
/// `links` controls generated-link footer content.
/// Relative paths are resolved against the directory containing `Sirno.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SirnoConfig {
    /// Configured monograph path.
    pub mono: PathBuf,
    /// Configured public Markdown entry store path.
    pub store: PathBuf,
    /// Generated-link footer settings.
    #[serde(default)]
    pub links: GeneratedLinkSettings,
}

impl SirnoConfig {
    /// Construct a config from explicit paths.
    pub fn new(mono: impl Into<PathBuf>, store: impl Into<PathBuf>) -> Self {
        Self { mono: mono.into(), store: store.into(), links: GeneratedLinkSettings::default() }
    }

    /// Default config for a new Sirno-managed repository.
    pub fn default_project() -> Self {
        Self::new("DESIGN.md", "docs")
    }

    /// Load a config from a specific file path.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        trace!("sirno config load begin: path={}", path.display());
        let source = fs::read_to_string(path)
            .map_err(|source| ConfigError::Read { path: path.to_path_buf(), source })?;
        let config = toml::from_str(&source)
            .map_err(|source| ConfigError::Parse { path: path.to_path_buf(), source })?;
        trace!("sirno config load end");
        Ok(config)
    }

    /// Write this config to a new file.
    ///
    /// Existing files are never overwritten.
    pub fn write_new(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let path = path.as_ref();
        trace!("sirno config write begin: path={}", path.display());
        let source = toml::to_string_pretty(self).map_err(ConfigError::Render)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|source| ConfigError::Create { path: path.to_path_buf(), source })?;
        file.write_all(source.as_bytes())
            .map_err(|source| ConfigError::Write { path: path.to_path_buf(), source })?;
        trace!("sirno config write end");
        Ok(())
    }

    /// Resolve the monograph path relative to a config file path.
    pub fn resolve_mono(&self, config_path: impl AsRef<Path>) -> PathBuf {
        resolve_config_relative(config_path.as_ref(), &self.mono)
    }

    /// Resolve the entry store path relative to a config file path.
    pub fn resolve_store(&self, config_path: impl AsRef<Path>) -> PathBuf {
        resolve_config_relative(config_path.as_ref(), &self.store)
    }
}

fn resolve_config_relative(config_path: &Path, configured_path: &Path) -> PathBuf {
    if configured_path.is_absolute() {
        return configured_path.to_path_buf();
    }
    config_path.parent().unwrap_or_else(|| Path::new(".")).join(configured_path)
}

/// Error raised by Sirno config operations.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The config file could not be read.
    #[error("failed to read config file {path}")]
    Read {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The config file could not be parsed as TOML.
    #[error("failed to parse config file {path}")]
    Parse {
        /// Path that could not be parsed.
        path: PathBuf,
        /// Underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },
    /// The config file could not be rendered.
    #[error("failed to render config file")]
    Render(#[source] toml::ser::Error),
    /// The config file could not be created.
    #[error("failed to create config file {path}")]
    Create {
        /// Path that could not be created.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The config file could not be written.
    #[error("failed to write config file {path}")]
    Write {
        /// Path that could not be written.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_config() {
        let config: SirnoConfig = toml::from_str(
            r#"
mono = "DESIGN.md"
store = "docs"
"#,
        )
        .unwrap();

        assert_eq!(config.mono, PathBuf::from("DESIGN.md"));
        assert_eq!(config.store, PathBuf::from("docs"));
        assert_eq!(config.links, GeneratedLinkSettings::default());
    }

    #[test]
    fn parses_link_settings() {
        let config: SirnoConfig = toml::from_str(
            r#"
mono = "DESIGN.md"
store = "docs"

[links]
category = true
clustee = false
refiner = true
"#,
        )
        .unwrap();

        assert_eq!(
            config.links,
            GeneratedLinkSettings { category: true, clustee: false, refiner: true }
        );
    }

    #[test]
    fn rejects_unknown_fields() {
        let error = toml::from_str::<SirnoConfig>(
            r#"
mono = "DESIGN.md"
store = "docs"
extra = "no"
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn resolves_relative_paths_against_config_directory() {
        let config = SirnoConfig::default_project();
        let config_path = Path::new("/tmp/project/Sirno.toml");

        assert_eq!(config.resolve_mono(config_path), PathBuf::from("/tmp/project/DESIGN.md"));
        assert_eq!(config.resolve_store(config_path), PathBuf::from("/tmp/project/docs"));
    }

    #[test]
    fn writes_and_reads_config_without_overwrite() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        let config = SirnoConfig::default_project();

        config.write_new(&path).unwrap();
        let read = SirnoConfig::from_file(&path).unwrap();

        assert_eq!(read, config);
        assert!(matches!(config.write_new(&path), Err(ConfigError::Create { .. })));
    }
}
