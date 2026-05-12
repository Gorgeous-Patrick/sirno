//! Entry identifiers.
//!
//! An entry id is the stable nominal handle for a Sirno entry.
//! It is derived from the filename stem in the public store model
//! and reused by relation metadata, generated footers, and witness lookup.

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use eter::filesystem::{FilesystemEntryId, FilesystemError};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use thiserror::Error;

/// Stable identifier for one Sirno entry.
///
/// Invariant: the identifier is non-empty lowercase ASCII kebab-case.
/// Digits are allowed inside segments. Hyphens separate non-empty segments.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EntryId(SmolStr);

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

    fn validate(raw: &str) -> Result<(), EntryIdError> {
        if raw.is_empty() {
            return Err(EntryIdError::Empty);
        }

        let mut last_was_hyphen = false;
        for (index, byte) in raw.bytes().enumerate() {
            match byte {
                | b'a'..=b'z' | b'0'..=b'9' => {
                    last_was_hyphen = false;
                }
                | b'-' => {
                    if index == 0 || last_was_hyphen {
                        return Err(EntryIdError::InvalidKebabCase(raw.to_owned()));
                    }
                    last_was_hyphen = true;
                }
                | _ => return Err(EntryIdError::InvalidCharacter(raw.to_owned())),
            }
        }

        if last_was_hyphen {
            return Err(EntryIdError::InvalidKebabCase(raw.to_owned()));
        }

        Ok(())
    }
}

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
    /// The id contains a character outside lowercase ASCII, digits, and hyphen.
    #[error("entry id contains an invalid character: {0}")]
    InvalidCharacter(String),
    /// The id is not segmented as lowercase kebab-case.
    #[error("entry id must be lowercase ASCII kebab-case: {0}")]
    InvalidKebabCase(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_lowercase_kebab_case_with_digits() {
        let id = EntryId::new("concept-2").unwrap();
        assert_eq!(id.as_str(), "concept-2");
    }

    #[test]
    fn rejects_empty_id() {
        assert_eq!(EntryId::new("").unwrap_err(), EntryIdError::Empty);
    }

    #[test]
    fn rejects_uppercase_id() {
        assert!(matches!(EntryId::new("Concept").unwrap_err(), EntryIdError::InvalidCharacter(_)));
    }

    #[test]
    fn rejects_leading_trailing_and_repeated_hyphens() {
        assert!(matches!(EntryId::new("-concept").unwrap_err(), EntryIdError::InvalidKebabCase(_)));
        assert!(matches!(EntryId::new("concept-").unwrap_err(), EntryIdError::InvalidKebabCase(_)));
        assert!(matches!(
            EntryId::new("concept--entry").unwrap_err(),
            EntryIdError::InvalidKebabCase(_)
        ));
    }
}
