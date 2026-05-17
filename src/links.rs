//! Generated Markdown links for entries.
//!
//! Sirno owns only the guard-bounded generated-link region.
//! Prose outside the region remains user-owned.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write};
use std::str::FromStr;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entry::Entry;
use crate::id::EntryId;

fn is_false(value: &bool) -> bool {
    !*value
}

/// Configured ripple sources for one structural edge direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
// sirno:witness:structural-edge-policy:begin
pub struct StructuralRippleSettings {
    /// Include waterline neighbors in tide workitems.
    #[serde(skip_serializing_if = "is_false")]
    pub lake: bool,
    /// Include frostline neighbors in tide workitems.
    #[serde(skip_serializing_if = "is_false")]
    pub frost: bool,
}
// sirno:witness:structural-edge-policy:end

impl StructuralRippleSettings {
    /// Construct ripple settings from explicit source flags.
    pub fn new(lake: bool, frost: bool) -> Self {
        Self { lake, frost }
    }

    /// Returns true when no ripple source is enabled.
    pub fn is_empty(&self) -> bool {
        !self.lake && !self.frost
    }
}

/// Tooling settings for one structural edge direction.
///
/// `render` includes the edge in generated footers.
/// `ripple` includes the edge in tide workitem generation.
// sirno:witness:structural-edge-policy:begin
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StructuralEdgeSettings {
    /// Include this edge direction in generated footer rendering.
    #[serde(skip_serializing_if = "is_false")]
    pub render: bool,
    /// Include this edge direction in tide workitem generation.
    #[serde(skip_serializing_if = "StructuralRippleSettings::is_empty")]
    pub ripple: StructuralRippleSettings,
}
// sirno:witness:structural-edge-policy:end

// sirno:witness:structural-edge-policy:begin
impl StructuralEdgeSettings {
    /// Construct structural edge settings from explicit render and ripple settings.
    pub fn new(render: bool, ripple: StructuralRippleSettings) -> Self {
        Self { render, ripple }
    }

    /// Construct an edge used only for generated footer rendering.
    pub fn render_only(enabled: bool) -> Self {
        Self::new(enabled, StructuralRippleSettings::default())
    }

    /// Construct an edge used for rendering and selected ripple sources.
    pub fn render_and_ripple(render: bool, lake: bool, frost: bool) -> Self {
        Self::new(render, StructuralRippleSettings::new(lake, frost))
    }
}
// sirno:witness:structural-edge-policy:end

impl fmt::Display for StructuralEdgeSettings {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.render {
            parts.push("render=true");
        }
        if self.ripple.lake {
            parts.push("ripple.lake=true");
        }
        if self.ripple.frost {
            parts.push("ripple.frost=true");
        }
        if parts.is_empty() {
            write!(formatter, "none")
        } else {
            write!(formatter, "{}", parts.join(" "))
        }
    }
}

/// Direction of one configured structural edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralEdgeDirection {
    /// Outgoing metadata targets from the current entry.
    To,
    /// Incoming metadata sources that point at the current entry.
    From,
    /// Entries connected through a shared target in the same field.
    Clique,
}

impl StructuralEdgeDirection {
    /// Directions in deterministic generated-footer and tide order.
    pub const ORDER: [Self; 3] = [Self::To, Self::From, Self::Clique];

    /// Lowercase direction label.
    pub fn label(self) -> &'static str {
        match self {
            | Self::To => "to",
            | Self::From => "from",
            | Self::Clique => "clique",
        }
    }
}

impl fmt::Display for StructuralEdgeDirection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

impl FromStr for StructuralEdgeDirection {
    type Err = StructuralEdgeDirectionParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            | "to" => Ok(Self::To),
            | "from" => Ok(Self::From),
            | "clique" => Ok(Self::Clique),
            | direction => Err(StructuralEdgeDirectionParseError(direction.to_owned())),
        }
    }
}

/// Error raised when text does not name a structural edge direction.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown structural edge direction `{0}`; expected to, from, or clique")]
pub struct StructuralEdgeDirectionParseError(String);

/// Settings for one configured structural field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
// sirno:witness:structural-edge-policy:begin
pub struct StructuralFieldSettings {
    /// Outgoing metadata target edge policy.
    #[serde(skip_serializing_if = "is_default")]
    pub to: StructuralEdgeSettings,
    /// Incoming metadata source edge policy.
    #[serde(skip_serializing_if = "is_default")]
    pub from: StructuralEdgeSettings,
    /// Shared-target clique edge policy.
    #[serde(skip_serializing_if = "is_default")]
    pub clique: StructuralEdgeSettings,
}
// sirno:witness:structural-edge-policy:end

