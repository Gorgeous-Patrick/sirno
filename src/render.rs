//! Rendered generated footers for entries.
//!
//! Sirno owns only the guard-bounded generated-link region.
//! Prose outside the region remains user-owned.

use std::collections::BTreeSet;
use std::fmt::Write;
use std::path::{Component, Path, PathBuf};

use thiserror::Error;

use crate::entry::Entry;
use crate::identifier::EntryAddress;
use crate::structural::{StructuralEdgeDirection, StructuralEdgeIndex, StructuralSettings};

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

impl StructuralEdgeIndex {
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
                        entry.id.clone(),
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
}

fn section_title(field: &str, direction: StructuralEdgeDirection) -> String {
    format!("{field} ({direction})")
}

#[derive(Debug)]
struct GeneratedLinkSection {
    title: String,
    source: EntryAddress,
    targets: BTreeSet<EntryAddress>,
}

impl GeneratedLinkSection {
    fn new(title: String, source: EntryAddress, targets: BTreeSet<EntryAddress>) -> Self {
        Self { title, source, targets }
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
            out.push_str(&render_markdown_entry_link(&self.source, id));
            out.push('\n');
        }
    }
    // sirno:witness:generated-footer:end
}

fn render_markdown_entry_link(source: &EntryAddress, target: &EntryAddress) -> String {
    format!(
        "[{}]({})",
        escape_markdown_link_label(target.as_str()),
        percent_encode_relative_path(&relative_entry_link(source, target))
    )
}

fn relative_entry_link(source: &EntryAddress, target: &EntryAddress) -> PathBuf {
    let source_path = source.to_lake_relative_path();
    let target_path = target.to_lake_relative_path();
    let source_parent = source_path.parent().unwrap_or_else(|| Path::new(""));
    relative_path(source_parent, &target_path)
}

fn relative_path(from_directory: &Path, target: &Path) -> PathBuf {
    let from = normal_components(from_directory);
    let target = normal_components(target);
    let common = from.iter().zip(&target).take_while(|(left, right)| left == right).count();
    let mut relative = PathBuf::new();
    for _ in &from[common..] {
        relative.push("..");
    }
    for segment in &target[common..] {
        relative.push(segment);
    }
    relative
}

fn normal_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            | Component::Normal(value) => value.to_str().map(ToOwned::to_owned),
            | _ => None,
        })
        .collect()
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

