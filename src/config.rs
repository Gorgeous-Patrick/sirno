//! Project configuration for a Sirno-managed repository.
//!
//! A repository is Sirno-managed when it contains `Sirno.toml`.
//! The config names the Sirno Lake.
//! It may also opt into repository witness members.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use indexmap::IndexMap;

use crate::entry::{DESC_FIELD, FROZEN_FIELD, META_FIELD, NAME_FIELD};
use crate::identifier::EntryAtom;
use crate::structural::{StructuralRenderSettings, StructuralSettings};

/// Canonical Sirno project config filename.
pub const CONFIG_FILE_NAME: &str = "Sirno.toml";

// sirno:witness:project-config:begin
macro_rules! witness_entry_address_capture_regex {
    () => {
        r#"([^\x00-\x1F\x7F<>:"/\\|?*,\r\n]+)"#
    };
}

/// Canonical witness delimiter capture for every legal entry address.
///
/// Reserved path checks that cannot fit Rust regex syntax are enforced by `EntryAddress`.
pub const WITNESS_ENTRY_ADDRESS_CAPTURE_REGEX: &str = witness_entry_address_capture_regex!();

/// Standard opening delimiter regex for line-comment repository witness blocks.
pub const STANDARD_LINE_WITNESS_BEGIN_REGEX: &str = concat!(
    r"(?m)^[ \t]*//[ \t]*sirno:witness:",
    witness_entry_address_capture_regex!(),
    r":begin"
);

/// Standard closing delimiter regex for line-comment repository witness blocks.
pub const STANDARD_LINE_WITNESS_END_REGEX: &str =
    concat!(r"(?m)^[ \t]*//[ \t]*sirno:witness:", witness_entry_address_capture_regex!(), r":end");

/// Standard opening delimiter regex for Markdown repository witness blocks.
pub const STANDARD_MARKDOWN_WITNESS_BEGIN_REGEX: &str = concat!(
    r"(?m)^[ \t]*<!--[ \t]*sirno:witness:",
    witness_entry_address_capture_regex!(),
    r":begin[ \t]*-->"
);

/// Standard closing delimiter regex for Markdown repository witness blocks.
pub const STANDARD_MARKDOWN_WITNESS_END_REGEX: &str = concat!(
    r"(?m)^[ \t]*<!--[ \t]*sirno:witness:",
    witness_entry_address_capture_regex!(),
    r":end[ \t]*-->"
);
// sirno:witness:project-config:end

/// Settings for optional check families.
///
/// Invariant: absent flags are enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CheckSettings {
    /// Check generated footer freshness.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render: Option<bool>,
    /// Check that each configured link relation has a matching structural entry.
    #[serde(rename = "structural-inhabitance", skip_serializing_if = "Option::is_none")]
    pub structural_inhabitance: Option<bool>,
}

impl CheckSettings {
    /// Return whether generated footer freshness checking is enabled.
    pub fn render_enabled(&self) -> bool {
        self.render.unwrap_or(true)
    }

    /// Return whether configured link relations must have matching structural entries.
    pub fn structural_inhabitance_enabled(&self) -> bool {
        self.structural_inhabitance.unwrap_or(true)
    }

    fn has_explicit_flags(&self) -> bool {
        self.render.is_some() || self.structural_inhabitance.is_some()
    }
}

/// Generated content render settings.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderSettings {
    /// Structural link directions rendered in generated footers.
    pub structural: StructuralRenderSettings,
}

impl RenderSettings {
    /// Return true when no render policy is configured.
    pub fn is_empty(&self) -> bool {
        self.structural.is_empty()
    }
}

/// Optional tutorial output settings.
///
/// Invariant: table presence enables configured tutorial text for recoverable command failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TutorialSettings {
    /// Show tutorial text when frost commit is blocked by open tide workitems.
    pub frost_commit_tide: bool,
    /// Include first-snapshot bootstrap context in the frost commit tide tutorial.
    pub frost_bootstrap_tide: bool,
}

impl TutorialSettings {
    /// Construct tutorial settings with every current tutorial enabled.
    pub fn all() -> Self {
        Self { frost_commit_tide: true, frost_bootstrap_tide: true }
    }
}

impl Default for TutorialSettings {
    fn default() -> Self {
        Self::all()
    }
}

/// Configured Sirno Lake settings.
///
/// Invariant: `path` points to the Sirno Lake.
/// `ignore` contains paths relative to the lake root that Sirno does not read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LakeSettings {
    /// Configured Sirno Lake path.
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

/// Ordered upstream lake declarations keyed by their glacier domain.
pub type UpstreamSettingsMap = IndexMap<EntryAtom, UpstreamSettings>;

