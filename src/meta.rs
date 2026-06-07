//! Generated meta-level lockfile for Sirno Lake parsing.
//!
//! The registry is derived from raw entry frontmatter before typed metadata parsing.
//! The lockfile records which entries define intrinsic metadata fields and structural relations.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entry::{EntryMetaType, FROZEN_FIELD, META_FIELD, META_TYPE_FIELD, RawEntry};
use crate::identifier::{EntryAddress, EntryAtom};
use crate::structural::StructuralSettings;

/// Generated meta registry lockfile name under `.sirno/`.
pub const META_FILE_NAME: &str = "meta.toml";
/// Current generated meta registry lockfile schema.
pub const META_FILE_SCHEMA: u32 = 1;

const META_FILE_HEADER: &str = "\
# This file is a generated Sirno lockfile.
# Sirno rewrites it from lake entry metadata when the registry changes.
# Edit the lake entries that define meta fields.

";

/// Ordered intrinsic metadata fields.
pub type IntrinsicFieldMap = IndexMap<String, EntryAddress>;

/// Discovered meta-level Sirno schema knowledge for one lake.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetaRegistry {
    intrinsics: IntrinsicFieldMap,
    structural: StructuralSettings,
}

/// File representation for the tracked `.sirno/meta.toml` lockfile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetaFile {
    /// Registry schema version.
    pub schema: u32,
    /// Discovered intrinsic metadata fields.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intrinsics: Vec<MetaFieldRecord>,
    /// Discovered structural relation fields.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structural: Vec<MetaFieldRecord>,
}

/// One generated meta-level field record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetaFieldRecord {
    /// Metadata field name.
    pub field: String,
    /// Entry that defines the field.
    pub entry: EntryAddress,
}

impl MetaFieldRecord {
    fn new(field: impl Into<String>, entry: EntryAddress) -> Self {
        Self { field: field.into(), entry }
    }
}

impl MetaFile {
    /// Load a generated meta registry lockfile from a specific path.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, MetaRegistryError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .map_err(|source| MetaRegistryError::Read { path: path.to_path_buf(), source })?;
        let file: Self = toml::from_str(&source)
            .map_err(|source| MetaRegistryError::Parse { path: path.to_path_buf(), source })?;
        file.validate()?;
        Ok(file)
    }

    fn validate(&self) -> Result<(), MetaRegistryError> {
        if self.schema != META_FILE_SCHEMA {
            return Err(MetaRegistryError::UnsupportedSchema { schema: self.schema });
        }
        Ok(())
    }
}

