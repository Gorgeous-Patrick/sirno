//! Lake-owned artifacts attached to Sirno entries.
//!
//! Entry artifacts live under `.artifacts/<entry-address>/` in the Sirno Lake.
//! The entry Markdown files stay flat at the lake root.

use std::fmt::{Display, Formatter};
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use smol_str::SmolStr;
use thiserror::Error;

use crate::identifier::EntryAddress;

/// Reserved lake directory that stores entry-owned artifacts.
// sirno:witness:entry-artifact:begin
pub const ARTIFACT_DIRECTORY_NAME: &str = ".artifacts";

/// Relative path for one artifact inside an entry artifact directory.
///
/// Invariant: the path is non-empty, relative, UTF-8, and contains only normal components.
/// It is stored with `/` separators so it can be compared and versioned deterministically.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EntryArtifactPath(SmolStr);

impl EntryArtifactPath {
    /// Construct a validated artifact-relative path.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, EntryArtifactPathError> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(EntryArtifactPathError::Empty);
        }
        if path.is_absolute() {
            return Err(EntryArtifactPathError::Absolute(path.to_path_buf()));
        }

        let mut parts = Vec::new();
        for component in path.components() {
            match component {
                | Component::Normal(component) => {
                    let Some(component) = component.to_str() else {
                        return Err(EntryArtifactPathError::NonUtf8(path.to_path_buf()));
                    };
                    if component.is_empty() {
                        return Err(EntryArtifactPathError::EmptyComponent(path.to_path_buf()));
                    }
                    parts.push(component);
                }
                | Component::CurDir
                | Component::ParentDir
                | Component::RootDir
                | Component::Prefix(_) => {
                    return Err(EntryArtifactPathError::NonRelative(path.to_path_buf()));
                }
            }
        }

        if parts.is_empty() {
            return Err(EntryArtifactPathError::Empty);
        }

        Ok(Self(SmolStr::new(parts.join("/"))))
    }

    /// Borrow the normalized artifact path as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert this artifact path to an OS path.
    pub fn to_path_buf(&self) -> PathBuf {
        self.as_str().split('/').collect()
    }
}

impl Display for EntryArtifactPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl FromStr for EntryArtifactPath {
    type Err = EntryArtifactPathError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::new(raw)
    }
}

impl<'de> Deserialize<'de> for EntryArtifactPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::new(raw.as_str()).map_err(serde::de::Error::custom)
    }
}

/// One lake-owned artifact attached to an entry.
///
/// Invariant: `owner` names the entry whose `.artifacts/<entry-address>/` tree contains `path`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryArtifact {
    /// Entry that owns this artifact.
    pub owner: EntryAddress,
    /// Path relative to the owning entry artifact directory.
    pub path: EntryArtifactPath,
    /// Opaque artifact bytes.
    pub content: Vec<u8>,
}
// sirno:witness:entry-artifact:end

impl EntryArtifact {
    /// Construct one typed entry artifact.
    pub fn new(owner: EntryAddress, path: EntryArtifactPath, content: impl Into<Vec<u8>>) -> Self {
        Self { owner, path, content: content.into() }
    }
}

/// Error raised when an artifact path cannot be represented in the Sirno Lake.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntryArtifactPathError {
    /// Artifact paths must contain at least one component.
    #[error("artifact path must not be empty")]
    Empty,
    /// Artifact paths are always relative to `.artifacts/<entry-address>/`.
    #[error("artifact path must be relative: {0}")]
    Absolute(PathBuf),
    /// Artifact paths must not contain parent, root, prefix, or current-directory components.
    #[error("artifact path must contain only normal relative components: {0}")]
    NonRelative(PathBuf),
    /// Artifact paths must be valid UTF-8 so Sirno can store stable path ids.
    #[error("artifact path must be valid UTF-8: {0}")]
    NonUtf8(PathBuf),
    /// Empty components cannot be represented in normalized artifact paths.
    #[error("artifact path contains an empty component: {0}")]
    EmptyComponent(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_path_normalizes_relative_paths() {
        let path = EntryArtifactPath::new(Path::new("images/logo.png")).unwrap();

        assert_eq!(path.as_str(), "images/logo.png");
        assert_eq!(path.to_path_buf(), PathBuf::from("images").join("logo.png"));
    }

    #[test]
    fn artifact_path_rejects_non_relative_components() {
        assert!(matches!(
            EntryArtifactPath::new(Path::new("../logo.png")).unwrap_err(),
            EntryArtifactPathError::NonRelative(_)
        ));
        assert!(matches!(
            EntryArtifactPath::new(Path::new("./logo.png")).unwrap_err(),
            EntryArtifactPathError::NonRelative(_)
        ));
    }

    #[test]
    fn artifact_path_deserialization_validates_invariant() {
        let error = serde_yaml::from_str::<EntryArtifactPath>("\"../logo.png\"").unwrap_err();

        assert!(error.to_string().contains("artifact path must contain only normal"));
    }
}
