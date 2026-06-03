//! Project-local lock state for upstream lakes.
//!
//! `Sirno.toml` configures paths and policy.
//! `Sirno.lock.toml` records generated upstream dependency state represented by the lake.

use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::config::UpstreamSettings;
use crate::identifier::EntryAtom;
/// Canonical Sirno project lock filename.
pub const LOCK_FILE_NAME: &str = "Sirno.lock.toml";

const LOCK_FILE_HEADER: &str = "\
# This file is generated and managed by Sirno.
# Do not edit it by hand.

";

/// Project-local generated state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:sirno-lock:begin
pub struct SirnoLock {
    /// Resolved upstream lake commits.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub upstreams: UpstreamLockMap,
}
// sirno:witness:sirno-lock:end

/// Ordered upstream lock records keyed by glacier domain.
pub type UpstreamLockMap = IndexMap<EntryAtom, UpstreamLock>;

impl SirnoLock {
    /// Resolve the lock path next to the config file.
    pub fn path_for_config(config_path: impl AsRef<Path>) -> PathBuf {
        config_path.as_ref().parent().unwrap_or_else(|| Path::new(".")).join(LOCK_FILE_NAME)
    }

    /// Load a lock from a specific file path.
    // sirno:witness:sirno-lock:begin
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, LockError> {
        let path = path.as_ref();
        trace!("sirno lock load begin: path={}", path.display());
        let source = fs::read_to_string(path)
            .map_err(|source| LockError::Read { path: path.to_path_buf(), source })?;
        let lock: Self = toml::from_str(&source)
            .map_err(|source| LockError::Parse { path: path.to_path_buf(), source })?;
        lock.validate()?;
        trace!("sirno lock load end");
        Ok(lock)
    }
    // sirno:witness:sirno-lock:end

    /// Load a lock from a file path when it exists.
    pub fn from_file_if_exists(path: impl AsRef<Path>) -> Result<Option<Self>, LockError> {
        match Self::from_file(path) {
            | Ok(lock) => Ok(Some(lock)),
            | Err(LockError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
                Ok(None)
            }
            | Err(source) => Err(source),
        }
    }

    /// Write this lock to an existing or new file.
    ///
    /// The lock is first written to a sibling temporary file.
    /// A rename then publishes the complete TOML file as one filesystem replacement.
    // sirno:witness:sirno-lock:begin
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), LockError> {
        let path = path.as_ref();
        trace!("sirno lock write begin: path={}", path.display());
        let source = self.to_toml()?;
        let temporary_path = Self::temporary_path(path);
        let mut file =
            OpenOptions::new().write(true).create_new(true).open(&temporary_path).map_err(
                |source| LockError::CreateTemporary { path: temporary_path.clone(), source },
            )?;
        if let Err(source) = file.write_all(source.as_bytes()) {
            drop(file);
            let _ = fs::remove_file(&temporary_path);
            return Err(LockError::WriteTemporary { path: temporary_path, source });
        }
        if let Err(source) = file.sync_all() {
            drop(file);
            let _ = fs::remove_file(&temporary_path);
            return Err(LockError::WriteTemporary { path: temporary_path, source });
        }
        drop(file);
        if let Err(source) = fs::rename(&temporary_path, path) {
            let _ = fs::remove_file(&temporary_path);
            return Err(LockError::Replace { path: path.to_path_buf(), temporary_path, source });
        }
        trace!("sirno lock write end");
        Ok(())
    }
    // sirno:witness:sirno-lock:end

    // sirno:witness:sirno-lock:begin
    fn validate(&self) -> Result<(), LockError> {
        for (domain, upstream) in &self.upstreams {
            upstream.validate(domain)?;
        }
        Ok(())
    }

    fn to_toml(&self) -> Result<String, LockError> {
        self.validate()?;
        let mut source = String::from(LOCK_FILE_HEADER);
        source.push_str(&toml::to_string_pretty(self).map_err(LockError::Render)?);
        Ok(source)
    }

    fn temporary_path(path: &Path) -> PathBuf {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = path.file_name().unwrap_or_else(|| OsStr::new(LOCK_FILE_NAME));
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let mut temporary_name = OsString::from(".");
        temporary_name.push(file_name);
        temporary_name.push(format!(".{}.{}.tmp", std::process::id(), nonce));
        parent.join(temporary_name)
    }
    // sirno:witness:sirno-lock:end
}

impl Default for SirnoLock {
    fn default() -> Self {
        Self { upstreams: UpstreamLockMap::new() }
    }
}

/// Resolved upstream state recorded in `Sirno.lock.toml`.
///
/// Invariant: `commit` is a non-empty Git commit id.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpstreamLock {
    /// Git source copied from `Sirno.toml`.
    pub git: String,
    /// Branch copied from `Sirno.toml`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Tag copied from `Sirno.toml`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Commit-ish copied from `Sirno.toml`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// Directory inside the Git tree containing `Sirno.toml`.
    pub project: PathBuf,
    /// Lake path read from the upstream project's `Sirno.toml`.
    pub lake: PathBuf,
    /// Exact Git commit crystallized into the glacier.
    pub commit: String,
}

