//! Structural checks for Sirno entries.
//!
//! Sirno checks the shape of entries and structural link targets.
//! It does not decide whether prose is true or whether code satisfies a claim.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::entry::{DESC_FIELD, Entry, EntryMetaType, NAME_FIELD};
use crate::identifier::EntryAddress;
use crate::structural::StructuralSettings;

const CATEGORY_FIELD: &str = "category";
const INTRINSIC_META_FIELDS: [&str; 2] = [NAME_FIELD, DESC_FIELD];

/// Boundary at which Sirno checks structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckMode {
    /// Editing checks keep local movement fast.
    Edit,
    /// Review checks treat dangling structural link references as errors.
    Review,
}

impl CheckMode {
    /// Diagnostic severity used by this check boundary.
    pub fn severity(self) -> CheckSeverity {
        match self {
            | Self::Edit => CheckSeverity::Warning,
            | Self::Review => CheckSeverity::Error,
        }
    }

    // sirno:witness:structural-check:begin
    /// Check structural link targets for a set of entries.
    ///
    /// Parsing already enforces required fields, accepted field shapes, and valid path syntax.
    /// This pass checks configured link relation entries, typed metadata, and target addresses.
    pub fn check_entries<'a>(
        self, entries: impl IntoIterator<Item = &'a Entry>, structural: &StructuralSettings,
    ) -> CheckReport {
        self.check_entries_with_structural_inhabitance(entries, structural, true)
    }

    /// Check structural link targets, with explicit structural-inhabitance policy.
    ///
    /// Structural inhabitance requires each configured link relation to name a structural entry.
    pub fn check_entries_with_structural_inhabitance<'a>(
        self, entries: impl IntoIterator<Item = &'a Entry>, structural: &StructuralSettings,
        structural_inhabitance: bool,
    ) -> CheckReport {
        let entries = entries.into_iter().collect::<Vec<_>>();
        let entries_by_id =
            entries.iter().map(|entry| (entry.id.clone(), *entry)).collect::<BTreeMap<_, _>>();
        let severity = self.severity();

        let mut report = CheckReport::new();
        if structural_inhabitance {
            for (field, _) in structural.fields() {
                let relation_entry = structural
                    .entry_for_field(field)
                    .expect("configured relation has defining entry");
                let Some(entry) = entries_by_id.get(relation_entry).copied() else {
                    report.push(CheckDiagnostic {
                        severity,
                        kind: CheckDiagnosticKind::MissingStructuralFieldEntry,
                        entry: None,
                        field: field.to_owned(),
                        target: Some(relation_entry.clone()),
                    });
                    continue;
                };
                if !entry.metadata.meta.is_structural_relation() {
                    report.push(CheckDiagnostic {
                        severity,
                        kind: CheckDiagnosticKind::MissingStructuralMeta,
                        entry: Some(entry.id.clone()),
                        field: field.to_owned(),
                        target: Some(relation_entry.clone()),
                    });
                }
            }
        }
        for entry in &entries {
            if Self::is_intrinsic_meta_field(entry.id.as_str())
                && !entry.metadata.meta.is_intrinsic_field()
            {
                report.push(CheckDiagnostic {
                    severity,
                    kind: CheckDiagnosticKind::MissingIntrinsicMeta,
                    entry: Some(entry.id.clone()),
                    field: entry.id.as_str().to_owned(),
                    target: None,
                });
            }
            if entry.metadata.meta.entry_type == Some(EntryMetaType::Intrinsic)
                && !Self::is_intrinsic_meta_field(entry.id.as_str())
            {
                report.push(CheckDiagnostic {
                    severity,
                    kind: CheckDiagnosticKind::UnregisteredIntrinsicMeta,
                    entry: Some(entry.id.clone()),
                    field: entry.id.as_str().to_owned(),
                    target: None,
                });
            }
            if entry.metadata.meta.is_structural_relation() && !structural.contains_entry(&entry.id)
            {
                report.push(CheckDiagnostic {
                    severity,
                    kind: CheckDiagnosticKind::UnregisteredStructuralMeta,
                    entry: Some(entry.id.clone()),
                    field: entry.id.as_str().to_owned(),
                    target: None,
                });
            }
            for (field, targets) in entry.metadata.structural_fields() {
                if !structural.contains_field(field) {
                    report.push(CheckDiagnostic {
                        severity: CheckSeverity::Warning,
                        kind: CheckDiagnosticKind::UnconfiguredStructuralField,
                        entry: Some(entry.id.clone()),
                        field: field.to_owned(),
                        target: None,
                    });
                    continue;
                }
                for target in targets {
                    if !entries_by_id.contains_key(target) {
                        report.push(CheckDiagnostic {
                            severity,
                            kind: CheckDiagnosticKind::MissingTarget,
                            entry: Some(entry.id.clone()),
                            field: field.to_owned(),
                            target: Some(target.clone()),
                        });
                    }
                }
            }
        }
        self.check_category_targets(&entries_by_id, structural, &mut report);
        report
    }

    fn check_category_targets(
        self, entries_by_id: &BTreeMap<EntryAddress, &Entry>, structural: &StructuralSettings,
        report: &mut CheckReport,
    ) {
        let category_id =
            structural.entry_for_field(CATEGORY_FIELD).cloned().unwrap_or_else(|| {
                EntryAddress::new(CATEGORY_FIELD).expect("built-in category entry address is valid")
            });
        let category_targets = entries_by_id
            .values()
            .flat_map(|entry| entry.metadata.structural_targets_for(CATEGORY_FIELD))
            .cloned()
            .collect::<BTreeSet<_>>();
        if category_targets.is_empty() && !structural.contains_field(CATEGORY_FIELD) {
            return;
        }
        if !entries_by_id.contains_key(&category_id) {
            report.push(CheckDiagnostic {
                severity: CheckSeverity::Warning,
                kind: CheckDiagnosticKind::MissingCategoryEntry,
                entry: None,
                field: CATEGORY_FIELD.to_owned(),
                target: Some(category_id.clone()),
            });
        }
        for target in category_targets {
            let Some(target_entry) = entries_by_id.get(&target) else {
                continue;
            };
            let has_category_marker = target_entry
                .metadata
                .structural_targets_for(CATEGORY_FIELD)
                .iter()
                .any(|id| id == &category_id);
            if !has_category_marker {
                report.push(CheckDiagnostic {
                    severity: self.severity(),
                    kind: CheckDiagnosticKind::CategoryTargetMissingCategoryMarker,
                    entry: Some(target.clone()),
                    field: CATEGORY_FIELD.to_owned(),
                    target: Some(category_id.clone()),
                });
            }
        }
    }
    // sirno:witness:structural-check:end

    fn is_intrinsic_meta_field(field: &str) -> bool {
        INTRINSIC_META_FIELDS.contains(&field)
    }
}