// sirno:witness:upstream-lake:begin
/// Configured upstream Git lake source.
///
/// Invariant: exactly one requested ref selector is present.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpstreamSettings {
    /// Git URI or local repository source accepted by Git.
    pub git: String,
    /// Branch name to resolve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Tag name to resolve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Commit-ish to resolve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// Directory inside the Git tree that contains `Sirno.toml`.
    #[serde(default = "default_upstream_project", skip_serializing_if = "is_default_project")]
    pub project: PathBuf,
}

impl UpstreamSettings {
    /// Construct branch-pinned upstream settings.
    pub fn branch(git: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            git: git.into(),
            branch: Some(branch.into()),
            tag: None,
            rev: None,
            project: default_upstream_project(),
        }
    }

    /// Construct tag-pinned upstream settings.
    pub fn tag(git: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            git: git.into(),
            branch: None,
            tag: Some(tag.into()),
            rev: None,
            project: default_upstream_project(),
        }
    }

    /// Construct commit-pinned upstream settings.
    pub fn rev(git: impl Into<String>, rev: impl Into<String>) -> Self {
        Self {
            git: git.into(),
            branch: None,
            tag: None,
            rev: Some(rev.into()),
            project: default_upstream_project(),
        }
    }

    /// Return the requested ref selector.
    pub fn selector(&self) -> UpstreamRef<'_> {
        if let Some(branch) = &self.branch {
            UpstreamRef::Branch(branch)
        } else if let Some(tag) = &self.tag {
            UpstreamRef::Tag(tag)
        } else if let Some(rev) = &self.rev {
            UpstreamRef::Rev(rev)
        } else {
            panic!("validated upstream settings always have one selector")
        }
    }

    fn validate(&self, domain: &EntryAtom) -> Result<(), ConfigError> {
        if self.git.trim().is_empty() {
            return Err(ConfigError::UpstreamGitSource(domain.clone()));
        }
        let ref_count = [self.branch.as_ref(), self.tag.as_ref(), self.rev.as_ref()]
            .into_iter()
            .flatten()
            .count();
        if ref_count != 1 {
            return Err(ConfigError::UpstreamRefSelector(domain.clone()));
        }
        for selector in
            [self.branch.as_ref(), self.tag.as_ref(), self.rev.as_ref()].into_iter().flatten()
        {
            if selector.trim().is_empty() {
                return Err(ConfigError::UpstreamRefSelector(domain.clone()));
            }
        }
        validate_upstream_project_path(domain, &self.project)?;
        Ok(())
    }
}
// sirno:witness:upstream-lake:end

/// Requested upstream Git ref selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpstreamRef<'a> {
    /// Branch name.
    Branch(&'a str),
    /// Tag name.
    Tag(&'a str),
    /// Commit-ish.
    Rev(&'a str),
}

fn default_upstream_project() -> PathBuf {
    PathBuf::from(".")
}

fn is_default_project(path: &PathBuf) -> bool {
    path == &default_upstream_project()
}

fn validate_upstream_project_path(domain: &EntryAtom, path: &Path) -> Result<(), ConfigError> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(ConfigError::UpstreamProjectPath { domain: domain.clone(), path: path.into() });
    }
    for component in path.components() {
        if matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)) {
            return Err(ConfigError::UpstreamProjectPath {
                domain: domain.clone(),
                path: path.into(),
            });
        }
    }
    Ok(())
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
/// Each regex captures the entry address as its first capture group.
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
/// Invariant: each delimiter pair is validated by `WitnessDelimiterSettings`.
/// An empty delimiter list disables repository witness lookup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:project-config:begin
pub struct WitnessSettings {
    /// Configured witness delimiter pairs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
        for (index, delimiter) in self.delimiters.iter().enumerate() {
            delimiter.validate(index)?;
        }
        Ok(())
    }
}

/// Sirno project configuration.
///
/// `lake.path` points to the configured lake path.
/// `lake.ignore` contains paths relative to the lake root that Sirno skips.
/// `repo.members`, when present, contains relative member paths or globs for witness lookup.
/// `witness` controls the delimiter syntax for repository witness blocks.
/// `check` controls optional structural check families.
/// `tutorial`, when present, enables tutorial output for recoverable command failures.
/// `structural` registers structural link relations and their defining entries.
/// `render` controls generated-link footer content.
/// Relative paths are resolved against the directory containing `Sirno.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:project-config:begin
pub struct SirnoConfig {
    /// Configured Sirno Lake settings.
    pub lake: LakeSettings,
    /// Configured upstream lakes.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub upstreams: UpstreamSettingsMap,
    /// Configured repository artifact members.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<RepoSettings>,
    /// Configured repository witness delimiter syntax.
    pub witness: WitnessSettings,
    /// Structural check settings.
    #[serde(default)]
    pub check: CheckSettings,
    /// Optional tutorial output settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tutorial: Option<TutorialSettings>,
    /// Structural link metadata relation settings.
    #[serde(default)]
    pub structural: StructuralSettings,
    /// Generated content render settings.
    #[serde(default, skip_serializing_if = "RenderSettings::is_empty")]
    pub render: RenderSettings,
}
// sirno:witness:project-config:end

