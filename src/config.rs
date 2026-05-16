//! Project configuration for a Sirno-managed repository.
//!
//! A repository is Sirno-managed when it contains `Sirno.toml`.
//! The config names the public Markdown entry lake.
//! It may also opt into a monograph, repository witness members, and Sirno Frost.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::entry::{DESC_FIELD, FROZEN_FIELD, NAME_FIELD};
use crate::links::StructuralSettings;

/// Canonical Sirno project config filename.
pub const CONFIG_FILE_NAME: &str = "Sirno.toml";

// sirno:witness:project-config:begin
macro_rules! witness_entry_id_capture_regex {
    () => {
        r#"([^\x00-\x1F\x7F<>:"/\\|?*\r\n]+)"#
    };
}

/// Canonical witness delimiter capture for every legal entry id.
///
/// Reserved filename checks that cannot fit Rust regex syntax are enforced by `EntryId`.
pub const WITNESS_ENTRY_ID_CAPTURE_REGEX: &str = witness_entry_id_capture_regex!();

/// Standard opening delimiter regex for line-comment repository witness blocks.
pub const STANDARD_LINE_WITNESS_BEGIN_REGEX: &str =
    concat!(r"(?m)^[ \t]*//[ \t]*sirno:witness:", witness_entry_id_capture_regex!(), r":begin");

/// Standard closing delimiter regex for line-comment repository witness blocks.
pub const STANDARD_LINE_WITNESS_END_REGEX: &str =
    concat!(r"(?m)^[ \t]*//[ \t]*sirno:witness:", witness_entry_id_capture_regex!(), r":end");

/// Standard opening delimiter regex for Markdown repository witness blocks.
pub const STANDARD_MARKDOWN_WITNESS_BEGIN_REGEX: &str = concat!(
    r"(?m)^[ \t]*<!--[ \t]*sirno:witness:",
    witness_entry_id_capture_regex!(),
    r":begin[ \t]*-->"
);

/// Standard closing delimiter regex for Markdown repository witness blocks.
pub const STANDARD_MARKDOWN_WITNESS_END_REGEX: &str = concat!(
    r"(?m)^[ \t]*<!--[ \t]*sirno:witness:",
    witness_entry_id_capture_regex!(),
    r":end[ \t]*-->"
);
// sirno:witness:project-config:end

// sirno:witness:mono:begin
/// Optional configured monograph settings.
///
/// Invariant: `path` points to the configured monograph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonoSettings {
    /// Configured monograph path.
    pub path: PathBuf,
}

impl MonoSettings {
    /// Construct monograph settings from a path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}
// sirno:witness:mono:end

/// Settings for structural checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CheckSettings {
    /// Check generated-link footer freshness.
    pub link: bool,
}

impl Default for CheckSettings {
    fn default() -> Self {
        Self { link: true }
    }
}

/// Configured public Markdown lake settings.
///
/// Invariant: `path` points to the public Markdown entry lake.
/// `ignore` contains paths relative to the lake root that Sirno does not read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LakeSettings {
    /// Configured public Markdown entry lake path.
    pub path: PathBuf,
    /// Lake-root-relative paths ignored by Sirno.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<PathBuf>,
}

impl LakeSettings {
    /// Construct lake settings from a lake path and no ignored paths.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), ignore: Vec::new() }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        for path in &self.ignore {
            if path.as_os_str().is_empty()
                || path.is_absolute()
                || path.components().any(|component| {
                    matches!(
                        component,
                        Component::ParentDir | Component::RootDir | Component::Prefix(_)
                    )
                })
            {
                return Err(ConfigError::LakeIgnorePath(path.clone()));
            }
        }
        Ok(())
    }
}

/// Configured Sirno Frost settings.
///
/// Invariant: `path` points to the private `eter` root used by Sirno Frost.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrostSettings {
    /// Configured Sirno Frost root path.
    pub path: PathBuf,
}

impl FrostSettings {
    /// Construct Sirno Frost settings from a root path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

/// One repository member that Sirno scans through `mosaika`.
///
/// Invariant: `pattern` is a non-empty config-relative path or glob.
/// It never names an absolute path or a parent-directory escape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
// sirno:witness:repo:begin
pub struct RepoMember {
    pattern: String,
}
// sirno:witness:repo:end

impl RepoMember {
    /// Construct one repo-member pattern.
    pub fn new(pattern: impl Into<String>) -> Result<Self, ConfigError> {
        let member = Self { pattern: pattern.into() };
        member.validate()?;
        Ok(member)
    }

