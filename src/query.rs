//! Query predicates for Sirno entries.
//!
//! Queries are typed predicates over parsed Markdown entries.
//! They select entries and leave presentation to the caller.

use std::collections::BTreeMap;

use tracing::trace;

use crate::entry::Entry;
use crate::identifier::EntryAddress;

/// Case-insensitive text term for an entry query.
///
/// Empty terms are ignored when a query stores text terms.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:query:begin
pub struct EntryTextTerm {
    normalized: String,
}
// sirno:witness:query:end

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

/// Structural link matcher for an entry query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryStructuralMatcher {
    /// The relation has any listed target.
    Targets(Vec<EntryAddress>),
    /// The relation is present with any target count.
    Present,
    /// The relation is present with no targets.
    Empty,
    /// The relation is absent.
    Missing,
}

impl EntryStructuralMatcher {
    fn matches(&self, entry_targets: Option<&[EntryAddress]>) -> bool {
        match self {
            | Self::Targets(query_targets) => entry_targets.is_some_and(|entry_targets| {
                query_targets.iter().any(|target| entry_targets.contains(target))
            }),
            | Self::Present => entry_targets.is_some(),
            | Self::Empty => entry_targets.is_some_and(|entry_targets| entry_targets.is_empty()),
            | Self::Missing => entry_targets.is_none(),
        }
    }
}

/// Exact predicate over Sirno entries.
///
/// Text terms are conjunctive.
/// Distinct metadata fields are conjunctive.
/// Structural matchers inside one link relation are disjunctive.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
// sirno:witness:query:begin
pub struct EntryQuery {
    text_terms: Vec<EntryTextTerm>,
    structural: BTreeMap<String, Vec<EntryStructuralMatcher>>,
}
// sirno:witness:query:end

impl EntryQuery {
    /// Construct an empty query that matches every entry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set text terms matched against id, intrinsic field values, and body.
    pub fn with_text_terms(mut self, terms: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.text_terms =
            terms.into_iter().map(EntryTextTerm::new).filter(|term| !term.is_empty()).collect();
        self
    }

    /// Add a target matcher for one link relation.
    pub fn with_structural_targets(
        mut self, field: impl Into<String>, targets: impl IntoIterator<Item = EntryAddress>,
    ) -> Self {
        let targets = targets.into_iter().collect::<Vec<_>>();
        if !targets.is_empty() {
            self.structural
                .entry(field.into())
                .or_default()
                .push(EntryStructuralMatcher::Targets(targets));
        }
        self
    }

    /// Add one matcher for one link relation.
    pub fn with_structural_matcher(
        mut self, field: impl Into<String>, matcher: EntryStructuralMatcher,
    ) -> Self {
        self.structural.entry(field.into()).or_default().push(matcher);
        self
    }

    /// Returns true when this query selects the entry.
    // sirno:witness:query:begin
    pub fn matches(&self, entry: &Entry) -> bool {
        self.matches_text(entry)
            && self.structural.iter().all(|(field, matchers)| {
                let entry_targets = entry.metadata.structural_field(field);
                matchers.iter().any(|matcher| matcher.matches(entry_targets))
            })
    }
    // sirno:witness:query:end

    /// Return entries selected by this exact query in input order.
    // sirno:witness:query:begin
    pub fn select_entries<'a>(
        &self, entries: impl IntoIterator<Item = &'a Entry>,
    ) -> Vec<&'a Entry> {
        let entries = entries.into_iter().collect::<Vec<_>>();
        trace!("query_entries begin: entries={}", entries.len());
        let matches = entries.into_iter().filter(|entry| self.matches(entry)).collect::<Vec<_>>();
        trace!("query_entries end: matches={}", matches.len());
        matches
    }
    // sirno:witness:query:end

    fn matches_text(&self, entry: &Entry) -> bool {
        if self.text_terms.is_empty() {
            return true;
        }

        let haystack = entry.query_text();
        self.text_terms.iter().all(|term| term.matches(&haystack))
    }
}

