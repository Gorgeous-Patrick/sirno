// sirno:witness:sirno-upstream:begin
//! Upstream lake resolution and crystallization.
//!
//! Upstream lakes are Git-backed Sirno projects declared by the current project.
//! Crystallization materializes each upstream lake into a managed glacier.
// sirno:witness:sirno-upstream:end

use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::artifact::{ARTIFACT_DIRECTORY_NAME, EntryArtifact, EntryArtifactPath};
use crate::check::CheckMode;
use crate::config::{CONFIG_FILE_NAME, SirnoConfig, UpstreamRef, UpstreamSettings};
use crate::entry::{Entry, FrozenMarker};
use crate::identifier::{EntryAddress, EntryAtom};
use crate::lake::{EntryDirectory, EntryDirectoryCheckSettings, GlacierReport};
use crate::lock::{SirnoLock, UpstreamLock};
use crate::render::GeneratedLinkBody;

/// Global Git cache for upstream sources.
#[derive(Clone, Debug)]
pub struct UpstreamGitCache {
    root: PathBuf,
}

/// Inputs for crystallizing selected upstreams.
pub(crate) struct CrystallizeUpstreams<'a> {
    /// Current project config path.
    pub(crate) config_path: &'a Path,
    /// Current project config.
    pub(crate) config: &'a SirnoConfig,
    /// Mutable project lock.
    pub(crate) lock: &'a mut SirnoLock,
    /// Current lake directory.
    pub(crate) lake: &'a EntryDirectory,
    /// Lake check settings used while replacing glaciers.
    pub(crate) settings: &'a EntryDirectoryCheckSettings,
    /// Global upstream Git cache.
    pub(crate) cache: &'a UpstreamGitCache,
    /// Selected glacier domains. Empty means every upstream.
    pub(crate) domains: &'a [EntryAtom],
    /// Use only existing lock records and cache mirrors.
    pub(crate) locked: bool,
}

/// Report from crystallizing upstream lakes into glaciers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamCrystallizeReport {
    /// Whether crystallization completed.
    pub ok: bool,
    /// Glacier domains that were crystallized.
    pub domains: Vec<String>,
    /// Paths changed in the current lake.
    pub changed_paths: Vec<String>,
    /// Human-readable summary.
    pub message: String,
}

/// Status report for configured upstream lakes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamStatusReport {
    /// Whether every upstream is locked, cached, and crystallized into its glacier.
    pub ok: bool,
    /// Per-upstream status rows.
    pub upstreams: Vec<UpstreamStatus>,
    /// Human-readable summary.
    pub message: String,
}

/// Status for one configured upstream lake.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamStatus {
    /// Glacier domain.
    pub domain: String,
    /// Configured Git source.
    pub git: String,
    /// Current status.
    pub state: UpstreamStatusState,
    /// Locked commit when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

/// Machine-readable upstream status state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpstreamStatusState {
    /// The upstream has a matching lock, cache mirror, and glacier.
    Ok,
    /// The upstream has no lock record.
    MissingLock,
    /// The lock record no longer matches `Sirno.toml`.
    StaleLock,
    /// The global cache mirror is absent.
    MissingCache,
    /// The upstream lock exists but the glacier has not been materialized.
    MissingGlacier,
    /// The glacier differs from the locked upstream commit.
    GlacierDrift,
}

impl UpstreamGitCache {
    /// Create a cache rooted at a specific path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Return the default global Sirno store under the user's home directory.
    pub fn default_global() -> Result<Self, UpstreamError> {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .ok_or(UpstreamError::HomeDirectory)?;
        Ok(Self::new(PathBuf::from(home).join(".sirno")))
    }

    /// Root path for the cache.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Return the mirror path for one normalized Git source.
    pub fn mirror_path_for_source(&self, source: &str) -> PathBuf {
        self.root.join("git").join(format!("{:016x}.git", stable_hash(source.as_bytes())))
    }

