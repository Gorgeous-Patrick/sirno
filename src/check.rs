//! Structural checks for Sirno entries.
//!
//! Sirno checks the shape of entries and structural link targets.
//! It does not decide whether prose is true or whether code satisfies a claim.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::entry::Entry;
use crate::identifier::EntryAddress;
use crate::meta::{MetaRegistry, validate_intrinsic_field_name};
use crate::structural::{StructuralSettings, validate_structural_field_name};

const CATEGORY_FIELD: &str = "category";

/// Boundary at which Sirno checks structure.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum CheckMode {
    /// Editing checks keep local movement fast.
    Edit,
    /// Review checks treat dangling structural link references as errors.
    #[default]
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
    /// This pass checks structural relation entries, typed metadata, and target addresses.
    pub fn check_entries<'a>(
        self, entries: impl IntoIterator<Item = &'a Entry>, meta: &MetaRegistry,
    ) -> CheckReport {
        let entries = entries.into_iter().collect::<Vec<_>>();
        let entries_by_id =
            entries.iter().map(|entry| (entry.id.clone(), *entry)).collect::<BTreeMap<_, _>>();
        let structural = meta.structural();
        let severity = self.severity();

        let mut report = CheckReport::new();
        for entry in &entries {
            if entry.metadata.meta.is_intrinsic_field()
                && validate_intrinsic_field_name(entry.id.as_str()).is_err()
            {
                report.push(CheckDiagnostic {
                    severity,
                    kind: CheckDiagnosticKind::InvalidIntrinsicField,
                    entry: Some(entry.id.clone()),
                    field: entry.id.as_str().to_owned(),
                    target: None,
                });
            }
            if entry.metadata.meta.is_structural_relation()
                && validate_structural_field_name(entry.id.as_str()).is_err()
            {
                report.push(CheckDiagnostic {
                    severity,
                    kind: CheckDiagnosticKind::InvalidStructuralRelationField,
                    entry: Some(entry.id.clone()),
                    field: entry.id.as_str().to_owned(),
                    target: None,
                });
            }
            for (field, targets) in entry.metadata.structural_fields() {
                if !structural.contains_field(field) {
                    report.push(CheckDiagnostic {
                        severity: CheckSeverity::Warning,
                        kind: CheckDiagnosticKind::UninhabitedStructuralField,
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
    /// A structural relation entry cannot be used as a metadata field.
    InvalidStructuralRelationField,
    /// An intrinsic metadata-field entry cannot be used as a metadata field.
    InvalidIntrinsicField,
    /// An entry uses a structural link relation without a matching relation entry.
    UninhabitedStructuralField,
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
    // sirno:witness:diagnostics:begin
    /// Stable diagnostic code.
    pub fn code(&self) -> &'static str {
        match self.kind {
            | CheckDiagnosticKind::InvalidStructuralRelationField => {
                "check.structural.field.invalid"
            }
            | CheckDiagnosticKind::InvalidIntrinsicField => "check.intrinsic.field.invalid",
            | CheckDiagnosticKind::UninhabitedStructuralField => "check.structural.field.undefined",
            | CheckDiagnosticKind::MissingTarget => "check.structural.target.missing",
            | CheckDiagnosticKind::MissingCategoryEntry => "check.category.entry.missing",
            | CheckDiagnosticKind::CategoryTargetMissingCategoryMarker => {
                "check.category.marker.missing"
            }
        }
    }

    /// Human-readable diagnostic message.
    pub fn message(&self) -> String {
        match self.kind {
            | CheckDiagnosticKind::InvalidStructuralRelationField => format!(
                "`{}` defines `meta.type: \"structural\"`, but its address is not a valid \
                 structural relation field",
                self.entry.as_ref().expect("invalid structural field diagnostic has entry")
            ),
            | CheckDiagnosticKind::InvalidIntrinsicField => format!(
                "`{}` defines `meta.type: \"intrinsic\"`, but its address is not a valid \
                 intrinsic metadata field",
                self.entry.as_ref().expect("invalid intrinsic field diagnostic has entry")
            ),
            | CheckDiagnosticKind::UninhabitedStructuralField => format!(
                "`{}` uses link relation `{}` without a structural relation entry",
                self.entry.as_ref().expect("uninhabited field diagnostic has entry"),
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

    /// Repair hint for human and agent-facing output.
    pub fn help(&self) -> Option<String> {
        match self.kind {
            | CheckDiagnosticKind::InvalidStructuralRelationField => {
                Some("Rename the entry to a valid structural metadata field name.".to_owned())
            }
            | CheckDiagnosticKind::InvalidIntrinsicField => {
                Some("Rename the entry to a valid intrinsic metadata field name.".to_owned())
            }
            | CheckDiagnosticKind::UninhabitedStructuralField => Some(format!(
                "Add entry `{}` with `meta.type: \"structural\"`, or remove this metadata field.",
                self.field
            )),
            | CheckDiagnosticKind::MissingTarget => Some(format!(
                "Create entry `{}` or remove it from `{}`.",
                self.target.as_ref().expect("missing target diagnostic has target"),
                self.field
            )),
            | CheckDiagnosticKind::MissingCategoryEntry => {
                Some("Create the default category entry with `sirno util entry`.".to_owned())
            }
            | CheckDiagnosticKind::CategoryTargetMissingCategoryMarker => Some(format!(
                "Add `category: [{}]` to `{}`.",
                self.target.as_ref().expect("category target diagnostic has target"),
                self.entry.as_ref().expect("category target diagnostic has entry")
            )),
        }
    }
    // sirno:witness:diagnostics:end
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
    use crate::entry::{DESC_FIELD, EntryMetaType, NAME_FIELD};
    use crate::structural::{StructuralFieldSettings, StructuralTideSettings};

    const FIELD_TOPIC: &str = "topic";
    const FIELD_CATEGORY: &str = "category";

    fn entry(id: &str) -> Entry {
        Entry::new(
            EntryAddress::new(id).unwrap(),
            crate::entry::seed_intrinsic_metadata(id, "desc").unwrap(),
            "",
        )
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

    fn meta_registry(structural: StructuralSettings) -> MetaRegistry {
        MetaRegistry::from_parts(
            [
                (NAME_FIELD, EntryAddress::new(NAME_FIELD).unwrap()),
                (DESC_FIELD, EntryAddress::new(DESC_FIELD).unwrap()),
            ],
            structural,
        )
    }

    #[test]
    fn clean_entries_produce_clean_report() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());
        let mut meta = entry("meta");
        meta.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());
        let topic = relation_entry(FIELD_TOPIC);

        let report = CheckMode::Review
            .check_entries([&concept, &meta, &topic], &meta_registry(structural_settings()));
        assert!(report.is_clean());
    }

    #[test]
    fn structural_type_without_tide_policy_is_clean() {
        let mut topic = entry(FIELD_TOPIC);
        topic.metadata.meta.entry_type = Some(EntryMetaType::Structural);

        let report =
            CheckMode::Review.check_entries([&topic], &meta_registry(structural_settings()));

        assert!(report.is_clean());
    }

    #[test]
    fn edit_mode_reports_dangling_reference_as_warning() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());
        let topic = relation_entry(FIELD_TOPIC);

        let report = CheckMode::Edit
            .check_entries([&concept, &topic], &meta_registry(structural_settings()));
        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::MissingTarget);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }

    #[test]
    fn review_mode_reports_dangling_reference_as_error() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());
        let topic = relation_entry(FIELD_TOPIC);

        let report = CheckMode::Review
            .check_entries([&concept, &topic], &meta_registry(structural_settings()));
        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::MissingTarget);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
    }

    #[test]
    fn uninhabited_structural_fields_warn() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryAddress::new("meta").unwrap());

        let report = CheckMode::Review
            .check_entries([&concept], &meta_registry(StructuralSettings::default()));

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::UninhabitedStructuralField);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }

    #[test]
    fn review_mode_reports_invalid_structural_relation_field_as_error() {
        let topic = relation_entry("meta");

        let report = CheckMode::Review
            .check_entries([&topic], &meta_registry(StructuralSettings::default()));

        assert_eq!(
            report.diagnostics()[0].kind,
            CheckDiagnosticKind::InvalidStructuralRelationField
        );
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Error);
        assert!(report.has_errors());
    }

    #[test]
    fn intrinsic_meta_fields_are_semantically_checked() {
        let name = intrinsic_entry(NAME_FIELD);
        let desc = intrinsic_entry(DESC_FIELD);

        let report = CheckMode::Review
            .check_entries([&name, &desc], &meta_registry(StructuralSettings::default()));

        assert!(report.is_clean());
    }

    #[test]
    fn review_mode_reports_invalid_intrinsic_field_as_error() {
        let meta = intrinsic_entry("meta");

        let report =
            CheckMode::Review.check_entries([&meta], &meta_registry(StructuralSettings::default()));

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::InvalidIntrinsicField);
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

        let report =
            CheckMode::Review.check_entries([&concept, &meta], &meta_registry(category_settings()));

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

        let report = CheckMode::Review
            .check_entries([&concept, &meta, &category], &meta_registry(category_settings()));

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

        let report = CheckMode::Edit
            .check_entries([&concept, &meta, &category], &meta_registry(category_settings()));

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
