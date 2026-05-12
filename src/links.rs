//! Generated Markdown links for entries.
//!
//! Sirno owns only the guard-bounded generated-link region.
//! Prose outside the region remains user-owned.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::entry::Entry;

/// Settings that choose which metadata fields become generated links.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeneratedLinkSettings {
    /// Include `category` relation targets.
    pub category: bool,
    /// Include `clustee` relation targets.
    pub clustee: bool,
    /// Expand enabled clustee links into clique edges.
    pub clique: bool,
    /// Include `refiner` relation targets.
    pub refiner: bool,
}

impl Default for GeneratedLinkSettings {
    fn default() -> Self {
        Self { category: false, clustee: true, clique: false, refiner: false }
    }
}

/// Store-wide context for generated-link rendering.
///
/// Invariant: each clustee closure maps to the closure id and every parsed entry that names it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GeneratedLinkIndex {
    cliques_by_closure: BTreeMap<crate::EntryId, BTreeSet<crate::EntryId>>,
}

impl GeneratedLinkIndex {
    /// Construct a generated-link index from parsed entries.
    pub fn from_entries(entries: &[Entry]) -> Self {
        let mut cliques_by_closure = BTreeMap::<crate::EntryId, BTreeSet<crate::EntryId>>::new();
        for entry in entries {
            for closure in &entry.metadata.clustee {
                let clique = cliques_by_closure.entry(closure.clone()).or_default();
                clique.insert(closure.clone());
                clique.insert(entry.id.clone());
            }
        }
        Self { cliques_by_closure }
    }

    /// Render the generated-link footer for one entry using this store-wide index.
    pub fn render_entry(&self, entry: &Entry, settings: &GeneratedLinkSettings) -> String {
        let mut out = String::new();
        out.push_str(BEGIN_LINKS_GUARD);
        out.push('\n');

        let mut rendered = 0_usize;
        let mut seen = BTreeSet::new();
        if settings.category {
            rendered += render_links(&mut out, entry.metadata.category.iter(), &mut seen);
        }
        if settings.clustee {
            if settings.clique {
                rendered += render_links(&mut out, self.clique_targets(entry).iter(), &mut seen);
            } else {
                rendered += render_links(&mut out, entry.metadata.clustee.iter(), &mut seen);
            }
        }
        if settings.refiner {
            rendered += render_links(&mut out, entry.metadata.refiner.iter(), &mut seen);
        }
        if rendered == 0 {
            out.push_str("- none\n");
        }

        out.push_str(END_LINKS_GUARD);
        out
    }

    fn clique_targets(&self, entry: &Entry) -> BTreeSet<crate::EntryId> {
        let mut targets = BTreeSet::new();
        for closure in &entry.metadata.clustee {
            if let Some(clique) = self.cliques_by_closure.get(closure) {
                targets.extend(clique.iter().filter(|id| *id != &entry.id).cloned());
            }
        }
        if let Some(clique) = self.cliques_by_closure.get(&entry.id) {
            targets.extend(clique.iter().filter(|id| *id != &entry.id).cloned());
        }
        targets
    }
}

/// Opening guard for Sirno-owned generated links.
pub const BEGIN_LINKS_GUARD: &str = "> **Sirno generated links begin. Do not edit this section.**";
/// Closing guard for Sirno-owned generated links.
pub const END_LINKS_GUARD: &str = "> **Sirno generated links end.**";

const GENERATED_LINK_DIVIDER: &str = "---";

/// Render the generated-link footer for one entry using only that entry as context.
///
/// Use `GeneratedLinkIndex::from_entries` when clique expansion needs the full store.
pub fn render_generated_links(entry: &Entry, settings: &GeneratedLinkSettings) -> String {
    GeneratedLinkIndex::from_entries(std::slice::from_ref(entry)).render_entry(entry, settings)
}

/// Validate generated-link guard boundaries in an entry body.
pub fn validate_generated_links(body: &str) -> Result<(), GeneratedLinkError> {
    validate_generated_link_bounds(body).map(|_| ())
}

/// Returns true when an existing generated-link region differs from `expected`.
///
/// Entries without a generated-link region are not stale.
pub fn generated_links_are_stale(body: &str, expected: &str) -> Result<bool, GeneratedLinkError> {
    let Some(bounds) = validate_generated_link_bounds(body)? else {
        return Ok(false);
    };
    Ok(&body[bounds.region_start..bounds.region_end] != expected)
}

fn validate_generated_link_bounds(
    body: &str,
) -> Result<Option<GeneratedLinkBounds>, GeneratedLinkError> {
    let begin = guard_positions(body, BEGIN_LINKS_GUARD);
    let end = guard_positions(body, END_LINKS_GUARD);
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
    Ok(Some(GeneratedLinkBounds { region_start: line_start(body, begin), region_end: end }))
}