    /// Return the member pattern as written in `Sirno.toml`.
    pub fn as_str(&self) -> &str {
        &self.pattern
    }

    fn validate(&self) -> Result<(), ConfigError> {
        let path = Path::new(&self.pattern);
        if self.pattern.is_empty()
            || path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(ConfigError::RepoMemberPath(self.pattern.clone()));
        }
        Ok(())
    }
}

/// Configured repository artifacts that can witness Sirno entries.
///
/// Invariant: every member is a config-relative path or glob.
/// Directory members are scanned recursively by witness lookup.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
// sirno:witness:repo:begin
pub struct RepoSettings {
    /// Config-relative paths or globs scanned through `mosaika`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<RepoMember>,
}
// sirno:witness:repo:end

impl RepoSettings {
    fn validate(&self) -> Result<(), ConfigError> {
        for member in &self.members {
            member.validate()?;
        }
        Ok(())
    }
}

/// Configured witness delimiter pair.
///
/// Invariant: `begin` and `end` are non-empty regex strings.
/// Each regex captures the entry id as its first capture group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:project-config:begin
pub struct WitnessDelimiterSettings {
    /// Regex that matches an opening witness delimiter.
    pub begin: String,
    /// Regex that matches a closing witness delimiter.
    pub end: String,
}
// sirno:witness:project-config:end

/// Configured witness delimiter syntax.
///
/// Invariant: `delimiters` is non-empty.
/// Each delimiter pair is validated by `WitnessDelimiterSettings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:project-config:begin
pub struct WitnessSettings {
    /// Configured witness delimiter pairs.
    pub delimiters: Vec<WitnessDelimiterSettings>,
}
// sirno:witness:project-config:end

impl WitnessDelimiterSettings {
    /// Construct one delimiter pair from regex strings.
    pub fn new(begin: impl Into<String>, end: impl Into<String>) -> Self {
        Self { begin: begin.into(), end: end.into() }
    }

    fn validate(&self, index: usize) -> Result<(), ConfigError> {
        Self::validate_regex("witness.delimiters.begin", index, &self.begin)?;
        Self::validate_regex("witness.delimiters.end", index, &self.end)?;
        Ok(())
    }

    fn validate_regex(field: &'static str, index: usize, source: &str) -> Result<(), ConfigError> {
        if source.trim().is_empty() {
            return Err(ConfigError::WitnessRegex { field, index });
        }
        let regex = Regex::new(source).map_err(|source| ConfigError::WitnessRegexSyntax {
            field,
            index,
            source,
        })?;
        if regex.captures_len() < 2 {
            return Err(ConfigError::WitnessRegexCapture { field, index });
        }
        if regex.is_match("") {
            return Err(ConfigError::WitnessRegexEmptyMatch { field, index });
        }
        Ok(())
    }
}

impl WitnessSettings {
    /// Construct the standard syntax written by generated configs.
    pub fn standard() -> Self {
        Self {
            delimiters: vec![
                WitnessDelimiterSettings::new(
                    STANDARD_LINE_WITNESS_BEGIN_REGEX,
                    STANDARD_LINE_WITNESS_END_REGEX,
                ),
                WitnessDelimiterSettings::new(
                    STANDARD_MARKDOWN_WITNESS_BEGIN_REGEX,
                    STANDARD_MARKDOWN_WITNESS_END_REGEX,
                ),
            ],
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.delimiters.is_empty() {
            return Err(ConfigError::WitnessDelimiterList);
        }
        for (index, delimiter) in self.delimiters.iter().enumerate() {
            delimiter.validate(index)?;
        }
        Ok(())
    }
}

/// Sirno project configuration.
///
/// `lake.path` points to the configured public Markdown entry lake path.
/// `mono.path`, when present, points to the configured monograph path.
/// `frost.path`, when present, points to the configured Sirno Frost root.
/// `lake.ignore` contains paths relative to the lake root that Sirno skips.
/// `repo.members`, when present, contains relative member paths or globs for witness lookup.
/// `witness` controls the delimiter syntax for repository witness blocks.
/// `check` controls optional structural check families.
/// `structural` controls structural metadata fields and generated-link footer content.
/// Relative paths are resolved against the directory containing `Sirno.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:project-config:begin
pub struct SirnoConfig {
    /// Configured monograph settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mono: Option<MonoSettings>,
    /// Configured public Markdown entry lake settings.
    pub lake: LakeSettings,
    /// Configured Sirno Frost settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frost: Option<FrostSettings>,
    /// Configured repository artifact members.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<RepoSettings>,
    /// Configured repository witness delimiter syntax.
    pub witness: WitnessSettings,
    /// Structural check settings.
    #[serde(default)]
    pub check: CheckSettings,
    /// Structural metadata and generated-link settings.
    #[serde(default)]
    pub structural: StructuralSettings,
}
// sirno:witness:project-config:end

impl SirnoConfig {
    /// Construct a config from the required lake path.
    // sirno:witness:project-config:begin
    pub fn new(lake: impl Into<PathBuf>) -> Self {
        Self {
            mono: None,
            lake: LakeSettings::new(lake),
            frost: None,
            repo: None,
            witness: WitnessSettings::standard(),
            check: CheckSettings::default(),
            structural: StructuralSettings::default(),
        }
    }
    // sirno:witness:project-config:end

