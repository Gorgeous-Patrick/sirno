//! Project configuration for a Sirno-managed repository.
//!
//! A repository is Sirno-managed when it contains `Sirno.toml`.
//! The config names the public Markdown entry store.
//! It may also opt into a monograph, repository witness members, and private `eter` history store.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::links::GeneratedLinkSettings;

/// Canonical Sirno project config filename.
pub const CONFIG_FILE_NAME: &str = "Sirno.toml";

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

/// Configured public Markdown store settings.
///
/// Invariant: `path` points to the public Markdown entry store.
/// `ignore` contains paths relative to the store root that Sirno does not read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreSettings {
    /// Configured public Markdown entry store path.
    pub path: PathBuf,
    /// Store-root-relative paths ignored by Sirno.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<PathBuf>,
}

impl StoreSettings {
    /// Construct store settings from a store path and no ignored paths.
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
                return Err(ConfigError::StoreIgnorePath(path.clone()));
            }
        }
        Ok(())
    }
}

/// Configured private history store settings.
///
/// Invariant: `path` points to the `eter` history root used for snapshots.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistorySettings {
    /// Configured private history store path.
    pub path: PathBuf,
}

impl HistorySettings {
    /// Construct history settings from a history root path.
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
// sirno:witness:code-form:begin
pub struct CodeMember {
    pattern: String,
}
// sirno:witness:code-form:end

impl CodeMember {
    /// Construct one code-member pattern.
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
            return Err(ConfigError::CodeMemberPath(self.pattern.clone()));
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
// sirno:witness:code-form:begin
pub struct CodeSettings {
    /// Config-relative paths or globs scanned through `mosaika`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<CodeMember>,
}
// sirno:witness:code-form:end

impl CodeSettings {
    fn validate(&self) -> Result<(), ConfigError> {
        for member in &self.members {
            member.validate()?;
        }
        Ok(())
    }
}

/// Sirno project configuration.
///
/// `store.path` points to the configured public Markdown entry store path.
/// `mono.path`, when present, points to the configured monograph path.
/// `history.path`, when present, points to the configured private `eter` history root.
/// `store.ignore` contains paths relative to the store root that Sirno skips.
/// `code.members`, when present, contains relative member paths or globs for witness lookup.
/// `check` controls optional structural check families.
/// `links` controls generated-link footer content.
/// Relative paths are resolved against the directory containing `Sirno.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:project-config:begin
pub struct SirnoConfig {
    /// Configured monograph settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mono: Option<MonoSettings>,
    /// Configured public Markdown entry store settings.
    pub store: StoreSettings,
    /// Configured private history store settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<HistorySettings>,
    /// Configured repository artifact members.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<CodeSettings>,
    /// Structural check settings.
    #[serde(default)]
    pub check: CheckSettings,
    /// Generated-link footer settings.
    #[serde(default)]
    pub links: GeneratedLinkSettings,
}
// sirno:witness:project-config:end

impl SirnoConfig {
    /// Construct a config from the required store path.
    // sirno:witness:project-config:begin
    pub fn new(store: impl Into<PathBuf>) -> Self {
        Self {
            mono: None,
            store: StoreSettings::new(store),
            history: None,
            code: None,
            check: CheckSettings::default(),
            links: GeneratedLinkSettings::default(),
        }
    }
    // sirno:witness:project-config:end

    /// Return this config with a configured monograph path.
    pub fn with_mono(mut self, mono: impl Into<PathBuf>) -> Self {
        self.mono = Some(MonoSettings::new(mono));
        self
    }

    /// Return this config with a configured public store path.
    pub fn with_store(mut self, store: impl Into<PathBuf>) -> Self {
        self.store.path = store.into();
        self
    }

    /// Return this config with a configured history root.
    pub fn with_history(mut self, history: impl Into<PathBuf>) -> Self {
        self.history = Some(HistorySettings::new(history));
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
        self.mono.as_ref().map(|mono| resolve_config_relative(config_path.as_ref(), &mono.path))
    }