fn percent_encode_relative_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            | Component::Normal(segment) => segment.to_str().map(percent_encode_path_segment),
            | Component::ParentDir => Some("..".to_owned()),
            | Component::CurDir => Some(".".to_owned()),
            | Component::RootDir | Component::Prefix(_) => None,
        })
        .collect::<Vec<_>>()
        .join("/")
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
    use crate::structural::{StructuralFieldSettings, StructuralSettings};
    use crate::{Entry, EntryAddress, EntryMetadata};

    const FIELD_KIND: &str = "kind";
    const FIELD_AREA: &str = "area";
    const FIELD_PARENT: &str = "parent";

    fn id(raw: &str) -> EntryAddress {
        EntryAddress::new(raw).unwrap()
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

    fn render_entry(entry: &Entry, settings: &StructuralSettings) -> String {
        StructuralEdgeIndex::from_entries(std::slice::from_ref(entry)).render_entry(entry, settings)
    }

    #[test]
    fn default_settings_render_no_sections() {
        let footer = render_entry(&entry(), &StructuralSettings::default());

        assert_eq!(footer, format!("{BEGIN_LINKS_GUARD}\n\n(none)\n\n{END_LINKS_GUARD}"));
    }

    #[test]
    fn configured_settings_render_selected_field_links() {
        let footer = render_entry(&entry(), &area_settings());

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
        let footer = render_entry(&entry(), &area_settings());

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
        let footer = render_entry(&entry(), &settings);

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
    fn renders_relative_links_between_root_and_domain_entries() {
        let settings = structural_settings([(FIELD_AREA, render_settings(true, false, false))]);
        let mut root_metadata = EntryMetadata::new("Concept", "A root entry.").unwrap();
        root_metadata.push_structural_target(FIELD_AREA, id("core.design"));
        let root = Entry::new(id("concept"), root_metadata, "Body.\n");
        let mut domain_metadata = EntryMetadata::new("Design", "A domain entry.").unwrap();
        domain_metadata.push_structural_target(FIELD_AREA, id("concept"));
        let domain = Entry::new(id("core.design"), domain_metadata, "Body.\n");
        let entries = vec![root.clone(), domain.clone()];
        let index = StructuralEdgeIndex::from_entries(&entries);

        let root_footer = index.render_entry(&root, &settings);
        let domain_footer = index.render_entry(&domain, &settings);

        assert!(root_footer.contains("  - [core.design](core/design.md)"));
        assert!(domain_footer.contains("  - [concept](../concept.md)"));
    }

    #[test]
    fn repeated_targets_render_once() {
        let mut entry = entry();
        entry.metadata.push_structural_target(FIELD_KIND, id("meta"));
        let settings = structural_settings([(FIELD_KIND, render_settings(true, true, false))]);

        let footer = render_entry(&entry, &settings);

        assert_eq!(footer.matches("[meta](meta.md)").count(), 1);
    }

    #[test]
    fn generated_links_escape_filename_like_entry_addresses() {
        let target = id("Spec [A] #1");
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target(FIELD_AREA, target);
        let entry = Entry::new(id("concept"), metadata, "Body.\n");

        let footer = render_entry(&entry, &area_settings());

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
        let index = StructuralEdgeIndex::from_entries(&entries);

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
        let index = StructuralEdgeIndex::from_entries(&entries);

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
        let index = StructuralEdgeIndex::from_entries(&entries);

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
        let index = StructuralEdgeIndex::from_entries(&entries);

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
        let index = StructuralEdgeIndex::from_entries(&entries);

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
        let entry = Entry::new(EntryAddress::new("meta").unwrap(), metadata, "Body.\n");

        let footer = render_entry(&entry, &area_settings());

        assert!(footer.contains("- area (from): (none)"));
        assert!(footer.contains("- area (to): (none)"));
        assert!(!footer.contains(&format!("{BEGIN_LINKS_GUARD}\n\n(none)\n\n{END_LINKS_GUARD}")));
        assert!(!footer.contains("- none"));
    }

    #[test]
    fn renders_region_none_when_no_sections_are_enabled() {
        let metadata = EntryMetadata::new("Meta", "A kind.").unwrap();
        let entry = Entry::new(EntryAddress::new("meta").unwrap(), metadata, "Body.\n");
        let settings = StructuralSettings::from_fields([
            (FIELD_KIND, StructuralFieldSettings::default()),
            (FIELD_AREA, StructuralFieldSettings::default()),
            (FIELD_PARENT, StructuralFieldSettings::default()),
        ]);

        let footer = render_entry(&entry, &settings);

        assert_eq!(footer, format!("{BEGIN_LINKS_GUARD}\n\n(none)\n\n{END_LINKS_GUARD}"));
        assert!(!footer.contains("- none"));
    }

    #[test]
    fn appends_footer_when_missing() {
        let footer = render_entry(&entry(), &StructuralSettings::default());

        let body = GeneratedLinkBody::new("Body.\n").apply(&footer).unwrap();

        assert_eq!(body, format!("Body.\n\n---\n\n{footer}\n"));
        assert_eq!(body.matches(BEGIN_LINKS_GUARD).count(), 1);
    }

    #[test]
    fn appends_footer_without_duplicate_divider() {
        let footer = render_entry(&entry(), &StructuralSettings::default());

        let body = GeneratedLinkBody::new("Body.\n\n---\n").apply(&footer).unwrap();

        assert_eq!(body, format!("Body.\n\n---\n\n{footer}\n"));
    }

    #[test]
    fn replaces_only_existing_footer_region() {
        let old = format!("{BEGIN_LINKS_GUARD}\nold\n{END_LINKS_GUARD}\n");
        let body = format!("Before.\n\n{old}\nAfter.\n");
        let footer = render_entry(&entry(), &StructuralSettings::default());

        let body = GeneratedLinkBody::new(&body).apply(&footer).unwrap();

        assert!(body.starts_with("Before.\n\n"));
        assert!(body.ends_with("After.\n"));
        assert!(!body.contains("old"));
        assert_eq!(body.matches(BEGIN_LINKS_GUARD).count(), 1);
    }

    #[test]
    fn deletes_existing_footer_region() {
        let footer = render_entry(&entry(), &StructuralSettings::default());
        let body = GeneratedLinkBody::new("Body.\n").apply(&footer).unwrap();

        let body = GeneratedLinkBody::new(&body).delete().unwrap();

        assert_eq!(body, "Body.\n\n---\n");
        assert!(!body.contains(BEGIN_LINKS_GUARD));
    }

    #[test]
    fn deletes_footer_without_touching_following_body() {
        let footer = render_entry(&entry(), &StructuralSettings::default());
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
        let footer = render_entry(&entry(), &StructuralSettings::default());
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
        let expected = render_entry(&entry(), &StructuralSettings::default());
        let current = GeneratedLinkBody::new("Body.\n").apply(&expected).unwrap();
        let stale = GeneratedLinkBody::new("Body.\n")
            .apply(&render_entry(
                &entry(),
                &structural_settings([
                    (FIELD_KIND, render_settings(true, true, false)),
                    (FIELD_AREA, render_settings(true, true, false)),
                    (FIELD_PARENT, render_settings(true, true, false)),
                ]),
            ))
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