    /// Return this config with a configured monograph path.
    pub fn with_mono(mut self, mono: impl Into<PathBuf>) -> Self {
        self.mono = Some(MonoSettings::new(mono));
        self
    }

    /// Return this config with a configured public lake path.
    pub fn with_lake(mut self, lake: impl Into<PathBuf>) -> Self {
        self.lake.path = lake.into();
        self
    }

    /// Return this config with a configured Sirno Frost root.
    pub fn with_frost(mut self, frost: impl Into<PathBuf>) -> Self {
        self.frost = Some(FrostSettings::new(frost));
        self
    }

    /// Default config for a new Sirno-managed repository.
    pub fn default_project() -> Self {
        Self::new("docs")
    }

    /// Load a config from a specific file path.
    // sirno:witness:project-config:begin
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        trace!("sirno config load begin: path={}", path.display());
        let source = fs::read_to_string(path)
            .map_err(|source| ConfigError::Read { path: path.to_path_buf(), source })?;
        let config: Self = toml::from_str(&source)
            .map_err(|source| ConfigError::Parse { path: path.to_path_buf(), source })?;
        config.validate_for_file(path)?;
        trace!("sirno config load end");
        Ok(config)
    }
    // sirno:witness:project-config:end

    /// Write this config to a new file.
    ///
    /// Existing files are never overwritten.
    // sirno:witness:project-config:begin
    pub fn write_new(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let path = path.as_ref();
        trace!("sirno config write begin: path={}", path.display());
        self.validate_for_file(path)?;
        let source = self.to_toml()?;
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
    // sirno:witness:project-config:end

    /// Write this config to an existing or new file.
    // sirno:witness:project-config:begin
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let path = path.as_ref();
        trace!("sirno config write replace begin: path={}", path.display());
        self.validate_for_file(path)?;
        let source = self.to_toml()?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|source| ConfigError::Create { path: path.to_path_buf(), source })?;
        file.write_all(source.as_bytes())
            .map_err(|source| ConfigError::Write { path: path.to_path_buf(), source })?;
        trace!("sirno config write replace end");
        Ok(())
    }
    // sirno:witness:project-config:end

    /// Resolve the monograph path relative to a config file path when configured.
    // sirno:witness:project-config:begin
    pub fn resolve_mono(&self, config_path: impl AsRef<Path>) -> Option<PathBuf> {
        self.mono
            .as_ref()
            .map(|mono| Self::resolve_config_relative(config_path.as_ref(), &mono.path))
    }

    /// Resolve the entry lake path relative to a config file path.
    pub fn resolve_lake(&self, config_path: impl AsRef<Path>) -> PathBuf {
        Self::resolve_config_relative(config_path.as_ref(), &self.lake.path)
    }

    /// Resolve the Sirno Frost root path relative to a config file path when configured.
    pub fn resolve_frost(&self, config_path: impl AsRef<Path>) -> Option<PathBuf> {
        self.frost
            .as_ref()
            .map(|frost| Self::resolve_config_relative(config_path.as_ref(), &frost.path))
    }
    // sirno:witness:project-config:end

    /// Validate this config as it would be used from a specific config file path.
    // sirno:witness:project-config:begin
    pub fn validate_for_file(&self, config_path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let config_path = config_path.as_ref();
        self.lake.validate()?;
        if let Some(repo) = &self.repo {
            repo.validate()?;
        }
        self.validate_structural_fields()?;
        self.witness.validate()?;
        if self.frost.is_some() {
            let lake = self.resolve_lake(config_path);
            let frost = self.resolve_frost(config_path).expect("frost path exists after is_some");
            if lake == frost || frost.starts_with(&lake) || lake.starts_with(&frost) {
                return Err(ConfigError::FrostLakePath { lake, frost });
            }
        }
        Ok(())
    }

    fn validate_structural_fields(&self) -> Result<(), ConfigError> {
        for (field, _) in self.structural.fields() {
            if field.is_empty() || field.contains('\n') || field.contains('\r') {
                return Err(ConfigError::StructuralFieldName(field.to_owned()));
            }
            if matches!(field, NAME_FIELD | DESC_FIELD | FROZEN_FIELD) {
                return Err(ConfigError::ReservedStructuralField(field.to_owned()));
            }
        }
        Ok(())
    }

    fn to_toml(&self) -> Result<String, ConfigError> {
        ConfigRenderer::render(self).map_err(ConfigError::Render)
    }

    fn resolve_config_relative(config_path: &Path, configured_path: &Path) -> PathBuf {
        if configured_path.is_absolute() {
            return configured_path.to_path_buf();
        }
        config_path.parent().unwrap_or_else(|| Path::new(".")).join(configured_path)
    }
}
// sirno:witness:project-config:end