impl MetaRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a registry from explicit parts.
    pub fn from_parts(
        intrinsics: impl IntoIterator<Item = (impl Into<String>, EntryAddress)>,
        structural: StructuralSettings,
    ) -> Self {
        Self {
            intrinsics: intrinsics
                .into_iter()
                .map(|(field, entry)| (field.into(), entry))
                .collect(),
            structural,
        }
    }

    /// Construct the conventional registry used by isolated parser callers.
    ///
    /// Lake loading discovers this information from entries instead.
    pub fn standard() -> Self {
        let mut registry = Self::new();
        registry.set_intrinsic_entry(
            crate::entry::NAME_FIELD,
            EntryAddress::new(crate::entry::NAME_FIELD)
                .expect("standard name entry address is valid"),
        );
        registry.set_intrinsic_entry(
            crate::entry::DESC_FIELD,
            EntryAddress::new(crate::entry::DESC_FIELD)
                .expect("standard desc entry address is valid"),
        );
        registry
    }

    /// Discover a registry from raw entries.
    ///
    /// Registry order is entry-address order.
    pub fn from_raw_entries<'a>(entries: impl IntoIterator<Item = &'a RawEntry>) -> Self {
        let mut intrinsic_entries = Vec::new();
        let mut structural_entries = Vec::new();
        for entry in entries {
            match entry.meta_type() {
                | Ok(Some(EntryMetaType::Intrinsic))
                    if validate_intrinsic_field_name(entry.id.as_str()).is_ok() =>
                {
                    intrinsic_entries.push(entry.id.clone());
                }
                | Ok(Some(EntryMetaType::Structural))
                    if validate_meta_field_name(entry.id.as_str()).is_ok() =>
                {
                    structural_entries.push(entry.id.clone());
                }
                | Ok(Some(_)) | Ok(None) | Err(_) => {}
            }
        }
        intrinsic_entries.sort();
        structural_entries.sort();

        let mut registry = Self::new();
        for entry in intrinsic_entries {
            registry.set_intrinsic_entry(entry.as_str().to_owned(), entry);
        }
        registry.structural = StructuralSettings::from_relations(
            structural_entries.into_iter().map(|entry| (entry.as_str().to_owned(), entry)),
        );
        registry
    }

    /// Return the discovered structural settings.
    pub fn structural(&self) -> &StructuralSettings {
        &self.structural
    }

    /// Iterate intrinsic fields in registry order.
    pub fn intrinsic_fields(&self) -> impl Iterator<Item = (&str, &EntryAddress)> {
        self.intrinsics.iter().map(|(field, entry)| (field.as_str(), entry))
    }

    /// Return true when a metadata field is intrinsic.
    pub fn contains_intrinsic_field(&self, field: &str) -> bool {
        self.intrinsics.contains_key(field)
    }

    /// Return true when an entry defines a discovered intrinsic field.
    pub fn contains_intrinsic_entry(&self, entry: &EntryAddress) -> bool {
        self.intrinsics.values().any(|defined| defined == entry)
    }

    /// Return the entry that defines one intrinsic metadata field.
    pub fn intrinsic_entry_for_field(&self, field: &str) -> Option<&EntryAddress> {
        self.intrinsics.get(field)
    }

    /// Return registry entries defined inside one managed domain.
    pub fn only_domain(&self, domain: &EntryAtom) -> Self {
        Self {
            intrinsics: self
                .intrinsics
                .iter()
                .filter(|(_, entry)| entry.starts_with_domain(domain))
                .map(|(field, entry)| (field.clone(), entry.clone()))
                .collect(),
            structural: StructuralSettings::from_relations(
                self.structural
                    .relations()
                    .filter(|(_, entry)| entry.starts_with_domain(domain))
                    .map(|(field, entry)| (field.to_owned(), entry.clone())),
            ),
        }
    }

    /// Return registry entries outside the given managed domains.
    pub fn without_domains<'a>(&self, domains: impl IntoIterator<Item = &'a EntryAtom>) -> Self {
        let domains = domains.into_iter().collect::<Vec<_>>();
        Self {
            intrinsics: self
                .intrinsics
                .iter()
                .filter(|(_, entry)| !domains.iter().any(|domain| entry.starts_with_domain(domain)))
                .map(|(field, entry)| (field.clone(), entry.clone()))
                .collect(),
            structural: StructuralSettings::from_relations(
                self.structural
                    .relations()
                    .filter(|(_, entry)| {
                        !domains.iter().any(|domain| entry.starts_with_domain(domain))
                    })
                    .map(|(field, entry)| (field.to_owned(), entry.clone())),
            ),
        }
    }

    /// Add or update one intrinsic field.
    pub fn set_intrinsic_entry(&mut self, field: impl Into<String>, entry: EntryAddress) -> bool {
        let field = field.into();
        let changed = self.intrinsics.get(&field) != Some(&entry);
        self.intrinsics.insert(field, entry);
        changed
    }

    /// Convert this registry into its lockfile representation.
    pub fn to_file(&self) -> MetaFile {
        MetaFile {
            schema: META_FILE_SCHEMA,
            intrinsics: self
                .intrinsics
                .iter()
                .map(|(field, entry)| MetaFieldRecord::new(field.clone(), entry.clone()))
                .collect(),
            structural: self
                .structural
                .relations()
                .map(|(field, entry)| MetaFieldRecord::new(field.to_owned(), entry.clone()))
                .collect(),
        }
    }

    /// Render this registry to TOML.
    pub fn to_toml(&self) -> Result<String, MetaRegistryError> {
        let mut source = String::from(META_FILE_HEADER);
        source
            .push_str(&toml::to_string_pretty(&self.to_file()).map_err(MetaRegistryError::Render)?);
        Ok(source)
    }

    /// Write this registry to its generated lockfile.
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), MetaRegistryError> {
        let path = path.as_ref();
        let source = self.to_toml()?;
        match fs::read_to_string(path) {
            | Ok(existing) if existing == source => return Ok(()),
            | Ok(_) => {}
            | Err(source) if source.kind() == ErrorKind::NotFound => {}
            | Err(source) => {
                return Err(MetaRegistryError::Read { path: path.to_path_buf(), source });
            }
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| MetaRegistryError::CreateDirectory {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(path, source)
            .map_err(|source| MetaRegistryError::Write { path: path.to_path_buf(), source })
    }
}

/// Validate a metadata key that can be discovered as an intrinsic field.
pub fn validate_intrinsic_field_name(field: &str) -> Result<(), MetaFieldNameError> {
    validate_meta_field_name(field)
}

/// Validate a metadata key that can be discovered at the meta level.
pub fn validate_meta_field_name(field: &str) -> Result<(), MetaFieldNameError> {
    if field.is_empty() || field.contains('\n') || field.contains('\r') || field.contains(',') {
        return Err(MetaFieldNameError::Invalid(field.to_owned()));
    }
    if field == META_FIELD || field == FROZEN_FIELD || field.starts_with("meta.") {
        return Err(MetaFieldNameError::Reserved(field.to_owned()));
    }
    if field == META_TYPE_FIELD {
        return Err(MetaFieldNameError::Reserved(field.to_owned()));
    }
    Ok(())
}

/// Error raised when a meta-level field cannot be used as a metadata key.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MetaFieldNameError {
    /// The field is empty, multiline, or contains a forbidden separator.
    #[error("meta field name must be a non-empty single-line metadata key: {0}")]
    Invalid(String),
    /// The field belongs to Sirno-managed metadata.
    #[error("meta field name is reserved for Sirno metadata: {0}")]
    Reserved(String),
}

