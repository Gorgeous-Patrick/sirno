//! Entry atoms and addresses.
//!
//! An entry atom is one local filename or domain segment.
//! An entry address is the dot-separated lookup form used by structural metadata,
//! generated footers, and witness lookup.

use std::fmt::{Display, Formatter};
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use eter::filesystem::{FilesystemEntryId, FilesystemError};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use thiserror::Error;

// sirno:witness:entry:begin
/// Maximum byte length for an entry atom before the `.md` extension is added.
pub const ENTRY_ATOM_MAX_BYTES: usize = 252;

const RESERVED_STORAGE_ENTRY_ATOM: &str = "Eter.lock.toml";

/// Dot-free segment of one Sirno entry address.
///
/// Invariant: the atom is a non-empty cross-platform filename stem.
/// Lowercase ASCII kebab-case is the recommended writing style.
/// The dot character is reserved for entry addresses.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EntryAtom(SmolStr);
// sirno:witness:entry:end

impl EntryAtom {
    /// Construct a validated entry atom.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, EntryAtomError> {
        let raw = raw.as_ref();
        Self::validate(raw)?;
        Ok(Self(SmolStr::new(raw)))
    }

    /// Borrow the atom as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert this atom into the filesystem-safe identifier used by `eter`.
    pub fn to_filesystem_id(&self) -> Result<FilesystemEntryId, FilesystemError> {
        FilesystemEntryId::new(self.as_str())
    }

    // sirno:witness:entry:begin
    fn validate(raw: &str) -> Result<(), EntryAtomError> {
        if raw.is_empty() {
            return Err(EntryAtomError::Empty);
        }

        if raw.len() > ENTRY_ATOM_MAX_BYTES {
            return Err(EntryAtomError::TooLong {
                atom: raw.to_owned(),
                max: ENTRY_ATOM_MAX_BYTES,
            });
        }

        if raw.ends_with(' ') {
            return Err(EntryAtomError::TrailingSpace(raw.to_owned()));
        }

        if raw.contains('.') {
            return Err(EntryAtomError::ReservedDot(raw.to_owned()));
        }

        if raw.eq_ignore_ascii_case(RESERVED_STORAGE_ENTRY_ATOM)
            || windows_device_name(raw).is_some()
        {
            return Err(EntryAtomError::ReservedFilename(raw.to_owned()));
        }

        for character in raw.chars() {
            if is_forbidden_filename_character(character) {
                return Err(EntryAtomError::InvalidCharacter { atom: raw.to_owned(), character });
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

impl Display for EntryAtom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for EntryAtom {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for EntryAtom {
    type Err = EntryAtomError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for EntryAtom {
    type Error = EntryAtomError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for EntryAtom {
    type Error = EntryAtomError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<FilesystemEntryId> for EntryAtom {
    type Error = EntryAtomError;

    fn try_from(value: FilesystemEntryId) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl<'de> Deserialize<'de> for EntryAtom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = SmolStr::deserialize(deserializer)?;
        Self::new(value.as_str()).map_err(serde::de::Error::custom)
    }
}

/// Dot-separated lookup address for one Sirno entry.
///
/// Invariant: every segment is a valid [`EntryAtom`].
/// The leading-dot form is reserved for Sirno built-in lake paths.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EntryAddress(SmolStr);

impl EntryAddress {
    /// Construct a validated entry address.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, EntryAddressError> {
        let raw = raw.as_ref();
        Self::validate(raw)?;
        Ok(Self(SmolStr::new(raw)))
    }

    /// Construct a one-segment entry address from a local atom.
    pub fn from_atom(atom: EntryAtom) -> Self {
        Self(SmolStr::new(atom.as_str()))
    }

    /// Borrow the address as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return the local atom in the final address segment.
    pub fn local_atom(&self) -> EntryAtom {
        EntryAtom::new(self.as_str().rsplit('.').next().expect("entry address is non-empty"))
            .expect("entry address segment is valid")
    }

    /// Return this address under a domain prefix.
    pub fn under_domain(&self, domain: &EntryAtom) -> Self {
        Self::new(format!("{}.{}", domain.as_str(), self.as_str()))
            .expect("domain and entry address compose into a valid entry address")
    }

    /// Return true when this address begins with a domain atom.
    pub fn starts_with_domain(&self, domain: &EntryAtom) -> bool {
        self.as_str() == domain.as_str()
            || self
                .as_str()
                .strip_prefix(domain.as_str())
                .is_some_and(|suffix| suffix.starts_with('.'))
    }

    /// Convert this address into a lake-root-relative Markdown file path.
    pub fn to_lake_relative_path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        let mut segments = self.as_str().split('.').peekable();
        while let Some(segment) = segments.next() {
            if segments.peek().is_some() {
                path.push(segment);
            } else {
                path.push(format!("{segment}.md"));
            }
        }
        path
    }

    /// Construct an entry address from a lake-root-relative Markdown file path.
    pub fn from_lake_relative_path(path: &Path) -> Result<Self, EntryAddressError> {
        if path.as_os_str().is_empty() || path.is_absolute() {
            return Err(EntryAddressError::InvalidRelativePath(path.to_path_buf()));
        }

        let mut segments = Vec::new();
        let mut components = path.components().peekable();
        while let Some(component) = components.next() {
            let Component::Normal(component) = component else {
                return Err(EntryAddressError::InvalidRelativePath(path.to_path_buf()));
            };
            let Some(component) = component.to_str() else {
                return Err(EntryAddressError::NonUtf8Path(path.to_path_buf()));
            };
            if component.starts_with('.') {
                return Err(EntryAddressError::ReservedBuiltinPath(component.to_owned()));
            }

            if components.peek().is_some() {
                segments.push(EntryAtom::new(component)?);
                continue;
            }

            let Some(stem) = component.strip_suffix(".md") else {
                return Err(EntryAddressError::MissingMarkdownExtension(path.to_path_buf()));
            };
            if stem.contains('.') {
                return Err(EntryAddressError::DottedFilename(component.to_owned()));
            }
            segments.push(EntryAtom::new(stem)?);
        }

        if segments.is_empty() {
            return Err(EntryAddressError::Empty);
        }
        Ok(Self::from_segments(segments))
    }

    /// Convert this address into the filesystem-safe identifier used by `eter`.
    pub fn to_filesystem_id(&self) -> Result<FilesystemEntryId, FilesystemError> {
        FilesystemEntryId::new(self.as_str())
    }

    fn from_segments(segments: Vec<EntryAtom>) -> Self {
        let path = segments.iter().map(EntryAtom::as_str).collect::<Vec<_>>().join(".");
        Self(SmolStr::new(path))
    }

    fn validate(raw: &str) -> Result<(), EntryAddressError> {
        if raw.is_empty() {
            return Err(EntryAddressError::Empty);
        }
        if raw.starts_with('.') {
            return Err(EntryAddressError::ReservedBuiltinPath(raw.to_owned()));
        }

        let mut saw_segment = false;
        for segment in raw.split('.') {
            if segment.is_empty() {
                return Err(EntryAddressError::EmptySegment(raw.to_owned()));
            }
            EntryAtom::new(segment)?;
            saw_segment = true;
        }
        if !saw_segment {
            return Err(EntryAddressError::Empty);
        }
        Ok(())
    }
}

impl Display for EntryAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for EntryAddress {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<EntryAtom> for EntryAddress {
    fn from(value: EntryAtom) -> Self {
        Self::from_atom(value)
    }
}

impl FromStr for EntryAddress {
    type Err = EntryAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for EntryAddress {
    type Error = EntryAddressError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for EntryAddress {
    type Error = EntryAddressError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<FilesystemEntryId> for EntryAddress {
    type Error = EntryAddressError;

    fn try_from(value: FilesystemEntryId) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl<'de> Deserialize<'de> for EntryAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = SmolStr::deserialize(deserializer)?;
        Self::new(value.as_str()).map_err(serde::de::Error::custom)
    }
}

/// Error raised when text cannot be used as an entry atom.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntryAtomError {
    /// Empty atoms cannot name an address segment.
    #[error("entry atom must not be empty")]
    Empty,
    /// The atom is longer than the cross-platform Markdown filename budget.
    #[error(
        "entry atom is too long for a cross-platform Markdown filename: {atom}; maximum is {max} bytes"
    )]
    TooLong {
        /// Invalid raw atom.
        atom: String,
        /// Maximum accepted byte length.
        max: usize,
    },
    /// The atom contains a character that is not valid in common filename components.
    #[error("entry atom contains invalid filename character `{character}`: {atom}")]
    InvalidCharacter {
        /// Invalid raw atom.
        atom: String,
        /// Invalid character.
        character: char,
    },
    /// Windows rejects filename components that end with a space.
    #[error("entry atom must not end with a space: {0}")]
    TrailingSpace(String),
    /// The dot character is reserved for Sirno entry address syntax.
    #[error("entry atom must not contain reserved dot character: {0}")]
    ReservedDot(String),
    /// The atom is reserved by Windows or Sirno storage.
    #[error("entry atom uses a reserved filename: {0}")]
    ReservedFilename(String),
}

/// Error raised when text cannot be used as an entry address.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntryAddressError {
    /// Empty addresses cannot find an entry.
    #[error("entry address must not be empty")]
    Empty,
    /// Empty address segments cannot name a domain or entry.
    #[error("entry address contains an empty segment: {0}")]
    EmptySegment(String),
    /// The leading-dot form belongs to Sirno built-in lake paths.
    #[error("entry address uses reserved built-in path form: {0}")]
    ReservedBuiltinPath(String),
    /// A lake path was not a normal relative path.
    #[error("entry address must be a normal lake-relative Markdown path: {0}")]
    InvalidRelativePath(PathBuf),
    /// A lake path was not valid UTF-8.
    #[error("entry address must be valid UTF-8: {0}")]
    NonUtf8Path(PathBuf),
    /// Entry files must use `.md`.
    #[error("entry address must point at a Markdown file: {0}")]
    MissingMarkdownExtension(PathBuf),
    /// Dots are represented by folders, not filename stems.
    #[error("entry filename must not contain dots: {0}")]
    DottedFilename(String),
    /// One address segment is not a valid entry atom.
    #[error(transparent)]
    EntryAtom(#[from] EntryAtomError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_cross_platform_filename_stems() {
        for raw in ["concept-2", "Concept 2", "api_v2+draft", "設計ノート"] {
            let id = EntryAtom::new(raw).unwrap();
            assert_eq!(id.as_str(), raw);
        }
    }

    #[test]
    fn rejects_empty_id() {
        assert_eq!(EntryAtom::new("").unwrap_err(), EntryAtomError::Empty);
    }

    #[test]
    fn rejects_too_long_id() {
        assert!(matches!(
            EntryAtom::new("a".repeat(ENTRY_ATOM_MAX_BYTES + 1)).unwrap_err(),
            EntryAtomError::TooLong { max: ENTRY_ATOM_MAX_BYTES, .. }
        ));
    }

    #[test]
    fn rejects_forbidden_filename_characters() {
        for raw in ["a/b", "a\\b", "a:b", "a*b", "a?b", "a\"b", "a<b", "a>b", "a|b", "a\nb"] {
            assert!(matches!(
                EntryAtom::new(raw).unwrap_err(),
                EntryAtomError::InvalidCharacter { .. }
            ));
        }
    }

    #[test]
    fn rejects_trailing_space() {
        assert!(matches!(
            EntryAtom::new("concept ").unwrap_err(),
            EntryAtomError::TrailingSpace(_)
        ));
    }

    #[test]
    fn rejects_reserved_dot() {
        assert!(matches!(EntryAtom::new("concept.").unwrap_err(), EntryAtomError::ReservedDot(_)));
        assert!(matches!(EntryAtom::new("api.v2").unwrap_err(), EntryAtomError::ReservedDot(_)));
        assert!(matches!(EntryAtom::new(".").unwrap_err(), EntryAtomError::ReservedDot(_)));
    }

    #[test]
    fn rejects_reserved_filenames() {
        for raw in ["con", "NUL", "com1", "LPT9"] {
            assert!(matches!(
                EntryAtom::new(raw).unwrap_err(),
                EntryAtomError::ReservedFilename(_)
            ));
        }
    }

    #[test]
    fn entry_address_accepts_one_or_more_atoms() {
        for raw in ["concept", "core.design", "core.design.routes"] {
            let path = EntryAddress::new(raw).unwrap();
            assert_eq!(path.as_str(), raw);
        }
    }

    #[test]
    fn entry_address_maps_to_lake_relative_markdown_path() {
        assert_eq!(
            EntryAddress::new("concept").unwrap().to_lake_relative_path(),
            Path::new("concept.md")
        );
        assert_eq!(
            EntryAddress::new("core.design").unwrap().to_lake_relative_path(),
            Path::new("core").join("design.md")
        );
    }

    #[test]
    fn entry_address_parses_lake_relative_markdown_path() {
        assert_eq!(
            EntryAddress::from_lake_relative_path(Path::new("core").join("design.md").as_path())
                .unwrap()
                .as_str(),
            "core.design"
        );
    }

    #[test]
    fn entry_address_rejects_reserved_builtins_and_empty_segments() {
        assert!(matches!(
            EntryAddress::new(".artifacts").unwrap_err(),
            EntryAddressError::ReservedBuiltinPath(_)
        ));
        assert!(matches!(
            EntryAddress::new("core.").unwrap_err(),
            EntryAddressError::EmptySegment(_)
        ));
        assert!(matches!(
            EntryAddress::new("core..design").unwrap_err(),
            EntryAddressError::EmptySegment(_)
        ));
    }

    #[test]
    fn entry_address_rejects_dotted_lake_filenames() {
        assert!(matches!(
            EntryAddress::from_lake_relative_path(Path::new("core.design.md")).unwrap_err(),
            EntryAddressError::DottedFilename(_)
        ));
    }
}
