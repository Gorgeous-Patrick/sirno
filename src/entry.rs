//! Sirno entry model and Markdown frontmatter syntax.
//!
//! An entry is the Sirno Lake unit of Sirno design storage.
//! The prose body carries design content.
//! The metadata block carries structure that tools read exactly.

use std::collections::BTreeSet;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use thiserror::Error;

use crate::identifier::{EntryAddress, EntryAddressError};
use crate::structural::{StructuralEdgeDirection, StructuralTideSettings};

pub const NAME_FIELD: &str = "name";
pub const DESC_FIELD: &str = "desc";
pub const META_FIELD: &str = "meta";
pub const FROZEN_FIELD: &str = "frozen";
pub const META_LAKE_FIELD: &str = "meta.lake";
pub const META_FROST_FIELD: &str = "meta.frost";

// sirno:witness:entry:begin
/// One Sirno entry.
///
/// Invariant: `id` is a valid entry address.
/// `metadata` contains typed entry metadata.
/// `body` is normal Markdown prose outside the metadata block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Lookup path for this entry.
    pub id: EntryAddress,
    /// Typed metadata read from the YAML block.
    pub metadata: EntryMetadata,
    /// Markdown body after the metadata block.
    pub body: String,
}
// sirno:witness:entry:end

impl Entry {
    /// Construct an entry from already typed parts.
    // sirno:witness:entry:begin
    pub fn new(
        id: impl Into<EntryAddress>, metadata: EntryMetadata, body: impl Into<String>,
    ) -> Self {
        Self { id: id.into(), metadata, body: body.into() }
    }
    // sirno:witness:entry:end

    /// Parse an entry from Markdown source with LF or CRLF line endings.
    ///
    /// Sirno accepts mixed line endings so tooling can still inspect the file.
    /// Lake checks warn when one file mixes LF and CRLF.
    // sirno:witness:entry:begin
    pub fn from_markdown(
        id: impl Into<EntryAddress>, source: &str,
    ) -> Result<Self, EntryParseError> {
        let (metadata_source, body) = split_frontmatter(source)?;
        let metadata = EntryMetadata::from_yaml_source(metadata_source)?;
        Ok(Self::new(id, metadata, body))
    }
    // sirno:witness:entry:end

    /// Render this entry to canonical Markdown source.
    // sirno:witness:entry:begin
    pub fn to_markdown(&self) -> Result<String, EntryRenderError> {
        Ok(format!("---\n{}---\n\n{}", self.metadata.to_yaml_source()?, self.body))
    }
    // sirno:witness:entry:end

    /// Replace the Markdown body in an existing entry source.
    ///
    /// The frontmatter region and its separator are preserved exactly.
    // sirno:witness:entry:begin
    pub fn replace_markdown_body(source: &str, body: &str) -> Result<String, EntryParseError> {
        let body_start = frontmatter_body_start(source)?;
        Ok(format!("{}{}", &source[..body_start], body))
    }
    // sirno:witness:entry:end

    /// Create the ordinary seed entries for a new Sirno Lake.
    ///
    /// The entries are normal entries.
    /// Later operations do not privilege them.
    pub fn default_seed_entries() -> Result<Vec<Self>, EntryParseError> {
        // sirno:witness:category:begin
        let category =
            EntryMetadata::new("Category", "An entry that other entries can be categorized by.")?;
        // sirno:witness:category:end

        // sirno:witness:meta:begin
        let meta = EntryMetadata::new(
            "Meta",
            "An entry that defines the project's principles, vocabulary, and documentation method.",
        )?;
        // sirno:witness:meta:end

        // sirno:witness:concept:begin
        let concept =
            EntryMetadata::new("Concept", "A named idea that compresses project knowledge.")?;
        // sirno:witness:concept:end

        // sirno:witness:narrative:begin
        let narrative = EntryMetadata::new("Narrative", "A route through concepts for a reader.")?;
        // sirno:witness:narrative:end

        Ok(vec![
            Self::new(
                seed_id("category"),
                category,
                "Categorize an entry by this entry to use it as a category target.\n",
            ),
            Self::new(
                seed_id("meta"),
                meta,
                "Defines how this project should be understood and developed.\n",
            ),
            Self::new(
                seed_id("concept"),
                concept,
                "A concept gives a stable name to compressed project knowledge.\n",
            ),
            Self::new(
                seed_id("narrative"),
                narrative,
                "A narrative records an order in which a reader can understand concepts.\n",
            ),
        ])
    }
}

/// Ordered structural link metadata for one entry.
pub type EntryStructuralFields = IndexMap<String, Vec<EntryAddress>>;

/// Metadata for one Sirno entry.
///
/// Invariant: `name` and `desc` are single-line plain strings.
/// `meta` carries Sirno-managed optional metadata.
/// Structural links map relation names to entry-path targets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryMetadata {
    /// Human-readable entry name.
    pub name: String,
    /// Short prose summary of the entry.
    pub desc: String,
    /// Sirno-managed metadata fields.
    pub meta: EntryMeta,
    // sirno:witness:structural:begin
    /// Structural link targets keyed by their Markdown metadata field name.
    ///
    /// Relation order follows the user-authored metadata order and is preserved when entries move
    /// through other storage forms.
    pub structural: EntryStructuralFields,
    // sirno:witness:structural:end
}