impl StructuralFieldSettings {
    /// Construct structural field settings from explicit edge policies.
    pub fn new(
        to: StructuralEdgeSettings, from: StructuralEdgeSettings, clique: StructuralEdgeSettings,
    ) -> Self {
        Self { to, from, clique }
    }

    /// Construct structural field settings from render-only edge flags.
    pub fn render_only(to: bool, from: bool, clique: bool) -> Self {
        Self::new(
            StructuralEdgeSettings::render_only(to),
            StructuralEdgeSettings::render_only(from),
            StructuralEdgeSettings::render_only(clique),
        )
    }

    /// Return settings for one structural edge direction.
    pub fn edge(&self, direction: StructuralEdgeDirection) -> &StructuralEdgeSettings {
        match direction {
            | StructuralEdgeDirection::To => &self.to,
            | StructuralEdgeDirection::From => &self.from,
            | StructuralEdgeDirection::Clique => &self.clique,
        }
    }
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

/// Ordered structural field settings from `Sirno.toml`.
pub type StructuralFieldMap = IndexMap<String, StructuralFieldSettings>;

/// Configured structural fields.
///
/// Each key names a metadata field that Sirno should treat as structural.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
// sirno:witness:structural-edge-policy:begin
pub struct StructuralSettings {
    fields: StructuralFieldMap,
}
// sirno:witness:structural-edge-policy:end

impl StructuralSettings {
    /// Construct structural settings from explicit field settings.
    pub fn from_fields(
        fields: impl IntoIterator<Item = (impl Into<String>, StructuralFieldSettings)>,
    ) -> Self {
        Self {
            fields: fields.into_iter().map(|(field, settings)| (field.into(), settings)).collect(),
        }
    }

    /// Iterate configured fields in user-authored order.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &StructuralFieldSettings)> {
        self.fields.iter().map(|(field, settings)| (field.as_str(), settings))
    }

    /// Return true when a metadata field is configured as structural.
    pub fn contains_field(&self, field: &str) -> bool {
        self.fields.contains_key(field)
    }

    /// Render the generated-link footer for one entry using only that entry as context.
    ///
    /// Use `GeneratedLinkIndex::from_entries` when clique expansion needs the full lake.
    // sirno:witness:generated-footer:begin
    pub fn render_entry(&self, entry: &Entry) -> String {
        GeneratedLinkIndex::from_entries(std::slice::from_ref(entry)).render_entry(entry, self)
    }
    // sirno:witness:generated-footer:end
}

/// Borrowed Markdown body whose generated-link footer can be inspected or changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeneratedLinkBody<'a> {
    body: &'a str,
}

impl<'a> GeneratedLinkBody<'a> {
    /// Borrow an entry body for generated-link operations.
    pub fn new(body: &'a str) -> Self {
        Self { body }
    }

    /// Validate generated-link guard boundaries.
    // sirno:witness:generated-footer-ownership:begin
    pub fn validate(&self) -> Result<(), GeneratedLinkError> {
        self.bounds().map(|_| ())
    }
    // sirno:witness:generated-footer-ownership:end

    /// Returns true when an existing generated-link region differs from `expected`.
    ///
    /// Bodies without a generated-link region are not stale.
    // sirno:witness:generated-footer:begin
    pub fn is_stale(&self, expected: &str) -> Result<bool, GeneratedLinkError> {
        let Some(bounds) = self.bounds()? else {
            return Ok(false);
        };
        Ok(&self.body[bounds.region_start..bounds.region_end] != expected)
    }
    // sirno:witness:generated-footer:end

    /// Apply generated links to an entry body.
    ///
    /// If no generated-link region exists, one is appended.
    /// If one valid generated-link region exists, only that region is replaced.
    // sirno:witness:generated-footer:begin
    pub fn apply(&self, footer: &str) -> Result<String, GeneratedLinkError> {
        let Some(bounds) = self.bounds()? else {
            return Ok(self.append_footer(footer));
        };
        let region_end = bounds.next_line_start(self.body);
        let before = self.body[..bounds.region_start].trim_end_matches('\n');
        let after = self.body[region_end..].trim_start_matches('\n');

        let mut out = String::new();
        if !before.is_empty() {
            out.push_str(before);
            out.push_str("\n\n");
        }
        out.push_str(footer);
        out.push('\n');
        if !after.is_empty() {
            out.push('\n');
            out.push_str(after);
        }
        Ok(out)
    }
    // sirno:witness:generated-footer:end

