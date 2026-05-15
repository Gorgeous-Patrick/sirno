//! Generated Markdown links for entries.
//!
//! Sirno owns only the guard-bounded generated-link region.
//! Prose outside the region remains user-owned.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entry::{BELONGS_FIELD, CATEGORY_FIELD, Entry, REFINES_FIELD};
use crate::id::EntryId;

fn is_false(value: &bool) -> bool {
    !*value
}

/// Generated-link settings for one structural field.
///
/// `to` includes links from the current entry to metadata targets.
/// `from` includes links from the current entry to entries that point at it.
/// `clique` includes entries connected through a shared target in the same field.
// sirno:witness:generated-link-policy:begin
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StructuralLinkSettings {
    /// Include outgoing metadata targets.
    #[serde(skip_serializing_if = "is_false")]
    pub to: bool,
    /// Include incoming metadata sources.
    #[serde(skip_serializing_if = "is_false")]
    pub from: bool,
    /// Include clique links derived from shared targets in this field.
    #[serde(skip_serializing_if = "is_false")]
    pub clique: bool,
}
// sirno:witness:generated-link-policy:end

// sirno:witness:generated-link-policy:begin
impl StructuralLinkSettings {
    /// Construct structural-field link settings from explicit sides.
    pub fn new(to: bool, from: bool, clique: bool) -> Self {
        Self { to, from, clique }
    }

    /// Construct structural-field link settings from one boolean applied to direct links.
    pub fn from_bool(enabled: bool) -> Self {
        Self::new(enabled, enabled, false)
    }

    /// Construct enabled structural-field link settings.
    pub fn enabled() -> Self {
        Self::from_bool(true)
    }

    /// Construct disabled structural-field link settings.
    pub fn disabled() -> Self {
        Self::from_bool(false)
    }
}
// sirno:witness:generated-link-policy:end

impl From<bool> for StructuralLinkSettings {
    fn from(value: bool) -> Self {
        Self::from_bool(value)
    }
}

impl fmt::Display for StructuralLinkSettings {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.from {
            parts.push("from=true");
        }
        if self.to {
            parts.push("to=true");
        }
        if self.clique {
            parts.push("clique=true");
        }
        if parts.is_empty() {
            write!(formatter, "none")
        } else {
            write!(formatter, "{}", parts.join(" "))
        }
    }
}

/// Settings for one configured structural field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
// sirno:witness:generated-link-policy:begin
pub struct StructuralFieldSettings {
    /// Generated-link policy for this structural field.
    pub link: StructuralLinkSettings,
}
// sirno:witness:generated-link-policy:end

impl StructuralFieldSettings {
    /// Construct structural field settings from a link policy.
    pub fn new(link: StructuralLinkSettings) -> Self {
        Self { link }
    }
}

/// Configured structural fields.
///
/// Each key names a metadata field that Sirno should treat as structural.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
// sirno:witness:generated-link-policy:begin
pub struct StructuralSettings {
    fields: BTreeMap<String, StructuralFieldSettings>,
}
// sirno:witness:generated-link-policy:end

impl Default for StructuralSettings {
    fn default() -> Self {
        Self::from_fields([
            (CATEGORY_FIELD, StructuralFieldSettings::default()),
            (BELONGS_FIELD, StructuralFieldSettings::new(StructuralLinkSettings::enabled())),
            (REFINES_FIELD, StructuralFieldSettings::default()),
        ])
    }
}

impl StructuralSettings {
    /// Construct structural settings from explicit field settings.
    pub fn from_fields(
        fields: impl IntoIterator<Item = (impl Into<String>, StructuralFieldSettings)>,
    ) -> Self {
        Self {
            fields: fields.into_iter().map(|(field, settings)| (field.into(), settings)).collect(),
        }
    }