    /// Resolve the entry store path relative to a config file path.
    pub fn resolve_store(&self, config_path: impl AsRef<Path>) -> PathBuf {
        resolve_config_relative(config_path.as_ref(), &self.store.path)
    }

    /// Resolve the history store path relative to a config file path when configured.
    pub fn resolve_history(&self, config_path: impl AsRef<Path>) -> Option<PathBuf> {
        self.history
            .as_ref()
            .map(|history| resolve_config_relative(config_path.as_ref(), &history.path))
    }
    // sirno:witness:project-config:end

    /// Validate this config as it would be used from a specific config file path.
    // sirno:witness:project-config:begin
    pub fn validate_for_file(&self, config_path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let config_path = config_path.as_ref();
        self.store.validate()?;
        if let Some(code) = &self.code {
            code.validate()?;
        }
        if self.history.is_some() {
            let store = self.resolve_store(config_path);
            let history =
                self.resolve_history(config_path).expect("history path exists after is_some");
            if store == history || history.starts_with(&store) || store.starts_with(&history) {
                return Err(ConfigError::HistoryStorePath { store, history });
            }
        }
        Ok(())
    }

    fn to_toml(&self) -> Result<String, ConfigError> {
        render_config(self).map_err(ConfigError::Render)
    }
}
// sirno:witness:project-config:end

fn resolve_config_relative(config_path: &Path, configured_path: &Path) -> PathBuf {
    if configured_path.is_absolute() {
        return configured_path.to_path_buf();
    }
    config_path.parent().unwrap_or_else(|| Path::new(".")).join(configured_path)
}

fn render_config(config: &SirnoConfig) -> Result<String, toml::ser::Error> {
    let mut out = String::new();

    if let Some(mono) = &config.mono {
        push_table(&mut out, "mono");
        // sirno:witness:project-config-comments:begin
        push_field(
            &mut out,
            "path",
            &mono.path,
            "Markdown monograph path, resolved relative to this config file.",
        )?;
        // sirno:witness:project-config-comments:end
        out.push('\n');
    }

    push_table(&mut out, "store");
    // sirno:witness:project-config-comments:begin
    push_field(
        &mut out,
        "path",
        &config.store.path,
        "Markdown entry store path, resolved relative to this config file.",
    )?;
    if !config.store.ignore.is_empty() {
        push_field(
            &mut out,
            "ignore",
            &config.store.ignore,
            "Store-root paths Sirno skips while reading, checking, querying, and generating links.",
        )?;
    }
    // sirno:witness:project-config-comments:end

    if let Some(history) = &config.history {
        out.push('\n');
        push_table(&mut out, "history");
        // sirno:witness:project-config-comments:begin
        push_field(
            &mut out,
            "path",
            &history.path,
            "Private eter history root, kept outside the public store.",
        )?;
        // sirno:witness:project-config-comments:end
    }

    if let Some(code) = &config.code
        && !code.members.is_empty()
    {
        out.push('\n');
        push_table(&mut out, "code");
        // sirno:witness:project-config-comments:begin
        push_field(
            &mut out,
            "members",
            &code.members,
            "Repository files, directories, or globs scanned for witness blocks.",
        )?;
        // sirno:witness:project-config-comments:end
    }

    out.push('\n');
    push_table(&mut out, "check");
    // sirno:witness:project-config-comments:begin
    push_field(
        &mut out,
        "link",
        &config.check.link,
        "Require generated footers to match current metadata during checks.",
    )?;
    // sirno:witness:project-config-comments:end

    out.push('\n');
    push_table(&mut out, "links");
    // sirno:witness:project-config-comments:begin
    push_field(
        &mut out,
        "category",
        &config.links.category,
        "Include category links; use a boolean or { to = bool, from = bool }.",
    )?;
    push_field(
        &mut out,
        "clustee",
        &config.links.clustee,
        "Include clustee links; use a boolean or { to = bool, from = bool }.",
    )?;
    push_field(
        &mut out,
        "clique",
        &config.links.clique,
        "Add clique sections derived from clustee closures.",
    )?;
    push_field(
        &mut out,
        "refiner",
        &config.links.refiner,
        "Include refiner links; use a boolean or { to = bool, from = bool }.",
    )?;
    // sirno:witness:project-config-comments:end

    Ok(out)
}

fn push_table(out: &mut String, name: &str) {
    out.push('[');
    out.push_str(name);
    out.push_str("]\n");
}

fn push_field<T: Serialize + ?Sized>(
    out: &mut String, name: &str, value: &T, comment: &str,
) -> Result<(), toml::ser::Error> {
    out.push_str("# ");
    out.push_str(comment);
    out.push('\n');
    out.push_str(name);
    out.push_str(" = ");
    out.push_str(&toml_value(value)?);
    out.push('\n');
    Ok(())
}

fn toml_value<T: Serialize + ?Sized>(value: &T) -> Result<String, toml::ser::Error> {
    Ok(toml::Value::try_from(value)?.to_string())
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
    /// A store ignore path is not relative to the store root.
    #[error("store.ignore path must be relative to the store root: {0}")]
    StoreIgnorePath(PathBuf),
    /// A code member path or glob is not relative to the config directory.
    #[error("code.members path must be relative to the config directory: {0}")]
    CodeMemberPath(String),
    /// The history root overlaps the public store path.
    #[error(
        "history path must be separate from public store path: store={store} history={history}"
    )]
    HistoryStorePath {
        /// Resolved public store path.
        store: PathBuf,
        /// Resolved history root path.
        history: PathBuf,
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

    #[test]
    fn parses_minimal_config() {
        let config: SirnoConfig = toml::from_str(
            r#"
[store]
path = "docs"
"#,
        )
        .unwrap();

        assert_eq!(config.mono, None);
        assert_eq!(config.store.path, PathBuf::from("docs"));
        assert_eq!(config.history, None);
        assert!(config.store.ignore.is_empty());
        assert_eq!(config.code, None);
        assert_eq!(config.check, CheckSettings::default());
        assert_eq!(config.links, GeneratedLinkSettings::default());
    }

    #[test]
    fn parses_optional_mono_settings() {
        let config: SirnoConfig = toml::from_str(
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"
"#,
        )
        .unwrap();

        assert_eq!(config.mono, Some(MonoSettings { path: PathBuf::from("DESIGN.md") }));
    }

    #[test]
    fn parses_history_settings() {
        let config: SirnoConfig = toml::from_str(
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"

[history]
path = "sirno-history"
"#,
        )
        .unwrap();

        assert_eq!(config.history, Some(HistorySettings { path: PathBuf::from("sirno-history") }));
    }

    #[test]
    fn parses_check_settings() {
        let config: SirnoConfig = toml::from_str(
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"

[check]
link = false
"#,
        )
        .unwrap();

        assert_eq!(config.check, CheckSettings { link: false });
    }

    #[test]
    fn parses_code_members() {
        let config: SirnoConfig = toml::from_str(
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"

[code]
members = ["src", "Cargo.toml", "crates/*/src"]
"#,
        )
        .unwrap();

        assert_eq!(
            config.code,
            Some(CodeSettings {
                members: vec![
                    CodeMember::new("src").unwrap(),
                    CodeMember::new("Cargo.toml").unwrap(),
                    CodeMember::new("crates/*/src").unwrap(),
                ],
            })
        );
    }

    #[test]
    fn parses_link_settings() {
        let config: SirnoConfig = toml::from_str(
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"

[links]
category = true
clustee = false
clique = true
refiner = true
"#,
        )
        .unwrap();

        assert_eq!(
            config.links,
            GeneratedLinkSettings {
                category: true.into(),
                clustee: false.into(),
                clique: true,
                refiner: true.into(),
            }
        );
    }

    #[test]
    fn parses_link_side_settings() {
        let config: SirnoConfig = toml::from_str(
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"

[links]
category = { to = true, from = false }
clustee = true
refiner = { to = false, from = true }
"#,
        )
        .unwrap();

        assert_eq!(
            config.links,
            GeneratedLinkSettings {
                category: crate::links::GeneratedLinkFieldSettings::new(true, false),
                clustee: crate::links::GeneratedLinkFieldSettings::new(true, true),
                clique: false,
                refiner: crate::links::GeneratedLinkFieldSettings::new(false, true),
            }
        );
    }

    #[test]
    fn parses_store_ignore_settings() {
        let config: SirnoConfig = toml::from_str(
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"
ignore = [".obsidian", "drafts"]
"#,
        )
        .unwrap();

        assert_eq!(config.store.path, PathBuf::from("docs"));
        assert_eq!(config.store.ignore, vec![PathBuf::from(".obsidian"), PathBuf::from("drafts")]);
    }

    #[test]
    fn rejects_unknown_fields() {
        let error = toml::from_str::<SirnoConfig>(
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"
extra = "no"
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn resolves_relative_paths_against_config_directory() {
        let config = SirnoConfig::default_project().with_mono("DESIGN.md");
        let config_path = Path::new("/tmp/project/Sirno.toml");

        assert_eq!(config.resolve_mono(config_path), Some(PathBuf::from("/tmp/project/DESIGN.md")));
        assert_eq!(config.resolve_store(config_path), PathBuf::from("/tmp/project/docs"));
        assert_eq!(config.resolve_history(config_path), None);
        assert_eq!(
            config.with_history("sirno-history").resolve_history(config_path),
            Some(PathBuf::from("/tmp/project/sirno-history"))
        );
    }

    #[test]
    fn rejects_ignore_paths_outside_store_root() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"
ignore = ["../outside"]
"#,
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(error, ConfigError::StoreIgnorePath(_)));
    }

    #[test]
    fn rejects_code_members_outside_config_root() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"

[code]
members = ["../outside"]
"#,
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(error, ConfigError::CodeMemberPath(_)));
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
    fn default_project_omits_optional_tables_when_rendered() {
        let source = SirnoConfig::default_project().to_toml().unwrap();

        assert!(source.contains("[store]"));
        assert!(source.contains("# Markdown entry store path"));
        assert!(source.contains("# Require generated footers"));
        assert!(source.contains("# Include clustee links"));
        assert!(!source.contains("[mono]"));
        assert!(!source.contains("[code]"));
    }

