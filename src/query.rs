//! Query predicates for Sirno entries.
//!
//! Queries are typed predicates over parsed Markdown entries.
//! They select entries and leave presentation to the caller.

use std::collections::BTreeMap;

use tracing::trace;

use crate::entry::Entry;
use crate::id::EntryId;

/// Case-insensitive text term for an entry query.
///
/// Empty terms are ignored when a query stores text terms.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:start query
pub struct EntryTextTerm {
    normalized: String,
}
// sirno:witness:end

impl EntryTextTerm {
    /// Construct a text term using Unicode lowercase conversion.
    pub fn new(raw: impl Into<String>) -> Self {
        Self { normalized: raw.into().to_lowercase() }
    }

    /// Normalized text used for matching.
    pub fn normalized(&self) -> &str {
        &self.normalized
    }

    fn is_empty(&self) -> bool {
        self.normalized.is_empty()
    }

    fn matches(&self, haystack: &str) -> bool {
        haystack.contains(&self.normalized)
    }
}

/// Exact predicate over Sirno entries.
///
/// Text terms are conjunctive.
/// Distinct metadata fields are conjunctive.
/// Repeated values inside one metadata field are disjunctive.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
// sirno:witness:start query
pub struct EntryQuery {
    text_terms: Vec<EntryTextTerm>,
    category: Vec<EntryId>,
    clustee: Vec<EntryId>,
    refiner: Vec<EntryId>,
    witness: bool,
}
// sirno:witness:end

impl EntryQuery {
    /// Construct an empty query that matches every entry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set text terms matched against id, name, description, and body.
    pub fn with_text_terms(mut self, terms: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.text_terms =
            terms.into_iter().map(EntryTextTerm::new).filter(|term| !term.is_empty()).collect();
        self
    }

    /// Set category targets.
    pub fn with_category(mut self, targets: impl IntoIterator<Item = EntryId>) -> Self {
        self.category = targets.into_iter().collect();
        self
    }

    /// Set clustee targets.
    pub fn with_clustee(mut self, targets: impl IntoIterator<Item = EntryId>) -> Self {
        self.clustee = targets.into_iter().collect();
        self
    }

    /// Set refiner targets.
    pub fn with_refiner(mut self, targets: impl IntoIterator<Item = EntryId>) -> Self {
        self.refiner = targets.into_iter().collect();
        self
    }

    /// Require the canonical witness marker.
    pub fn with_witness(mut self, witness: bool) -> Self {
        self.witness = witness;
        self
    }

    /// Returns true when this query selects the entry.
    // sirno:witness:start query
    pub fn matches(&self, entry: &Entry) -> bool {
        self.matches_text(entry)
            && matches_targets(&entry.metadata.category, &self.category)
            && matches_targets(&entry.metadata.clustee, &self.clustee)
            && matches_targets(&entry.metadata.refiner, &self.refiner)
            && (!self.witness || entry.metadata.witness.is_some())
    }
    // sirno:witness:end

    fn matches_text(&self, entry: &Entry) -> bool {
        if self.text_terms.is_empty() {
            return true;
        }

        let haystack = entry_text(entry);
        self.text_terms.iter().all(|term| term.matches(&haystack))
    }
}

/// Vague predicate over Sirno entries.
///
/// Vague text terms match an entry plus the ids, names, and descriptions of structural targets.
/// Each text term must match somewhere in that expanded text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
// sirno:witness:start query
pub struct VagueEntryQuery {
    text_terms: Vec<EntryTextTerm>,
}
// sirno:witness:end

impl VagueEntryQuery {
    /// Construct an empty vague query that matches every entry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set text terms matched against expanded entry text.
    pub fn with_text_terms(mut self, terms: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.text_terms =
            terms.into_iter().map(EntryTextTerm::new).filter(|term| !term.is_empty()).collect();
        self
    }

    /// Returns true when this query selects the entry.
    // sirno:witness:start query
    pub fn matches<'a>(
        &self, entry: &'a Entry, entries_by_id: &BTreeMap<&'a EntryId, &'a Entry>,
    ) -> bool {
        if self.text_terms.is_empty() {
            return true;
        }

        let haystack = vague_entry_text(entry, entries_by_id);
        self.text_terms.iter().all(|term| term.matches(&haystack))
    }
    // sirno:witness:end
}

/// Return entries selected by an exact query in input order.
// sirno:witness:start query
pub fn query_entries<'a>(
    entries: impl IntoIterator<Item = &'a Entry>, query: &EntryQuery,
) -> Vec<&'a Entry> {
    let entries = entries.into_iter().collect::<Vec<_>>();
    trace!("query_entries begin: entries={}", entries.len());
    let matches = entries.into_iter().filter(|entry| query.matches(entry)).collect::<Vec<_>>();
    trace!("query_entries end: matches={}", matches.len());
    matches
}
// sirno:witness:end