    /// Iterate configured fields in deterministic order.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &StructuralFieldSettings)> {
        let mut fields = Vec::new();
        for field in [CATEGORY_FIELD, BELONGS_FIELD, REFINES_FIELD] {
            if let Some(settings) = self.fields.get(field) {
                fields.push((field, settings));
            }
        }
        fields.extend(self.fields.iter().filter_map(|(field, settings)| {
            (!matches!(field.as_str(), CATEGORY_FIELD | BELONGS_FIELD | REFINES_FIELD))
                .then_some((field.as_str(), settings))
        }));
        fields.into_iter()
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
            let link = field_settings.link;
            if link.to {
                sections.push(GeneratedLinkSection::new(
                    section_title(field, "to"),
                    entry.metadata.structural_targets_for(field).iter().cloned().collect(),
                ));
            }
            if link.from {
                sections.push(GeneratedLinkSection::new(
                    section_title(field, "from"),
                    self.incoming_targets(field, entry),
                ));
            }
            if link.clique {
                sections.push(GeneratedLinkSection::new(
                    section_title(field, "clique"),
                    self.clique_targets(field, entry),
                ));
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

    fn incoming_targets(&self, field: &str, entry: &Entry) -> BTreeSet<EntryId> {
        self.sources_by_field_target
            .get(field)
            .and_then(|sources_by_target| sources_by_target.get(&entry.id))
            .cloned()
            .unwrap_or_default()
    }

    // sirno:witness:belongs:begin
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
    // sirno:witness:belongs:end
}

fn section_title(field: &str, direction: &str) -> String {
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
        if self.targets.is_empty() {
            out.push_str(&self.title);
            out.push_str(": (none)\n\n");
            return;
        }

        out.push_str(&self.title);
        out.push(':');
        out.push('\n');
        for id in &self.targets {
            out.push_str("- ");
            out.push_str(&format!("[{}]({}.md)", id.as_str(), id.as_str()));
            out.push('\n');
        }
        out.push('\n');
    }
    // sirno:witness:generated-footer:end
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

    fn id(raw: &str) -> EntryId {
        EntryId::new(raw).unwrap()
    }

    fn entry() -> Entry {
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target(CATEGORY_FIELD, id("meta"));
        metadata.push_structural_target(BELONGS_FIELD, id("core"));
        metadata.push_structural_target(REFINES_FIELD, id("metadata"));
        Entry::new(id("concept"), metadata, "Body.\n")
    }