/// Error raised while reading or writing the generated meta registry lockfile.
#[derive(Debug, Error)]
pub enum MetaRegistryError {
    /// The control directory could not be created.
    #[error("failed to create meta registry lockfile directory {path}")]
    CreateDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// TOML rendering failed.
    #[error("failed to render meta registry lockfile")]
    Render(#[source] toml::ser::Error),
    /// TOML parsing failed.
    #[error("failed to parse meta registry lockfile {path}")]
    Parse {
        /// Registry path.
        path: PathBuf,
        /// Underlying TOML error.
        #[source]
        source: toml::de::Error,
    },
    /// The registry lockfile schema is unsupported.
    #[error("unsupported meta registry lockfile schema {schema}")]
    UnsupportedSchema {
        /// Unsupported schema version.
        schema: u32,
    },
    /// The existing registry file could not be read.
    #[error("failed to read meta registry lockfile {path}")]
    Read {
        /// Registry path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The registry file could not be written.
    #[error("failed to write meta registry lockfile {path}")]
    Write {
        /// Registry path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_entry(id: &str, meta_type: &str) -> RawEntry {
        RawEntry::from_markdown(
            EntryAddress::new(id).unwrap(),
            &format!(
                "\
---
name: {id}
desc: Test entry.
meta.type: \"{meta_type}\"
---

Body.
"
            ),
        )
        .unwrap()
    }

    #[test]
    fn discovers_meta_entries_in_entry_address_order() {
        let registry = MetaRegistry::from_raw_entries([
            &raw_entry("name", "intrinsic"),
            &raw_entry("category", "structural"),
            &raw_entry("desc", "intrinsic"),
            &raw_entry("belongs", "structural"),
        ]);

        let intrinsics = registry.intrinsic_fields().map(|(field, _)| field).collect::<Vec<_>>();
        let structural =
            registry.structural().relations().map(|(field, _)| field).collect::<Vec<_>>();

        assert_eq!(intrinsics, ["desc", "name"]);
        assert_eq!(structural, ["belongs", "category"]);
    }

    #[test]
    fn renders_generated_registry_lockfile_toml() {
        let registry = MetaRegistry::from_raw_entries([
            &raw_entry("name", "intrinsic"),
            &raw_entry("category", "structural"),
        ]);

        let source = registry.to_toml().unwrap();
        let parsed: MetaFile = toml::from_str(&source).unwrap();

        assert_eq!(parsed.schema, META_FILE_SCHEMA);
        assert_eq!(parsed.intrinsics[0].field, "name");
        assert_eq!(parsed.structural[0].field, "category");
        assert!(source.starts_with("# This file is a generated Sirno lockfile."));
    }
}
