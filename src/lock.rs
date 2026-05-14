//! Project-local lock state for Sirno history.
//!
//! `Sirno.toml` configures paths and policy.
//! `Sirno.lock` records the history snapshot represented by the public store.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use eter::Eterator;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

/// Canonical Sirno project lock filename.
pub const LOCK_FILE_NAME: &str = "Sirno.lock";

const LOCK_FILE_HEADER: &str = "\
# This file is generated and managed by Sirno.
# Do not edit it by hand.

";

/// Project-local history state.
///
/// Invariant: `history.version` names the `eter` snapshot represented by the public store.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:sirno-lock:begin
pub struct SirnoLock {
    /// Current public-store history state.
    pub history: HistoryLock,
}
// sirno:witness:sirno-lock:end

impl SirnoLock {
    /// Construct a lock for the current editable public store.
    // sirno:witness:sirno-lock:begin
    pub fn current(version: Eterator) -> Self {
        Self { history: HistoryLock::current(version) }
    }
    // sirno:witness:sirno-lock:end

    /// Construct a lock for a checked-out history snapshot.
    // sirno:witness:sirno-lock:begin
    pub fn checked_out(version: Eterator, mutable: bool) -> Self {
        Self { history: HistoryLock::checked_out(version, mutable) }
    }
    // sirno:witness:sirno-lock:end

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

    /// Write this lock to an existing or new file.
    // sirno:witness:sirno-lock:begin
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), LockError> {
        let path = path.as_ref();
        trace!("sirno lock write begin: path={}", path.display());
        let source = self.to_toml()?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|source| LockError::Create { path: path.to_path_buf(), source })?;
        file.write_all(source.as_bytes())
            .map_err(|source| LockError::Write { path: path.to_path_buf(), source })?;
        trace!("sirno lock write end");
        Ok(())
    }
    // sirno:witness:sirno-lock:end

    // sirno:witness:sirno-lock:begin
    fn validate(&self) -> Result<(), LockError> {
        self.history.validate()
    }

    fn to_toml(&self) -> Result<String, LockError> {
        self.validate()?;
        let mut source = String::from(LOCK_FILE_HEADER);
        source.push_str(&toml::to_string_pretty(self).map_err(LockError::Render)?);
        Ok(source)
    }
    // sirno:witness:sirno-lock:end
}

/// History state recorded in `Sirno.lock`.
///
/// Invariant: `mutable` is true only for checked-out snapshots created with `--unsafe-mutable`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// sirno:witness:versioning:begin
pub struct HistoryLock {
    /// Public store status relative to the configured history root.
    pub status: HistoryLockStatus,
    /// Raw `Eterator` version represented by the public store.
    pub version: u64,
    /// Whether a checked-out historical snapshot was intentionally left writable.
    #[serde(default, skip_serializing_if = "is_false")]
    pub mutable: bool,
}
// sirno:witness:versioning:end

impl HistoryLock {
    /// Construct state for the current editable public store.
    // sirno:witness:versioning:begin
    pub fn current(version: Eterator) -> Self {
        Self { status: HistoryLockStatus::Current, version: version.version(), mutable: false }
    }
    // sirno:witness:versioning:end

    /// Construct state for a checked-out history snapshot.
    // sirno:witness:versioning:begin
    pub fn checked_out(version: Eterator, mutable: bool) -> Self {
        Self { status: HistoryLockStatus::CheckedOut, version: version.version(), mutable }
    }
    // sirno:witness:versioning:end

    /// Return the stored version as an `Eterator`.
    // sirno:witness:versioning:begin
    pub fn eterator(&self) -> Eterator {
        Eterator(self.version)
    }
    // sirno:witness:versioning:end

    /// Returns true when the public store is a historical checkout.
    // sirno:witness:versioning:begin
    pub fn is_checked_out(&self) -> bool {
        self.status == HistoryLockStatus::CheckedOut
    }

    /// Returns true when the public store is a writable historical checkout.
    pub fn is_unsafe_mutable_checkout(&self) -> bool {
        self.is_checked_out() && self.mutable
    }
    // sirno:witness:versioning:end

    // sirno:witness:versioning:begin
    fn validate(&self) -> Result<(), LockError> {
        if self.status == HistoryLockStatus::Current && self.mutable {
            return Err(LockError::CurrentMutable);
        }
        Ok(())
    }
    // sirno:witness:versioning:end
}

/// Public-store status relative to history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
// sirno:witness:versioning:begin
pub enum HistoryLockStatus {
    /// The public store is the current editable version.
    Current,
    /// The public store is a materialized historical snapshot.
    CheckedOut,
}
// sirno:witness:versioning:end

/// Resolve the lock path next to the config file.
pub fn resolve_lock_path(config_path: impl AsRef<Path>) -> PathBuf {
    config_path.as_ref().parent().unwrap_or_else(|| Path::new(".")).join(LOCK_FILE_NAME)
}

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
    /// Current public-store state must be editable.
    #[error("current history state cannot be marked mutable")]
    CurrentMutable,
    /// The lock file could not be created.
    #[error("failed to create lock file {path}")]
    Create {
        /// Path that could not be created.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The lock file could not be written.
    #[error("failed to write lock file {path}")]
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
    fn renders_current_history_lock() {
        let lock = SirnoLock::current(Eterator(7));
        let rendered = lock.to_toml().unwrap();

        assert_eq!(
            rendered,
            "\
# This file is generated and managed by Sirno.
# Do not edit it by hand.

[history]
status = \"current\"
version = 7
"
        );
    }

    #[test]
    fn renders_mutable_checkout_lock() {
        let lock = SirnoLock::checked_out(Eterator(3), true);
        let rendered = lock.to_toml().unwrap();

        assert_eq!(
            rendered,
            "\
# This file is generated and managed by Sirno.
# Do not edit it by hand.

[history]
status = \"checked-out\"
version = 3
mutable = true
"
        );
    }

    #[test]
    fn rejects_mutable_current_lock() {
        let error = toml::from_str::<SirnoLock>(
            r#"
[history]
status = "current"
version = 3
mutable = true
"#,
        )
        .unwrap()
        .validate()
        .unwrap_err();

        assert!(matches!(error, LockError::CurrentMutable));
    }
}