    fn ensure_mirror(&self, source: &str) -> Result<PathBuf, UpstreamError> {
        let mirror = self.mirror_path_for_source(source);
        if mirror.exists() {
            run_git(["-C", path_arg(&mirror), "fetch", "--prune", "origin"])?;
            return Ok(mirror);
        }

        let parent = mirror.parent().expect("mirror path has parent");
        fs::create_dir_all(parent)
            .map_err(|source| UpstreamError::CreateCache { path: parent.to_path_buf(), source })?;
        run_git(["clone", "--mirror", source, path_arg(&mirror)])?;
        Ok(mirror)
    }

    fn require_mirror(&self, source: &str) -> Result<PathBuf, UpstreamError> {
        let mirror = self.mirror_path_for_source(source);
        if mirror.is_dir() {
            Ok(mirror)
        } else {
            Err(UpstreamError::MissingCache(source.to_owned()))
        }
    }
}

// sirno:witness:crystallization:begin
/// Crystallize selected upstream lake domains into glaciers.
pub(crate) fn crystallize_upstreams(
    input: CrystallizeUpstreams<'_>,
) -> Result<(UpstreamCrystallizeReport, Vec<GlacierReport>), UpstreamError> {
    let CrystallizeUpstreams { config_path, config, lock, lake, settings, cache, domains, locked } =
        input;
    let selected = select_upstreams(config, domains)?;
    let mut changed_paths = Vec::new();
    let mut reports = Vec::new();
    let mut names = Vec::new();

    for (domain, upstream) in selected {
        trace!("crystallize upstream begin: domain={domain}");
        let loaded = load_upstream(config_path, domain, upstream, lock, cache, locked)?;
        let report = lake.replace_glacier(domain, &loaded.entries, &loaded.artifacts, settings)?;
        changed_paths.extend(report.changed_paths().iter().map(|path| display_path(path)));
        reports.push(report);
        lock.upstreams.insert(domain.clone(), loaded.lock);
        names.push(domain.to_string());
        trace!("crystallize upstream end: domain={domain}");
    }

    Ok((
        UpstreamCrystallizeReport {
            ok: true,
            domains: names.clone(),
            changed_paths,
            message: format!(
                "crystallized {} {}",
                names.len(),
                plural_named(names.len(), "glacier", "glaciers")
            ),
        },
        reports,
    ))
}
// sirno:witness:crystallization:end

// sirno:witness:lake-system:begin
/// Return status for configured upstream lakes.
pub fn upstream_status(
    config_path: &Path, config: &SirnoConfig, lock: Option<&SirnoLock>, cache: &UpstreamGitCache,
    lake: Option<(&EntryDirectory, &EntryDirectoryCheckSettings)>,
) -> Result<UpstreamStatusReport, UpstreamError> {
    let mut upstreams = Vec::new();
    for (domain, upstream) in &config.upstreams {
        let source = normalize_git_source(config_path, &upstream.git)?;
        let locked = lock.and_then(|lock| lock.upstreams.get(domain));
        let (mut state, commit) = match locked {
            | None => (UpstreamStatusState::MissingLock, None),
            | Some(locked) if !locked.matches_settings(upstream) => {
                (UpstreamStatusState::StaleLock, Some(locked.commit.clone()))
            }
            | Some(locked) if !cache.mirror_path_for_source(&source).is_dir() => {
                (UpstreamStatusState::MissingCache, Some(locked.commit.clone()))
            }
            | Some(locked) => (UpstreamStatusState::Ok, Some(locked.commit.clone())),
        };
        if state == UpstreamStatusState::Ok
            && let (Some(lock), Some((lake, settings))) = (lock, lake)
        {
            state = glacier_status(config_path, domain, upstream, lock, cache, lake, settings)?;
        }
        upstreams.push(UpstreamStatus {
            domain: domain.to_string(),
            git: upstream.git.clone(),
            state,
            commit,
        });
    }
    let ok = upstreams.iter().all(|status| status.state == UpstreamStatusState::Ok);
    Ok(UpstreamStatusReport {
        ok,
        message: format!("{} upstream {}", upstreams.len(), plural(upstreams.len())),
        upstreams,
    })
}
// sirno:witness:lake-system:end