impl EntryMetadata {
    /// Construct metadata with required fields and no structural link values.
    // sirno:witness:metadata:begin
    pub fn new(name: impl Into<String>, desc: impl Into<String>) -> Result<Self, EntryParseError> {
        let name = name.into();
        let desc = desc.into();
        validate_plain_string(NAME_FIELD, &name)?;
        validate_plain_string(DESC_FIELD, &desc)?;
        Ok(Self {
            name,
            desc,
            meta: EntryMeta::default(),
            structural: EntryStructuralFields::new(),
        })
    }
    // sirno:witness:metadata:end

    /// Parse metadata from YAML source without surrounding `---` sentinels.
    // sirno:witness:metadata:begin
    pub fn from_yaml_source(source: &str) -> Result<Self, EntryParseError> {
        let value: Value = serde_yaml::from_str(source).map_err(EntryParseError::Yaml)?;
        let mut mapping = match value {
            | Value::Mapping(mapping) => mapping,
            | _ => return Err(EntryParseError::MetadataMustBeMapping),
        };

        let name = take_required_string(&mut mapping, NAME_FIELD)?;
        let desc = take_required_string(&mut mapping, DESC_FIELD)?;
        validate_plain_string(NAME_FIELD, &name)?;
        validate_plain_string(DESC_FIELD, &desc)?;

        if mapping.contains_key(Value::String(FROZEN_FIELD.to_owned())) {
            return Err(EntryParseError::TopLevelFrozenMarker);
        }
        let mut meta = take_entry_meta(&mut mapping)?;
        meta.tide = take_flat_structural_tide_settings(&mut mapping)?;
        let structural = take_structural_fields(mapping)?;

        Ok(Self { name, desc, meta, structural })
    }
    // sirno:witness:metadata:end

    /// Render this metadata block to canonical YAML source.
    // sirno:witness:metadata:begin
    pub fn to_yaml_source(&self) -> Result<String, EntryRenderError> {
        validate_plain_string(NAME_FIELD, &self.name)?;
        validate_plain_string(DESC_FIELD, &self.desc)?;

        let mut out = String::new();
        out.push_str(&format!("name: {}\n", render_yaml_scalar(&self.name)?));
        out.push_str(&format!("desc: {}\n", render_yaml_scalar(&self.desc)?));
        render_entry_meta(&mut out, &self.meta);
        render_structural_fields(&mut out, &self.structural)?;
        Ok(out)
    }
    // sirno:witness:metadata:end

    /// Returns every entry address mentioned by structural links.
    // sirno:witness:metadata:begin
    pub fn structural_targets(&self) -> impl Iterator<Item = (&str, &EntryAddress)> {
        self.structural
            .iter()
            .flat_map(|(field, targets)| targets.iter().map(move |id| (field.as_str(), id)))
    }
    // sirno:witness:metadata:end

    /// Return link relation names and their targets in user-authored order.
    pub fn structural_fields(&self) -> impl Iterator<Item = (&str, &[EntryAddress])> {
        self.structural.iter().map(|(field, targets)| (field.as_str(), targets.as_slice()))
    }

    /// Return targets for one link relation.
    pub fn structural_targets_for(&self, field: &str) -> &[EntryAddress] {
        self.structural.get(field).map(Vec::as_slice).unwrap_or_default()
    }

    /// Return targets for one link relation while preserving field presence.
    ///
    /// `None` means the field is absent.
    /// `Some([])` means the field is present and has no targets.
    pub fn structural_field(&self, field: &str) -> Option<&[EntryAddress]> {
        self.structural.get(field).map(Vec::as_slice)
    }

    /// Return a mutable target list for one link relation.
    pub fn structural_targets_for_mut(
        &mut self, field: impl Into<String>,
    ) -> &mut Vec<EntryAddress> {
        self.structural.entry(field.into()).or_default()
    }

    /// Set the targets for one link relation.
    ///
    /// An empty target list records a present empty field.
    pub fn set_structural_targets(
        &mut self, field: impl Into<String>, targets: impl IntoIterator<Item = EntryAddress>,
    ) {
        self.structural.insert(field.into(), targets.into_iter().collect::<Vec<_>>());
    }

    /// Add one target to one link relation.
    pub fn push_structural_target(
        &mut self, field: impl Into<String>, target: impl Into<EntryAddress>,
    ) {
        self.structural_targets_for_mut(field).push(target.into());
    }

    /// Rename every structural link target that matches `old_id`.
    pub fn rename_structural_target(
        &mut self, old_id: &EntryAddress, new_id: &EntryAddress,
    ) -> bool {
        let mut changed = false;
        for targets in self.structural.values_mut() {
            for target in targets {
                if target == old_id {
                    *target = new_id.clone();
                    changed = true;
                }
            }
        }
        changed
    }

