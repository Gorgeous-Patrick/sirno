//! Structural checks for Sirno entries.
//!
//! Sirno checks the shape of entries and structural targets.
//! It does not decide whether prose is true or whether code satisfies a claim.

use std::collections::BTreeMap;

use crate::entry::Entry;
use crate::id::EntryId;
use crate::structural::StructuralSettings;

/// Boundary at which Sirno checks structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckMode {
    /// Editing checks keep local movement fast.
    Edit,
    /// Review checks treat dangling structural references as errors.
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
    /// Check structural metadata targets for a set of entries.
    ///
    /// Parsing already enforces required fields, accepted field shapes, and valid id syntax.
    /// This pass checks configured structural field entries and entry ids named by those fields.
    pub fn check_entries<'a>(
        self, entries: impl IntoIterator<Item = &'a Entry>, structural: &StructuralSettings,
    ) -> CheckReport {
        self.check_entries_with_structural_inhabitance(entries, structural, true)
    }

    /// Check structural metadata targets, with explicit structural-inhabitance policy.
    ///
    /// Structural inhabitance requires each configured structural field to name an existing entry.
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
                if !entries_by_id.keys().any(|id| id.as_str() == field) {
                    report.push(CheckDiagnostic {
                        severity,
                        kind: CheckDiagnosticKind::MissingStructuralFieldEntry,
                        entry: None,
                        field: field.to_owned(),
                        target: None,
                    });
                }
            }
        }
        for entry in entries {
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
        report
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
    /// A configured structural field does not name an existing entry.
    MissingStructuralFieldEntry,
    /// An entry uses a structural metadata field not configured in `Sirno.toml`.
    UnconfiguredStructuralField,
    /// A structural target id does not name an entry.
    MissingTarget,
}

/// One structural diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckDiagnostic {
    /// Diagnostic severity.
    pub severity: CheckSeverity,
    /// Structural problem detected by the check.
    pub kind: CheckDiagnosticKind,
    /// Entry whose metadata produced the diagnostic.
    pub entry: Option<EntryId>,
    /// Metadata field that produced the diagnostic.
    pub field: String,
    /// Referenced id that produced the diagnostic.
    pub target: Option<EntryId>,
}

impl CheckDiagnostic {
    /// Human-readable diagnostic message.
    pub fn message(&self) -> String {
        match self.kind {
            | CheckDiagnosticKind::MissingStructuralFieldEntry => format!(
                "`Sirno.toml` configures structural field `{}`, but entry `{}` does not exist",
                self.field, self.field
            ),
            | CheckDiagnosticKind::UnconfiguredStructuralField => format!(
                "`{}` uses structural field `{}` that is not configured in `Sirno.toml`",
                self.entry.as_ref().expect("unconfigured field diagnostic has entry"),
                self.field
            ),
            | CheckDiagnosticKind::MissingTarget => format!(
                "`{}` references missing entry `{}` through `{}`",
                self.entry.as_ref().expect("missing target diagnostic has entry"),
                self.target.as_ref().expect("missing target diagnostic has target"),
                self.field
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
    use crate::entry::EntryMetadata;
    use crate::structural::StructuralFieldSettings;

    const FIELD_TOPIC: &str = "topic";

    fn entry(id: &str) -> Entry {
        Entry::new(EntryId::new(id).unwrap(), EntryMetadata::new(id, "desc").unwrap(), "")
    }

    fn structural_settings() -> StructuralSettings {
        StructuralSettings::from_fields([(FIELD_TOPIC, StructuralFieldSettings::default())])
    }

    #[test]
    fn clean_entries_produce_clean_report() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryId::new("meta").unwrap());
        let mut meta = entry("meta");
        meta.metadata.push_structural_target(FIELD_TOPIC, EntryId::new("meta").unwrap());
        let topic = entry(FIELD_TOPIC);

        let report =
            CheckMode::Review.check_entries([&concept, &meta, &topic], &structural_settings());
        assert!(report.is_clean());
    }

    #[test]
    fn edit_mode_reports_dangling_reference_as_warning() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryId::new("meta").unwrap());
        let topic = entry(FIELD_TOPIC);

        let report = CheckMode::Edit.check_entries([&concept, &topic], &structural_settings());
        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::MissingTarget);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }

    #[test]
    fn review_mode_reports_dangling_reference_as_error() {
        let mut concept = entry("concept");
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryId::new("meta").unwrap());
        let topic = entry(FIELD_TOPIC);

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
        concept.metadata.push_structural_target(FIELD_TOPIC, EntryId::new("meta").unwrap());

        let report = CheckMode::Review.check_entries([&concept], &StructuralSettings::default());

        assert_eq!(report.diagnostics()[0].kind, CheckDiagnosticKind::UnconfiguredStructuralField);
        assert_eq!(report.diagnostics()[0].severity, CheckSeverity::Warning);
        assert!(!report.has_errors());
    }
}