/// Vague predicate over Sirno entries.
///
/// Vague text terms match an entry plus the ids and intrinsic values of structural link targets.
/// Each text term must match somewhere in that expanded text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
// sirno:witness:query:begin
pub struct VagueEntryQuery {
    text_terms: Vec<EntryTextTerm>,
}
// sirno:witness:query:end

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
    // sirno:witness:query:begin
    pub fn matches<'a>(
        &self, entry: &'a Entry, entries_by_id: &BTreeMap<&'a EntryAddress, &'a Entry>,
    ) -> bool {
        if self.text_terms.is_empty() {
            return true;
        }

        let haystack = entry.vague_query_text(entries_by_id);
        self.text_terms.iter().all(|term| term.matches(&haystack))
    }
    // sirno:witness:query:end

    /// Return entries selected by this vague query in input order.
    // sirno:witness:query:begin
    pub fn select_entries<'a>(&self, entries: &'a [Entry]) -> Vec<&'a Entry> {
        trace!("vague_query_entries begin: entries={}", entries.len());
        let entries_by_id =
            entries.iter().map(|entry| (&entry.id, entry)).collect::<BTreeMap<_, _>>();
        let matches =
            entries.iter().filter(|entry| self.matches(entry, &entries_by_id)).collect::<Vec<_>>();
        trace!("vague_query_entries end: matches={}", matches.len());
        matches
    }
    // sirno:witness:query:end
}

// sirno:witness:query:begin
impl Entry {
    fn query_text(&self) -> String {
        let mut text = self.id.to_string();
        push_intrinsic_values(&mut text, self);
        text.push('\n');
        text.push_str(&self.body);
        text.to_lowercase()
    }

    fn vague_query_text(&self, entries_by_id: &BTreeMap<&EntryAddress, &Entry>) -> String {
        let mut text = self.query_text();
        for target in self.metadata.structural_targets().map(|(_, target)| target) {
            text.push('\n');
            text.push_str(target.as_str());
            if let Some(target_entry) = entries_by_id.get(target) {
                push_intrinsic_values(&mut text, target_entry);
            }
        }
        text.to_lowercase()
    }
}

fn push_intrinsic_values(text: &mut String, entry: &Entry) {
    for (_, value) in entry.metadata.intrinsic_fields() {
        text.push('\n');
        text.push_str(value);
    }
}
// sirno:witness:query:end

#[cfg(test)]
mod tests {
    use super::*;

    const FIELD_KIND: &str = "kind";
    const FIELD_AREA: &str = "area";

    fn id(raw: &str) -> EntryAddress {
        EntryAddress::new(raw).unwrap()
    }