/// Severity of one structural diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckSeverity {
    /// A condition worth showing during editing.
    Warning,
    /// A structural violation at the selected boundary.
    Error,
}

impl CheckSeverity {
    /// Lowercase label used in human-readable diagnostic output.
    pub fn label(self) -> &'static str {
        match self {
            | Self::Warning => "warning",
            | Self::Error => "error",
        }
    }
}

/// Reason for one structural diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckDiagnosticKind {
    /// A configured link relation does not name an existing entry.
    MissingStructuralFieldEntry,
    /// A configured link relation entry does not define entry-side tide policy.
    MissingStructuralMeta,
    /// An entry defines structural type without being a configured link relation.
    UnregisteredStructuralMeta,
    /// An intrinsic metadata-field entry does not define the intrinsic meta type.
    MissingIntrinsicMeta,
    /// An entry defines intrinsic metadata-field type without being an intrinsic field entry.
    UnregisteredIntrinsicMeta,
    /// An entry uses a structural link relation not configured in `Sirno.toml`.
    UnconfiguredStructuralField,
    /// A structural link target id does not name an entry.
    MissingTarget,
    /// Category metadata is present but the `category` entry is missing.
    MissingCategoryEntry,
    /// An entry used as a category target is not itself marked as a category.
    CategoryTargetMissingCategoryMarker,
}

/// One structural diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckDiagnostic {
    /// Diagnostic severity.
    pub severity: CheckSeverity,
    /// Structural problem detected by the check.
    pub kind: CheckDiagnosticKind,
    /// Entry whose metadata produced the diagnostic.
    pub entry: Option<EntryAddress>,
    /// Metadata field that produced the diagnostic.
    pub field: String,
    /// Referenced path that produced the diagnostic.
    pub target: Option<EntryAddress>,
}