    /// Delete generated links from an entry body.
    ///
    /// If no generated-link region exists, the body is returned unchanged.
    // sirno:witness:generated-footer:begin
    pub fn delete(&self) -> Result<String, GeneratedLinkError> {
        let Some(bounds) = self.bounds()? else {
            return Ok(self.body.to_owned());
        };
        let region_end = bounds.next_line_start(self.body);
        let before = self.body[..bounds.region_start].trim_end_matches('\n');
        let after = self.body[region_end..].trim_start_matches('\n');

        let mut out = String::new();
        if !before.is_empty() {
            out.push_str(before);
        }
        if !before.is_empty() && !after.is_empty() {
            out.push_str("\n\n");
        }
        if !after.is_empty() {
            out.push_str(after);
        }
        if after.is_empty() && self.body.ends_with('\n') && !out.is_empty() {
            out.push('\n');
        }
        Ok(out)
    }
    // sirno:witness:generated-footer:end

    /// Replace the generated-link region with whitespace while preserving byte length and newlines.
    ///
    /// If no generated-link region exists, the body is returned unchanged.
    // sirno:witness:generated-footer:begin
    pub fn mask(&self) -> Result<String, GeneratedLinkError> {
        let Some(bounds) = self.bounds()? else {
            return Ok(self.body.to_owned());
        };
        let region_end = bounds.next_line_start(self.body);
        let body = self.body.as_bytes();
        let mut out = Vec::with_capacity(body.len());

        out.extend_from_slice(&body[..bounds.region_start]);
        for byte in &body[bounds.region_start..region_end] {
            out.push(if *byte == b'\n' { b'\n' } else { b' ' });
        }
        out.extend_from_slice(&body[region_end..]);

        Ok(String::from_utf8(out).expect("masked generated-link body remains UTF-8"))
    }
    // sirno:witness:generated-footer:end

    fn bounds(&self) -> Result<Option<GeneratedLinkBounds>, GeneratedLinkError> {
        GeneratedLinkBounds::find(self.body)
    }

    // sirno:witness:generated-footer:begin
    fn append_footer(&self, footer: &str) -> String {
        let before = self.body.trim_end_matches('\n');
        let mut out = String::new();
        if !before.is_empty() {
            out.push_str(before);
            out.push_str("\n\n");
            if !Self::ends_with_divider(before) {
                out.push_str(GENERATED_LINK_DIVIDER);
                out.push_str("\n\n");
            }
        }
        out.push_str(footer);
        out.push('\n');
        out
    }
    // sirno:witness:generated-footer:end

    fn ends_with_divider(body: &str) -> bool {
        body.lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .is_some_and(|line| line.trim() == GENERATED_LINK_DIVIDER)
    }
}

/// Lake-wide context for generated-link rendering.
///
/// Invariant: each clique target maps to itself and every parsed entry that names it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
// sirno:witness:generated-footer:begin
pub struct GeneratedLinkIndex {
    sources_by_field_target: BTreeMap<String, BTreeMap<EntryId, BTreeSet<EntryId>>>,
    cliques_by_field_target: BTreeMap<String, BTreeMap<EntryId, BTreeSet<EntryId>>>,
}
// sirno:witness:generated-footer:end

impl GeneratedLinkIndex {
    /// Construct a generated-link index from parsed entries.
    // sirno:witness:generated-footer:begin
    pub fn from_entries(entries: &[Entry]) -> Self {
        let mut sources_by_field_target =
            BTreeMap::<String, BTreeMap<EntryId, BTreeSet<EntryId>>>::new();
        let mut cliques_by_field_target =
            BTreeMap::<String, BTreeMap<EntryId, BTreeSet<EntryId>>>::new();
        for entry in entries {
            for (field, targets) in entry.metadata.structural_fields() {
                Self::insert_sources(
                    sources_by_field_target.entry(field.to_owned()).or_default(),
                    &entry.id,
                    targets,
                );
                Self::insert_cliques(
                    cliques_by_field_target.entry(field.to_owned()).or_default(),
                    &entry.id,
                    targets,
                );
            }
            // sirno:witness:generated-footer:end
            // sirno:witness:generated-footer:begin
        }
        Self { sources_by_field_target, cliques_by_field_target }
    }
    // sirno:witness:generated-footer:end

    /// Render the generated-link footer for one entry using this lake-wide index.
    pub fn render_entry(&self, entry: &Entry, settings: &StructuralSettings) -> String {
        // sirno:witness:generated-footer:begin
        let mut out = String::new();
        out.push_str(BEGIN_LINKS_GUARD);
        out.push_str("\n\n");
        // sirno:witness:generated-footer:end

        // sirno:witness:generated-footer:begin
        let mut sections = Vec::new();
        for (field, field_settings) in settings.fields() {
            for direction in StructuralEdgeDirection::ORDER {
                if field_settings.edge(direction).render {
                    sections.push(GeneratedLinkSection::new(
                        section_title(field, direction),
                        self.edge_targets(field, direction, entry),
                    ));
                }
            }
        }
        // sirno:witness:generated-footer:end

        // sirno:witness:generated-footer:begin
        if sections.is_empty() {
            out.push_str("(none)\n\n");
        } else {
            for section in &sections {
                section.render(&mut out);
            }
            out.push('\n');
        }

        out.push_str(END_LINKS_GUARD);
        out
        // sirno:witness:generated-footer:end
    }