struct ConfigRenderer {
    out: String,
}

impl ConfigRenderer {
    fn render(config: &SirnoConfig) -> Result<String, toml::ser::Error> {
        let mut renderer = Self { out: String::new() };
        renderer.push_config(config)?;
        Ok(renderer.out)
    }

    fn push_config(&mut self, config: &SirnoConfig) -> Result<(), toml::ser::Error> {
        if let Some(mono) = &config.mono {
            self.push_table("mono");
            // sirno:witness:project-config-comments:begin
            self.push_field(
                "path",
                &mono.path,
                "Markdown monograph path, resolved relative to this config file.",
            )?;
            // sirno:witness:project-config-comments:end
            self.out.push('\n');
        }

        self.push_table("lake");
        // sirno:witness:project-config-comments:begin
        self.push_field(
            "path",
            &config.lake.path,
            "Markdown entry lake path, resolved relative to this config file.",
        )?;
        if !config.lake.ignore.is_empty() {
            self.push_field(
                "ignore",
                &config.lake.ignore,
                "Lake-root paths Sirno skips while reading, checking, querying, and generating links.",
            )?;
        }
        // sirno:witness:project-config-comments:end

        if let Some(frost) = &config.frost {
            self.out.push('\n');
            self.push_table("frost");
            // sirno:witness:project-config-comments:begin
            self.push_field(
                "path",
                &frost.path,
                "Sirno Frost root, kept outside the public lake.",
            )?;
            // sirno:witness:project-config-comments:end
        }

        if let Some(repo) = &config.repo
            && !repo.members.is_empty()
        {
            self.out.push('\n');
            self.push_table("repo");
            // sirno:witness:project-config-comments:begin
            self.push_field(
                "members",
                &repo.members,
                "Repository files, directories, or globs scanned for witness blocks.",
            )?;
            // sirno:witness:project-config-comments:end
        }

        self.out.push('\n');
        self.push_table("witness");
        // sirno:witness:project-config-comments:begin
        self.push_witness_delimiters(&config.witness.delimiters)?;
        // sirno:witness:project-config-comments:end

        self.out.push('\n');
        self.push_table("check");
        // sirno:witness:project-config-comments:begin
        self.push_field(
            "link",
            &config.check.link,
            "Require generated footers to match current metadata during checks.",
        )?;
        // sirno:witness:project-config-comments:end

        self.out.push('\n');
        self.push_table("structural");
        // sirno:witness:project-config-comments:begin
        for (field, settings) in config.structural.fields() {
            self.push_field(
                field,
                settings,
                "Structural metadata field; link.to, link.from, and link.clique default to false.",
            )?;
        }
        // sirno:witness:project-config-comments:end

        Ok(())
    }

    fn push_table(&mut self, name: &str) {
        self.out.push('[');
        self.out.push_str(name);
        self.out.push_str("]\n");
    }

