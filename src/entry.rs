//! Sirno entry model and Markdown frontmatter syntax.
//!
//! An entry is the public unit of Sirno design storage.
//! The prose body carries design content.
//! The metadata block carries structure that tools read exactly.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use thiserror::Error;

use crate::id::{EntryId, EntryIdError};

const NAME_FIELD: &str = "name";
const DESCRIPTION_FIELD: &str = "description";
const CATEGORY_FIELD: &str = "category";
const CLUSTEE_FIELD: &str = "clustee";
const REFINER_FIELD: &str = "refiner";
const WITNESS_FIELD: &str = "witness";

// sirno:witness:start entry
/// One Sirno entry.
///
/// Invariant: `id` is a valid entry id.
/// `metadata` contains typed structural fields.
/// `body` is normal Markdown prose outside the metadata block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Stable nominal id for this entry.
    pub id: EntryId,
    /// Typed metadata read from the YAML block.
    pub metadata: EntryMetadata,
    /// Markdown body after the metadata block.
    pub body: String,
}
// sirno:witness:end

impl Entry {
    /// Construct an entry from already typed parts.
    // sirno:witness:start entry
    pub fn new(id: EntryId, metadata: EntryMetadata, body: impl Into<String>) -> Self {
        Self { id, metadata, body: body.into() }
    }
    // sirno:witness:end

    /// Parse an entry from canonical Markdown source.
    // sirno:witness:start entry
    pub fn from_markdown(id: EntryId, source: &str) -> Result<Self, EntryParseError> {
        let (metadata_source, body) = split_frontmatter(source)?;
        let metadata = EntryMetadata::from_yaml_source(metadata_source)?;
        Ok(Self::new(id, metadata, body))
    }
    // sirno:witness:end

    /// Render this entry to canonical Markdown source.
    // sirno:witness:start entry
    pub fn to_markdown(&self) -> Result<String, EntryRenderError> {
        Ok(format!("---\n{}---\n\n{}", self.metadata.to_yaml_source()?, self.body))
    }
    // sirno:witness:end

    /// Replace the Markdown body in an existing entry source.
    ///
    /// The frontmatter region and its separator are preserved exactly.
    // sirno:witness:start entry
    pub fn replace_markdown_body(source: &str, body: &str) -> Result<String, EntryParseError> {
        let body_start = frontmatter_body_start(source)?;
        Ok(format!("{}{}", &source[..body_start], body))
    }
    // sirno:witness:end
}

/// Metadata for one Sirno entry.
///
/// Invariant: `name` and `description` are single-line plain strings.
/// Structural vectors contain entry ids and therefore cannot contain invalid targets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryMetadata {
    /// Human-readable entry name.
    pub name: String,
    /// Short prose description of the entry.
    pub description: String,
    // sirno:witness:start category
    /// Categories that classify this entry.
    pub category: Vec<EntryId>,
    // sirno:witness:end
    // sirno:witness:start clustee
    /// Clique closures that group this entry.
    pub clustee: Vec<EntryId>,
    // sirno:witness:end
    // sirno:witness:start refiner
    /// Broader entries refined by this entry.
    pub refiner: Vec<EntryId>,
    // sirno:witness:end
    /// Witness marker declaring that this entry has repository evidence.
    pub witness: Option<WitnessMarker>,
}

impl EntryMetadata {
    /// Construct metadata with required fields and no structural field values.
    // sirno:witness:start metadata
    pub fn new(
        name: impl Into<String>, description: impl Into<String>,
    ) -> Result<Self, EntryParseError> {
        let name = name.into();
        let description = description.into();
        validate_plain_string(NAME_FIELD, &name)?;
        validate_plain_string(DESCRIPTION_FIELD, &description)?;
        Ok(Self {
            name,
            description,
            category: Vec::new(),
            clustee: Vec::new(),
            refiner: Vec::new(),
            witness: None,
        })
    }
    // sirno:witness:end