fn glacier_status(
    config_path: &Path, domain: &EntryAtom, upstream: &UpstreamSettings, lock: &SirnoLock,
    cache: &UpstreamGitCache, lake: &EntryDirectory, settings: &EntryDirectoryCheckSettings,
) -> Result<UpstreamStatusState, UpstreamError> {
    let loaded = load_upstream(config_path, domain, upstream, lock, cache, true)?;
    let Some(actual) = read_glacier(lake, domain, settings)? else {
        return Ok(UpstreamStatusState::MissingGlacier);
    };
    if actual.entries == loaded.entries && actual.artifacts == loaded.artifacts {
        Ok(UpstreamStatusState::Ok)
    } else {
        Ok(UpstreamStatusState::GlacierDrift)
    }
}

fn read_glacier(
    lake: &EntryDirectory, domain: &EntryAtom, settings: &EntryDirectoryCheckSettings,
) -> Result<Option<LoadedUpstreamFiles>, UpstreamError> {
    let mut check_settings = settings.clone();
    check_settings.render = false;
    check_settings.witness = None;
    let checked = lake.check_with_settings(CheckMode::Edit, &check_settings)?;
    if checked.has_errors() {
        return Ok(Some(LoadedUpstreamFiles { entries: Vec::new(), artifacts: Vec::new() }));
    }

    let mut entries = Vec::new();
    for entry in checked.entries().iter().filter(|entry| entry.id.starts_with_domain(domain)) {
        if !entry.metadata.meta.frozen.as_ref().is_some_and(|marker| marker.is_managed()) {
            return Ok(Some(LoadedUpstreamFiles { entries: Vec::new(), artifacts: Vec::new() }));
        }
        let mut entry = entry.clone();
        entry.body = strip_generated_footer_for_import(&entry.body)?;
        entries.push(entry);
    }
    let mut artifacts = checked
        .artifacts()
        .iter()
        .filter(|artifact| artifact.owner.starts_with_domain(domain))
        .cloned()
        .collect::<Vec<_>>();
    if entries.is_empty() && artifacts.is_empty() {
        return Ok(None);
    }
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    artifacts.sort_by(|left, right| {
        left.owner.cmp(&right.owner).then_with(|| left.path.cmp(&right.path))
    });
    Ok(Some(LoadedUpstreamFiles { entries, artifacts }))
}

// sirno:witness:upstream-lake:begin
fn select_upstreams<'a>(
    config: &'a SirnoConfig, domains: &[EntryAtom],
) -> Result<Vec<(&'a EntryAtom, &'a UpstreamSettings)>, UpstreamError> {
    if domains.is_empty() {
        return Ok(config.upstreams.iter().collect());
    }
    domains
        .iter()
        .map(|domain| {
            config
                .upstreams
                .get_key_value(domain)
                .ok_or_else(|| UpstreamError::UnknownDomain(domain.clone()))
        })
        .collect()
}

fn load_upstream(
    config_path: &Path, domain: &EntryAtom, settings: &UpstreamSettings, lock: &SirnoLock,
    cache: &UpstreamGitCache, locked: bool,
) -> Result<LoadedUpstream, UpstreamError> {
    let source = normalize_git_source(config_path, &settings.git)?;
    let mirror =
        if locked { cache.require_mirror(&source)? } else { cache.ensure_mirror(&source)? };
    let commit = if locked {
        let locked =
            lock.upstreams.get(domain).ok_or_else(|| UpstreamError::MissingLock(domain.clone()))?;
        if !locked.matches_settings(settings) {
            return Err(UpstreamError::StaleLock(domain.clone()));
        }
        verify_commit(&mirror, &locked.commit)?;
        locked.commit.clone()
    } else {
        resolve_commit(&mirror, settings.selector())?
    };

    let project = git_tree_path(&settings.project)?;
    let config_tree_path = join_tree_path(&project, CONFIG_FILE_NAME);
    let config_source = git_show_text(&mirror, &commit, &config_tree_path)?;
    let upstream_config = SirnoConfig::from_source(Path::new(&config_tree_path), &config_source)?;
    let lake = git_tree_path(&upstream_config.lake.path)?;
    let lake_tree_path = join_tree_path(&project, &lake);
    let files = git_list_files(&mirror, &commit, &lake_tree_path)?;
    let loaded =
        load_upstream_files(&mirror, &commit, &lake_tree_path, &files, domain, &upstream_config)?;
    let lock = UpstreamLock::new(settings, upstream_config.lake.path.clone(), commit);
    Ok(LoadedUpstream { entries: loaded.entries, artifacts: loaded.artifacts, lock })
}
// sirno:witness:upstream-lake:end