    fn insert_sources(
        sources_by_target: &mut BTreeMap<EntryId, BTreeSet<EntryId>>, source: &EntryId,
        targets: &[EntryId],
    ) {
        for target in targets {
            sources_by_target.entry(target.clone()).or_default().insert(source.clone());
        }
    }

    fn insert_cliques(
        cliques_by_target: &mut BTreeMap<EntryId, BTreeSet<EntryId>>, source: &EntryId,
        targets: &[EntryId],
    ) {
        for target in targets {
            let clique = cliques_by_target.entry(target.clone()).or_default();
            clique.insert(target.clone());
            clique.insert(source.clone());
        }
    }

    /// Return target entries for one structural edge direction.
    pub fn edge_targets(
        &self, field: &str, direction: StructuralEdgeDirection, entry: &Entry,
    ) -> BTreeSet<EntryId> {
        match direction {
            | StructuralEdgeDirection::To => {
                entry.metadata.structural_targets_for(field).iter().cloned().collect()
            }
            | StructuralEdgeDirection::From => self.incoming_targets(field, entry),
            | StructuralEdgeDirection::Clique => self.clique_targets(field, entry),
        }
    }

    fn incoming_targets(&self, field: &str, entry: &Entry) -> BTreeSet<EntryId> {
        self.sources_by_field_target
            .get(field)
            .and_then(|sources_by_target| sources_by_target.get(&entry.id))
            .cloned()
            .unwrap_or_default()
    }

    // sirno:witness:structural-edge-policy:begin
    fn clique_targets(&self, field: &str, entry: &Entry) -> BTreeSet<EntryId> {
        let mut targets = BTreeSet::new();
        let Some(cliques_by_target) = self.cliques_by_field_target.get(field) else {
            return targets;
        };
        for target in entry.metadata.structural_targets_for(field) {
            if let Some(clique) = cliques_by_target.get(target) {
                targets.extend(clique.iter().filter(|id| *id != &entry.id).cloned());
            }
        }
        if let Some(clique) = cliques_by_target.get(&entry.id) {
            targets.extend(clique.iter().filter(|id| *id != &entry.id).cloned());
        }
        targets
    }
    // sirno:witness:structural-edge-policy:end
}

fn section_title(field: &str, direction: StructuralEdgeDirection) -> String {
    format!("{field} ({direction})")
}

#[derive(Debug)]
struct GeneratedLinkSection {
    title: String,
    targets: BTreeSet<EntryId>,
}

impl GeneratedLinkSection {
    fn new(title: String, targets: BTreeSet<EntryId>) -> Self {
        Self { title, targets }
    }

    // sirno:witness:generated-footer:begin
    fn render(&self, out: &mut String) {
        out.push_str("- ");
        out.push_str(&self.title);
        out.push(':');

        if self.targets.is_empty() {
            out.push_str(" (none)\n");
            return;
        }

        out.push('\n');
        for id in &self.targets {
            out.push_str("  - ");
            out.push_str(&render_markdown_entry_link(id));
            out.push('\n');
        }
    }
    // sirno:witness:generated-footer:end
}

fn render_markdown_entry_link(id: &EntryId) -> String {
    format!(
        "[{}]({}.md)",
        escape_markdown_link_label(id.as_str()),
        percent_encode_path_segment(id.as_str())
    )
}

fn escape_markdown_link_label(value: &str) -> String {
    let mut out = String::new();
    for character in value.chars() {
        if matches!(character, '[' | ']') {
            out.push('\\');
        }
        out.push(character);
    }
    out
}

fn percent_encode_path_segment(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            write!(&mut out, "%{byte:02X}").expect("writing to a string cannot fail");
        }
    }
    out
}

/// Opening guard for Sirno-owned generated links.
// sirno:witness:generated-footer-ownership:begin
pub const BEGIN_LINKS_GUARD: &str = "> **Sirno generated links begin. Do not edit this section.**";
/// Closing guard for Sirno-owned generated links.
pub const END_LINKS_GUARD: &str = "> **Sirno generated links end.**";
// sirno:witness:generated-footer-ownership:end

const GENERATED_LINK_DIVIDER: &str = "---";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GeneratedLinkBounds {
    region_start: usize,
    region_end: usize,
}