    /// Parse metadata from YAML source without surrounding `---` sentinels.
    // sirno:witness:start metadata
    pub fn from_yaml_source(source: &str) -> Result<Self, EntryParseError> {
        let canonical_witness = has_canonical_witness_marker(source);
        let value: Value = serde_yaml::from_str(source).map_err(EntryParseError::Yaml)?;
        let mut mapping = match value {
            | Value::Mapping(mapping) => mapping,
            | _ => return Err(EntryParseError::MetadataMustBeMapping),
        };

        reject_unknown_fields(&mapping)?;

        let name = take_required_string(&mut mapping, NAME_FIELD)?;
        let description = take_required_string(&mut mapping, DESCRIPTION_FIELD)?;
        validate_plain_string(NAME_FIELD, &name)?;
        validate_plain_string(DESCRIPTION_FIELD, &description)?;

        let category = take_optional_id_list(&mut mapping, CATEGORY_FIELD)?;
        let clustee = take_optional_id_list(&mut mapping, CLUSTEE_FIELD)?;
        let refiner = take_optional_id_list(&mut mapping, REFINER_FIELD)?;
        let witness = take_witness_marker(&mut mapping, canonical_witness)?;

        Ok(Self { name, description, category, clustee, refiner, witness })
    }
    // sirno:witness:end

    /// Render this metadata block to canonical YAML source.
    // sirno:witness:start metadata
    pub fn to_yaml_source(&self) -> Result<String, EntryRenderError> {
        validate_plain_string(NAME_FIELD, &self.name)?;
        validate_plain_string(DESCRIPTION_FIELD, &self.description)?;

        let mut out = String::new();
        out.push_str(&format!("name: {}\n", render_yaml_scalar(&self.name)?));
        out.push_str(&format!("description: {}\n", render_yaml_scalar(&self.description)?));
        render_id_list(&mut out, CATEGORY_FIELD, &self.category);
        render_id_list(&mut out, CLUSTEE_FIELD, &self.clustee);
        render_id_list(&mut out, REFINER_FIELD, &self.refiner);
        if self.witness.is_some() {
            out.push_str("witness:\n");
        }
        Ok(out)
    }
    // sirno:witness:end

    /// Returns every entry id mentioned by structural metadata.
    // sirno:witness:start metadata
    pub fn structural_targets(&self) -> impl Iterator<Item = (&'static str, &EntryId)> {
        self.category
            .iter()
            .map(|id| (CATEGORY_FIELD, id))
            .chain(self.clustee.iter().map(|id| (CLUSTEE_FIELD, id)))
            .chain(self.refiner.iter().map(|id| (REFINER_FIELD, id)))
    }
    // sirno:witness:end
}

/// Marker for the canonical `witness:` metadata field.
///
/// The public Markdown syntax has no value for this marker.
/// Storage backends may encode the presence bit internally,
/// but rendered entry metadata normalizes back to `witness:`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitnessMarker {
    /// The entry has repository evidence queried by entry id.
    Present,
}

/// Create the ordinary seed entries for a new Sirno store.
///
/// The entries are normal entries.
/// Later operations do not privilege them.
pub fn default_seed_entries() -> Result<Vec<Entry>, EntryParseError> {
    // sirno:witness:start meta
    let mut meta =
        EntryMetadata::new("Meta", "A category for entries that define project vocabulary.")?;
    meta.category.push(seed_id("meta"));
    meta.witness = None;
    // sirno:witness:end

    // sirno:witness:start concept
    let mut concept =
        EntryMetadata::new("Concept", "A named idea that compresses project knowledge.")?;
    concept.category.push(seed_id("meta"));
    // sirno:witness:end

    // sirno:witness:start narrative
    let mut narrative = EntryMetadata::new("Narrative", "A route through concepts for a reader.")?;
    narrative.category.push(seed_id("meta"));
    // sirno:witness:end

    Ok(vec![
        Entry::new(seed_id("meta"), meta, "Defines entries that classify other entries.\n"),
        Entry::new(
            seed_id("concept"),
            concept,
            "A concept gives a stable name to compressed project knowledge.\n",
        ),
        Entry::new(
            seed_id("narrative"),
            narrative,
            "A narrative records an order in which a reader can understand concepts.\n",
        ),
    ])
}

fn seed_id(raw: &str) -> EntryId {
    EntryId::new(raw).unwrap_or_else(|error| panic!("invalid built-in seed id `{raw}`: {error}"))
}

fn split_frontmatter(source: &str) -> Result<(&str, String), EntryParseError> {
    let body_start = frontmatter_body_start(source)?;
    let rest = source.strip_prefix("---\n").ok_or(EntryParseError::MissingFrontmatter)?;
    let index = rest.find("\n---\n").ok_or(EntryParseError::UnterminatedFrontmatter)?;
    let metadata = &rest[..index];
    Ok((metadata, source[body_start..].to_owned()))
}