struct LoadedUpstream {
    entries: Vec<Entry>,
    artifacts: Vec<EntryArtifact>,
    lock: UpstreamLock,
}

struct LoadedUpstreamFiles {
    entries: Vec<Entry>,
    artifacts: Vec<EntryArtifact>,
}

// sirno:witness:lake-sheaf:begin
fn load_upstream_files(
    mirror: &Path, commit: &str, lake_tree_path: &str, files: &[String], domain: &EntryAtom,
    config: &SirnoConfig,
) -> Result<LoadedUpstreamFiles, UpstreamError> {
    let mut entries = Vec::new();
    let mut artifacts = Vec::new();
    for file in files {
        let Some(relative) = strip_tree_prefix(file, lake_tree_path) else {
            continue;
        };
        let relative_path = PathBuf::from(relative);
        if config.lake.ignore.iter().any(|ignored| relative_path.starts_with(ignored)) {
            continue;
        }
        if first_component_starts_with_dot(&relative_path)
            && !relative.starts_with(ARTIFACT_DIRECTORY_NAME)
        {
            continue;
        }
        if relative.starts_with(&format!("{ARTIFACT_DIRECTORY_NAME}/")) {
            let artifact = load_artifact(mirror, commit, file, relative, domain)?;
            artifacts.push(artifact);
            continue;
        }
        if relative_path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            return Err(UpstreamError::UnsupportedLakePath(file.clone()));
        }
        let source = git_show_text(mirror, commit, file)?;
        let source_address = EntryAddress::from_lake_relative_path(&relative_path)?;
        let mut entry = Entry::from_markdown(source_address.under_domain(domain), &source)?;
        entry.body = strip_generated_footer_for_import(&entry.body)?;
        rebase_structural_targets(&mut entry, domain);
        match &mut entry.metadata.meta.frozen {
            | Some(marker) => marker.insert_managed(),
            | None => entry.metadata.meta.frozen = Some(FrozenMarker::managed()),
        }
        entries.push(entry);
    }
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    artifacts.sort_by(|left, right| {
        left.owner.cmp(&right.owner).then_with(|| left.path.cmp(&right.path))
    });
    Ok(LoadedUpstreamFiles { entries, artifacts })
}
// sirno:witness:lake-sheaf:end

fn load_artifact(
    mirror: &Path, commit: &str, file: &str, relative: &str, domain: &EntryAtom,
) -> Result<EntryArtifact, UpstreamError> {
    let rest = relative.strip_prefix(ARTIFACT_DIRECTORY_NAME).unwrap().trim_start_matches('/');
    let Some((owner, path)) = rest.split_once('/') else {
        return Err(UpstreamError::UnsupportedLakePath(file.to_owned()));
    };
    let owner = EntryAddress::new(owner)?.under_domain(domain);
    let artifact_path = EntryArtifactPath::new(Path::new(path))?;
    let content = git_show_bytes(mirror, commit, file)?;
    Ok(EntryArtifact::new(owner, artifact_path, content))
}

fn rebase_structural_targets(entry: &mut Entry, domain: &EntryAtom) {
    for targets in entry.metadata.structural.values_mut() {
        for target in targets {
            *target = target.under_domain(domain);
        }
    }
}

fn strip_generated_footer_for_import(body: &str) -> Result<String, UpstreamError> {
    let stripped = GeneratedLinkBody::new(body).delete()?;
    let trimmed = stripped.trim_end_matches('\n');
    if let Some(before) = trimmed.strip_suffix("\n\n---") {
        let mut out = before.to_owned();
        if !out.is_empty() {
            out.push('\n');
        }
        return Ok(out);
    }
    Ok(stripped)
}