impl CheckDiagnostic {
    /// Human-readable diagnostic message.
    pub fn message(&self) -> String {
        match self.kind {
            | CheckDiagnosticKind::MissingStructuralFieldEntry => format!(
                "`Sirno.toml` configures link relation `{}`, but entry `{}` does not exist",
                self.field,
                self.target.as_ref().expect("missing structural field entry diagnostic has target")
            ),
            | CheckDiagnosticKind::MissingStructuralMeta => format!(
                "`Sirno.toml` configures link relation `{}`, but entry `{}` does not define \
                 `meta.type: \"structural\"`",
                self.field,
                self.entry.as_ref().expect("missing structural meta diagnostic has entry")
            ),
            | CheckDiagnosticKind::UnregisteredStructuralMeta => format!(
                "`{}` defines `meta.type: \"structural\"`, but it is not configured in `Sirno.toml`",
                self.entry.as_ref().expect("unregistered structural meta diagnostic has entry")
            ),
            | CheckDiagnosticKind::MissingIntrinsicMeta => format!(
                "entry `{}` defines required metadata field `{}`, but does not define \
                 `meta.type: \"intrinsic\"`",
                self.entry.as_ref().expect("missing intrinsic meta diagnostic has entry"),
                self.field
            ),
            | CheckDiagnosticKind::UnregisteredIntrinsicMeta => format!(
                "`{}` defines `meta.type: \"intrinsic\"`, but it is not an intrinsic metadata field entry",
                self.entry.as_ref().expect("unregistered intrinsic meta diagnostic has entry")
            ),
            | CheckDiagnosticKind::UnconfiguredStructuralField => format!(
                "`{}` uses link relation `{}` that is not configured in `Sirno.toml`",
                self.entry.as_ref().expect("unconfigured field diagnostic has entry"),
                self.field
            ),
            | CheckDiagnosticKind::MissingTarget => format!(
                "`{}` references missing entry `{}` through `{}`",
                self.entry.as_ref().expect("missing target diagnostic has entry"),
                self.target.as_ref().expect("missing target diagnostic has target"),
                self.field
            ),
            | CheckDiagnosticKind::MissingCategoryEntry => {
                "`category` metadata needs entry `category`; add it with `sirno util entry`"
                    .to_owned()
            }
            | CheckDiagnosticKind::CategoryTargetMissingCategoryMarker => format!(
                "`{}` is used as a category target, but it is not categorized by `{}`",
                self.entry.as_ref().expect("category target diagnostic has entry"),
                self.target.as_ref().expect("category target diagnostic has target")
            ),
        }
    }
}

/// Result of checking a set of entries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckReport {
    diagnostics: Vec<CheckDiagnostic>,
}