impl UpstreamLock {
    /// Construct a lock record from config and a resolved commit.
    pub fn new(settings: &UpstreamSettings, lake: PathBuf, commit: impl Into<String>) -> Self {
        Self {
            git: settings.git.clone(),
            branch: settings.branch.clone(),
            tag: settings.tag.clone(),
            rev: settings.rev.clone(),
            project: settings.project.clone(),
            lake,
            commit: commit.into(),
        }
    }

    /// Return whether this lock still corresponds to a config declaration.
    pub fn matches_settings(&self, settings: &UpstreamSettings) -> bool {
        self.git == settings.git
            && self.branch == settings.branch
            && self.tag == settings.tag
            && self.rev == settings.rev
            && self.project == settings.project
    }

    fn validate(&self, domain: &EntryAtom) -> Result<(), LockError> {
        if self.git.trim().is_empty() {
            return Err(LockError::UpstreamGitSource(domain.clone()));
        }
        if self.commit.trim().is_empty() {
            return Err(LockError::UpstreamCommit(domain.clone()));
        }
        let ref_count = [self.branch.as_ref(), self.tag.as_ref(), self.rev.as_ref()]
            .into_iter()
            .flatten()
            .count();
        if ref_count != 1 {
            return Err(LockError::UpstreamRefSelector(domain.clone()));
        }
        Ok(())
    }
}

/// Error raised by Sirno lock operations.
#[derive(Debug, Error)]
pub enum LockError {
    /// The lock file could not be read.
    #[error("failed to read lock file {path}")]
    Read {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The lock file could not be parsed as TOML.
    #[error("failed to parse lock file {path}")]
    Parse {
        /// Path that could not be parsed.
        path: PathBuf,
        /// Underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },
    /// The lock file could not be rendered.
    #[error("failed to render lock file")]
    Render(#[source] toml::ser::Error),
    /// An upstream Git source is empty.
    #[error("locked upstream `{0}` git source must not be empty")]
    UpstreamGitSource(EntryAtom),
    /// An upstream must have exactly one ref selector.
    #[error("locked upstream `{0}` must configure exactly one of branch, tag, or rev")]
    UpstreamRefSelector(EntryAtom),
    /// An upstream commit is empty.
    #[error("locked upstream `{0}` commit must not be empty")]
    UpstreamCommit(EntryAtom),
    /// The temporary lock file could not be created.
    #[error("failed to create temporary lock file {path}")]
    CreateTemporary {
        /// Temporary path that could not be created.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The temporary lock file could not be written.
    #[error("failed to write temporary lock file {path}")]
    WriteTemporary {
        /// Temporary path that could not be written.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The temporary lock file could not replace the public lock file.
    #[error("failed to replace lock file {path} with temporary lock file {temporary_path}")]
    Replace {
        /// Lock path that could not be replaced.
        path: PathBuf,
        /// Complete temporary lock path.
        temporary_path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_empty_lock() {
        let lock = SirnoLock::default();
        let rendered = lock.to_toml().unwrap();

        assert_eq!(
            rendered,
            "\
# This file is generated and managed by Sirno.
# Do not edit it by hand.

"
        );
    }

    #[test]
    fn lock_path_uses_toml_suffix() {
        let path = SirnoLock::path_for_config("/project/Sirno.toml");

        assert_eq!(path, PathBuf::from("/project/Sirno.lock.toml"));
    }

    #[test]
    fn renders_upstream_lock() {
        let settings = UpstreamSettings::branch("../core.git", "main");
        let lock = SirnoLock {
            upstreams: UpstreamLockMap::from([(
                EntryAtom::new("core").unwrap(),
                UpstreamLock::new(&settings, PathBuf::from("docs"), "0123456789abcdef"),
            )]),
        };
        let rendered = lock.to_toml().unwrap();
        let read: SirnoLock = toml::from_str(&rendered).unwrap();

        assert_eq!(read, lock);
        assert!(rendered.contains("[upstreams.core]"));
        assert!(rendered.contains("git = \"../core.git\""));
        assert!(rendered.contains("branch = \"main\""));
        assert!(rendered.contains("lake = \"docs\""));
        assert!(rendered.contains("commit = \"0123456789abcdef\""));
    }

    #[test]
    fn rejects_anchor_lock_state() {
        let error = toml::from_str::<SirnoLock>(
            r#"
[anchor]
path = ".sirno/anchor.toml"
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn rejects_tide_lock_state() {
        let error = toml::from_str::<SirnoLock>(
            r#"
[[tide.resolved]]
ripple = "ripple"
field = "belongs"
direction = "to"
neighbor = "neighbor"
fingerprint = "sha256:abc"
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn lock_write_replaces_existing_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(LOCK_FILE_NAME);
        let settings = UpstreamSettings::branch("../core.git", "main");
        let first = SirnoLock {
            upstreams: UpstreamLockMap::from([(
                EntryAtom::new("core").unwrap(),
                UpstreamLock::new(&settings, PathBuf::from("docs"), "1"),
            )]),
        };
        first.write(&path).unwrap();

        let second = SirnoLock {
            upstreams: UpstreamLockMap::from([(
                EntryAtom::new("core").unwrap(),
                UpstreamLock::new(&settings, PathBuf::from("docs"), "2"),
            )]),
        };
        second.write(&path).unwrap();

        let rendered = fs::read_to_string(&path).unwrap();
        assert!(rendered.contains("commit = \"2\""));
        assert!(!rendered.contains("commit = \"1\""));
        let paths = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(paths, 1);
    }
}