fn resolve_commit(mirror: &Path, selector: UpstreamRef<'_>) -> Result<String, UpstreamError> {
    let rev = match selector {
        | UpstreamRef::Branch(branch) => format!("refs/heads/{branch}^{{commit}}"),
        | UpstreamRef::Tag(tag) => format!("refs/tags/{tag}^{{commit}}"),
        | UpstreamRef::Rev(rev) => format!("{rev}^{{commit}}"),
    };
    let output = run_git_output(["-C", path_arg(mirror), "rev-parse", "--verify", &rev])?;
    Ok(String::from_utf8_lossy(&output).trim().to_owned())
}

fn verify_commit(mirror: &Path, commit: &str) -> Result<(), UpstreamError> {
    run_git_output([
        "-C",
        path_arg(mirror),
        "rev-parse",
        "--verify",
        &format!("{commit}^{{commit}}"),
    ])?;
    Ok(())
}

fn git_show_text(mirror: &Path, commit: &str, path: &str) -> Result<String, UpstreamError> {
    let bytes = git_show_bytes(mirror, commit, path)?;
    String::from_utf8(bytes)
        .map_err(|source| UpstreamError::NonUtf8GitObject { path: path.to_owned(), source })
}

fn git_show_bytes(mirror: &Path, commit: &str, path: &str) -> Result<Vec<u8>, UpstreamError> {
    run_git_output(["-C", path_arg(mirror), "show", &format!("{commit}:{path}")])
}

fn git_list_files(mirror: &Path, commit: &str, path: &str) -> Result<Vec<String>, UpstreamError> {
    let bytes = run_git_output([
        "-C",
        path_arg(mirror),
        "ls-tree",
        "-r",
        "-z",
        "--name-only",
        commit,
        "--",
        path,
    ])?;
    bytes
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| {
            String::from_utf8(part.to_vec())
                .map_err(|source| UpstreamError::NonUtf8GitObject { path: path.to_owned(), source })
        })
        .collect()
}

fn normalize_git_source(config_path: &Path, source: &str) -> Result<String, UpstreamError> {
    if source.contains("://") || source.contains('@') && source.contains(':') {
        return Ok(source.to_owned());
    }
    let path = Path::new(source);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        config_path.parent().unwrap_or_else(|| Path::new(".")).join(path)
    };
    let path = fs::canonicalize(&path).unwrap_or(path);
    Ok(path.to_string_lossy().into_owned())
}

fn git_tree_path(path: &Path) -> Result<String, UpstreamError> {
    if path.as_os_str().is_empty() {
        return Ok(String::new());
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            | Component::CurDir => {}
            | Component::Normal(part) => parts.push(part_to_utf8(part)?),
            | _ => return Err(UpstreamError::GitTreePath(path.to_path_buf())),
        }
    }
    Ok(parts.join("/"))
}

fn join_tree_path(left: &str, right: impl AsRef<Path>) -> String {
    let right = git_tree_path(right.as_ref()).expect("validated tree path");
    match (left.is_empty(), right.is_empty()) {
        | (true, _) => right,
        | (_, true) => left.to_owned(),
        | (false, false) => format!("{left}/{right}"),
    }
}

fn strip_tree_prefix<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.is_empty() {
        return Some(path);
    }
    path.strip_prefix(prefix)?.strip_prefix('/')
}

fn first_component_starts_with_dot(path: &Path) -> bool {
    path.components().next().is_some_and(|component| match component {
        | Component::Normal(name) => name.to_str().is_some_and(|name| name.starts_with('.')),
        | _ => false,
    })
}

fn part_to_utf8(part: &OsStr) -> Result<String, UpstreamError> {
    part.to_str().map(str::to_owned).ok_or_else(|| UpstreamError::GitTreePath(PathBuf::from(part)))
}

fn path_arg(path: &Path) -> &str {
    path.to_str().expect("Sirno paths passed to git are UTF-8")
}

fn run_git<const N: usize>(args: [&str; N]) -> Result<(), UpstreamError> {
    run_git_output(args).map(|_| ())
}

fn run_git_output<const N: usize>(args: [&str; N]) -> Result<Vec<u8>, UpstreamError> {
    let output = Command::new("git").args(args).output().map_err(UpstreamError::StartGit)?;
    if output.status.success() {
        return Ok(output.stdout);
    }
    Err(UpstreamError::Git {
        status: output.status.code(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    })
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "lake" } else { "lakes" }
}