impl CheckReport {
    /// Construct an empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one diagnostic to the report.
    pub fn push(&mut self, diagnostic: CheckDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// All diagnostics in deterministic check order.
    pub fn diagnostics(&self) -> &[CheckDiagnostic] {
        &self.diagnostics
    }

    /// Returns true when the report contains no diagnostics.
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns true when at least one diagnostic is an error.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| diagnostic.severity == CheckSeverity::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{EntryMetaType, EntryMetadata};
    use crate::structural::{StructuralFieldSettings, StructuralTideSettings};

    const FIELD_TOPIC: &str = "topic";
    const FIELD_CATEGORY: &str = "category";

    fn entry(id: &str) -> Entry {
        Entry::new(EntryAddress::new(id).unwrap(), EntryMetadata::new(id, "desc").unwrap(), "")
    }

    fn relation_entry(id: &str) -> Entry {
        let mut entry = entry(id);
        entry.metadata.meta.entry_type = Some(EntryMetaType::Structural);
        entry.metadata.meta.tide = Some(StructuralTideSettings::default());
        entry
    }

    fn intrinsic_entry(id: &str) -> Entry {
        let mut entry = entry(id);
        entry.metadata.meta.entry_type = Some(EntryMetaType::Intrinsic);
        entry
    }

    fn structural_settings() -> StructuralSettings {
        StructuralSettings::from_fields([(FIELD_TOPIC, StructuralFieldSettings::default())])
    }

    fn category_settings() -> StructuralSettings {
        StructuralSettings::from_fields([(FIELD_CATEGORY, StructuralFieldSettings::default())])
    }

    #[test]
    fn clean_entries_produce_clean_report() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());
        let mut meta = entry("meta");
        meta.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());
        let topic = relation_entry(FIELD_TOPIC);

        let report =
            CheckMode::Review.check_entries([&concept, &meta, &topic], &structural_settings());
        assert!(report.is_clean());
    }

    #[test]
    fn configured_structural_type_without_tide_policy_is_clean() {
        let mut topic = entry(FIELD_TOPIC);
        topic.metadata.meta.entry_type = Some(EntryMetaType::Structural);

        let report = CheckMode::Review.check_entries([&topic], &structural_settings());

        assert!(report.is_clean());
    }

    #[test]
    fn structural_inhabitance_uses_configured_relation_entry() {
        let mut relation = entry("metadata-topic");
        relation.metadata.meta.entry_type = Some(EntryMetaType::Structural);
        let settings = StructuralSettings::from_relations([(
            FIELD_TOPIC,
            EntryAddress::new("metadata-topic").unwrap(),
        )]);

        let report = CheckMode::Review.check_entries([&relation], &settings);

        assert!(report.is_clean());
    }

    #[test]
    fn edit_mode_reports_dangling_reference_as_warning() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());
        let topic = relation_entry(FIELD_TOPIC);

        let report = CheckMode::Edit.check_entries([&concept, &topic], &structural_settings());
        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::MissingTarget);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }

    #[test]
    fn review_mode_reports_dangling_reference_as_error() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());
        let topic = relation_entry(FIELD_TOPIC);

        let report = CheckMode::Review.check_entries([&concept, &topic], &structural_settings());
        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::MissingTarget);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
    }

    #[test]
    fn edit_mode_reports_missing_structural_field_entry_as_warning() {
        let concept = entry("concept");

        let report = CheckMode::Edit.check_entries([&concept], &structural_settings());

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::MissingStructuralFieldEntry);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }

    #[test]
    fn review_mode_reports_missing_structural_field_entry_as_error() {
        let concept = entry("concept");

        let report = CheckMode::Review.check_entries([&concept], &structural_settings());

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::MissingStructuralFieldEntry);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
        assert!(report.diagnostics()[0].message().contains("entry `topic` does not exist"));
    }

    #[test]
    fn review_mode_reports_missing_structural_meta_as_error() {
        let topic = entry(FIELD_TOPIC);

        let report = CheckMode::Review.check_entries([&topic], &structural_settings());

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::MissingStructuralMeta);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
        assert!(report.diagnostics()[0].message().contains("does not define `meta.type"));
    }

    #[test]
    fn structural_inhabitance_can_be_skipped() {
        let concept = entry("concept");

        let report = CheckMode::Review.check_entries_with_structural_inhabitance(
            [&concept],
            &structural_settings(),
            false,
        );

        assert!(report.is_clean());
    }

    #[test]
    fn unconfigured_structural_fields_warn() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());

        let report = CheckMode::Review.check_entries([&concept], &StructuralSettings::default());

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::UnconfiguredStructuralField);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }

    #[test]
    fn review_mode_reports_unregistered_structural_meta_as_error() {
        let topic = relation_entry(FIELD_TOPIC);

        let report = CheckMode::Review.check_entries([&topic], &StructuralSettings::default());

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::UnregisteredStructuralMeta);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
    }

    #[test]
    fn intrinsic_meta_fields_are_semantically_checked() {
        let name = intrinsic_entry(NAME_FIELD);
        let desc = intrinsic_entry(DESC_FIELD);

        let report =
            CheckMode::Review.check_entries([&name, &desc], &StructuralSettings::default());

        assert!(report.is_clean());
    }

    #[test]
    fn review_mode_reports_intrinsic_field_without_intrinsic_meta_as_error() {
        let name = entry(NAME_FIELD);

        let report = CheckMode::Review.check_entries([&name], &StructuralSettings::default());

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::MissingIntrinsicMeta);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
        assert!(report.diagnostics()[0].message().contains("meta.type: \"intrinsic\""));
    }

    #[test]
    fn review_mode_reports_unregistered_intrinsic_meta_as_error() {
        let concept = intrinsic_entry("concept");

        let report = CheckMode::Review.check_entries([&concept], &StructuralSettings::default());

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::UnregisteredIntrinsicMeta);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
    }

    #[test]
    fn category_metadata_warns_when_category_entry_is_missing() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_CATEGORY, EntryAddress::new("meta").unwrap());
        let mut meta = entry("meta");
        meta.metadata
            .push_structural_target(FIELD_CATEGORY, EntryAddress::new("category").unwrap());

        let report = CheckMode::Review.check_entries([&concept, &meta], &category_settings());

        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.kind == CheckDiagnosticKind::MissingCategoryEntry
                    && diagnostic.severity == CheckSeverity::Warning)
        );
    }

    #[test]
    fn review_mode_reports_category_target_without_category_marker_as_error() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_CATEGORY, EntryAddress::new("meta").unwrap());
        let meta = entry("meta");
        let mut category = relation_entry("category");
        category
            .metadata
            .push_structural_target(FIELD_CATEGORY, EntryAddress::new("category").unwrap());

        let report =
            CheckMode::Review.check_entries([&concept, &meta, &category], &category_settings());

        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| {
                diagnostic.kind == CheckDiagnosticKind::CategoryTargetMissingCategoryMarker
            })
            .expect("category target marker diagnostic");
        assert_eq!(diagnostic.entry.as_ref().unwrap().as_str(), "meta");
        assert_eq!(diagnostic.severity, CheckSeverity::Error);
        assert!(report.has_errors());
    }

    #[test]
    fn edit_mode_reports_category_target_without_category_marker_as_warning() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_CATEGORY, EntryAddress::new("meta").unwrap());
        let meta = entry("meta");
        let mut category = relation_entry("category");
        category
            .metadata
            .push_structural_target(FIELD_CATEGORY, EntryAddress::new("category").unwrap());

        let report =
            CheckMode::Edit.check_entries([&concept, &meta, &category], &category_settings());

        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| {
                diagnostic.kind == CheckDiagnosticKind::CategoryTargetMissingCategoryMarker
            })
            .expect("category target marker diagnostic");
        assert_eq!(diagnostic.severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }
}
