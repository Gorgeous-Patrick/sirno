//! Generated Markdown links for entries.
//!
//! Sirno owns only the guard-bounded generated-link region.
//! Prose outside the region remains user-owned.

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
    /// Include `refiner` relation targets.
    pub refiner: bool,
}

impl Default for GeneratedLinkSettings {
    fn default() -> Self {
        Self { category: false, clustee: true, refiner: false }
    }
}

/// Opening guard for Sirno-owned generated links.
pub const BEGIN_LINKS_GUARD: &str = "> **Sirno generated links begin. Do not edit this section.**";
/// Closing guard for Sirno-owned generated links.
pub const END_LINKS_GUARD: &str = "> **Sirno generated links end.**";

/// Render the generated-link footer for one entry.
pub fn render_generated_links(entry: &Entry, settings: &GeneratedLinkSettings) -> String {
    let mut out = String::new();
    out.push_str(BEGIN_LINKS_GUARD);
    out.push('\n');
    out.push_str("## Sirno Links\n\n");

    let mut rendered = 0_usize;
    if settings.category {
        rendered += render_relation(&mut out, "category", &entry.metadata.category);
    }
    if settings.clustee {
        rendered += render_relation(&mut out, "clustee", &entry.metadata.clustee);
    }
    if settings.refiner {
        rendered += render_relation(&mut out, "refiner", &entry.metadata.refiner);
    }
    if rendered == 0 {
        out.push_str("- none\n");
    }

    out.push_str(END_LINKS_GUARD);
    out
}

/// Validate generated-link guard boundaries in an entry body.
pub fn validate_generated_links(body: &str) -> Result<(), GeneratedLinkError> {
    let begin = guard_positions(body, BEGIN_LINKS_GUARD);
    let end = guard_positions(body, END_LINKS_GUARD);
    match (begin.as_slice(), end.as_slice()) {
        | ([], []) => Ok(()),
        | ([begin], [end]) if begin < end => Ok(()),
        | ([begin], [end]) if begin > end => Err(GeneratedLinkError::EndBeforeBegin),
        | ([], [_]) => Err(GeneratedLinkError::MissingBegin),
        | ([_], []) => Err(GeneratedLinkError::MissingEnd),
        | (_, _) if begin.len() > 1 => Err(GeneratedLinkError::DuplicateBegin),
        | (_, _) if end.len() > 1 => Err(GeneratedLinkError::DuplicateEnd),
        | _ => Err(GeneratedLinkError::Malformed),
    }
}

/// Apply generated links to an entry body.
///
/// If no generated-link region exists, one is appended.
/// If one valid generated-link region exists, only that region is replaced.
pub fn apply_generated_links(body: &str, footer: &str) -> Result<String, GeneratedLinkError> {
    validate_generated_links(body)?;
    let begin = guard_positions(body, BEGIN_LINKS_GUARD);
    if begin.is_empty() {
        return Ok(append_footer(body, footer));
    }

    let begin = begin[0];
    let end = guard_positions(body, END_LINKS_GUARD)[0] + END_LINKS_GUARD.len();
    let region_start = line_start(body, begin);
    let region_end = next_line_start(body, end);
    let before = body[..region_start].trim_end_matches('\n');
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

fn render_relation(out: &mut String, label: &str, ids: &[crate::EntryId]) -> usize {
    if ids.is_empty() {
        return 0;
    }

    let links = ids
        .iter()
        .map(|id| format!("[{}]({}.md)", id.as_str(), id.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str("- ");
    out.push_str(label);
    out.push_str(": ");
    out.push_str(&links);
    out.push('\n');
    ids.len()
}

fn append_footer(body: &str, footer: &str) -> String {
    let before = body.trim_end_matches('\n');
    let mut out = String::new();
    if !before.is_empty() {
        out.push_str(before);
        out.push_str("\n\n");
    }
    out.push_str(footer);
    out.push('\n');
    out
}

fn guard_positions(body: &str, guard: &str) -> Vec<usize> {
    body.match_indices(guard).map(|(index, _)| index).collect()
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

    fn entry() -> Entry {
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.category.push(EntryId::new("meta").unwrap());
        metadata.clustee.push(EntryId::new("core").unwrap());
        metadata.refiner.push(EntryId::new("relation").unwrap());
        Entry::new(EntryId::new("concept").unwrap(), metadata, "Body.\n")
    }

    #[test]
    fn default_settings_render_only_clustee_links() {
        let footer = render_generated_links(&entry(), &GeneratedLinkSettings::default());

        assert!(!footer.contains("- category: [meta](meta.md)"));
        assert!(footer.contains("- clustee: [core](core.md)"));
        assert!(!footer.contains("- refiner: [relation](relation.md)"));
        assert!(footer.contains(BEGIN_LINKS_GUARD));
        assert!(footer.contains(END_LINKS_GUARD));
        assert!(footer.contains("> **Sirno generated links begin."));
    }

    #[test]
    fn settings_can_enable_each_relation_field() {
        let settings = GeneratedLinkSettings { category: true, clustee: true, refiner: true };
        let footer = render_generated_links(&entry(), &settings);

        assert!(footer.contains("- category: [meta](meta.md)"));
        assert!(footer.contains("- clustee: [core](core.md)"));
        assert!(footer.contains("- refiner: [relation](relation.md)"));
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

        assert_eq!(body.matches(BEGIN_LINKS_GUARD).count(), 1);
        assert!(body.starts_with("Body.\n\n"));
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