    fn structural_settings(
        fields: impl IntoIterator<Item = (&'static str, StructuralLinkSettings)>,
    ) -> StructuralSettings {
        StructuralSettings::from_fields(
            fields.into_iter().map(|(field, link)| (field, StructuralFieldSettings::new(link))),
        )
    }

    #[test]
    fn default_settings_render_only_belongs_links() {
        let footer = StructuralSettings::default().render_entry(&entry());

        assert!(!footer.contains("[meta](meta.md)"));
        assert!(footer.contains("- [core](core.md)"));
        assert!(!footer.contains("[metadata](metadata.md)"));
        assert!(!footer.contains("## Sirno Links"));
        assert!(footer.contains("belongs (from): (none)"));
        assert!(footer.contains("belongs (to):\n- [core](core.md)"));
        assert!(footer.contains(BEGIN_LINKS_GUARD));
        assert!(footer.contains(END_LINKS_GUARD));
        assert!(footer.contains("> **Sirno generated links begin."));
    }

    #[test]
    fn quoted_guards_are_separated_from_link_list() {
        let footer = StructuralSettings::default().render_entry(&entry());

        assert!(footer.contains(&format!(
            "{BEGIN_LINKS_GUARD}\n\nbelongs (to):\n- [core](core.md)\n\nbelongs (from): (none)"
        )));
        assert!(footer.contains(&format!("belongs (from): (none)\n\n{END_LINKS_GUARD}")));
    }

    #[test]
    fn settings_can_enable_each_structural_field() {
        let settings = structural_settings([
            (CATEGORY_FIELD, StructuralLinkSettings::enabled()),
            (BELONGS_FIELD, StructuralLinkSettings::enabled()),
            (REFINES_FIELD, StructuralLinkSettings::enabled()),
        ]);
        let footer = settings.render_entry(&entry());

        assert!(footer.contains("- [meta](meta.md)"));
        assert!(footer.contains("- [core](core.md)"));
        assert!(footer.contains("- [metadata](metadata.md)"));
        assert!(footer.contains("category (from): (none)"));
        assert!(footer.contains("category (to):"));
        assert!(footer.contains("belongs (from): (none)"));
        assert!(footer.contains("belongs (to):"));
        assert!(footer.contains("refines (from): (none)"));
        assert!(footer.contains("refines (to):"));
    }

    #[test]
    fn repeated_targets_render_once() {
        let mut entry = entry();
        entry.metadata.push_structural_target(CATEGORY_FIELD, id("meta"));
        let settings = structural_settings([(CATEGORY_FIELD, StructuralLinkSettings::enabled())]);

        let footer = settings.render_entry(&entry);

        assert_eq!(footer.matches("[meta](meta.md)").count(), 1);
    }

    #[test]
    fn boolean_field_settings_render_to_and_from_edges() {
        let settings = structural_settings([(CATEGORY_FIELD, StructuralLinkSettings::enabled())]);
        let category =
            Entry::new(id("meta"), EntryMetadata::new("Meta", "A category.").unwrap(), "Body.\n");
        let mut member_metadata = EntryMetadata::new("Member", "A category member.").unwrap();
        member_metadata.push_structural_target(CATEGORY_FIELD, id("meta"));
        let member = Entry::new(id("member"), member_metadata, "Body.\n");
        let entries = vec![category.clone(), member.clone()];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let category_footer = index.render_entry(&category, &settings);
        let member_footer = index.render_entry(&member, &settings);

        assert!(category_footer.contains("category (from):"));
        assert!(category_footer.contains("- [member](member.md)"));
        assert!(category_footer.contains("category (to): (none)"));
        assert!(member_footer.contains("category (from): (none)"));
        assert!(member_footer.contains("category (to):"));
        assert!(member_footer.contains("- [meta](meta.md)"));
    }

    #[test]
    fn table_field_settings_can_choose_one_side() {
        let settings = structural_settings([(
            CATEGORY_FIELD,
            StructuralLinkSettings::new(false, true, false),
        )]);
        let category =
            Entry::new(id("meta"), EntryMetadata::new("Meta", "A category.").unwrap(), "Body.\n");
        let mut member_metadata = EntryMetadata::new("Member", "A category member.").unwrap();
        member_metadata.push_structural_target(CATEGORY_FIELD, id("meta"));
        let member = Entry::new(id("member"), member_metadata, "Body.\n");
        let entries = vec![category.clone(), member.clone()];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let category_footer = index.render_entry(&category, &settings);
        let member_footer = index.render_entry(&member, &settings);

        assert!(category_footer.contains("category (from):"));
        assert!(category_footer.contains("- [member](member.md)"));
        assert!(member_footer.contains("category (from): (none)"));
        assert!(!member_footer.contains("[meta](meta.md)"));
    }

    #[test]
    fn clique_setting_expands_belongs_targets_to_edges() {
        let settings =
            structural_settings([(BELONGS_FIELD, StructuralLinkSettings::new(false, false, true))]);

        let closure = Entry::new(
            id("core"),
            EntryMetadata::new("Core", "A review neighborhood.").unwrap(),
            "Body.\n",
        );
        let mut left_metadata = EntryMetadata::new("Left", "A neighborhood member.").unwrap();
        left_metadata.push_structural_target(BELONGS_FIELD, id("core"));
        let left = Entry::new(id("left"), left_metadata, "Body.\n");
        let mut right_metadata = EntryMetadata::new("Right", "A neighborhood member.").unwrap();
        right_metadata.push_structural_target(BELONGS_FIELD, id("core"));
        let right = Entry::new(id("right"), right_metadata, "Body.\n");
        let mut outside_metadata = EntryMetadata::new("Outside", "Another member.").unwrap();
        outside_metadata.push_structural_target(BELONGS_FIELD, id("other"));
        let outside = Entry::new(id("outside"), outside_metadata, "Body.\n");
        let entries = vec![closure.clone(), left.clone(), right.clone(), outside];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let closure_footer = index.render_entry(&closure, &settings);
        let left_footer = index.render_entry(&left, &settings);

        assert!(closure_footer.contains("belongs (clique):"));
        assert!(!closure_footer.contains("belongs (from)"));
        assert!(closure_footer.contains("- [left](left.md)"));
        assert!(closure_footer.contains("- [right](right.md)"));
        assert!(!closure_footer.contains("[core](core.md)"));
        assert!(!closure_footer.contains("[outside](outside.md)"));
        assert!(left_footer.contains("belongs (clique):"));
        assert!(!left_footer.contains("belongs (to)"));
        assert!(left_footer.contains("- [core](core.md)"));
        assert!(left_footer.contains("- [right](right.md)"));
        assert!(!left_footer.contains("[left](left.md)"));
        assert!(!left_footer.contains("[outside](outside.md)"));
    }

    #[test]
    fn belongs_sections_remain_direct_when_clique_is_enabled() {
        let settings =
            structural_settings([(BELONGS_FIELD, StructuralLinkSettings::new(true, true, true))]);

        let closure = Entry::new(
            id("core"),
            EntryMetadata::new("Core", "A review neighborhood.").unwrap(),
            "Body.\n",
        );
        let mut left_metadata = EntryMetadata::new("Left", "A neighborhood member.").unwrap();
        left_metadata.push_structural_target(BELONGS_FIELD, id("core"));
        let left = Entry::new(id("left"), left_metadata, "Body.\n");
        let mut right_metadata = EntryMetadata::new("Right", "A neighborhood member.").unwrap();
        right_metadata.push_structural_target(BELONGS_FIELD, id("core"));
        let right = Entry::new(id("right"), right_metadata, "Body.\n");
        let entries = vec![closure, left.clone(), right];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let left_footer = index.render_entry(&left, &settings);

        assert!(left_footer.contains("belongs (to):\n- [core](core.md)"));
        assert!(left_footer.contains("belongs (clique):"));
        assert!(left_footer.contains("- [right](right.md)"));
    }

    #[test]
    fn sections_render_to_from_clique_order() {
        let settings =
            structural_settings([(BELONGS_FIELD, StructuralLinkSettings::new(true, true, true))]);

        let closure = Entry::new(
            id("core"),
            EntryMetadata::new("Core", "A review neighborhood.").unwrap(),
            "Body.\n",
        );
        let mut left_metadata = EntryMetadata::new("Left", "A neighborhood member.").unwrap();
        left_metadata.push_structural_target(BELONGS_FIELD, id("core"));
        let left = Entry::new(id("left"), left_metadata, "Body.\n");
        let mut right_metadata = EntryMetadata::new("Right", "A neighborhood member.").unwrap();
        right_metadata.push_structural_target(BELONGS_FIELD, id("core"));
        let right = Entry::new(id("right"), right_metadata, "Body.\n");
        let entries = vec![closure, left.clone(), right];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let footer = index.render_entry(&left, &settings);
        let to = footer.find("belongs (to):").unwrap();
        let from = footer.find("belongs (from):").unwrap();
        let clique = footer.find("belongs (clique):").unwrap();

        assert!(to < from);
        assert!(from < clique);
    }

    #[test]
    fn renders_empty_enabled_sections_when_entry_has_no_structural_targets() {
        let metadata = EntryMetadata::new("Meta", "A category.").unwrap();
        let entry = Entry::new(EntryId::new("meta").unwrap(), metadata, "Body.\n");

        let footer = StructuralSettings::default().render_entry(&entry);

        assert!(footer.contains("belongs (from): (none)"));
        assert!(footer.contains("belongs (to): (none)"));
        assert!(!footer.contains(&format!("{BEGIN_LINKS_GUARD}\n\n(none)\n\n{END_LINKS_GUARD}")));
        assert!(!footer.contains("- none"));
    }

    #[test]
    fn renders_region_none_when_no_sections_are_enabled() {
        let metadata = EntryMetadata::new("Meta", "A category.").unwrap();
        let entry = Entry::new(EntryId::new("meta").unwrap(), metadata, "Body.\n");
        let settings = StructuralSettings::from_fields([
            (CATEGORY_FIELD, StructuralFieldSettings::default()),
            (BELONGS_FIELD, StructuralFieldSettings::default()),
            (REFINES_FIELD, StructuralFieldSettings::default()),
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
    fn reports_generated_links_staleness() {
        let expected = StructuralSettings::default().render_entry(&entry());
        let current = GeneratedLinkBody::new("Body.\n").apply(&expected).unwrap();
        let stale = GeneratedLinkBody::new("Body.\n")
            .apply(
                &structural_settings([
                    (CATEGORY_FIELD, StructuralLinkSettings::enabled()),
                    (BELONGS_FIELD, StructuralLinkSettings::enabled()),
                    (REFINES_FIELD, StructuralLinkSettings::enabled()),
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