    #[test]
    fn rendered_config_comments_each_written_field() {
        let config = SirnoConfig {
            mono: Some(MonoSettings::new("DESIGN.md")),
            store: StoreSettings {
                path: PathBuf::from("docs"),
                ignore: vec![PathBuf::from(".obsidian")],
            },
            history: Some(HistorySettings::new("sirno-history")),
            code: Some(CodeSettings { members: vec![CodeMember::new("src").unwrap()] }),
            check: CheckSettings { link: false },
            links: GeneratedLinkSettings {
                category: true.into(),
                clustee: crate::links::GeneratedLinkFieldSettings::new(true, false),
                clique: true,
                refiner: false.into(),
            },
        };

        let source = config.to_toml().unwrap();
        let read: SirnoConfig = toml::from_str(&source).unwrap();

        assert_eq!(read, config);
        assert!(source.contains("# Markdown monograph path"));
        assert!(source.contains("# Markdown entry store path"));
        assert!(source.contains("# Store-root paths Sirno skips"));
        assert!(source.contains("# Private eter history root"));
        assert!(source.contains("# Repository files, directories, or globs"));
        assert!(source.contains("# Require generated footers"));
        assert!(source.contains("# Include category links"));
        assert!(source.contains("# Include clustee links"));
        assert!(source.contains("# Add clique sections"));
        assert!(source.contains("# Include refiner links"));
    }

    #[test]
    fn rejects_history_path_inside_public_store() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            r#"
[mono]
path = "DESIGN.md"

[store]
path = "docs"

[history]
path = "docs/history"
"#,
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(matches!(error, ConfigError::HistoryStorePath { .. }));
    }
}