    fn push_field<T: Serialize + ?Sized>(
        &mut self, name: &str, value: &T, comment: &str,
    ) -> Result<(), toml::ser::Error> {
        self.out.push_str("# ");
        self.out.push_str(comment);
        self.out.push('\n');
        self.out.push_str(name);
        self.out.push_str(" = ");
        self.out.push_str(&Self::toml_value(value)?);
        self.out.push('\n');
        Ok(())
    }

    // sirno:witness:project-config-comments:begin
    fn push_witness_delimiters(
        &mut self, delimiters: &[WitnessDelimiterSettings],
    ) -> Result<(), toml::ser::Error> {
        self.out.push_str(
            "# Witness delimiter regex pairs; each first capture group is the entry id.\n",
        );
        self.out.push_str("# Canonical filename entry-id capture: ");
        self.out.push_str(WITNESS_ENTRY_ID_CAPTURE_REGEX);
        self.out.push('\n');
        for (index, delimiter) in delimiters.iter().enumerate() {
            if index > 0 {
                self.out.push('\n');
            }
            self.push_array_table("witness.delimiters");
            self.push_bare_field("begin", &delimiter.begin)?;
            self.push_bare_field("end", &delimiter.end)?;
        }
        Ok(())
    }
    // sirno:witness:project-config-comments:end

    fn push_bare_field<T: Serialize + ?Sized>(
        &mut self, name: &str, value: &T,
    ) -> Result<(), toml::ser::Error> {
        self.out.push_str(name);
        self.out.push_str(" = ");
        self.out.push_str(&Self::toml_value(value)?);
        self.out.push('\n');
        Ok(())
    }

    fn push_array_table(&mut self, name: &str) {
        self.out.push_str("[[");
        self.out.push_str(name);
        self.out.push_str("]]\n");
    }

    fn toml_value<T: Serialize + ?Sized>(value: &T) -> Result<String, toml::ser::Error> {
        Ok(toml::Value::try_from(value)?.to_string())
    }
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
    #[error("failed to parse config file {path}: {source}")]
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
    /// A lake ignore path is not relative to the lake root.
    #[error("lake.ignore path must be relative to the lake root: {0}")]
    LakeIgnorePath(PathBuf),
    /// A repo member path or glob is not relative to the config directory.
    #[error("repo.members path must be relative to the config directory: {0}")]
    RepoMemberPath(String),
    /// A structural field name cannot be used as a metadata key.
    #[error("structural field name must be a non-empty single-line metadata key: {0}")]
    StructuralFieldName(String),
    /// A structural field name is reserved for scalar Sirno metadata.
    #[error("structural field name is reserved for Sirno metadata: {0}")]
    ReservedStructuralField(String),
    /// No witness delimiter pairs are configured.
    #[error("witness.delimiters must contain at least one delimiter pair")]
    WitnessDelimiterList,
    /// A witness delimiter regex is empty.
    #[error("{field} at index {index} must not be empty")]
    WitnessRegex {
        /// Config field that contained an empty regex.
        field: &'static str,
        /// Zero-based delimiter pair index.
        index: usize,
    },
    /// A witness delimiter regex is invalid.
    #[error("{field} at index {index} contains an invalid regex")]
    WitnessRegexSyntax {
        /// Config field that contained an invalid regex.
        field: &'static str,
        /// Zero-based delimiter pair index.
        index: usize,
        /// Regex parser error.
        #[source]
        source: regex::Error,
    },
    /// A witness delimiter regex does not capture an entry id.
    #[error("{field} at index {index} must capture the entry id")]
    WitnessRegexCapture {
        /// Config field that did not declare a capture group.
        field: &'static str,
        /// Zero-based delimiter pair index.
        index: usize,
    },
    /// A witness delimiter regex can match empty text.
    #[error("{field} at index {index} must not match empty text")]
    WitnessRegexEmptyMatch {
        /// Config field that can match empty text.
        field: &'static str,
        /// Zero-based delimiter pair index.
        index: usize,
    },
    /// The Sirno Frost root overlaps the public lake path.
    #[error("frost path must be separate from public lake path: lake={lake} frost={frost}")]
    FrostLakePath {
        /// Resolved public lake path.
        lake: PathBuf,
        /// Resolved Sirno Frost root path.
        frost: PathBuf,
    },
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