    fn entry(raw_id: &str, name: &str, desc: &str, body: &str) -> Entry {
        Entry::new(id(raw_id), crate::entry::seed_intrinsic_metadata(name, desc).unwrap(), body)
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
    fn text_terms_match_dynamic_intrinsic_fields() {
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept
            .metadata
            .intrinsic
            .insert("summary".to_owned(), "Dynamic project route.".to_owned());

        let query = EntryQuery::new().with_text_terms(["dynamic"]);

        assert!(query.matches(&concept));
    }

    #[test]
    fn structural_values_are_disjunctive_inside_one_field() {
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept.metadata.push_structural_target(FIELD_KIND, id("meta"));

        let query =
            EntryQuery::new().with_structural_targets(FIELD_KIND, [id("narrative"), id("meta")]);

        assert!(query.matches(&concept));
    }

    #[test]
    fn structural_state_matchers_preserve_field_presence() {
        let missing = entry("missing", "Missing", "No field.", "");
        let mut empty = entry("empty", "Empty", "Present empty field.", "");
        empty.metadata.set_structural_targets(FIELD_KIND, []);
        let mut targeted = entry("targeted", "Targeted", "Present field.", "");
        targeted.metadata.push_structural_target(FIELD_KIND, id("meta"));

        let present =
            EntryQuery::new().with_structural_matcher(FIELD_KIND, EntryStructuralMatcher::Present);
        let empty_query =
            EntryQuery::new().with_structural_matcher(FIELD_KIND, EntryStructuralMatcher::Empty);
        let missing_query =
            EntryQuery::new().with_structural_matcher(FIELD_KIND, EntryStructuralMatcher::Missing);

        assert!(!present.matches(&missing));
        assert!(present.matches(&empty));
        assert!(present.matches(&targeted));
        assert!(!empty_query.matches(&missing));
        assert!(empty_query.matches(&empty));
        assert!(!empty_query.matches(&targeted));
        assert!(missing_query.matches(&missing));
        assert!(!missing_query.matches(&empty));
        assert!(!missing_query.matches(&targeted));
    }

    #[test]
    fn structural_state_and_target_matchers_are_disjunctive_inside_one_field() {
        let mut empty = entry("empty", "Empty", "Present empty field.", "");
        empty.metadata.set_structural_targets(FIELD_KIND, []);
        let mut targeted = entry("targeted", "Targeted", "Present field.", "");
        targeted.metadata.push_structural_target(FIELD_KIND, id("meta"));

        let query = EntryQuery::new()
            .with_structural_targets(FIELD_KIND, [id("meta")])
            .with_structural_matcher(FIELD_KIND, EntryStructuralMatcher::Empty);

        assert!(query.matches(&empty));
        assert!(query.matches(&targeted));
    }

    #[test]
    fn structural_fields_are_conjunctive_across_fields() {
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept.metadata.push_structural_target(FIELD_KIND, id("meta"));
        concept.metadata.push_structural_target(FIELD_AREA, id("knowledge"));

        let matching = EntryQuery::new()
            .with_structural_targets(FIELD_KIND, [id("meta")])
            .with_structural_targets(FIELD_AREA, [id("knowledge")]);
        let missing = EntryQuery::new()
            .with_structural_targets(FIELD_KIND, [id("meta")])
            .with_structural_targets(FIELD_AREA, [id("reader")]);

        assert!(matching.matches(&concept));
        assert!(!missing.matches(&concept));
    }

    #[test]
    fn query_entries_preserves_input_order() {
        let first = entry("first", "First", "A first idea.", "");
        let second = entry("second", "Second", "A second idea.", "");
        let entries = [&first, &second];

        let matches = EntryQuery::new().with_text_terms(["idea"]).select_entries(entries);

        assert_eq!(matches, vec![&first, &second]);
    }

    #[test]
    fn vague_query_matches_structural_target_id() {
        let meta = entry("meta", "Meta", "A kind.", "");
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept.metadata.push_structural_target(FIELD_KIND, id("meta"));
        let entries = vec![concept, meta];

        let matches = VagueEntryQuery::new().with_text_terms(["meta"]).select_entries(&entries);

        assert_eq!(
            matches.iter().map(|entry| &entry.id).collect::<Vec<_>>(),
            vec![&id("concept"), &id("meta")]
        );
    }

    #[test]
    fn vague_query_matches_structural_target_metadata() {
        let meta = entry("meta", "Meta", "Project vocabulary.", "");
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept.metadata.push_structural_target(FIELD_KIND, id("meta"));
        let entries = vec![concept, meta];

        let matches =
            VagueEntryQuery::new().with_text_terms(["vocabulary"]).select_entries(&entries);

        assert_eq!(
            matches.iter().map(|entry| &entry.id).collect::<Vec<_>>(),
            vec![&id("concept"), &id("meta")]
        );
    }

    #[test]
    fn vague_query_matches_structural_target_dynamic_intrinsic_fields() {
        let mut meta = entry("meta", "Meta", "Project vocabulary.", "");
        meta.metadata.intrinsic.insert("summary".to_owned(), "Dynamic glossary.".to_owned());
        let mut concept = entry("concept", "Concept", "A named idea.", "");
        concept.metadata.push_structural_target(FIELD_KIND, id("meta"));
        let entries = vec![concept, meta];

        let matches = VagueEntryQuery::new().with_text_terms(["glossary"]).select_entries(&entries);

        assert_eq!(
            matches.iter().map(|entry| &entry.id).collect::<Vec<_>>(),
            vec![&id("concept"), &id("meta")]
        );
    }
}
