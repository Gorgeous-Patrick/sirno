//! Entry identifiers.
//!
//! An entry id is the stable nominal handle for a Sirno entry.
//! It is derived from the filename stem in the public lake model
//! and reused by structural metadata, generated footers, and witness lookup.

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use eter::filesystem::{FilesystemEntryId, FilesystemError};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use thiserror::Error;

// sirno:witness:entry:begin
/// Maximum byte length for an entry id before the `.md` extension is added.
pub const ENTRY_ID_MAX_BYTES: usize = 252;

const RESERVED_STORAGE_ENTRY_ID: &str = "Eter.lock.toml";

/// Stable identifier for one Sirno entry.
///
/// Invariant: the identifier is a non-empty cross-platform filename stem.
/// Lowercase ASCII kebab-case is the recommended writing style.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EntryId(SmolStr);
// sirno:witness:entry:end

impl EntryId {
    /// Construct a validated entry id.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, EntryIdError> {
        let raw = raw.as_ref();
        Self::validate(raw)?;
        Ok(Self(SmolStr::new(raw)))
    }

    /// Borrow the id as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert this id into the filesystem-safe identifier used by `eter`.
    pub fn to_filesystem_id(&self) -> Result<FilesystemEntryId, FilesystemError> {
        FilesystemEntryId::new(self.as_str())
    }

    // sirno:witness:entry:begin
    fn validate(raw: &str) -> Result<(), EntryIdError> {
        if raw.is_empty() {
            return Err(EntryIdError::Empty);
        }

        if raw.len() > ENTRY_ID_MAX_BYTES {
            return Err(EntryIdError::TooLong { id: raw.to_owned(), max: ENTRY_ID_MAX_BYTES });
        }

        if raw.ends_with([' ', '.']) {
            return Err(EntryIdError::TrailingSpaceOrPeriod(raw.to_owned()));
        }

        if raw.eq_ignore_ascii_case(RESERVED_STORAGE_ENTRY_ID) || windows_device_name(raw).is_some()
        {
            return Err(EntryIdError::ReservedFilename(raw.to_owned()));
        }

        for character in raw.chars() {
            if is_forbidden_filename_character(character) {
                return Err(EntryIdError::InvalidCharacter { id: raw.to_owned(), character });
            }
        }

        Ok(())
    }
    // sirno:witness:entry:end
}

// sirno:witness:entry:begin
fn is_forbidden_filename_character(character: char) -> bool {
    character.is_control()
        || matches!(character, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | ',')
}

fn windows_device_name(raw: &str) -> Option<&str> {
    let basename = raw.split('.').next().unwrap_or(raw);
    let uppercase = basename.to_ascii_uppercase();
    matches!(
        uppercase.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
    .then_some(basename)
}
// sirno:witness:entry:end

impl Display for EntryId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for EntryId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for EntryId {
    type Err = EntryIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for EntryId {
    type Error = EntryIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for EntryId {
    type Error = EntryIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<FilesystemEntryId> for EntryId {
    type Error = EntryIdError;

    fn try_from(value: FilesystemEntryId) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl<'de> Deserialize<'de> for EntryId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = SmolStr::deserialize(deserializer)?;
        Self::new(value.as_str()).map_err(serde::de::Error::custom)
    }
}

/// Error raised when text cannot be used as an entry id.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntryIdError {
    /// Empty identifiers cannot name an entry.
    #[error("entry id must not be empty")]
    Empty,
    /// The id is longer than the cross-platform Markdown filename budget.
    #[error(
        "entry id is too long for a cross-platform Markdown filename: {id}; maximum is {max} bytes"
    )]
    TooLong {
        /// Invalid raw id.
        id: String,
        /// Maximum accepted byte length.
        max: usize,
    },
    /// The id contains a character that is not valid in common filename components.
    #[error("entry id contains invalid filename character `{character}`: {id}")]
    InvalidCharacter {
        /// Invalid raw id.
        id: String,
        /// Invalid character.
        character: char,
    },
    /// Windows rejects filename components that end with a space or period.
    #[error("entry id must not end with a space or period: {0}")]
    TrailingSpaceOrPeriod(String),
    /// The id is reserved by Windows or Sirno storage.
    #[error("entry id uses a reserved filename: {0}")]
    ReservedFilename(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_cross_platform_filename_stems() {
        for raw in ["concept-2", "Concept 2", "api_v2.1+draft", "設計ノート"] {
            let id = EntryId::new(raw).unwrap();
            assert_eq!(id.as_str(), raw);
        }
    }

    #[test]
    fn rejects_empty_id() {
        assert_eq!(EntryId::new("").unwrap_err(), EntryIdError::Empty);
    }

    #[test]
    fn rejects_too_long_id() {
        assert!(matches!(
            EntryId::new("a".repeat(ENTRY_ID_MAX_BYTES + 1)).unwrap_err(),
            EntryIdError::TooLong { max: ENTRY_ID_MAX_BYTES, .. }
        ));
    }

    #[test]
    fn rejects_forbidden_filename_characters() {
        for raw in ["a/b", "a\\b", "a:b", "a*b", "a?b", "a\"b", "a<b", "a>b", "a|b", "a\nb"] {
            assert!(matches!(
                EntryId::new(raw).unwrap_err(),
                EntryIdError::InvalidCharacter { .. }
            ));
        }
    }

    #[test]
    fn rejects_trailing_space_or_period() {
        assert!(matches!(
            EntryId::new("concept ").unwrap_err(),
            EntryIdError::TrailingSpaceOrPeriod(_)
        ));
        assert!(matches!(
            EntryId::new("concept.").unwrap_err(),
            EntryIdError::TrailingSpaceOrPeriod(_)
        ));
        assert!(matches!(EntryId::new(".").unwrap_err(), EntryIdError::TrailingSpaceOrPeriod(_)));
    }

    #[test]
    fn rejects_reserved_filenames() {
        for raw in ["con", "NUL", "com1", "LPT9.notes", "eter.lock.toml"] {
            assert!(matches!(EntryId::new(raw).unwrap_err(), EntryIdError::ReservedFilename(_)));
        }
    }
}