/// Apply generated links to an entry body.
///
/// If no generated-link region exists, one is appended.
/// If one valid generated-link region exists, only that region is replaced.
pub fn apply_generated_links(body: &str, footer: &str) -> Result<String, GeneratedLinkError> {
    validate_generated_links(body)?;
    let Some(bounds) = validate_generated_link_bounds(body)? else {
        return Ok(append_footer(body, footer));
    };
    let region_end = next_line_start(body, bounds.region_end);
    let before = body[..bounds.region_start].trim_end_matches('\n');
    let after = body[region_end..].trim_start_matches('\n');

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

/// Delete generated links from an entry body.
///
/// If no generated-link region exists, the body is returned unchanged.
pub fn delete_generated_links(body: &str) -> Result<String, GeneratedLinkError> {
    let Some(bounds) = validate_generated_link_bounds(body)? else {
        return Ok(body.to_owned());
    };
    let region_end = next_line_start(body, bounds.region_end);
    let before = body[..bounds.region_start].trim_end_matches('\n');
    let after = body[region_end..].trim_start_matches('\n');

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
    if after.is_empty() && body.ends_with('\n') && !out.is_empty() {
        out.push('\n');
    }
    Ok(out)
}

fn render_links<'a>(
    out: &mut String, ids: impl IntoIterator<Item = &'a crate::EntryId>,
    seen: &mut BTreeSet<crate::EntryId>,
) -> usize {
    let mut rendered = 0_usize;
    for id in ids {
        if !seen.insert(id.clone()) {
            continue;
        }
        out.push_str("- ");
        out.push_str(&format!("[{}]({}.md)", id.as_str(), id.as_str()));
        out.push('\n');
        rendered += 1;
    }
    rendered
}

fn append_footer(body: &str, footer: &str) -> String {
    let before = body.trim_end_matches('\n');
    let mut out = String::new();
    if !before.is_empty() {
        out.push_str(before);
        out.push_str("\n\n");
        if !ends_with_divider(before) {
            out.push_str(GENERATED_LINK_DIVIDER);
            out.push_str("\n\n");
        }
    }
    out.push_str(footer);
    out.push('\n');
    out
}

fn ends_with_divider(body: &str) -> bool {
    body.lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.trim() == GENERATED_LINK_DIVIDER)
}