impl SirnoConfig {
    /// Construct a config from the required lake path.
    // sirno:witness:project-config:begin
    pub fn new(lake: impl Into<PathBuf>) -> Self {
        Self {
            lake: LakeSettings::new(lake),
            upstreams: UpstreamSettingsMap::new(),
            repo: None,
            witness: WitnessSettings::standard(),
            check: CheckSettings::default(),
            tutorial: None,
            structural: StructuralSettings::default(),
            render: RenderSettings::default(),
        }
    }
    // sirno:witness:project-config:end

    /// Return this config with a configured Sirno Lake path.
    pub fn with_lake(mut self, lake: impl Into<PathBuf>) -> Self {
        self.lake.path = lake.into();
        self
    }

    /// Return this config with tutorial output enabled.
    pub fn with_tutorial(mut self) -> Self {
        self.tutorial = Some(TutorialSettings::all());
        self
    }

    /// Return this config with one upstream declaration set.
    pub fn with_upstream(mut self, domain: EntryAtom, settings: UpstreamSettings) -> Self {
        self.upstreams.insert(domain, settings);
        self
    }

    /// Remove one upstream declaration.
    pub fn remove_upstream(&mut self, domain: &EntryAtom) -> Option<UpstreamSettings> {
        self.upstreams.shift_remove(domain)
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
        let config = Self::from_source(path, &source)?;
        trace!("sirno config load end");
        Ok(config)
    }

    /// Load a config from source text and the path it represents.
    pub fn from_source(path: impl AsRef<Path>, source: &str) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let config: Self = toml::from_str(source)
            .map_err(|source| ConfigError::Parse { path: path.to_path_buf(), source })?;
        config.validate_for_file(path)?;
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

    // sirno:witness:project-config-comments:begin
    /// Render this config as canonical commented TOML.
    pub fn to_commented_toml(&self) -> Result<String, ConfigError> {
        self.to_toml()
    }

    /// Return canonical comment text missing from an existing config source.
    pub fn missing_comments_in(&self, source: &str) -> Result<Vec<String>, ConfigError> {
        let expected = self
            .to_commented_toml()?
            .lines()
            .filter_map(|line| line.strip_prefix("# ").map(str::to_owned))
            .collect::<Vec<_>>();
        let current = source.lines().map(str::trim).collect::<Vec<_>>();

        Ok(expected
            .into_iter()
            .filter(|comment| {
                let line = format!("# {comment}");
                !current.iter().any(|current| *current == line)
            })
            .collect())
    }
    // sirno:witness:project-config-comments:end

    // sirno:witness:project-config:begin
    /// Resolve the entry lake path relative to a config file path.
    pub fn resolve_lake(&self, config_path: impl AsRef<Path>) -> PathBuf {
        Self::resolve_config_relative(config_path.as_ref(), &self.lake.path)
    }

    // sirno:witness:project-config:end

    /// Validate this config as it would be used from a specific config file path.
    // sirno:witness:project-config:begin
    pub fn validate_for_file(&self, _config_path: impl AsRef<Path>) -> Result<(), ConfigError> {
        self.lake.validate()?;
        if let Some(repo) = &self.repo {
            repo.validate()?;
        }
        for (domain, upstream) in &self.upstreams {
            upstream.validate(domain)?;
        }
        self.validate_structural_fields()?;
        self.validate_render_settings()?;
        self.witness.validate()?;
        Ok(())
    }

    fn validate_structural_fields(&self) -> Result<(), ConfigError> {
        for (field, _) in self.structural.fields() {
            if field.is_empty()
                || field.contains('\n')
                || field.contains('\r')
                || field.contains(',')
            {
                return Err(ConfigError::StructuralFieldName(field.to_owned()));
            }
            if matches!(field, NAME_FIELD | DESC_FIELD | META_FIELD | FROZEN_FIELD) {
                return Err(ConfigError::ReservedStructuralField(field.to_owned()));
            }
        }
        Ok(())
    }

    fn validate_render_settings(&self) -> Result<(), ConfigError> {
        for (field, directions) in self.render.structural.fields() {
            if !self.structural.contains_field(field) {
                return Err(ConfigError::RenderStructuralField(field.to_owned()));
            }
            let mut seen = Vec::new();
            for direction in directions {
                if seen.contains(direction) {
                    return Err(ConfigError::DuplicateRenderStructuralDirection {
                        field: field.to_owned(),
                        direction: direction.to_string(),
                    });
                }
                seen.push(*direction);
            }
        }
        Ok(())
    }

