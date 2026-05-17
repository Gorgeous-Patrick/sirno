//! Project-local lock state for Sirno Frost.
//!
//! `Sirno.toml` configures paths and policy.
//! `Sirno.lock.toml` records the Frost snapshot reference represented by the public lake.

use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use eter::{Eterator, GcGeneration, SnapshotRef};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::tide::TideResolution;

/// Canonical Sirno project lock filename.
pub const LOCK_FILE_NAME: &str = "Sirno.lock.toml";

const LOCK_FILE_HEADER: &str = "\
# This file is generated and managed by Sirno.
# Do not edit it by hand.

";

/// Project-local Frost state.
///
/// Invariant: `frost.generation` and `frost.version` name the `eter` snapshot represented
/// by the public lake.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:sirno-lock:begin
pub struct SirnoLock {
    /// Current public-lake Frost state.
    pub frost: FrostLock,
    /// Explicit dependency review resolutions for the current lake edit session.
    #[serde(default, skip_serializing_if = "TideLock::is_empty")]
    pub tide: TideLock,
}
// sirno:witness:sirno-lock:end

impl SirnoLock {
    /// Construct a lock for the current editable public lake.
    // sirno:witness:sirno-lock:begin
    pub fn current(snapshot: SnapshotRef) -> Self {
        Self { frost: FrostLock::current(snapshot), tide: TideLock::default() }
    }
    // sirno:witness:sirno-lock:end

    /// Construct a lock for a checked-out Frost snapshot.
    // sirno:witness:sirno-lock:begin
    pub fn checked_out(snapshot: SnapshotRef, mutable: bool) -> Self {
        Self { frost: FrostLock::checked_out(snapshot, mutable), tide: TideLock::default() }
    }
    // sirno:witness:sirno-lock:end

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
        self.frost.validate()
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

/// Tide state recorded in `Sirno.lock.toml`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TideLock {
    /// Explicitly resolved tide workitems.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolved: Vec<TideResolution>,
}

impl TideLock {
    /// Returns true when no tide state is stored.
    pub fn is_empty(&self) -> bool {
        self.resolved.is_empty()
    }

    /// Replace stored resolutions with a deterministic list.
    pub fn set_resolved(&mut self, mut resolved: Vec<TideResolution>) {
        resolved.sort();
        resolved.dedup();
        self.resolved = resolved;
    }

    /// Clear all tide state.
    pub fn clear(&mut self) {
        self.resolved.clear();
    }
}

/// Frost state recorded in `Sirno.lock.toml`.
///
/// Invariant: `mutable` is true only for checked-out snapshots created with `--unsafe-mutable`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:versioning:begin
pub struct FrostLock {
    /// Public lake status relative to the configured Frost path.
    pub status: FrostLockStatus,
    /// GC generation for the represented snapshot.
    pub generation: u64,
    /// Raw `Eterator` coordinate represented by the public lake.
    pub version: u64,
    /// Whether a checked-out frozen snapshot was intentionally left writable.
    #[serde(default, skip_serializing_if = "is_false")]
    pub mutable: bool,
}
// sirno:witness:versioning:end

impl FrostLock {
    /// Construct state for the current editable public lake.
    // sirno:witness:versioning:begin
    pub fn current(snapshot: SnapshotRef) -> Self {
        Self {
            status: FrostLockStatus::Current,
            generation: snapshot.generation.number(),
            version: snapshot.version(),
            mutable: false,
        }
    }
    // sirno:witness:versioning:end

    /// Construct state for a checked-out Frost snapshot.
    // sirno:witness:versioning:begin
    pub fn checked_out(snapshot: SnapshotRef, mutable: bool) -> Self {
        Self {
            status: FrostLockStatus::CheckedOut,
            generation: snapshot.generation.number(),
            version: snapshot.version(),
            mutable,
        }
    }
    // sirno:witness:versioning:end

    /// Return the stored snapshot reference.
    // sirno:witness:versioning:begin
    pub fn snapshot_ref(&self) -> SnapshotRef {
        SnapshotRef::new(GcGeneration(self.generation), Eterator(self.version))
    }
    // sirno:witness:versioning:end

    /// Returns true when the public lake is a Frost checkout.
    // sirno:witness:versioning:begin
    pub fn is_checked_out(&self) -> bool {
        self.status == FrostLockStatus::CheckedOut
    }

    /// Returns true when the public lake is a writable historical checkout.
    pub fn is_unsafe_mutable_checkout(&self) -> bool {
        self.is_checked_out() && self.mutable
    }
    // sirno:witness:versioning:end

    // sirno:witness:versioning:begin
    fn validate(&self) -> Result<(), LockError> {
        if self.status == FrostLockStatus::Current && self.mutable {
            return Err(LockError::CurrentMutable);
        }
        Ok(())
    }
    // sirno:witness:versioning:end
}

/// Public-lake status relative to Sirno Frost.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
// sirno:witness:versioning:begin
pub enum FrostLockStatus {
    /// The public lake is the current editable version.
    Current,
    /// The public lake is a materialized frozen snapshot.
    CheckedOut,
}
// sirno:witness:versioning:end

fn is_false(value: &bool) -> bool {
    !*value
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
    /// Current public-lake state must be editable.
    #[error("current frost state cannot be marked mutable")]
    CurrentMutable,
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
    fn renders_current_frost_lock() {
        let lock = SirnoLock::current(SnapshotRef::new(GcGeneration::INITIAL, Eterator(7)));
        let rendered = lock.to_toml().unwrap();

        assert_eq!(
            rendered,
            "\
# This file is generated and managed by Sirno.
# Do not edit it by hand.

[frost]
status = \"current\"
generation = 0
version = 7
"
        );
    }

    #[test]
    fn lock_path_uses_toml_suffix() {
        let path = SirnoLock::path_for_config("/project/Sirno.toml");

        assert_eq!(path, PathBuf::from("/project/Sirno.lock.toml"));
    }

    #[test]
    fn renders_mutable_checkout_lock() {
        let lock = SirnoLock::checked_out(SnapshotRef::new(GcGeneration(2), Eterator(3)), true);
        let rendered = lock.to_toml().unwrap();

        assert_eq!(
            rendered,
            "\
# This file is generated and managed by Sirno.
# Do not edit it by hand.

[frost]
status = \"checked-out\"
generation = 2
version = 3
mutable = true
"
        );
    }

    #[test]
    fn rejects_mutable_current_lock() {
        let error = toml::from_str::<SirnoLock>(
            r#"
[frost]
status = "current"
generation = 0
version = 3
mutable = true
"#,
        )
        .unwrap()
        .validate()
        .unwrap_err();

        assert!(matches!(error, LockError::CurrentMutable));
    }

    #[test]
    fn lock_write_replaces_existing_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(LOCK_FILE_NAME);
        SirnoLock::current(SnapshotRef::new(GcGeneration::INITIAL, Eterator(1)))
            .write(&path)
            .unwrap();

        SirnoLock::current(SnapshotRef::new(GcGeneration::INITIAL, Eterator(2)))
            .write(&path)
            .unwrap();

        let rendered = fs::read_to_string(&path).unwrap();
        assert!(rendered.contains("version = 2"));
        assert!(!rendered.contains("version = 1"));
        let paths = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(paths, 1);
    }
}