fn guard_positions(body: &str, guard: &str) -> Vec<usize> {
    body.match_indices(guard).map(|(index, _)| index).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GeneratedLinkBounds {
    region_start: usize,
    region_end: usize,
}

fn line_start(body: &str, index: usize) -> usize {
    body[..index].rfind('\n').map_or(0, |position| position + 1)
}

fn next_line_start(body: &str, index: usize) -> usize {
    body[index..].find('\n').map_or(body.len(), |position| index + position + 1)
}

/// Error raised by generated-link footer handling.
#[derive(Debug, Error, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entry, EntryId, EntryMetadata};

    fn id(raw: &str) -> EntryId {
        EntryId::new(raw).unwrap()
    }

    fn entry() -> Entry {
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.category.push(id("meta"));
        metadata.clustee.push(id("core"));
        metadata.refiner.push(id("relation"));
        Entry::new(id("concept"), metadata, "Body.\n")
    }

    #[test]
    fn default_settings_render_only_clustee_links() {
        let footer = render_generated_links(&entry(), &GeneratedLinkSettings::default());

        assert!(!footer.contains("[meta](meta.md)"));
        assert!(footer.contains("- [core](core.md)"));
        assert!(!footer.contains("[relation](relation.md)"));
        assert!(!footer.contains("## Sirno Links"));
        assert!(!footer.contains("clustee:"));
        assert!(footer.contains(BEGIN_LINKS_GUARD));
        assert!(footer.contains(END_LINKS_GUARD));
        assert!(footer.contains("> **Sirno generated links begin."));
    }

    #[test]
    fn settings_can_enable_each_relation_field() {
        let settings =
            GeneratedLinkSettings { category: true, clustee: true, clique: false, refiner: true };
        let footer = render_generated_links(&entry(), &settings);

        assert!(footer.contains("- [meta](meta.md)"));
        assert!(footer.contains("- [core](core.md)"));
        assert!(footer.contains("- [relation](relation.md)"));
        assert!(!footer.contains("category:"));
        assert!(!footer.contains("clustee:"));
        assert!(!footer.contains("refiner:"));
    }

    #[test]
    fn repeated_targets_render_once() {
        let mut entry = entry();
        entry.metadata.category.push(id("core"));
        let settings =
            GeneratedLinkSettings { category: true, clustee: true, clique: false, refiner: false };

        let footer = render_generated_links(&entry, &settings);

        assert_eq!(footer.matches("[core](core.md)").count(), 1);
    }

    #[test]
    fn clique_setting_expands_clustee_closures_to_edges() {
        let settings =
            GeneratedLinkSettings { category: false, clustee: true, clique: true, refiner: false };

        let closure = Entry::new(
            id("core"),
            EntryMetadata::new("Core", "A clique closure.").unwrap(),
            "Body.\n",
        );
        let mut left_metadata = EntryMetadata::new("Left", "A clique member.").unwrap();
        left_metadata.clustee.push(id("core"));
        let left = Entry::new(id("left"), left_metadata, "Body.\n");
        let mut right_metadata = EntryMetadata::new("Right", "A clique member.").unwrap();
        right_metadata.clustee.push(id("core"));
        let right = Entry::new(id("right"), right_metadata, "Body.\n");
        let mut outside_metadata = EntryMetadata::new("Outside", "Another member.").unwrap();
        outside_metadata.clustee.push(id("other"));
        let outside = Entry::new(id("outside"), outside_metadata, "Body.\n");
        let entries = vec![closure.clone(), left.clone(), right.clone(), outside];
        let index = GeneratedLinkIndex::from_entries(&entries);

        let closure_footer = index.render_entry(&closure, &settings);
        let left_footer = index.render_entry(&left, &settings);

        assert!(closure_footer.contains("- [left](left.md)"));
        assert!(closure_footer.contains("- [right](right.md)"));
        assert!(!closure_footer.contains("[core](core.md)"));
        assert!(!closure_footer.contains("[outside](outside.md)"));
        assert!(left_footer.contains("- [core](core.md)"));
        assert!(left_footer.contains("- [right](right.md)"));
        assert!(!left_footer.contains("[left](left.md)"));
        assert!(!left_footer.contains("[outside](outside.md)"));
    }

    #[test]
    fn renders_none_when_entry_has_no_entry_relations() {
        let metadata = EntryMetadata::new("Meta", "A category.").unwrap();
        let entry = Entry::new(EntryId::new("meta").unwrap(), metadata, "Body.\n");

        let footer = render_generated_links(&entry, &GeneratedLinkSettings::default());

        assert!(footer.contains("- none"));
    }

    #[test]
    fn appends_footer_when_missing() {
        let footer = render_generated_links(&entry(), &GeneratedLinkSettings::default());

        let body = apply_generated_links("Body.\n", &footer).unwrap();

        assert_eq!(body, format!("Body.\n\n---\n\n{footer}\n"));
        assert_eq!(body.matches(BEGIN_LINKS_GUARD).count(), 1);
    }

    #[test]
    fn appends_footer_without_duplicate_divider() {
        let footer = render_generated_links(&entry(), &GeneratedLinkSettings::default());

        let body = apply_generated_links("Body.\n\n---\n", &footer).unwrap();

        assert_eq!(body, format!("Body.\n\n---\n\n{footer}\n"));
    }

    #[test]
    fn replaces_only_existing_footer_region() {
        let old = format!("{BEGIN_LINKS_GUARD}\nold\n{END_LINKS_GUARD}\n");
        let body = format!("Before.\n\n{old}\nAfter.\n");
        let footer = render_generated_links(&entry(), &GeneratedLinkSettings::default());

        let body = apply_generated_links(&body, &footer).unwrap();

        assert!(body.starts_with("Before.\n\n"));
        assert!(body.ends_with("After.\n"));
        assert!(!body.contains("old"));
        assert_eq!(body.matches(BEGIN_LINKS_GUARD).count(), 1);
    }

    #[test]
    fn deletes_existing_footer_region() {
        let footer = render_generated_links(&entry(), &GeneratedLinkSettings::default());
        let body = apply_generated_links("Body.\n", &footer).unwrap();

        let body = delete_generated_links(&body).unwrap();

        assert_eq!(body, "Body.\n\n---\n");
        assert!(!body.contains(BEGIN_LINKS_GUARD));
    }

    #[test]
    fn deletes_footer_without_touching_following_body() {
        let footer = render_generated_links(&entry(), &GeneratedLinkSettings::default());
        let body = format!("Before.\n\n{footer}\nAfter.\n");

        let body = delete_generated_links(&body).unwrap();

        assert_eq!(body, "Before.\n\nAfter.\n");
    }

    #[test]
    fn delete_is_noop_when_footer_is_missing() {
        let body = delete_generated_links("Body.\n").unwrap();

        assert_eq!(body, "Body.\n");
    }

    #[test]
    fn reports_generated_links_staleness() {
        let expected = render_generated_links(&entry(), &GeneratedLinkSettings::default());
        let current = apply_generated_links("Body.\n", &expected).unwrap();
        let stale = apply_generated_links(
            "Body.\n",
            &render_generated_links(
                &entry(),
                &GeneratedLinkSettings {
                    category: true,
                    clustee: true,
                    clique: false,
                    refiner: true,
                },
            ),
        )
        .unwrap();

        assert!(!generated_links_are_stale("Body.\n", &expected).unwrap());
        assert!(!generated_links_are_stale(&current, &expected).unwrap());
        assert!(generated_links_are_stale(&stale, &expected).unwrap());
    }

    #[test]
    fn rejects_missing_end_guard() {
        let error = validate_generated_links(BEGIN_LINKS_GUARD).unwrap_err();

        assert_eq!(error, GeneratedLinkError::MissingEnd);
    }

    #[test]
    fn rejects_duplicate_begin_guard() {
        let body = format!("{BEGIN_LINKS_GUARD}\n{BEGIN_LINKS_GUARD}\n{END_LINKS_GUARD}\n");
        let error = validate_generated_links(&body).unwrap_err();

        assert_eq!(error, GeneratedLinkError::DuplicateBegin);
    }
}