    const TEST_WITNESS_BEGIN_REGEX: &str = "(?m)^BEGIN ([A-Za-z0-9_-]+)$";
    const TEST_WITNESS_END_REGEX: &str = "(?m)^END ([A-Za-z0-9_-]+)$";

    fn test_witness_syntax() -> WitnessSettings {
        WitnessSettings {
            delimiters: vec![WitnessDelimiterSettings::new(
                TEST_WITNESS_BEGIN_REGEX,
                TEST_WITNESS_END_REGEX,
            )],
        }
    }

    fn config_source(source: &str) -> String {
        format!(
            "{source}\n[witness]\n[[witness.delimiters]]\nbegin = '{begin}'\nend = '{end}'\n",
            begin = TEST_WITNESS_BEGIN_REGEX,
            end = TEST_WITNESS_END_REGEX,
        )
    }

    fn parse_config(source: &str) -> SirnoConfig {
        toml::from_str(&config_source(source)).unwrap()
    }

    #[test]
    fn parses_minimal_config() {
        let config = parse_config(
            r#"
[lake]
path = "docs"
"#,
        );

        assert_eq!(config.mono, None);
        assert_eq!(config.lake.path, PathBuf::from("docs"));
        assert_eq!(config.frost, None);
        assert!(config.lake.ignore.is_empty());
        assert_eq!(config.repo, None);
        assert_eq!(config.witness, test_witness_syntax());
        assert_eq!(config.check, CheckSettings::default());
        assert_eq!(config.structural, StructuralSettings::default());
    }

    #[test]
    fn parses_optional_mono_settings() {
        let config = parse_config(
            r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"
"#,
        );

        assert_eq!(config.mono, Some(MonoSettings { path: PathBuf::from("DESIGN.md") }));
    }

    #[test]
    fn parses_frost_settings() {
        let config = parse_config(
            r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"

[frost]
path = "sirno-frost"
"#,
        );

        assert_eq!(config.frost, Some(FrostSettings { path: PathBuf::from("sirno-frost") }));
    }

    #[test]
    fn parses_check_settings() {
        let config = parse_config(
            r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"

[check]
link = false
"#,
        );

        assert_eq!(config.check, CheckSettings { link: false });
    }

    #[test]
    fn parses_repo_members() {
        let config = parse_config(
            r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"

[repo]
members = ["src", "Cargo.toml", "crates/*/src"]
"#,
        );

        assert_eq!(
            config.repo,
            Some(RepoSettings {
                members: vec![
                    RepoMember::new("src").unwrap(),
                    RepoMember::new("Cargo.toml").unwrap(),
                    RepoMember::new("crates/*/src").unwrap(),
                ],
            })
        );
    }

    #[test]
    fn parses_witness_syntax_settings() {
        let config: SirnoConfig = toml::from_str(
            r#"
[lake]
path = "docs"

[witness]
[[witness.delimiters]]
begin = '(?m)^BEGIN ([A-Za-z0-9_-]+)$'
end = '(?m)^END ([A-Za-z0-9_-]+)$'

[[witness.delimiters]]
begin = '(?m)^START ([A-Za-z0-9_-]+)$'
end = '(?m)^STOP ([A-Za-z0-9_-]+)$'
"#,
        )
        .unwrap();

        assert_eq!(
            config.witness,
            WitnessSettings {
                delimiters: vec![
                    WitnessDelimiterSettings::new(
                        "(?m)^BEGIN ([A-Za-z0-9_-]+)$",
                        "(?m)^END ([A-Za-z0-9_-]+)$",
                    ),
                    WitnessDelimiterSettings::new(
                        "(?m)^START ([A-Za-z0-9_-]+)$",
                        "(?m)^STOP ([A-Za-z0-9_-]+)$",
                    ),
                ],
            }
        );
    }

    #[test]
    fn parses_structural_settings() {
        let config = parse_config(
            r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"

[structural]
kind = { link = { to = true } }
area = { link = { from = true, to = true, clique = true } }
parent = { link = { from = true } }
"#,
        );

        assert_eq!(
            config.structural,
            StructuralSettings::from_fields([
                (
                    "kind",
                    crate::links::StructuralFieldSettings::new(
                        crate::links::StructuralLinkSettings::new(true, false, false),
                    ),
                ),
                (
                    "area",
                    crate::links::StructuralFieldSettings::new(
                        crate::links::StructuralLinkSettings::new(true, true, true),
                    ),
                ),
                (
                    "parent",
                    crate::links::StructuralFieldSettings::new(
                        crate::links::StructuralLinkSettings::new(false, true, false),
                    ),
                ),
            ])
        );
    }