    /// Return structural settings with generated-footer render policy applied.
    pub fn effective_structural_settings(&self) -> StructuralSettings {
        self.structural.with_render_settings(&self.render.structural)
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
        self.push_table("lake");
        // sirno:witness:project-config-comments:begin
        self.push_field(
            "path",
            &config.lake.path,
            "Sirno Lake path, resolved relative to this config file.",
        )?;
        if !config.lake.ignore.is_empty() {
            self.push_field(
                "ignore",
                &config.lake.ignore,
                "Paths in lake that Sirno skips while reading, checking, querying, and rendering footers.",
            )?;
        }
        // sirno:witness:project-config-comments:end

        if !config.upstreams.is_empty() {
            self.out.push('\n');
            self.push_upstreams(&config.upstreams)?;
        }

        if let Some(repo) = &config.repo {
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

        if config.check.has_explicit_flags() {
            self.out.push('\n');
            self.push_table("check");
            // sirno:witness:project-config-comments:begin
            if let Some(render) = config.check.render {
                self.push_field(
                    "render",
                    &render,
                    "Require generated footers to match current metadata during checks.",
                )?;
            }
            if let Some(structural_inhabitance) = config.check.structural_inhabitance {
                self.push_field(
                    "structural-inhabitance",
                    &structural_inhabitance,
                    "Require each configured link relation to have a matching structural relation entry during checks.",
                )?;
            }
            // sirno:witness:project-config-comments:end
        }

        if let Some(tutorial) = config.tutorial {
            self.out.push('\n');
            self.push_table("tutorial");
            // sirno:witness:project-config-comments:begin
            self.out.push_str(
                "# Presence of this table enables tutorial text for recoverable command failures.\n",
            );
            self.out.push_str("# Remove this table to keep CLI errors terse.\n");
            self.push_field(
                "frost_commit_tide",
                &tutorial.frost_commit_tide,
                "Show tutorial text when frost commit is blocked by open tide workitems.",
            )?;
            self.push_field(
                "frost_bootstrap_tide",
                &tutorial.frost_bootstrap_tide,
                "Include first-snapshot bootstrap context in the frost commit tide tutorial.",
            )?;
            // sirno:witness:project-config-comments:end
        }

        self.out.push('\n');
        self.push_table("structural");
        // sirno:witness:project-config-comments:begin
        self.push_structural_fields(&config.structural)?;
        // sirno:witness:project-config-comments:end

        if !config.render.is_empty() {
            self.out.push('\n');
            self.push_render_settings(&config.render)?;
        }

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
            "# Witness delimiter regex pairs; each first capture group is the entry address.\n",
        );
        self.out.push_str("# Canonical entry-address capture: ");
        self.out.push_str(WITNESS_ENTRY_ADDRESS_CAPTURE_REGEX);
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

    // sirno:witness:project-config-comments:begin
    fn push_structural_fields(
        &mut self, structural: &StructuralSettings,
    ) -> Result<(), toml::ser::Error> {
        let mut fields = structural.relations().peekable();
        if fields.peek().is_some() {
            for comment in [
                "Structural link relations.",
                "Add one [structural.FIELD] subtable for each metadata relation Sirno treats as structure.",
                "FIELD must be a non-empty single-line metadata key with no comma.",
                "FIELD cannot be name, desc, meta, or frozen.",
                "entry names the lake entry that documents the relation.",
                "Entry metadata values for FIELD must be lists of entry addresses; targets must exist by review.",
                "Tide policy lives in structural relation entry meta.ripple.lake \
                 and meta.ripple.frost direction lists.",
            ] {
                self.out.push_str("# ");
                self.out.push_str(comment);
                self.out.push('\n');
            }
        }
        for (field, entry) in fields {
            self.out.push('\n');
            self.push_table(&format!("structural.{field}"));
            self.push_bare_field("entry", entry)?;
        }
        Ok(())
    }

    fn push_render_settings(&mut self, render: &RenderSettings) -> Result<(), toml::ser::Error> {
        let mut fields = render.structural.fields().peekable();
        if fields.peek().is_none() {
            return Ok(());
        }
        self.push_table("render.structural");
        for comment in [
            "Generated-footer structural link render policy.",
            "Each key names a configured structural relation.",
            "Values are direction lists: to, from, and clique.",
        ] {
            self.out.push_str("# ");
            self.out.push_str(comment);
            self.out.push('\n');
        }
        for (field, directions) in fields {
            self.push_bare_field(field, directions)?;
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

    fn push_upstreams(&mut self, upstreams: &UpstreamSettingsMap) -> Result<(), toml::ser::Error> {
        for (index, (domain, upstream)) in upstreams.iter().enumerate() {
            if index > 0 {
                self.out.push('\n');
            }
            self.push_table(&format!("upstreams.{domain}"));
            // sirno:witness:project-config-comments:begin
            if index == 0 {
                self.out
                    .push_str(
                        "# Git-backed upstream lake crystallized into a glacier under this entry domain.\n",
                    );
            }
            // sirno:witness:project-config-comments:end
            self.push_bare_field("git", &upstream.git)?;
            if let Some(branch) = &upstream.branch {
                self.push_bare_field("branch", branch)?;
            }
            if let Some(tag) = &upstream.tag {
                self.push_bare_field("tag", tag)?;
            }
            if let Some(rev) = &upstream.rev {
                self.push_bare_field("rev", rev)?;
            }
            if !is_default_project(&upstream.project) {
                self.push_bare_field("project", &upstream.project)?;
            }
        }
        Ok(())
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
    /// A link relation name cannot be used as a metadata key.
    #[error("link relation name must be a non-empty single-line metadata key: {0}")]
    StructuralFieldName(String),
    /// A link relation name is reserved for Sirno-managed metadata.
    #[error("link relation name is reserved for Sirno metadata: {0}")]
    ReservedStructuralField(String),
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
    /// A witness delimiter regex does not capture an entry address.
    #[error("{field} at index {index} must capture the entry address")]
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
    /// An upstream Git source is empty.
    #[error("upstream `{0}` git source must not be empty")]
    UpstreamGitSource(EntryAtom),
    /// An upstream must have exactly one ref selector.
    #[error("upstream `{0}` must configure exactly one of branch, tag, or rev")]
    UpstreamRefSelector(EntryAtom),
    /// An upstream project path is not a normal Git-tree-relative path.
    #[error("upstream `{domain}` project path must be relative within the Git tree: {path}")]
    UpstreamProjectPath {
        /// Glacier domain.
        domain: EntryAtom,
        /// Invalid project path.
        path: PathBuf,
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

    fn assert_before(source: &str, before: &str, after: &str) {
        assert!(source.find(before).unwrap() < source.find(after).unwrap());
    }

    #[test]
    fn parses_minimal_config() {
        let config = parse_config(
            r#"
[lake]
path = "docs"
"#,
        );

        assert_eq!(config.lake.path, PathBuf::from("docs"));
        assert!(config.upstreams.is_empty());
        assert!(config.lake.ignore.is_empty());
        assert_eq!(config.repo, None);
        assert_eq!(config.witness, test_witness_syntax());
        assert_eq!(config.check, CheckSettings::default());
        assert!(config.check.render_enabled());
        assert!(config.check.structural_inhabitance_enabled());
        assert_eq!(config.tutorial, None);
        assert_eq!(config.structural, StructuralSettings::default());
        assert_eq!(config.render, RenderSettings::default());
    }

    #[test]
    fn rejects_frost_settings() {
        let error = SirnoConfig::from_source(
            Path::new("Sirno.toml"),
            r#"
[lake]
path = "docs"

[frost]
path = "sirno-frost"
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn parses_upstream_settings() {
        let config = parse_config(
            r#"
[lake]
path = "docs"

[upstreams.core]
git = "https://example.invalid/core.git"
branch = "main"
project = "packages/core"

[upstreams.std]
git = "../std.git"
tag = "stable"
"#,
        );

        assert_eq!(
            config.upstreams.get(&EntryAtom::new("core").unwrap()),
            Some(&UpstreamSettings {
                git: "https://example.invalid/core.git".to_owned(),
                branch: Some("main".to_owned()),
                tag: None,
                rev: None,
                project: PathBuf::from("packages/core"),
            })
        );
        assert_eq!(
            config.upstreams.get(&EntryAtom::new("std").unwrap()),
            Some(&UpstreamSettings::tag("../std.git", "stable"))
        );
    }

    #[test]
    fn parses_check_settings() {
        let config = parse_config(
            r#"
[lake]
path = "docs"

[check]
render = false
structural-inhabitance = false
"#,
        );

        assert_eq!(
            config.check,
            CheckSettings { render: Some(false), structural_inhabitance: Some(false) }
        );
        assert!(!config.check.render_enabled());
        assert!(!config.check.structural_inhabitance_enabled());
    }

    #[test]
    fn omitted_check_flags_default_to_enabled() {
        let config = parse_config(
            r#"
[lake]
path = "docs"

[check]
structural-inhabitance = false
"#,
        );

        assert_eq!(
            config.check,
            CheckSettings { render: None, structural_inhabitance: Some(false) }
        );
        assert!(config.check.render_enabled());
        assert!(!config.check.structural_inhabitance_enabled());
    }

    #[test]
    fn parses_tutorial_settings() {
        let default_tutorial = parse_config(
            r#"
[lake]
path = "docs"

[tutorial]
"#,
        );
        let selected_tutorial = parse_config(
            r#"
[lake]
path = "docs"

[tutorial]
frost_commit_tide = false
frost_bootstrap_tide = true
"#,
        );

        assert_eq!(default_tutorial.tutorial, Some(TutorialSettings::all()));
        assert_eq!(
            selected_tutorial.tutorial,
            Some(TutorialSettings { frost_commit_tide: false, frost_bootstrap_tide: true })
        );
    }

    #[test]
    fn parses_repo_members() {
        let config = parse_config(
            r#"
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
    fn parses_empty_witness_syntax_settings() {
        let bare: SirnoConfig = toml::from_str(
            r#"
[lake]
path = "docs"

[witness]
"#,
        )
        .unwrap();
        let explicit: SirnoConfig = toml::from_str(
            r#"
[lake]
path = "docs"

[witness]
delimiters = []
"#,
        )
        .unwrap();

        assert!(bare.witness.delimiters.is_empty());
        assert!(explicit.witness.delimiters.is_empty());
    }

    #[test]
    fn parses_structural_settings() {
        let config = parse_config(
            r#"
[lake]
path = "docs"

[structural.kind]
entry = "kind-entry"

[structural.area]
entry = "area"

[structural.parent]
entry = "parent"

[render.structural]
kind = ["to"]
area = ["to", "from", "clique"]
parent = ["from"]
"#,
        );

        assert_eq!(
            config.structural,
            StructuralSettings::from_relations([
                ("kind", crate::EntryAddress::new("kind-entry").unwrap()),
                ("area", crate::EntryAddress::new("area").unwrap()),
                ("parent", crate::EntryAddress::new("parent").unwrap()),
            ])
        );
        assert_eq!(
            config.render.structural,
            crate::StructuralRenderSettings::from_fields([
                ("kind", [crate::StructuralEdgeDirection::To].as_slice().iter().copied()),
                (
                    "area",
                    [
                        crate::StructuralEdgeDirection::To,
                        crate::StructuralEdgeDirection::From,
                        crate::StructuralEdgeDirection::Clique,
                    ]
                    .as_slice()
                    .iter()
                    .copied(),
                ),
                ("parent", [crate::StructuralEdgeDirection::From].as_slice().iter().copied()),
            ])
        );
        let effective = config.effective_structural_settings();
        let effective_fields = effective.fields().collect::<Vec<_>>();
        assert!(effective_fields[0].1.to.render);
        assert!(effective_fields[1].1.from.render);
        assert!(effective_fields[1].1.clique.render);
        assert!(effective_fields[2].1.from.render);
    }

    #[test]
    fn rejects_old_structural_edge_settings_in_config() {
        let error = toml::from_str::<SirnoConfig>(&config_source(
            r#"
[lake]
path = "docs"

[structural.topic]
entry = "topic"
to = { render = true }
"#,
        ))
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `to`"));
    }

    #[test]
    fn structural_relations_default_to_no_rendering() {
        let config = parse_config(
            r#"
[lake]
path = "docs"

[structural.topic]
entry = "topic"
"#,
        );

        assert_eq!(
            config.structural,
            StructuralSettings::from_relations([(
                "topic",
                crate::EntryAddress::new("topic").unwrap()
            )])
        );
        assert_eq!(config.render, RenderSettings::default());
        assert!(!config.effective_structural_settings().fields().next().unwrap().1.to.render);
    }

    #[test]
    fn parses_structural_subtables() {
        let config = parse_config(
            r#"
[lake]
path = "docs"

[structural.kind]
entry = "kind"

[structural.topic]
entry = "topic-entry"

[render.structural]
kind = ["to"]
topic = ["clique"]
"#,
        );

        assert_eq!(
            config.structural,
            StructuralSettings::from_relations([
                ("kind", crate::EntryAddress::new("kind").unwrap()),
                ("topic", crate::EntryAddress::new("topic-entry").unwrap()),
            ])
        );
        let effective = config.effective_structural_settings();
        let effective_fields = effective.fields().collect::<Vec<_>>();
        assert!(effective_fields[0].1.to.render);
        assert!(effective_fields[1].1.clique.render);
    }

    #[test]
    fn preserves_configured_structural_field_order() {
        let config = parse_config(
            r#"
[lake]
path = "docs"

[structural.zeta]
entry = "zeta"

[structural.alpha]
entry = "alpha"

[structural.middle]
entry = "middle"

[render.structural]
zeta = ["to"]
alpha = ["from"]
middle = ["clique"]
"#,
        );

        let fields = config.structural.fields().map(|(field, _)| field).collect::<Vec<_>>();
        let rendered = config.to_toml().unwrap();

        assert_eq!(fields, ["zeta", "alpha", "middle"]);
        assert_before(&rendered, "[structural.zeta]", "[structural.alpha]");
        assert_before(&rendered, "[structural.alpha]", "[structural.middle]");
    }

    #[test]
    fn parses_lake_ignore_settings() {
        let config = parse_config(
            r#"
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
        let config = SirnoConfig::default_project();
        let config_path = Path::new("/tmp/project/Sirno.toml");

        assert_eq!(config.resolve_lake(config_path), PathBuf::from("/tmp/project/docs"));
    }

    #[test]
    fn rejects_ignore_paths_outside_lake_root() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            config_source(
                r#"
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
    fn rejects_invalid_upstream_ref_selectors_and_project_paths() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            config_source(
                r#"
[lake]
path = "docs"

[upstreams.core]
git = "../core.git"
branch = "main"
tag = "stable"
"#,
            ),
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();
        assert!(
            matches!(error, ConfigError::UpstreamRefSelector(domain) if domain.as_str() == "core")
        );

        fs::write(
            &path,
            config_source(
                r#"
[lake]
path = "docs"

[upstreams.core]
git = "../core.git"
branch = "main"
project = "../core"
"#,
            ),
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();
        assert!(
            matches!(error, ConfigError::UpstreamProjectPath { domain, .. } if domain.as_str() == "core")
        );
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
    fn validates_empty_witness_delimiter_list() {
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

        let config = SirnoConfig::from_file(&path).unwrap();

        assert!(config.witness.delimiters.is_empty());
    }

    #[test]
    fn standard_witness_regexes_use_canonical_entry_address_capture() {
        let syntax = WitnessSettings::standard();

        for delimiter in syntax.delimiters {
            assert!(delimiter.begin.contains(WITNESS_ENTRY_ADDRESS_CAPTURE_REGEX));
            assert!(delimiter.end.contains(WITNESS_ENTRY_ADDRESS_CAPTURE_REGEX));
        }
    }

    #[test]
    fn standard_witness_regexes_accept_dotted_paths_and_reject_other_separators() {
        let line_begin = Regex::new(STANDARD_LINE_WITNESS_BEGIN_REGEX).unwrap();
        let markdown_begin = Regex::new(STANDARD_MARKDOWN_WITNESS_BEGIN_REGEX).unwrap();

        assert!(line_begin.is_match("// sirno:witness:valid-entry:begin"));
        assert!(line_begin.is_match("// sirno:witness:core.design:begin"));
        assert!(!line_begin.is_match("// sirno:witness:bad,id:begin"));
        assert!(!line_begin.is_match("// sirno:witness:bad\rid:begin"));
        assert!(!line_begin.is_match("// sirno:witness:bad\nid:begin"));

        assert!(markdown_begin.is_match("<!-- sirno:witness:valid-entry:begin -->"));
        assert!(markdown_begin.is_match("<!-- sirno:witness:core.design:begin -->"));
        assert!(!markdown_begin.is_match("<!-- sirno:witness:bad,id:begin -->"));
        assert!(!markdown_begin.is_match("<!-- sirno:witness:bad\rid:begin -->"));
        assert!(!markdown_begin.is_match("<!-- sirno:witness:bad\nid:begin -->"));
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
        assert!(source.contains("# Sirno Lake path"));
        assert!(source.contains("# Witness delimiter regex pairs"));
        assert!(source.contains(&format!(
            "# Canonical entry-address capture: {WITNESS_ENTRY_ADDRESS_CAPTURE_REGEX}"
        )));
        assert!(!source.contains("# Opening witness delimiter regex."));
        assert!(!source.contains("# Closing witness delimiter regex."));
        assert!(!source.contains("[check]"));
        assert!(!source.contains("# Require generated footers"));
        assert!(!source.contains("structural-inhabitance"));
        assert!(!source.contains("[tutorial]"));
        assert!(source.contains("[structural]"));
        assert!(!source.contains("# Structural metadata field"));
        assert!(!source.contains("[repo]"));
    }

    #[test]
    fn rendered_config_keeps_selected_comments_and_structural_link_order() {
        let mut upstream = UpstreamSettings::branch("https://example.invalid/core.git", "main");
        upstream.project = PathBuf::from("packages/core");
        let upstreams = UpstreamSettingsMap::from([(EntryAtom::new("core").unwrap(), upstream)]);
        let config = SirnoConfig {
            lake: LakeSettings {
                path: PathBuf::from("docs"),
                ignore: vec![PathBuf::from(".obsidian")],
            },
            upstreams,
            repo: Some(RepoSettings { members: vec![RepoMember::new("src").unwrap()] }),
            witness: test_witness_syntax(),
            check: CheckSettings { render: Some(false), structural_inhabitance: Some(false) },
            tutorial: Some(TutorialSettings {
                frost_commit_tide: true,
                frost_bootstrap_tide: false,
            }),
            structural: StructuralSettings::from_relations([
                ("kind", crate::EntryAddress::new("kind-entry").unwrap()),
                ("area", crate::EntryAddress::new("area").unwrap()),
                ("parent", crate::EntryAddress::new("parent").unwrap()),
            ]),
            render: RenderSettings {
                structural: crate::StructuralRenderSettings::from_fields([
                    (
                        "kind",
                        [crate::StructuralEdgeDirection::To, crate::StructuralEdgeDirection::From]
                            .as_slice()
                            .iter()
                            .copied(),
                    ),
                    (
                        "area",
                        [
                            crate::StructuralEdgeDirection::To,
                            crate::StructuralEdgeDirection::From,
                            crate::StructuralEdgeDirection::Clique,
                        ]
                        .as_slice()
                        .iter()
                        .copied(),
                    ),
                ]),
            },
        };

        let source = config.to_toml().unwrap();
        let read: SirnoConfig = toml::from_str(&source).unwrap();

        assert_eq!(read, config);
        assert!(source.contains("# Sirno Lake path"));
        assert!(source.contains("# Paths in lake that Sirno skips"));
        assert!(!source.contains("[frost]"));
        assert!(source.contains("[upstreams.core]"));
        assert!(source.contains(
            "# Git-backed upstream lake crystallized into a glacier under this entry domain."
        ));
        assert!(source.contains("git = \"https://example.invalid/core.git\""));
        assert!(source.contains("branch = \"main\""));
        assert!(source.contains("project = \"packages/core\""));
        assert!(source.contains("# Repository files, directories, or globs"));
        assert!(source.contains("# Witness delimiter regex pairs"));
        assert!(source.contains(&format!(
            "# Canonical entry-address capture: {WITNESS_ENTRY_ADDRESS_CAPTURE_REGEX}"
        )));
        assert!(!source.contains("# Opening witness delimiter regex."));
        assert!(!source.contains("# Closing witness delimiter regex."));
        assert!(source.contains("# Require generated footers"));
        assert!(source.contains("render = false"));
        assert!(source.contains("# Require each configured link relation"));
        assert!(source.contains("structural-inhabitance = false"));
        assert!(source.contains("[tutorial]"));
        assert!(source.contains(
            "# Presence of this table enables tutorial text for recoverable command failures."
        ));
        assert!(source.contains("# Remove this table to keep CLI errors terse."));
        assert!(
            source.contains(
                "# Show tutorial text when frost commit is blocked by open tide workitems."
            )
        );
        assert!(source.contains(
            "# Include first-snapshot bootstrap context in the frost commit tide tutorial."
        ));
        assert!(source.contains("frost_commit_tide = true"));
        assert!(source.contains("frost_bootstrap_tide = false"));
        assert!(source.contains("[structural]"));
        assert!(source.contains("# Structural link relations."));
        assert!(source.contains(
            "# Add one [structural.FIELD] subtable for each metadata relation Sirno treats as structure."
        ));
        assert!(
            source.contains("# FIELD must be a non-empty single-line metadata key with no comma.")
        );
        assert!(source.contains("# FIELD cannot be name, desc, meta, or frozen."));
        assert!(source.contains("# entry names the lake entry that documents the relation."));
        assert!(source.contains(
            "# Entry metadata values for FIELD must be lists of entry addresses; targets must exist by review."
        ));
        assert!(source.contains(
            "# Tide policy lives in structural relation entry meta.ripple.lake and meta.ripple.frost direction lists."
        ));
        assert!(source.contains("[render.structural]"));
        assert!(source.contains("# Generated-footer structural link render policy."));
        assert!(source.contains("# Each key names a configured structural relation."));
        assert!(source.contains("# Values are direction lists: to, from, and clique."));
        assert_before(&source, "[upstreams.core]", "[repo]");
        assert_before(&source, "[tutorial]", "[structural]");
        assert_before(&source, "[structural.parent]", "[render.structural]");
        assert_eq!(source.matches("# Structural link relations.").count(), 1);
        assert_before(&source, "# Structural link relations", "[structural.kind]");
        assert!(source.contains("[structural.kind]\nentry = \"kind-entry\"\n"));
        assert!(source.contains("[structural.area]\nentry = \"area\"\n"));
        assert!(source.contains("[structural.parent]\nentry = \"parent\"\n"));
        assert!(source.contains("[render.structural]\n"));
        assert!(source.contains("kind = [\"to\", \"from\"]"));
        assert!(source.contains("area = [\"to\", \"from\", \"clique\"]"));
        assert!(!source.contains("kind = {"));
        assert!(!source.contains("area = {"));
        assert!(!source.contains("parent = {"));
        assert_before(&source, "[structural.kind]", "[structural.area]");
        assert_before(&source, "[structural.area]", "[structural.parent]");
    }

    #[test]
    fn detects_missing_generated_comments() {
        let config = SirnoConfig::default_project();
        let source = config
            .to_commented_toml()
            .unwrap()
            .replace("# Sirno Lake path, resolved relative to this config file.\n", "");

        let missing = config.missing_comments_in(&source).unwrap();

        assert_eq!(
            missing,
            vec!["Sirno Lake path, resolved relative to this config file.".to_owned()]
        );
    }

    #[test]
    fn rejects_frost_table() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(
            &path,
            config_source(
                r#"
[lake]
path = "docs"

[frost]
path = "docs/frost"
"#,
            ),
        )
        .unwrap();

        let error = SirnoConfig::from_file(&path).unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }
}