    /// Rename one structural link relation.
    ///
    /// The field stays in its original order position.
    pub fn rename_structural_field(
        &mut self, old_id: &EntryAddress, new_id: &EntryAddress,
    ) -> bool {
        let old_field = old_id.as_str();
        if !self.structural.contains_key(old_field) {
            return false;
        }

        let mut renamed = EntryStructuralFields::with_capacity(self.structural.len());
        for (field, targets) in std::mem::take(&mut self.structural) {
            if field == old_field {
                renamed.insert(new_id.as_str().to_owned(), targets);
            } else {
                renamed.insert(field, targets);
            }
        }
        self.structural = renamed;
        true
    }
}

/// Sirno-managed metadata for one entry.
///
/// Empty managed metadata is omitted from rendered entry frontmatter.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryMeta {
    /// Freeze reasons declaring that this Sirno Lake entry file is protected.
    pub frozen: Option<FrozenMarker>,
    /// Tide policy for a structural relation defined by this entry.
    pub tide: Option<StructuralTideSettings>,
}

impl EntryMeta {
    /// Return whether no managed metadata is set.
    pub fn is_empty(&self) -> bool {
        self.frozen.is_none() && self.tide.is_none()
    }
}

/// Protection reasons stored in the canonical `meta.frozen` metadata field.
///
/// Invariant: the reason set is non-empty.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenMarker {
    reasons: BTreeSet<FrozenReason>,
}

impl FrozenMarker {
    /// Construct a marker for a reviewed entry.
    pub fn reviewed() -> Self {
        Self::from_reason(FrozenReason::Reviewed)
    }

    /// Construct a marker for a managed entry.
    pub fn managed() -> Self {
        Self::from_reason(FrozenReason::Managed)
    }

    /// Return whether this marker includes `reviewed`.
    pub fn is_reviewed(&self) -> bool {
        self.reasons.contains(&FrozenReason::Reviewed)
    }

    /// Return whether this marker includes `managed`.
    pub fn is_managed(&self) -> bool {
        self.reasons.contains(&FrozenReason::Managed)
    }

    /// Add `reviewed` to this marker.
    pub fn insert_reviewed(&mut self) {
        self.reasons.insert(FrozenReason::Reviewed);
    }

    /// Add `managed` to this marker.
    pub fn insert_managed(&mut self) {
        self.reasons.insert(FrozenReason::Managed);
    }

    /// Remove `reviewed` from this marker.
    ///
    /// Returns true when at least one protection reason remains.
    pub fn remove_reviewed(&mut self) -> bool {
        self.reasons.remove(&FrozenReason::Reviewed);
        !self.reasons.is_empty()
    }

    /// Iterate over reasons in canonical render order.
    pub fn reasons(&self) -> impl Iterator<Item = FrozenReason> + '_ {
        [FrozenReason::Reviewed, FrozenReason::Managed]
            .into_iter()
            .filter(|reason| self.reasons.contains(reason))
    }

    fn from_reason(reason: FrozenReason) -> Self {
        let mut reasons = BTreeSet::new();
        reasons.insert(reason);
        Self { reasons }
    }
}

/// One reason why a Sirno entry is protected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FrozenReason {
    /// The entry matches the current frost snapshot.
    Reviewed,
    /// The entry is owned by crystallization.
    Managed,
}

fn seed_id(raw: &str) -> EntryAddress {
    EntryAddress::new(raw)
        .unwrap_or_else(|error| panic!("invalid built-in seed path `{raw}`: {error}"))
}

/// Byte ranges for one Markdown frontmatter block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrontmatterBounds {
    /// First byte after the opening fence line.
    metadata_start: usize,
    /// First byte after metadata text, before the closing fence line.
    metadata_end: usize,
    /// First byte of the Markdown body after the optional blank separator.
    body_start: usize,
}

impl FrontmatterBounds {
    /// Locate the frontmatter block while preserving source byte offsets.
    fn parse(source: &str) -> Result<Self, EntryParseError> {
        let opening_line =
            source.split_inclusive('\n').next().ok_or(EntryParseError::MissingFrontmatter)?;
        if !opening_line.ends_with('\n') || line_text(opening_line) != "---" {
            return Err(EntryParseError::MissingFrontmatter);
        }

        let metadata_start = opening_line.len();
        let mut metadata_end = metadata_start;
        let mut cursor = metadata_start;

        for line in source[metadata_start..].split_inclusive('\n') {
            if line.ends_with('\n') && line_text(line) == "---" {
                let body_start =
                    cursor + line.len() + line_break_len_at(&source[cursor + line.len()..]);
                return Ok(Self { metadata_start, metadata_end, body_start });
            }

            if !line.ends_with('\n') {
                break;
            }

            metadata_end = cursor + line.len() - line_ending_len(line);
            cursor += line.len();
        }

        Err(EntryParseError::UnterminatedFrontmatter)
    }
}

fn split_frontmatter(source: &str) -> Result<(&str, String), EntryParseError> {
    let bounds = FrontmatterBounds::parse(source)?;
    Ok((bounds.metadata(source), bounds.body(source).to_owned()))
}