    #[test]
    fn structural_link_fields_default_to_false() {
        let config = parse_config(
            r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"

[structural]
topic = { link = {} }
"#,
        );

        assert_eq!(
            config.structural,
            StructuralSettings::from_fields([(
                "topic",
                crate::links::StructuralFieldSettings::default(),
            )])
        );
    }

    #[test]
    fn parses_structural_subtables() {
        let config = parse_config(
            r#"
[lake]
path = "docs"

[structural.kind]
link = { to = true }

[structural.topic]
link = { clique = true }
"#,
        );

        assert_eq!(
            config.structural,
            StructuralSettings::from_fields([
                (
                    "kind",
                    crate::links::StructuralFieldSettings::new(
                        crate::links::StructuralLinkSettings::new(true, false, false),
                    ),
                ),
                (
                    "topic",
                    crate::links::StructuralFieldSettings::new(
                        crate::links::StructuralLinkSettings::new(false, false, true),
                    ),
                ),
            ])
        );
    }

    #[test]
    fn parses_lake_ignore_settings() {
        let config = parse_config(
            r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"
ignore = [".obsidian", "drafts"]
"#,
        );

        assert_eq!(config.lake.path, PathBuf::from("docs"));
        assert_eq!(config.lake.ignore, vec![PathBuf::from(".obsidian"), PathBuf::from("drafts")]);
    }

    #[test]
    fn rejects_unknown_fields() {
        let source = config_source(
            r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"
extra = "no"
"#,
        );
        let error = toml::from_str::<SirnoConfig>(&source).unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn rejects_missing_witness_syntax() {
        let error = toml::from_str::<SirnoConfig>(
            r#"
[lake]
path = "docs"
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("missing field `witness`"));
    }

    #[test]
    fn resolves_relative_paths_against_config_directory() {
        let config = SirnoConfig::default_project().with_mono("DESIGN.md");
        let config_path = Path::new("/tmp/project/Sirno.toml");

        assert_eq!(config.resolve_mono(config_path), Some(PathBuf::from("/tmp/project/DESIGN.md")));
        assert_eq!(config.resolve_lake(config_path), PathBuf::from("/tmp/project/docs"));
        assert_eq!(config.resolve_frost(config_path), None);
        assert_eq!(
            config.with_frost("sirno-frost").resolve_frost(config_path),
            Some(PathBuf::from("/tmp/project/sirno-frost"))
        );
    }

    #[test]
    fn rejects_ignore_paths_outside_lake_root() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            config_source(
                r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"
ignore = ["../outside"]
"#,
            ),
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(error, ConfigError::LakeIgnorePath(_)));
    }

    #[test]
    fn rejects_repo_members_outside_config_root() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            config_source(
                r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"

[repo]
members = ["../outside"]
"#,
            ),
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(error, ConfigError::RepoMemberPath(_)));
    }

    #[test]
    fn rejects_empty_witness_regex() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            r#"
[lake]
path = "docs"

[witness]
[[witness.delimiters]]
begin = ""
end = '(?m)^END ([A-Za-z0-9_-]+)$'
"#,
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(
            error,
            ConfigError::WitnessRegex { field, index: 0 }
                if field == "witness.delimiters.begin"
        ));
    }

    #[test]
    fn rejects_invalid_witness_regex() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            r#"
[lake]
path = "docs"

[witness]
[[witness.delimiters]]
begin = '('
end = '(?m)^END ([A-Za-z0-9_-]+)$'
"#,
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(
            error,
            ConfigError::WitnessRegexSyntax { field, index: 0, .. }
                if field == "witness.delimiters.begin"
        ));
    }

    #[test]
    fn rejects_witness_regex_without_capture() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            r#"
[lake]
path = "docs"

[witness]
[[witness.delimiters]]
begin = '(?m)^BEGIN$'
end = '(?m)^END ([A-Za-z0-9_-]+)$'
"#,
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(
            error,
            ConfigError::WitnessRegexCapture { field, index: 0 }
                if field == "witness.delimiters.begin"
        ));
    }

    #[test]
    fn rejects_empty_matching_witness_regex() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            r#"