fn frontmatter_body_start(source: &str) -> Result<usize, EntryParseError> {
    let rest = source.strip_prefix("---\n").ok_or(EntryParseError::MissingFrontmatter)?;
    let index = rest.find("\n---\n").ok_or(EntryParseError::UnterminatedFrontmatter)?;
    let mut body_start = "---\n".len() + index + "\n---\n".len();
    if source[body_start..].starts_with('\n') {
        body_start += 1;
    }
    Ok(body_start)
}

fn reject_unknown_fields(mapping: &Mapping) -> Result<(), EntryParseError> {
    let allowed = BTreeSet::from([
        NAME_FIELD,
        DESCRIPTION_FIELD,
        CATEGORY_FIELD,
        CLUSTEE_FIELD,
        REFINER_FIELD,
        WITNESS_FIELD,
    ]);
    for key in mapping.keys() {
        let Value::String(key) = key else {
            return Err(EntryParseError::MetadataKeyMustBeString);
        };
        if !allowed.contains(key.as_str()) {
            return Err(EntryParseError::UnknownField(key.clone()));
        }
    }
    Ok(())
}

fn take_required_string(
    mapping: &mut Mapping, field: &'static str,
) -> Result<String, EntryParseError> {
    let value = mapping
        .remove(Value::String(field.to_owned()))
        .ok_or(EntryParseError::MissingField(field))?;
    match value {
        | Value::String(value) => Ok(value),
        | _ => Err(EntryParseError::FieldMustBeString(field)),
    }
}

fn take_optional_id_list(
    mapping: &mut Mapping, field: &'static str,
) -> Result<Vec<EntryId>, EntryParseError> {
    let Some(value) = mapping.remove(Value::String(field.to_owned())) else {
        return Ok(Vec::new());
    };
    let Value::Sequence(values) = value else {
        return Err(EntryParseError::FieldMustBeList(field));
    };

    values
        .into_iter()
        .map(|value| match value {
            | Value::String(raw) => EntryId::new(&raw).map_err(|source| {
                EntryParseError::InvalidStructuralId { field, value: raw, source }
            }),
            | _ => Err(EntryParseError::ListItemMustBeString(field)),
        })
        .collect()
}

fn take_witness_marker(
    mapping: &mut Mapping, canonical_witness: bool,
) -> Result<Option<WitnessMarker>, EntryParseError> {
    let Some(value) = mapping.remove(Value::String(WITNESS_FIELD.to_owned())) else {
        return Ok(None);
    };
    if value != Value::Null || !canonical_witness {
        return Err(EntryParseError::InvalidWitnessMarker);
    }
    Ok(Some(WitnessMarker::Present))
}

fn has_canonical_witness_marker(source: &str) -> bool {
    source.lines().any(|line| line.trim_end() == "witness:")
}

fn validate_plain_string(field: &'static str, value: &str) -> Result<(), EntryParseError> {
    if value.contains('\n') || value.contains('\r') {
        return Err(EntryParseError::FieldMustBePlainString(field));
    }
    Ok(())
}

fn render_id_list(out: &mut String, field: &str, values: &[EntryId]) {
    if values.is_empty() {
        return;
    }
    out.push_str(field);
    out.push_str(":\n");
    for id in values {
        out.push_str("  - ");
        out.push_str(id.as_str());
        out.push('\n');
    }
}

fn render_yaml_scalar(value: &str) -> Result<String, EntryRenderError> {
    let mut rendered = serde_yaml::to_string(value).map_err(EntryRenderError::Yaml)?;
    if let Some(stripped) = rendered.strip_suffix("\n...\n") {
        rendered = stripped.to_owned();
    }
    Ok(rendered.trim_end_matches('\n').to_owned())
}