impl GeneratedLinkBounds {
    // sirno:witness:generated-footer-ownership:begin
    fn find(body: &str) -> Result<Option<Self>, GeneratedLinkError> {
        let begin = Self::guard_positions(body, BEGIN_LINKS_GUARD);
        let end = Self::guard_positions(body, END_LINKS_GUARD);
        let bounds = match (begin.as_slice(), end.as_slice()) {
            | ([], []) => Ok(()),
            | ([begin], [end]) if begin < end => Ok(()),
            | ([begin], [end]) if begin > end => Err(GeneratedLinkError::EndBeforeBegin),
            | ([], [_]) => Err(GeneratedLinkError::MissingBegin),
            | ([_], []) => Err(GeneratedLinkError::MissingEnd),
            | (_, _) if begin.len() > 1 => Err(GeneratedLinkError::DuplicateBegin),
            | (_, _) if end.len() > 1 => Err(GeneratedLinkError::DuplicateEnd),
            | _ => Err(GeneratedLinkError::Malformed),
        };
        bounds?;

        if begin.is_empty() {
            return Ok(None);
        }

        let begin = begin[0];
        let end = end[0] + END_LINKS_GUARD.len();
        Ok(Some(Self { region_start: Self::line_start(body, begin), region_end: end }))
    }
    // sirno:witness:generated-footer-ownership:end

    fn next_line_start(self, body: &str) -> usize {
        body[self.region_end..]
            .find('\n')
            .map_or(body.len(), |position| self.region_end + position + 1)
    }

    fn guard_positions(body: &str, guard: &str) -> Vec<usize> {
        body.match_indices(guard).map(|(index, _)| index).collect()
    }

    fn line_start(body: &str, index: usize) -> usize {
        body[..index].rfind('\n').map_or(0, |position| position + 1)
    }
}