/// Return entries selected by a vague query in input order.
// sirno:witness:start query
pub fn vague_query_entries<'a>(entries: &'a [Entry], query: &VagueEntryQuery) -> Vec<&'a Entry> {
    trace!("vague_query_entries begin: entries={}", entries.len());
    let entries_by_id = entries.iter().map(|entry| (&entry.id, entry)).collect::<BTreeMap<_, _>>();
    let matches =
        entries.iter().filter(|entry| query.matches(entry, &entries_by_id)).collect::<Vec<_>>();
    trace!("vague_query_entries end: matches={}", matches.len());
    matches
}
// sirno:witness:end

fn matches_targets(entry_targets: &[EntryId], query_targets: &[EntryId]) -> bool {
    query_targets.is_empty() || query_targets.iter().any(|target| entry_targets.contains(target))
}

fn entry_text(entry: &Entry) -> String {
    format!("{}\n{}\n{}\n{}", entry.id, entry.metadata.name, entry.metadata.description, entry.body)
        .to_lowercase()
}

fn vague_entry_text(entry: &Entry, entries_by_id: &BTreeMap<&EntryId, &Entry>) -> String {
    let mut text = entry_text(entry);
    for target in entry.metadata.structural_targets().map(|(_, target)| target) {
        text.push('\n');
        text.push_str(target.as_str());
        if let Some(target_entry) = entries_by_id.get(target) {
            text.push('\n');
            text.push_str(&target_entry.metadata.name);
            text.push('\n');
            text.push_str(&target_entry.metadata.description);
        }
    }
    text.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{EntryMetadata, WitnessMarker};

    fn id(raw: &str) -> EntryId {
        EntryId::new(raw).unwrap()
    }

    fn entry(raw_id: &str, name: &str, description: &str, body: &str) -> Entry {
        Entry::new(id(raw_id), EntryMetadata::new(name, description).unwrap(), body)
    }

    #[test]
    fn empty_query_matches_every_entry() {
        let concept = entry("concept", "Concept", "A named idea.", "");

        assert!(EntryQuery::new().matches(&concept));
    }

    #[test]
    fn text_terms_match_entry_text_case_insensitively() {
        let concept = entry(
            "concept",
            "Concept",
            "A named idea.",
            "A cognitive route through project knowledge.",
        );

        let query = EntryQuery::new().with_text_terms(["ROUTE", "project"]);

        assert!(query.matches(&concept));
        assert!(!EntryQuery::new().with_text_terms(["missing"]).matches(&concept));
    }

    #[test]
    fn structural_values_are_disjunctive_inside_one_field() {
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept.metadata.category.push(id("meta"));

        let query = EntryQuery::new().with_category([id("narrative"), id("meta")]);

        assert!(query.matches(&concept));
    }

    #[test]
    fn structural_fields_are_conjunctive_across_fields() {
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept.metadata.category.push(id("meta"));
        concept.metadata.clustee.push(id("knowledge"));

        let matching =
            EntryQuery::new().with_category([id("meta")]).with_clustee([id("knowledge")]);
        let missing = EntryQuery::new().with_category([id("meta")]).with_clustee([id("reader")]);

        assert!(matching.matches(&concept));
        assert!(!missing.matches(&concept));
    }

    #[test]
    fn witness_filter_requires_marker() {
        let plain = entry("concept", "Concept", "A named idea.", "");
        let mut witnessed = entry("witnessed", "Witnessed", "A witnessed idea.", "");
        witnessed.metadata.witness = Some(WitnessMarker::Present);

        let query = EntryQuery::new().with_witness(true);

        assert!(!query.matches(&plain));
        assert!(query.matches(&witnessed));
    }

    #[test]
    fn query_entries_preserves_input_order() {
        let first = entry("first", "First", "A first idea.", "");
        let second = entry("second", "Second", "A second idea.", "");
        let entries = [&first, &second];

        let matches = query_entries(entries, &EntryQuery::new().with_text_terms(["idea"]));

        assert_eq!(matches, vec![&first, &second]);
    }

    #[test]
    fn vague_query_matches_structural_target_id() {
        let meta = entry("meta", "Meta", "A category.", "");
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept.metadata.category.push(id("meta"));
        let entries = vec![concept, meta];

        let matches =
            vague_query_entries(&entries, &VagueEntryQuery::new().with_text_terms(["meta"]));

        assert_eq!(
            matches.iter().map(|entry| &entry.id).collect::<Vec<_>>(),
            vec![&id("concept"), &id("meta")]
        );
    }

    #[test]
    fn vague_query_matches_structural_target_metadata() {
        let meta = entry("meta", "Meta", "Project vocabulary.", "");
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept.metadata.category.push(id("meta"));
        let entries = vec![concept, meta];

        let matches =
            vague_query_entries(&entries, &VagueEntryQuery::new().with_text_terms(["vocabulary"]));

        assert_eq!(
            matches.iter().map(|entry| &entry.id).collect::<Vec<_>>(),
            vec![&id("concept"), &id("meta")]
        );
    }
}