[lake]
path = "docs"

[witness]
[[witness.delimiters]]
begin = '()'
end = '(?m)^END ([A-Za-z0-9_-]+)$'
"#,
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(
            error,
            ConfigError::WitnessRegexEmptyMatch { field, index: 0 }
                if field == "witness.delimiters.begin"
        ));
    }

    #[test]
    fn rejects_empty_witness_delimiter_list() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            r#"
[lake]
path = "docs"

[witness]
delimiters = []
"#,
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(error, ConfigError::WitnessDelimiterList));
    }

    #[test]
    fn standard_witness_regexes_use_canonical_entry_id_capture() {
        let syntax = WitnessSettings::standard();

        for delimiter in syntax.delimiters {
            assert!(delimiter.begin.contains(WITNESS_ENTRY_ID_CAPTURE_REGEX));
            assert!(delimiter.end.contains(WITNESS_ENTRY_ID_CAPTURE_REGEX));
        }
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

    #[test]
    fn default_project_writes_witness_syntax_and_omits_optional_tables() {
        let source = SirnoConfig::default_project().to_toml().unwrap();

        assert!(source.contains("[lake]"));
        assert!(source.contains("[witness]"));
        assert!(source.contains("[[witness.delimiters]]"));
        assert!(source.contains("# Markdown entry lake path"));
        assert!(source.contains("# Witness delimiter regex pairs"));
        assert!(source.contains(&format!(
            "# Canonical filename entry-id capture: {WITNESS_ENTRY_ID_CAPTURE_REGEX}"
        )));
        assert!(!source.contains("# Opening witness delimiter regex."));
        assert!(!source.contains("# Closing witness delimiter regex."));
        assert!(source.contains("# Require generated footers"));
        assert!(source.contains("[structural]"));
        assert!(!source.contains("# Structural metadata field"));
        assert!(!source.contains("[mono]"));
        assert!(!source.contains("[repo]"));
    }

    #[test]
    fn rendered_config_keeps_selected_comments_and_structural_link_order() {
        let config = SirnoConfig {
            mono: Some(MonoSettings::new("DESIGN.md")),
            lake: LakeSettings {
                path: PathBuf::from("docs"),
                ignore: vec![PathBuf::from(".obsidian")],
            },
            frost: Some(FrostSettings::new("sirno-frost")),
            repo: Some(RepoSettings { members: vec![RepoMember::new("src").unwrap()] }),
            witness: test_witness_syntax(),
            check: CheckSettings { link: false },
            structural: StructuralSettings::from_fields([
                (
                    "kind",
                    crate::links::StructuralFieldSettings::new(
                        crate::links::StructuralLinkSettings::enabled(),
                    ),
                ),
                (
                    "area",
                    crate::links::StructuralFieldSettings::new(
                        crate::links::StructuralLinkSettings::new(true, true, true),
                    ),
                ),
                ("parent", crate::links::StructuralFieldSettings::default()),
            ]),
        };

        let source = config.to_toml().unwrap();
        let read: SirnoConfig = toml::from_str(&source).unwrap();

        assert_eq!(read, config);
        assert!(source.contains("# Markdown monograph path"));
        assert!(source.contains("# Markdown entry lake path"));
        assert!(source.contains("# Lake-root paths Sirno skips"));
        assert!(source.contains("# Sirno Frost root"));
        assert!(source.contains("# Repository files, directories, or globs"));
        assert!(source.contains("# Witness delimiter regex pairs"));
        assert!(source.contains(&format!(
            "# Canonical filename entry-id capture: {WITNESS_ENTRY_ID_CAPTURE_REGEX}"
        )));
        assert!(!source.contains("# Opening witness delimiter regex."));
        assert!(!source.contains("# Closing witness delimiter regex."));
        assert!(source.contains("# Require generated footers"));
        assert!(source.contains("[structural]"));
        assert!(source.contains("# Structural metadata field"));
        assert!(source.contains("kind = { link = { to = true, from = true } }"));
        assert!(source.contains("area = { link = { to = true, from = true, clique = true } }"));
    }

    #[test]
    fn rejects_frost_path_inside_public_lake() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            config_source(
                r#"
[mono]
path = "DESIGN.md"

[lake]
path = "docs"

[frost]
path = "docs/frost"
"#,
            ),
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(error, ConfigError::FrostLakePath { .. }));
    }
}