/// Error raised by generated-link footer handling.
#[derive(Debug, Error, PartialEq, Eq)]
// sirno:witness:generated-footer:begin
pub enum GeneratedLinkError {
    /// A closing guard appears without an opening guard.
    #[error("generated-link footer is missing its opening guard")]
    MissingBegin,
    /// An opening guard appears without a closing guard.
    #[error("generated-link footer is missing its closing guard")]
    MissingEnd,
    /// More than one opening guard appears.
    #[error("generated-link footer has duplicate opening guards")]
    DuplicateBegin,
    /// More than one closing guard appears.
    #[error("generated-link footer has duplicate closing guards")]
    DuplicateEnd,
    /// The closing guard appears before the opening guard.
    #[error("generated-link footer closing guard appears before opening guard")]
    EndBeforeBegin,
    /// The generated-link guard state is malformed.
    #[error("generated-link footer boundaries are malformed")]
    Malformed,
}
// sirno:witness:generated-footer:end

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entry, EntryId, EntryMetadata};

    const FIELD_KIND: &str = "kind";
    const FIELD_AREA: &str = "area";
    const FIELD_PARENT: &str = "parent";

    fn id(raw: &str) -> EntryId {
        EntryId::new(raw).unwrap()
    }

    fn entry() -> Entry {
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target(FIELD_KIND, id("meta"));
        metadata.push_structural_target(FIELD_AREA, id("core"));
        metadata.push_structural_target(FIELD_PARENT, id("metadata"));
        Entry::new(id("concept"), metadata, "Body.\n")
    }

    fn structural_settings(
        fields: impl IntoIterator<Item = (&'static str, StructuralFieldSettings)>,
    ) -> StructuralSettings {
        StructuralSettings::from_fields(fields)
    }

    fn area_settings() -> StructuralSettings {
        structural_settings([(FIELD_AREA, render_settings(true, true, false))])
    }

    fn render_settings(to: bool, from: bool, clique: bool) -> StructuralFieldSettings {
        StructuralFieldSettings::render_only(to, from, clique)
    }

    #[test]
    fn default_settings_render_no_sections() {
        let footer = StructuralSettings::default().render_entry(&entry());

        assert_eq!(footer, format!("{BEGIN_LINKS_GUARD}\n\n(none)\n\n{END_LINKS_GUARD}"));
    }

    #[test]
    fn configured_settings_render_selected_field_links() {
        let footer = area_settings().render_entry(&entry());

        assert!(!footer.contains("[meta](meta.md)"));
        assert!(footer.contains("  - [core](core.md)"));
        assert!(!footer.contains("[metadata](metadata.md)"));
        assert!(!footer.contains("## Sirno Links"));
        assert!(footer.contains("- area (from): (none)"));
        assert!(footer.contains("- area (to):\n  - [core](core.md)"));
        assert!(footer.contains(BEGIN_LINKS_GUARD));
        assert!(footer.contains(END_LINKS_GUARD));
        assert!(footer.contains("> **Sirno generated links begin."));
    }

    #[test]
    fn quoted_guards_are_separated_from_link_list() {
        let footer = area_settings().render_entry(&entry());

        assert!(footer.contains(&format!(
            "{BEGIN_LINKS_GUARD}\n\n- area (to):\n  - [core](core.md)\n- area (from): (none)"
        )));
        assert!(footer.contains(&format!("- area (from): (none)\n\n{END_LINKS_GUARD}")));
    }

    #[test]
    fn settings_can_enable_each_structural_field() {
        let settings = structural_settings([
            (FIELD_KIND, render_settings(true, true, false)),
            (FIELD_AREA, render_settings(true, true, false)),
            (FIELD_PARENT, render_settings(true, true, false)),
        ]);
        let footer = settings.render_entry(&entry());

        assert!(footer.contains("  - [meta](meta.md)"));
        assert!(footer.contains("  - [core](core.md)"));
        assert!(footer.contains("  - [metadata](metadata.md)"));
        assert!(footer.contains("- kind (from): (none)"));
        assert!(footer.contains("- kind (to):"));
        assert!(footer.contains("- area (from): (none)"));
        assert!(footer.contains("- area (to):"));
        assert!(footer.contains("- parent (from): (none)"));
        assert!(footer.contains("- parent (to):"));
    }

    #[test]
    fn repeated_targets_render_once() {
        let mut entry = entry();
        entry.metadata.push_structural_target(FIELD_KIND, id("meta"));
        let settings = structural_settings([(FIELD_KIND, render_settings(true, true, false))]);

        let footer = settings.render_entry(&entry);

        assert_eq!(footer.matches("[meta](meta.md)").count(), 1);
    }

    #[test]
    fn generated_links_escape_filename_like_entry_ids() {
        let target = id("Spec [A] #1");
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target(FIELD_AREA, target);
        let entry = Entry::new(id("concept"), metadata, "Body.\n");

        let footer = area_settings().render_entry(&entry);

        assert!(footer.contains("  - [Spec \\[A\\] #1](Spec%20%5BA%5D%20%231.md)"));
    }

    #[test]
    fn boolean_field_settings_render_to_and_from_edges() {
        let settings = structural_settings([(FIELD_KIND, render_settings(true, true, false))]);
        let target_entry =
            Entry::new(id("meta"), EntryMetadata::new("Meta", "A kind.").unwrap(), "Body.\n");
        let mut member_metadata = EntryMetadata::new("Member", "A kind member.").unwrap();
        member_metadata.push_structural_target(FIELD_KIND, id("meta"));
        let member = Entry::new(id("member"), member_metadata, "Body.\n");
        let entries = vec![target_entry.clone(), member.clone()];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let target_footer = index.render_entry(&target_entry, &settings);
        let member_footer = index.render_entry(&member, &settings);

        assert!(target_footer.contains("- kind (from):"));
        assert!(target_footer.contains("  - [member](member.md)"));
        assert!(target_footer.contains("- kind (to): (none)"));
        assert!(member_footer.contains("- kind (from): (none)"));
        assert!(member_footer.contains("- kind (to):"));
        assert!(member_footer.contains("  - [meta](meta.md)"));
    }

    #[test]
    fn table_field_settings_can_choose_one_side() {
        let settings = structural_settings([(FIELD_KIND, render_settings(false, true, false))]);
        let target_entry =
            Entry::new(id("meta"), EntryMetadata::new("Meta", "A kind.").unwrap(), "Body.\n");
        let mut member_metadata = EntryMetadata::new("Member", "A kind member.").unwrap();
        member_metadata.push_structural_target(FIELD_KIND, id("meta"));
        let member = Entry::new(id("member"), member_metadata, "Body.\n");
        let entries = vec![target_entry.clone(), member.clone()];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let target_footer = index.render_entry(&target_entry, &settings);
        let member_footer = index.render_entry(&member, &settings);

        assert!(target_footer.contains("- kind (from):"));
        assert!(target_footer.contains("  - [member](member.md)"));
        assert!(member_footer.contains("- kind (from): (none)"));
        assert!(!member_footer.contains("[meta](meta.md)"));
    }

    #[test]
    fn clique_setting_expands_field_targets_to_edges() {
        let settings = structural_settings([(FIELD_AREA, render_settings(false, false, true))]);

        let closure = Entry::new(
            id("core"),
            EntryMetadata::new("Core", "A review neighborhood.").unwrap(),
            "Body.\n",
        );
        let mut left_metadata = EntryMetadata::new("Left", "A neighborhood member.").unwrap();
        left_metadata.push_structural_target(FIELD_AREA, id("core"));
        let left = Entry::new(id("left"), left_metadata, "Body.\n");
        let mut right_metadata = EntryMetadata::new("Right", "A neighborhood member.").unwrap();
        right_metadata.push_structural_target(FIELD_AREA, id("core"));
        let right = Entry::new(id("right"), right_metadata, "Body.\n");
        let mut outside_metadata = EntryMetadata::new("Outside", "Another member.").unwrap();
        outside_metadata.push_structural_target(FIELD_AREA, id("other"));
        let outside = Entry::new(id("outside"), outside_metadata, "Body.\n");
        let entries = vec![closure.clone(), left.clone(), right.clone(), outside];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let closure_footer = index.render_entry(&closure, &settings);
        let left_footer = index.render_entry(&left, &settings);

        assert!(closure_footer.contains("- area (clique):"));
        assert!(!closure_footer.contains("area (from)"));
        assert!(closure_footer.contains("  - [left](left.md)"));
        assert!(closure_footer.contains("  - [right](right.md)"));
        assert!(!closure_footer.contains("[core](core.md)"));
        assert!(!closure_footer.contains("[outside](outside.md)"));
        assert!(left_footer.contains("- area (clique):"));
        assert!(!left_footer.contains("area (to)"));
        assert!(left_footer.contains("  - [core](core.md)"));
        assert!(left_footer.contains("  - [right](right.md)"));
        assert!(!left_footer.contains("[left](left.md)"));
        assert!(!left_footer.contains("[outside](outside.md)"));
    }

    #[test]
    fn direct_sections_remain_when_clique_is_enabled() {
        let settings = structural_settings([(FIELD_AREA, render_settings(true, true, true))]);

        let closure = Entry::new(
            id("core"),
            EntryMetadata::new("Core", "A review neighborhood.").unwrap(),
            "Body.\n",
        );
        let mut left_metadata = EntryMetadata::new("Left", "A neighborhood member.").unwrap();
        left_metadata.push_structural_target(FIELD_AREA, id("core"));
        let left = Entry::new(id("left"), left_metadata, "Body.\n");
        let mut right_metadata = EntryMetadata::new("Right", "A neighborhood member.").unwrap();
        right_metadata.push_structural_target(FIELD_AREA, id("core"));
        let right = Entry::new(id("right"), right_metadata, "Body.\n");
        let entries = vec![closure, left.clone(), right];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let left_footer = index.render_entry(&left, &settings);

        assert!(left_footer.contains("- area (to):\n  - [core](core.md)"));
        assert!(left_footer.contains("- area (clique):"));
        assert!(left_footer.contains("  - [right](right.md)"));
    }

    #[test]
    fn sections_render_to_from_clique_order() {
        let settings = structural_settings([(FIELD_AREA, render_settings(true, true, true))]);

        let closure = Entry::new(
            id("core"),
            EntryMetadata::new("Core", "A review neighborhood.").unwrap(),
            "Body.\n",
        );
        let mut left_metadata = EntryMetadata::new("Left", "A neighborhood member.").unwrap();
        left_metadata.push_structural_target(FIELD_AREA, id("core"));
        let left = Entry::new(id("left"), left_metadata, "Body.\n");
        let mut right_metadata = EntryMetadata::new("Right", "A neighborhood member.").unwrap();
        right_metadata.push_structural_target(FIELD_AREA, id("core"));
        let right = Entry::new(id("right"), right_metadata, "Body.\n");
        let entries = vec![closure, left.clone(), right];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let footer = index.render_entry(&left, &settings);
        let to = footer.find("area (to):").unwrap();
        let from = footer.find("area (from):").unwrap();
        let clique = footer.find("area (clique):").unwrap();

        assert!(to < from);
        assert!(from < clique);
    }

    #[test]
    fn renders_empty_enabled_sections_when_entry_has_no_structural_targets() {
        let metadata = EntryMetadata::new("Meta", "A kind.").unwrap();
        let entry = Entry::new(EntryId::new("meta").unwrap(), metadata, "Body.\n");

        let footer = area_settings().render_entry(&entry);

        assert!(footer.contains("- area (from): (none)"));
        assert!(footer.contains("- area (to): (none)"));
        assert!(!footer.contains(&format!("{BEGIN_LINKS_GUARD}\n\n(none)\n\n{END_LINKS_GUARD}")));
        assert!(!footer.contains("- none"));
    }

    #[test]
    fn renders_region_none_when_no_sections_are_enabled() {
        let metadata = EntryMetadata::new("Meta", "A kind.").unwrap();
        let entry = Entry::new(EntryId::new("meta").unwrap(), metadata, "Body.\n");
        let settings = StructuralSettings::from_fields([
            (FIELD_KIND, StructuralFieldSettings::default()),
            (FIELD_AREA, StructuralFieldSettings::default()),
            (FIELD_PARENT, StructuralFieldSettings::default()),
        ]);

        let footer = settings.render_entry(&entry);

        assert_eq!(footer, format!("{BEGIN_LINKS_GUARD}\n\n(none)\n\n{END_LINKS_GUARD}"));
        assert!(!footer.contains("- none"));
    }

    #[test]
    fn appends_footer_when_missing() {
        let footer = StructuralSettings::default().render_entry(&entry());

        let body = GeneratedLinkBody::new("Body.\n").apply(&footer).unwrap();

        assert_eq!(body, format!("Body.\n\n---\n\n{footer}\n"));
        assert_eq!(body.matches(BEGIN_LINKS_GUARD).count(), 1);
    }

    #[test]
    fn appends_footer_without_duplicate_divider() {
        let footer = StructuralSettings::default().render_entry(&entry());

        let body = GeneratedLinkBody::new("Body.\n\n---\n").apply(&footer).unwrap();

        assert_eq!(body, format!("Body.\n\n---\n\n{footer}\n"));
    }

    #[test]
    fn replaces_only_existing_footer_region() {
        let old = format!("{BEGIN_LINKS_GUARD}\nold\n{END_LINKS_GUARD}\n");
        let body = format!("Before.\n\n{old}\nAfter.\n");
        let footer = StructuralSettings::default().render_entry(&entry());

        let body = GeneratedLinkBody::new(&body).apply(&footer).unwrap();

        assert!(body.starts_with("Before.\n\n"));
        assert!(body.ends_with("After.\n"));
        assert!(!body.contains("old"));
        assert_eq!(body.matches(BEGIN_LINKS_GUARD).count(), 1);
    }

    #[test]
    fn deletes_existing_footer_region() {
        let footer = StructuralSettings::default().render_entry(&entry());
        let body = GeneratedLinkBody::new("Body.\n").apply(&footer).unwrap();

        let body = GeneratedLinkBody::new(&body).delete().unwrap();

        assert_eq!(body, "Body.\n\n---\n");
        assert!(!body.contains(BEGIN_LINKS_GUARD));
    }

    #[test]
    fn deletes_footer_without_touching_following_body() {
        let footer = StructuralSettings::default().render_entry(&entry());
        let body = format!("Before.\n\n{footer}\nAfter.\n");

        let body = GeneratedLinkBody::new(&body).delete().unwrap();

        assert_eq!(body, "Before.\n\nAfter.\n");
    }

    #[test]
    fn delete_is_noop_when_footer_is_missing() {
        let body = GeneratedLinkBody::new("Body.\n").delete().unwrap();

        assert_eq!(body, "Body.\n");
    }

    #[test]
    fn masks_existing_footer_region() {
        let footer = StructuralSettings::default().render_entry(&entry());
        let body = format!("Before.\n\n{footer}\nAfter.\n");

        let masked = GeneratedLinkBody::new(&body).mask().unwrap();

        assert_eq!(masked.len(), body.len());
        assert_eq!(masked.lines().count(), body.lines().count());
        assert!(masked.contains("Before."));
        assert!(masked.contains("After."));
        assert!(!masked.contains(BEGIN_LINKS_GUARD));
        assert!(!masked.contains("(none)"));
    }

    #[test]
    fn mask_is_noop_when_footer_is_missing() {
        let body = GeneratedLinkBody::new("Body.\n").mask().unwrap();

        assert_eq!(body, "Body.\n");
    }

    #[test]
    fn reports_generated_links_staleness() {
        let expected = StructuralSettings::default().render_entry(&entry());
        let current = GeneratedLinkBody::new("Body.\n").apply(&expected).unwrap();
        let stale = GeneratedLinkBody::new("Body.\n")
            .apply(
                &structural_settings([
                    (FIELD_KIND, render_settings(true, true, false)),
                    (FIELD_AREA, render_settings(true, true, false)),
                    (FIELD_PARENT, render_settings(true, true, false)),
                ])
                .render_entry(&entry()),
            )
            .unwrap();

        assert!(!GeneratedLinkBody::new("Body.\n").is_stale(&expected).unwrap());
        assert!(!GeneratedLinkBody::new(&current).is_stale(&expected).unwrap());
        assert!(GeneratedLinkBody::new(&stale).is_stale(&expected).unwrap());
    }

    #[test]
    fn rejects_missing_end_guard() {
        let error = GeneratedLinkBody::new(BEGIN_LINKS_GUARD).validate().unwrap_err();

        assert_eq!(error, GeneratedLinkError::MissingEnd);
    }

    #[test]
    fn rejects_duplicate_begin_guard() {
        let body = format!("{BEGIN_LINKS_GUARD}\n{BEGIN_LINKS_GUARD}\n{END_LINKS_GUARD}\n");
        let error = GeneratedLinkBody::new(&body).validate().unwrap_err();

        assert_eq!(error, GeneratedLinkError::DuplicateBegin);
    }
}