/// Error raised when entry Markdown cannot be parsed into the Sirno model.
#[derive(Debug, Error)]
pub enum EntryParseError {
    /// The entry source does not start with a frontmatter block.
    #[error("entry is missing a YAML metadata block")]
    MissingFrontmatter,
    /// The entry source has an opening metadata block without a closing sentinel.
    #[error("entry metadata block is not closed")]
    UnterminatedFrontmatter,
    /// YAML metadata failed to parse.
    #[error("invalid YAML metadata: {0}")]
    Yaml(serde_yaml::Error),
    /// The YAML metadata root must be a mapping.
    #[error("entry metadata must be a mapping")]
    MetadataMustBeMapping,
    /// Metadata keys must be strings.
    #[error("entry metadata keys must be strings")]
    MetadataKeyMustBeString,
    /// A required field is absent.
    #[error("missing required metadata field `{0}`")]
    MissingField(&'static str),
    /// A required string field has another YAML type.
    #[error("metadata field `{0}` must be a string")]
    FieldMustBeString(&'static str),
    /// A string field is not a single-line plain string.
    #[error("metadata field `{0}` must be a single-line plain string")]
    FieldMustBePlainString(&'static str),
    /// A structural field is not a YAML list.
    #[error("metadata field `{0}` must be a list")]
    FieldMustBeList(&'static str),
    /// A structural list item is not a string.
    #[error("items in metadata field `{0}` must be strings")]
    ListItemMustBeString(&'static str),
    /// A structural field item is not a valid entry id.
    #[error("metadata field `{field}` contains invalid entry id `{value}`")]
    InvalidStructuralId {
        /// Structural field containing the invalid id.
        field: &'static str,
        /// Invalid raw id.
        value: String,
        /// Entry id validation error.
        #[source]
        source: EntryIdError,
    },
    /// The metadata block contains a field outside Sirno's exact schema.
    #[error("unknown metadata field `{0}`")]
    UnknownField(String),
    /// The witness field is present with a value or noncanonical spelling.
    #[error("metadata field `witness` must be written as canonical marker `witness:`")]
    InvalidWitnessMarker,
}

/// Error raised when typed entry data cannot be rendered.
#[derive(Debug, Error)]
pub enum EntryRenderError {
    /// The entry metadata violates a plain-string invariant.
    #[error(transparent)]
    InvalidMetadata(#[from] EntryParseError),
    /// YAML scalar rendering failed.
    #[error("failed to render YAML scalar: {0}")]
    Yaml(serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_id() -> EntryId {
        EntryId::new("witness").unwrap()
    }

    #[test]
    fn parses_canonical_entry_metadata() {
        let source = "\
---
name: Witness
description: An entry whose claim is evidenced by repository artifacts.
category:
  - concept
witness:
---

Body.
";

        let entry = Entry::from_markdown(entry_id(), source).unwrap();
        assert_eq!(entry.metadata.name, "Witness");
        assert_eq!(entry.metadata.category, vec![EntryId::new("concept").unwrap()]);
        assert_eq!(entry.metadata.witness, Some(WitnessMarker::Present));
        assert_eq!(entry.body, "Body.\n");
    }

    #[test]
    fn rejects_scalar_structural_field() {
        let source = "\
---
name: Bad
description: Bad category.
category: concept
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();
        assert!(matches!(error, EntryParseError::FieldMustBeList("category")));
    }

    #[test]
    fn rejects_noncanonical_witness_value() {
        let source = "\
---
name: Bad
description: Bad witness.
witness: true
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();
        assert!(matches!(error, EntryParseError::InvalidWitnessMarker));
    }

    #[test]
    fn rejects_explicit_null_witness_value() {
        let source = "\
---
name: Bad
description: Bad witness.
witness: null
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();
        assert!(matches!(error, EntryParseError::InvalidWitnessMarker));
    }

    #[test]
    fn renders_canonical_witness_marker() {
        let mut metadata = EntryMetadata::new("Witness", "Repository evidence.").unwrap();
        metadata.category.push(EntryId::new("concept").unwrap());
        metadata.witness = Some(WitnessMarker::Present);
        let entry = Entry::new(entry_id(), metadata, "Body.\n");

        let rendered = entry.to_markdown().unwrap();
        assert!(rendered.contains("witness:\n"));
        assert!(!rendered.contains("witness: null"));
        assert!(!rendered.contains("witness: true"));
    }

    #[test]
    fn replaces_body_without_rewriting_frontmatter() {
        let source = "\
---
name: Old
description: Existing description.
---

Old body.
";

        let replaced = Entry::replace_markdown_body(source, "New body.\n").unwrap();

        assert!(
            replaced.starts_with("---\nname: Old\ndescription: Existing description.\n---\n\n")
        );
        assert!(replaced.ends_with("New body.\n"));
        assert!(!replaced.contains("Old body."));
    }
}