fn plural_named<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

/// Error raised while resolving or crystallizing upstream lakes.
#[derive(Debug, Error)]
pub enum UpstreamError {
    /// The user home directory could not be resolved.
    #[error("failed to locate home directory for ~/.sirno")]
    HomeDirectory,
    /// A configured glacier domain does not exist.
    #[error("upstream `{0}` is not configured")]
    UnknownDomain(EntryAtom),
    /// A locked crystallization requested a missing lock.
    #[error("upstream `{0}` is not locked; run `sirno upstream crystallize {0}`")]
    MissingLock(EntryAtom),
    /// A locked crystallization found stale lock state.
    #[error("upstream `{0}` lock is stale; run `sirno upstream update {0}`")]
    StaleLock(EntryAtom),
    /// The upstream cache mirror is missing.
    #[error("upstream cache is missing for {0}")]
    MissingCache(String),
    /// A cache directory could not be created.
    #[error("failed to create upstream cache directory {path}")]
    CreateCache {
        /// Cache directory path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Git could not be started.
    #[error("failed to run git")]
    StartGit(#[source] std::io::Error),
    /// Git returned an error status.
    #[error("git command failed: {stderr}")]
    Git {
        /// Git process status code.
        status: Option<i32>,
        /// Git stderr.
        stderr: String,
    },
    /// A Git object was not UTF-8 where text was required.
    #[error("git object is not valid UTF-8: {path}")]
    NonUtf8GitObject {
        /// Git tree path.
        path: String,
        /// UTF-8 conversion error.
        #[source]
        source: std::string::FromUtf8Error,
    },
    /// A Git-tree-relative path was invalid.
    #[error("invalid Git tree path: {0}")]
    GitTreePath(PathBuf),
    /// An upstream lake contained an unsupported file.
    #[error("upstream lake contains unsupported path: {0}")]
    UnsupportedLakePath(String),
    /// Config parsing failed.
    #[error(transparent)]
    Config(#[from] crate::ConfigError),
    /// Entry parsing failed.
    #[error(transparent)]
    Entry(#[from] crate::EntryParseError),
    /// Entry address parsing failed.
    #[error(transparent)]
    EntryAddress(#[from] crate::EntryAddressError),
    /// Artifact path parsing failed.
    #[error(transparent)]
    ArtifactPath(#[from] crate::EntryArtifactPathError),
    /// Generated-link footer handling failed.
    #[error(transparent)]
    GeneratedLink(#[from] crate::GeneratedLinkError),
    /// Lake writing failed.
    #[error(transparent)]
    Lake(#[from] crate::EntryDirectoryError),
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;
    use crate::surface::{
        CommandError, SurfaceContext, UpstreamAddRequest, UpstreamCrystallizeRequest,
    };
    use crate::{EntryDirectoryError, LOCK_FILE_NAME, StructuralSettings, UpstreamSettingsMap};

    fn run_git(root: &Path, args: &[&str]) {
        let output = Command::new("git").current_dir(root).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn write_upstream_repo(root: &Path) -> String {
        fs::create_dir_all(root.join("docs/.artifacts/design")).unwrap();
        SirnoConfig::new("docs").write_new(root.join(CONFIG_FILE_NAME)).unwrap();
        fs::write(
            root.join("docs/design.md"),
            "\
---
name: Design
desc: Upstream design entry.
meta:
  frozen:
    - reviewed
belongs:
  - alpha
---

Body.
",
        )
        .unwrap();
        fs::write(
            root.join("docs/alpha.md"),
            "\
---
name: Alpha
desc: Upstream alpha entry.
---

Body.
",
        )
        .unwrap();
        fs::write(root.join("docs/.artifacts/design/logo.bin"), b"logo").unwrap();

        run_git(root, &["init"]);
        run_git(root, &["checkout", "-b", "main"]);
        run_git(root, &["config", "user.email", "sirno@example.invalid"]);
        run_git(root, &["config", "user.name", "Sirno Test"]);
        run_git(root, &["add", "."]);
        run_git(root, &["commit", "-m", "seed"]);
        let output =
            Command::new("git").current_dir(root).args(["rev-parse", "HEAD"]).output().unwrap();
        assert!(output.status.success());
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }

    #[test]
    fn crystallizes_local_git_upstream_through_cache_and_lock() {
        let temp = tempfile::tempdir().unwrap();
        let upstream_root = temp.path().join("upstream");
        fs::create_dir(&upstream_root).unwrap();
        let commit = write_upstream_repo(&upstream_root);
        let project_root = temp.path().join("project");
        fs::create_dir(&project_root).unwrap();
        let config_path = project_root.join(CONFIG_FILE_NAME);
        let domain = EntryAtom::new("core").unwrap();
        let config = SirnoConfig {
            upstreams: UpstreamSettingsMap::from([(
                domain.clone(),
                UpstreamSettings::branch(upstream_root.to_string_lossy(), "main"),
            )]),
            structural: StructuralSettings::default(),
            ..SirnoConfig::new("lake")
        };
        config.write_new(&config_path).unwrap();

        let result = SurfaceContext::new(&config_path)
            .with_upstream_store_path(temp.path().join("store"))
            .upstream_crystallize(UpstreamCrystallizeRequest {
                domains: vec![domain.clone()],
                locked: false,
            })
            .unwrap();

        assert_eq!(result.domains, ["core"]);
        let design = fs::read_to_string(project_root.join("lake/core/design.md")).unwrap();
        assert!(design.contains("  - reviewed"));
        assert!(design.contains("  - managed"));
        assert!(design.contains("belongs:\n  - core.alpha"));
        assert_eq!(
            fs::read(project_root.join("lake/.artifacts/core.design/logo.bin")).unwrap(),
            b"logo"
        );
        let lock = SirnoLock::from_file(project_root.join(LOCK_FILE_NAME)).unwrap();
        let upstream = lock.upstreams.get(&domain).unwrap();
        assert_eq!(upstream.commit, commit);

        let status = SurfaceContext::new(&config_path)
            .with_upstream_store_path(temp.path().join("store"))
            .upstream_status()
            .unwrap();
        assert!(status.ok, "{status:?}");
        assert_eq!(status.upstreams[0].state, UpstreamStatusState::Ok);

        let result = SurfaceContext::new(&config_path)
            .with_upstream_store_path(temp.path().join("store"))
            .upstream_crystallize(UpstreamCrystallizeRequest {
                domains: vec![domain],
                locked: true,
            })
            .unwrap();
        assert_eq!(result.domains, ["core"]);
        assert_eq!(fs::read_dir(temp.path().join("store/git")).unwrap().count(), 1);
    }

    #[test]
    fn upstream_add_rejects_implicit_local_lakelet_collision_before_config_write() {
        let temp = tempfile::tempdir().unwrap();
        let upstream_root = temp.path().join("upstream");
        fs::create_dir(&upstream_root).unwrap();
        write_upstream_repo(&upstream_root);
        let project_root = temp.path().join("project");
        fs::create_dir(&project_root).unwrap();
        let config_path = project_root.join(CONFIG_FILE_NAME);
        SirnoConfig::new("lake").write_new(&config_path).unwrap();
        fs::create_dir_all(project_root.join("lake/core")).unwrap();
        fs::write(
            project_root.join("lake/core/local.md"),
            "\
---
name: Local
desc: Local lakelet entry.
---

Body.
",
        )
        .unwrap();
        let domain = EntryAtom::new("core").unwrap();

        let error = SurfaceContext::new(&config_path)
            .with_upstream_store_path(temp.path().join("store"))
            .upstream_add(UpstreamAddRequest {
                domain: domain.clone(),
                settings: UpstreamSettings::branch(upstream_root.to_string_lossy(), "main"),
            })
            .unwrap_err();

        assert!(matches!(
            error,
            CommandError::EntryDirectory(EntryDirectoryError::UnmanagedGlacierPath(path))
                if path.ends_with("lake/core/local.md")
        ));
        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert!(config.upstreams.is_empty());
        assert!(!project_root.join(LOCK_FILE_NAME).exists());
        assert!(project_root.join("lake/core/local.md").exists());
    }
}