fn frontmatter_body_start(source: &str) -> Result<usize, EntryParseError> {
    Ok(FrontmatterBounds::parse(source)?.body_start)
}

impl FrontmatterBounds {
    fn metadata(self, source: &str) -> &str {
        &source[self.metadata_start..self.metadata_end]
    }

    fn body(self, source: &str) -> &str {
        &source[self.body_start..]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineEnding {
    Lf,
    Crlf,
}

/// Return true when one source uses both LF-only and CRLF line endings.
pub(crate) fn has_mixed_line_endings(source: &str) -> bool {
    let mut first = None;
    for line in source.split_inclusive('\n') {
        let Some(ending) = line_ending(line) else {
            continue;
        };
        if let Some(first) = first {
            if first != ending {
                return true;
            }
        } else {
            first = Some(ending);
        }
    }
    false
}

fn line_text(line: &str) -> &str {
    match line_ending(line) {
        | Some(LineEnding::Crlf) => &line[..line.len() - "\r\n".len()],
        | Some(LineEnding::Lf) => &line[..line.len() - "\n".len()],
        | None => line,
    }
}

fn line_ending_len(line: &str) -> usize {
    match line_ending(line) {
        | Some(LineEnding::Crlf) => "\r\n".len(),
        | Some(LineEnding::Lf) => "\n".len(),
        | None => 0,
    }
}

fn line_break_len_at(source: &str) -> usize {
    if source.starts_with("\r\n") {
        "\r\n".len()
    } else if source.starts_with('\n') {
        "\n".len()
    } else {
        0
    }
}

fn line_ending(line: &str) -> Option<LineEnding> {
    if line.ends_with("\r\n") {
        Some(LineEnding::Crlf)
    } else if line.ends_with('\n') {
        Some(LineEnding::Lf)
    } else {
        None
    }
}

fn take_required_string(
    mapping: &mut Mapping, field: &'static str,
) -> Result<String, EntryParseError> {
    let value = mapping
        .shift_remove(Value::String(field.to_owned()))
        .ok_or(EntryParseError::MissingField(field))?;
    match value {
        | Value::String(value) => Ok(value),
        | _ => Err(EntryParseError::FieldMustBeString(field)),
    }
}

fn take_structural_fields(mapping: Mapping) -> Result<EntryStructuralFields, EntryParseError> {
    let mut structural = EntryStructuralFields::new();
    for (key, value) in mapping {
        let Value::String(field) = key else {
            return Err(EntryParseError::MetadataKeyMustBeString);
        };
        structural.insert(field.clone(), parse_id_list(field, value)?);
    }
    Ok(structural)
}

fn take_entry_meta(mapping: &mut Mapping) -> Result<EntryMeta, EntryParseError> {
    let Some(value) = mapping.shift_remove(Value::String(META_FIELD.to_owned())) else {
        return Ok(EntryMeta::default());
    };
    let Value::Mapping(mut meta_mapping) = value else {
        return Err(EntryParseError::MetaMustBeMapping);
    };

    let frozen = take_meta_frozen_marker(&mut meta_mapping)?;
    if let Some((key, _)) = meta_mapping.into_iter().next() {
        let Value::String(field) = key else {
            return Err(EntryParseError::MetaKeyMustBeString);
        };
        return Err(EntryParseError::UnknownMetaField(field));
    }

    Ok(EntryMeta { frozen, tide: None })
}

fn parse_id_list(field: String, value: Value) -> Result<Vec<EntryAddress>, EntryParseError> {
    let Value::Sequence(values) = value else {
        return Err(EntryParseError::FieldMustBeList(field));
    };

    values
        .into_iter()
        .map(|value| match value {
            | Value::String(raw) => EntryAddress::new(&raw).map_err(|source| {
                EntryParseError::InvalidStructuralId { field: field.clone(), value: raw, source }
            }),
            | _ => Err(EntryParseError::ListItemMustBeString(field.clone())),
        })
        .collect()
}

fn take_meta_frozen_marker(mapping: &mut Mapping) -> Result<Option<FrozenMarker>, EntryParseError> {
    let Some(value) = mapping.shift_remove(Value::String(FROZEN_FIELD.to_owned())) else {
        return Ok(None);
    };
    parse_frozen_marker_value(value).map(Some)
}

fn take_flat_structural_tide_settings(
    mapping: &mut Mapping,
) -> Result<Option<StructuralTideSettings>, EntryParseError> {
    let keys = mapping
        .keys()
        .filter_map(|key| match key {
            | Value::String(field) if field.starts_with("meta.") => Some(field.clone()),
            | _ => None,
        })
        .collect::<Vec<_>>();
    let mut settings = None;
    for field in keys {
        let value = mapping
            .shift_remove(Value::String(field.clone()))
            .expect("metadata key was collected from this mapping");
        parse_flat_structural_tide_field(&mut settings, &field, value)?;
    }
    Ok(settings)
}

fn parse_flat_structural_tide_field(
    settings: &mut Option<StructuralTideSettings>, field: &str, value: Value,
) -> Result<(), EntryParseError> {
    let meta_field =
        field.strip_prefix("meta.").expect("flat structural tide field has meta prefix");
    let parts = meta_field.split('.').collect::<Vec<_>>();
    match parts.as_slice() {
        | ["lake"] | ["frost"] => {
            if parse_tide_bool_field(field, value)? {
                return Err(EntryParseError::InvalidStructuralTideField(field.to_owned()));
            }
            settings.get_or_insert_with(StructuralTideSettings::default);
            Ok(())
        }
        | [line @ ("lake" | "frost"), direction] => {
            let enabled = parse_tide_bool_field(field, value)?;
            let direction = direction
                .parse::<StructuralEdgeDirection>()
                .map_err(|_| EntryParseError::UnknownMetaField(meta_field.to_owned()))?;
            set_structural_tide_setting(
                settings.get_or_insert_with(StructuralTideSettings::default),
                line,
                direction,
                enabled,
            );
            Ok(())
        }
        | _ => Err(EntryParseError::UnknownMetaField(meta_field.to_owned())),
    }
}

fn parse_tide_bool_field(field: &str, value: Value) -> Result<bool, EntryParseError> {
    match value {
        | Value::Bool(enabled) => Ok(enabled),
        | _ => Err(EntryParseError::InvalidStructuralTideField(field.to_owned())),
    }
}

fn set_structural_tide_setting(
    settings: &mut StructuralTideSettings, line: &str, direction: StructuralEdgeDirection,
    enabled: bool,
) {
    let ripple = match direction {
        | StructuralEdgeDirection::To => &mut settings.to,
        | StructuralEdgeDirection::From => &mut settings.from,
        | StructuralEdgeDirection::Clique => &mut settings.clique,
    };
    match line {
        | "lake" => ripple.lake = enabled,
        | "frost" => ripple.frost = enabled,
        | _ => unreachable!("line was parsed before setting tide value"),
    }
}

fn parse_frozen_marker_value(value: Value) -> Result<FrozenMarker, EntryParseError> {
    let Value::Sequence(values) = value else {
        return Err(EntryParseError::InvalidFrozenMarker);
    };
    if values.is_empty() {
        return Err(EntryParseError::InvalidFrozenMarker);
    }

    let mut reasons = BTreeSet::new();
    for value in values {
        let Value::String(raw) = value else {
            return Err(EntryParseError::InvalidFrozenMarker);
        };
        let reason = match raw.as_str() {
            | "reviewed" => FrozenReason::Reviewed,
            | "managed" => FrozenReason::Managed,
            | _ => return Err(EntryParseError::InvalidFrozenMarker),
        };
        if !reasons.insert(reason) {
            return Err(EntryParseError::InvalidFrozenMarker);
        }
    }
    Ok(FrozenMarker { reasons })
}

fn validate_plain_string(field: &'static str, value: &str) -> Result<(), EntryParseError> {
    if value.contains('\n') || value.contains('\r') {
        return Err(EntryParseError::FieldMustBePlainString(field));
    }
    Ok(())
}

fn render_id_list(
    out: &mut String, field: &str, values: &[EntryAddress],
) -> Result<(), EntryRenderError> {
    if values.is_empty() {
        out.push_str(field);
        out.push_str(": []\n");
        return Ok(());
    }
    out.push_str(field);
    out.push_str(":\n");
    for id in values {
        out.push_str("  - ");
        out.push_str(&render_yaml_scalar(id.as_str())?);
        out.push('\n');
    }
    Ok(())
}

fn render_structural_fields(
    out: &mut String, structural: &EntryStructuralFields,
) -> Result<(), EntryRenderError> {
    for (field, values) in structural {
        render_id_list(out, field, values)?;
    }
    Ok(())
}

fn render_entry_meta(out: &mut String, meta: &EntryMeta) {
    if meta.is_empty() {
        return;
    }

    if let Some(marker) = &meta.frozen {
        out.push_str("meta:\n");
        render_meta_frozen_marker(out, marker);
    }
    if let Some(tide) = &meta.tide {
        render_flat_structural_tide_settings(out, tide);
    }
}

fn render_meta_frozen_marker(out: &mut String, marker: &FrozenMarker) {
    out.push_str("  frozen:\n");
    for reason in marker.reasons() {
        out.push_str("    - ");
        out.push_str(match reason {
            | FrozenReason::Reviewed => "reviewed",
            | FrozenReason::Managed => "managed",
        });
        out.push('\n');
    }
}

fn render_flat_structural_tide_settings(out: &mut String, settings: &StructuralTideSettings) {
    if settings.is_empty() {
        out.push_str("meta.lake: false\n");
        out.push_str("meta.frost: false\n");
        return;
    }
    render_flat_structural_tide_line(
        out,
        "lake",
        settings.to.lake,
        settings.from.lake,
        settings.clique.lake,
    );
    render_flat_structural_tide_line(
        out,
        "frost",
        settings.to.frost,
        settings.from.frost,
        settings.clique.frost,
    );
}

fn render_flat_structural_tide_line(
    out: &mut String, line: &str, to: bool, from: bool, clique: bool,
) {
    for (direction, enabled) in [("to", to), ("from", from), ("clique", clique)] {
        if enabled {
            out.push_str("meta.");
            out.push_str(line);
            out.push('.');
            out.push_str(direction);
            out.push_str(": true\n");
        }
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
    /// The managed meta field must be a YAML mapping.
    #[error("metadata field `meta` must be a mapping")]
    MetaMustBeMapping,
    /// Managed meta keys must be strings.
    #[error("metadata field `meta` keys must be strings")]
    MetaKeyMustBeString,
    /// A managed meta field is not supported.
    #[error("unknown Sirno-managed metadata field `meta.{0}`")]
    UnknownMetaField(String),
    /// A required field is absent.
    #[error("missing required metadata field `{0}`")]
    MissingField(&'static str),
    /// A required string field has another YAML type.
    #[error("metadata field `{0}` must be a string")]
    FieldMustBeString(&'static str),
    /// A string field is not a single-line plain string.
    #[error("metadata field `{0}` must be a single-line plain string")]
    FieldMustBePlainString(&'static str),
    /// A structural link relation is not a YAML list.
    #[error("metadata field `{0}` must be a list")]
    FieldMustBeList(String),
    /// A structural list item is not a string.
    #[error("items in metadata field `{0}` must be strings")]
    ListItemMustBeString(String),
    /// A structural link target is not a valid entry address.
    #[error("metadata field `{field}` contains invalid entry address `{value}`")]
    InvalidStructuralId {
        /// Link relation containing the invalid path.
        field: String,
        /// Invalid raw path.
        value: String,
        /// Entry address validation error.
        #[source]
        source: EntryAddressError,
    },
    /// The frozen field is present with invalid protection reasons.
    #[error("metadata field `meta.frozen` must be a non-empty list of reviewed or managed reasons")]
    InvalidFrozenMarker,
    /// A flat tide metadata field has an invalid value.
    #[error("metadata field `{0}` must be a boolean tide setting")]
    InvalidStructuralTideField(String),
    /// The old top-level frozen field is no longer valid.
    #[error("metadata field `frozen` moved to `meta.frozen`")]
    TopLevelFrozenMarker,
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

    fn entry_id() -> EntryAddress {
        EntryAddress::new("witness").unwrap()
    }

    #[test]
    fn parses_canonical_entry_metadata() {
        let source = "\
---
name: Witness
desc: An entry whose claim is evidenced by repository artifacts.
topic:
  - concept
---

Body.
";

        let entry = Entry::from_markdown(entry_id(), source).unwrap();
        assert_eq!(entry.metadata.name, "Witness");
        assert_eq!(
            entry.metadata.structural_targets_for("topic"),
            &[EntryAddress::new("concept").unwrap()]
        );
        assert_eq!(entry.body, "Body.\n");
    }

    #[test]
    fn parses_crlf_entry_metadata() {
        let source = concat!(
            "---\r\n",
            "name: Witness\r\n",
            "desc: An entry whose claim is evidenced by repository artifacts.\r\n",
            "topic:\r\n",
            "  - concept\r\n",
            "---\r\n",
            "\r\n",
            "Body.\r\n",
        );

        let entry = Entry::from_markdown(entry_id(), source).unwrap();

        assert_eq!(entry.metadata.name, "Witness");
        assert_eq!(
            entry.metadata.structural_targets_for("topic"),
            &[EntryAddress::new("concept").unwrap()]
        );
        assert_eq!(entry.body, "Body.\r\n");
    }

    #[test]
    fn rejects_scalar_structural_link_relation() {
        let source = "\
---
name: Bad
desc: Bad structural metadata.
topic: concept
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();
        assert!(matches!(error, EntryParseError::FieldMustBeList(field) if field == "topic"));
    }

    #[test]
    fn parses_extra_list_metadata_as_structural_link_relation() {
        let source = "\
---
name: Evidence
desc: Metadata with a project-defined structural link relation.
witness:
  - repository-evidence
---
";

        let entry = Entry::from_markdown(entry_id(), source).unwrap();

        assert_eq!(
            entry.metadata.structural_targets_for("witness"),
            &[EntryAddress::new("repository-evidence").unwrap()]
        );
    }

    #[test]
    fn preserves_structural_link_relation_order_when_rendering() {
        let source = "\
---
name: Ordered
desc: Metadata with user-authored structural link relation order.
zeta:
  - concept
alpha:
  - meta
---

Body.
";

        let entry = Entry::from_markdown(entry_id(), source).unwrap();
        let fields = entry.metadata.structural_fields().map(|(field, _)| field).collect::<Vec<_>>();
        let rendered = entry.to_markdown().unwrap();

        assert_eq!(fields, ["zeta", "alpha"]);
        assert!(rendered.find("zeta:\n").unwrap() < rendered.find("alpha:\n").unwrap());
    }

    #[test]
    fn preserves_present_empty_structural_link_relation_when_rendering() {
        let source = "\
---
name: Empty Field
desc: Metadata with a present empty structural link relation.
topic: []
---

Body.
";

        let entry = Entry::from_markdown(entry_id(), source).unwrap();
        let rendered = entry.to_markdown().unwrap();
        let reparsed = Entry::from_markdown(entry_id(), &rendered).unwrap();

        assert!(
            matches!(entry.metadata.structural_field("topic"), Some(targets) if targets.is_empty())
        );
        assert!(
            matches!(reparsed.metadata.structural_field("topic"), Some(targets) if targets.is_empty())
        );
        assert!(rendered.contains("topic: []\n"));
    }

    #[test]
    fn renders_structural_ids_as_yaml_scalars() {
        let target = EntryAddress::new("Design Note #1").unwrap();
        let mut metadata =
            EntryMetadata::new("Evidence", "Metadata with a quoted target.").unwrap();
        metadata.push_structural_target("witness", target.clone());
        let entry = Entry::new(entry_id(), metadata, "Body.\n");

        let rendered = entry.to_markdown().unwrap();
        let reparsed = Entry::from_markdown(entry_id(), &rendered).unwrap();

        assert_eq!(reparsed.metadata.structural_targets_for("witness"), &[target]);
    }

    #[test]
    fn renames_structural_targets() {
        let old_id = EntryAddress::new("old-entry").unwrap();
        let new_id = EntryAddress::new("new-entry").unwrap();
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target("belongs", old_id.clone());
        metadata.push_structural_target("belongs", EntryAddress::new("other-entry").unwrap());
        metadata.push_structural_target("refines", old_id.clone());

        assert!(metadata.rename_structural_target(&old_id, &new_id));

        assert_eq!(
            metadata.structural_targets_for("belongs"),
            &[new_id.clone(), EntryAddress::new("other-entry").unwrap()]
        );
        assert_eq!(metadata.structural_targets_for("refines"), &[new_id]);
    }

    #[test]
    fn renames_structural_fields() {
        let old_id = EntryAddress::new("refines").unwrap();
        let new_id = EntryAddress::new("prerequisite").unwrap();
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target("category", EntryAddress::new("concept").unwrap());
        metadata.push_structural_target("refines", EntryAddress::new("broader").unwrap());
        metadata.push_structural_target("belongs", EntryAddress::new("area").unwrap());

        assert!(metadata.rename_structural_field(&old_id, &new_id));

        let fields = metadata.structural_fields().map(|(field, _)| field).collect::<Vec<_>>();
        assert_eq!(fields, ["category", "prerequisite", "belongs"]);
        assert_eq!(
            metadata.structural_targets_for("prerequisite"),
            &[EntryAddress::new("broader").unwrap()]
        );
        assert!(metadata.structural_field("refines").is_none());
    }

    #[test]
    fn parses_canonical_frozen_marker() {
        let source = "\
---
name: Frozen
desc: A protected entry.
meta:
  frozen:
    - reviewed
---

Body.
";

        let entry = Entry::from_markdown(entry_id(), source).unwrap();

        assert_eq!(entry.metadata.meta.frozen, Some(FrozenMarker::reviewed()));
        assert!(entry.metadata.meta.frozen.as_ref().unwrap().is_reviewed());
    }

    #[test]
    fn parses_entry_without_managed_meta() {
        let source = "\
---
name: Plain
desc: No managed metadata.
topic:
  - concept
---

Body.
";

        let entry = Entry::from_markdown(entry_id(), source).unwrap();

        assert!(entry.metadata.meta.is_empty());
        assert_eq!(
            entry.metadata.structural_targets_for("topic"),
            &[EntryAddress::new("concept").unwrap()]
        );
    }

    #[test]
    fn rejects_top_level_frozen_marker() {
        let source = "\
---
name: Old
desc: Old frozen marker.
frozen:
  - reviewed
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();

        assert!(matches!(error, EntryParseError::TopLevelFrozenMarker));
    }

    #[test]
    fn rejects_noncanonical_frozen_value() {
        let source = "\
---
name: Bad
desc: Bad frozen marker.
meta:
  frozen: true
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();

        assert!(matches!(error, EntryParseError::InvalidFrozenMarker));
    }

    #[test]
    fn rejects_explicit_null_frozen_value() {
        let source = "\
---
name: Bad
desc: Bad frozen marker.
meta:
  frozen: null
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();

        assert!(matches!(error, EntryParseError::InvalidFrozenMarker));
    }

    #[test]
    fn parses_managed_frozen_reason() {
        let source = "\
---
name: Managed
desc: A managed entry.
meta:
  frozen:
    - reviewed
    - managed
---

Body.
";

        let entry = Entry::from_markdown(entry_id(), source).unwrap();
        let frozen = entry.metadata.meta.frozen.as_ref().unwrap();

        assert!(frozen.is_reviewed());
        assert!(frozen.is_managed());
    }

    #[test]
    fn parses_structural_tide_settings() {
        let source = "\
---
name: Belongs
desc: A structural relation.
meta.lake.to: true
meta.lake.from: true
meta.lake.clique: true
meta.frost.from: true
---

Body.
";

        let entry = Entry::from_markdown(entry_id(), source).unwrap();
        let structural = entry.metadata.meta.tide.unwrap();

        assert_eq!(
            structural,
            StructuralTideSettings::new(
                crate::structural::StructuralRippleSettings::new(true, false),
                crate::structural::StructuralRippleSettings::new(true, true),
                crate::structural::StructuralRippleSettings::new(true, false),
            )
        );
    }

    #[test]
    fn renders_empty_structural_tide_settings() {
        let mut metadata = EntryMetadata::new("Category", "A structural relation.").unwrap();
        metadata.meta.tide = Some(StructuralTideSettings::default());
        let entry = Entry::new(entry_id(), metadata, "Body.\n");

        let rendered = entry.to_markdown().unwrap();
        let reparsed = Entry::from_markdown(entry_id(), &rendered).unwrap();

        assert!(rendered.contains("meta.lake: false\nmeta.frost: false\n"));
        assert_eq!(reparsed.metadata.meta.tide, Some(StructuralTideSettings::default()));
    }

    #[test]
    fn renders_structural_tide_settings() {
        let mut metadata = EntryMetadata::new("Belongs", "A structural relation.").unwrap();
        metadata.meta.tide = Some(StructuralTideSettings::new(
            crate::structural::StructuralRippleSettings::new(true, false),
            crate::structural::StructuralRippleSettings::new(true, true),
            crate::structural::StructuralRippleSettings::new(true, false),
        ));
        let entry = Entry::new(entry_id(), metadata, "Body.\n");

        let rendered = entry.to_markdown().unwrap();

        assert!(rendered.contains(
            "\
meta.lake.to: true
meta.lake.from: true
meta.lake.clique: true
meta.frost.from: true
"
        ));
    }

    #[test]
    fn renders_canonical_frozen_marker() {
        let mut metadata = EntryMetadata::new("Frozen", "Protected entry.").unwrap();
        metadata.meta.frozen = Some(FrozenMarker::reviewed());
        let entry = Entry::new(entry_id(), metadata, "Body.\n");

        let rendered = entry.to_markdown().unwrap();

        assert!(rendered.contains("meta:\n  frozen:\n    - reviewed\n"));
        assert!(
            rendered.find("desc: Protected entry.\n").unwrap() < rendered.find("meta:\n").unwrap()
        );
        assert!(!rendered.contains("frozen: null"));
        assert!(!rendered.contains("frozen: true"));
    }

    #[test]
    fn omits_empty_managed_meta_when_rendering() {
        let metadata = EntryMetadata::new("Plain", "No managed metadata.").unwrap();
        let entry = Entry::new(entry_id(), metadata, "Body.\n");

        let rendered = entry.to_markdown().unwrap();

        assert!(!rendered.contains("meta:\n"));
    }

    #[test]
    fn rejects_non_mapping_meta() {
        let source = "\
---
name: Bad
desc: Bad meta.
meta: true
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();

        assert!(matches!(error, EntryParseError::MetaMustBeMapping));
    }

    #[test]
    fn rejects_unknown_meta_field() {
        let source = "\
---
name: Bad
desc: Bad meta.
meta:
  owner: sirno
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();

        assert!(matches!(error, EntryParseError::UnknownMetaField(field) if field == "owner"));
    }

    #[test]
    fn rejects_old_structural_tide_block() {
        let source = "\
---
name: Bad
desc: Bad tide metadata.
meta:
  structural: {}
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();

        assert!(matches!(error, EntryParseError::UnknownMetaField(field) if field == "structural"));
    }

    #[test]
    fn rejects_non_boolean_flat_tide_field() {
        let source = "\
---
name: Bad
desc: Bad tide metadata.
meta.lake.to: \"yes\"
---
";

        let error = Entry::from_markdown(entry_id(), source).unwrap_err();

        assert!(
            matches!(error, EntryParseError::InvalidStructuralTideField(field) if field == "meta.lake.to")
        );
    }

    #[test]
    fn replaces_body_without_rewriting_frontmatter() {
        let source = "\
---
name: Old
desc: Existing desc.
---

Old body.
";

        let replaced = Entry::replace_markdown_body(source, "New body.\n").unwrap();

        assert!(replaced.starts_with("---\nname: Old\ndesc: Existing desc.\n---\n\n"));
        assert!(replaced.ends_with("New body.\n"));
        assert!(!replaced.contains("Old body."));
    }

    #[test]
    fn replaces_crlf_body_without_rewriting_frontmatter() {
        let source = "---\r\nname: Old\r\ndesc: Existing desc.\r\n---\r\n\r\nOld body.\r\n";

        let replaced = Entry::replace_markdown_body(source, "New body.\n").unwrap();

        assert!(replaced.starts_with("---\r\nname: Old\r\ndesc: Existing desc.\r\n---\r\n\r\n"));
        assert!(replaced.ends_with("New body.\n"));
        assert!(!replaced.contains("Old body."));
    }

    #[test]
    fn detects_mixed_line_endings() {
        assert!(!has_mixed_line_endings("---\nname: Entry\n---\n"));
        assert!(!has_mixed_line_endings("---\r\nname: Entry\r\n---\r\n"));
        assert!(has_mixed_line_endings("---\r\nname: Entry\n---\r\n"));
    }
}
