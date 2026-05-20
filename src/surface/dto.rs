//! Request and result types for command callers.

use std::path::PathBuf;
use std::str::FromStr;

use clap::ValueEnum;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::surface::error::CommandError;
use crate::surface::output::{
    diagnostics_from_entry_report, display_path, display_paths, format_gen_link_report,
    format_query_json, query_result_records,
};
use crate::{
    CheckMode, EntryDirectoryReport, EntryId, EntryIdError, EntryStructuralMatcher,
    GenLinkDirectoryReport, StructuralEdgeSettings, Tide, TideStatus, TideWorkitem, WitnessRecord,
};

/// Shared human-or-JSON output renderer.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum StructuredOutputFormat {
    /// Print JSON for machine-oriented callers.
    Json,
    /// Print terminal-oriented human text.
    #[default]
    Human,
}

pub(crate) type QueryOutputFormat = StructuredOutputFormat;
pub(crate) type TideOutputFormat = StructuredOutputFormat;

/// Tide status detail selected by command callers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum TideStatusMode {
    /// Show only entry ids that need review.
    #[default]
    Review,
    /// Show full open workitem statuses.
    Full,
    /// Show full open and resolved workitem statuses.
    All,
}

impl TideStatusMode {
    pub(crate) fn includes_workitems(self) -> bool {
        matches!(self, Self::Full | Self::All)
    }

    pub(crate) fn includes_resolved(self) -> bool {
        matches!(self, Self::All)
    }
}

/// Result of reading or changing the process current working directory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CwdResult {
    /// Whether the current working directory was read successfully.
    pub ok: bool,
    /// Whether the command changed the current working directory before reading it.
    pub changed: bool,
    /// Current working directory after the command completed.
    pub path: String,
    /// Human-readable summary.
    pub message: String,
}

/// Result of checking canonical comments in `Sirno.toml`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigCommentResult {
    /// Whether the config now has every canonical comment.
    pub ok: bool,
    /// Whether this run rewrote the config file.
    pub changed: bool,
    /// Checked config path.
    pub config_path: String,
    /// Canonical comment texts that were missing before any repair.
    pub missing_comments: Vec<String>,
    /// Human-readable summary.
    pub message: String,
}

/// Query output column list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryColumns {
    pub(crate) columns: Vec<QueryColumn>,
}

impl QueryColumns {
    /// Build a column list from explicit query columns.
    pub fn new(columns: Vec<QueryColumn>) -> Self {
        Self { columns }
    }

    /// Return the selected columns in display order.
    pub fn columns(&self) -> &[QueryColumn] {
        &self.columns
    }

    /// Return stable output field labels in display order.
    pub fn labels(&self) -> Vec<String> {
        self.columns.iter().map(|column| column.label().to_owned()).collect()
    }
}

impl Default for QueryColumns {
    fn default() -> Self {
        Self { columns: vec![QueryColumn::Id, QueryColumn::Path, QueryColumn::Name] }
    }
}

impl FromStr for QueryColumns {
    type Err = QueryColumnsParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if raw.trim().is_empty() {
            return Err(QueryColumnsParseError::Empty);
        }

        let mut columns = Vec::new();
        for raw_column in raw.split(',') {
            let column = raw_column.trim();
            if column.is_empty() {
                return Err(QueryColumnsParseError::EmptyColumn);
            }
            columns.push(column.parse()?);
        }

        Ok(Self { columns })
    }
}

/// One column printable by `sirno query`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryColumn {
    /// Entry id.
    Id,
    /// Human-readable entry name.
    Name,
    /// Markdown path.
    Path,
    /// Short entry desc.
    Desc,
}

impl FromStr for QueryColumn {
    type Err = QueryColumnsParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            | "id" => Ok(Self::Id),
            | "name" => Ok(Self::Name),
            | "path" => Ok(Self::Path),
            | "desc" => Ok(Self::Desc),
            | column => Err(QueryColumnsParseError::UnknownColumn(column.to_owned())),
        }
    }
}

impl QueryColumn {
    /// Return the stable output field name for this column.
    pub fn label(self) -> &'static str {
        match self {
            | Self::Id => "id",
            | Self::Name => "name",
            | Self::Path => "path",
            | Self::Desc => "desc",
        }
    }
}

/// Error raised while parsing one `--columns` list.
#[derive(Debug, Error)]
pub enum QueryColumnsParseError {
    /// The list contains no columns.
    #[error("query columns must include at least one column")]
    Empty,
    /// The list contains a separator without a column.
    #[error("query columns contain an empty column")]
    EmptyColumn,
    /// The list contains an unknown output column.
    #[error("unknown query column `{0}`; expected id, name, path, or desc")]
    UnknownColumn(String),
}

/// Structural query filter parsed from `FIELD=ENTRY_ID[,ENTRY_ID]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructuralFilter {
    /// Structural field name.
    pub field: String,
    /// Accepted target entry ids for this field.
    pub targets: Vec<EntryId>,
}

impl FromStr for StructuralFilter {
    type Err = StructuralFilterParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let Some((field, targets)) = raw.split_once('=') else {
            return Err(StructuralFilterParseError::MissingEquals);
        };
        let field = field.trim();
        if field.is_empty() {
            return Err(StructuralFilterParseError::EmptyField);
        }
        let targets = parse_structural_filter_targets(targets)?;
        Ok(Self { field: field.to_owned(), targets })
    }
}

fn parse_structural_filter_targets(raw: &str) -> Result<Vec<EntryId>, StructuralFilterParseError> {
    let mut targets = Vec::new();
    for raw_target in raw.split(',') {
        let target = raw_target.trim();
        if target.is_empty() {
            return Err(StructuralFilterParseError::EmptyTarget);
        }
        targets.push(EntryId::new(target)?);
    }
    Ok(targets)
}

/// Error raised while parsing one structural query filter.
#[derive(Debug, Error)]
pub enum StructuralFilterParseError {
    /// The argument does not contain the field-target separator.
    #[error("expected FIELD=ENTRY_ID[,ENTRY_ID]")]
    MissingEquals,
    /// The structural field name is empty.
    #[error("structural field name must not be empty")]
    EmptyField,
    /// The target entry id list contains a separator without a target.
    #[error("structural filter contains an empty target")]
    EmptyTarget,
    /// A target entry id is invalid.
    #[error(transparent)]
    EntryId(#[from] EntryIdError),
}

/// Structural query state filter parsed from `FIELD=STATE`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructuralStateFilter {
    /// Structural field name.
    pub field: String,
    /// Accepted state for this field.
    pub state: StructuralFieldState,
}

impl FromStr for StructuralStateFilter {
    type Err = StructuralStateFilterParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let Some((field, state)) = raw.split_once('=') else {
            return Err(StructuralStateFilterParseError::MissingEquals);
        };
        let field = field.trim();
        if field.is_empty() {
            return Err(StructuralStateFilterParseError::EmptyField);
        }
        Ok(Self { field: field.to_owned(), state: state.trim().parse()? })
    }
}

/// Structural field state matched by `sirno query --is`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralFieldState {
    /// The field is present with any target count.
    Present,
    /// The field is present with no targets.
    Empty,
    /// The field is absent.
    Missing,
}

impl FromStr for StructuralFieldState {
    type Err = StructuralStateFilterParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            | "present" => Ok(Self::Present),
            | "empty" => Ok(Self::Empty),
            | "missing" => Ok(Self::Missing),
            | state => Err(StructuralStateFilterParseError::UnknownState(state.to_owned())),
        }
    }
}

impl From<StructuralFieldState> for EntryStructuralMatcher {
    fn from(value: StructuralFieldState) -> Self {
        match value {
            | StructuralFieldState::Present => Self::Present,
            | StructuralFieldState::Empty => Self::Empty,
            | StructuralFieldState::Missing => Self::Missing,
        }
    }
}

/// Error raised while parsing one structural query state filter.
#[derive(Debug, Error)]
pub enum StructuralStateFilterParseError {
    /// The argument does not contain the field-state separator.
    #[error("expected FIELD=present, FIELD=empty, or FIELD=missing")]
    MissingEquals,
    /// The structural field name is empty.
    #[error("structural field name must not be empty")]
    EmptyField,
    /// The structural field state is not recognized.
    #[error("unknown structural field state `{0}`; expected present, empty, or missing")]
    UnknownState(String),
}

/// Entry query request shared by CLI and tool callers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QueryRequest {
    /// Vague text terms matched against expanded entry text.
    pub terms: Vec<String>,
    /// Exact text terms matched against entry-local text.
    pub exact_terms: Vec<String>,
    /// Structural target filters.
    pub has: Vec<StructuralFilter>,
    /// Structural field state filters.
    pub is: Vec<StructuralStateFilter>,
    /// Output columns to materialize.
    pub columns: QueryColumns,
}

/// Query execution result before presentation rendering.
#[derive(Debug)]
pub enum QueryRun {
    /// The lake did not pass the edit-mode checks needed for query.
    InvalidLake(EntryDirectoryReport),
    /// The query completed and produced rows.
    Results(QueryResults),
}

/// Structured query rows plus the selected column order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryResults {
    pub(crate) columns: QueryColumns,
    pub(crate) rows: Vec<Vec<String>>,
}

impl QueryResults {
    /// Build query results from selected columns and materialized rows.
    pub fn new(columns: QueryColumns, rows: Vec<Vec<String>>) -> Self {
        Self { columns, rows }
    }

    /// Return selected columns in display order.
    pub fn columns(&self) -> &QueryColumns {
        &self.columns
    }

    /// Return raw row values in selected column order.
    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    /// Return JSON-ready records keyed by selected column labels.
    pub fn records(&self) -> Vec<IndexMap<String, String>> {
        query_result_records(&self.columns, &self.rows)
    }

    /// Render the result rows as pretty JSON.
    pub fn to_json(&self) -> Result<String, CommandError> {
        format_query_json(&self.columns, &self.rows)
    }
}

/// Entry path lookup request shared by CLI and tool callers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryPathRequest {
    /// Entry id whose paths should be returned.
    pub id: EntryId,
    /// Selected path classes.
    pub selection: PathSelection,
    /// Whether returned paths should be absolute.
    pub absolute: bool,
}

impl EntryPathRequest {
    /// Build a path lookup request from explicit typed fields.
    pub fn new(id: EntryId, selection: PathSelection, absolute: bool) -> Self {
        Self { id, selection, absolute }
    }
}

/// Lake initialization request shared by non-CLI front ends.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LakeInitRequest {
    /// Sirno Lake path written to `Sirno.toml`.
    pub lake: Option<PathBuf>,
}

/// Result of creating a Sirno Lake.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LakeInitResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Config file that was written.
    pub config_path: String,
    /// Sirno Lake directory that was initialized.
    pub lake_path: String,
    /// Number of seed entries written.
    pub entry_count: usize,
    /// Concise human-readable summary.
    pub message: String,
}

/// Structural metadata target for typed command callers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralTarget {
    /// Structural field name.
    pub field: String,
    /// Target entry id.
    pub target: EntryId,
}

/// Entry creation request shared by the CLI and tool callers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryNewRequest {
    /// Entry id and filename stem.
    pub id: EntryId,
    /// Human-readable entry name.
    pub name: Option<String>,
    /// Short entry description.
    pub desc: String,
    /// Structural metadata targets.
    #[serde(default)]
    pub structural: Vec<StructuralTarget>,
    /// Initial Markdown body.
    pub body: Option<String>,
}

/// Result that points at one entry file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPathResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry id affected by the command.
    pub id: String,
    /// Sirno Lake entry path affected by the command.
    pub path: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of reading one Sirno Lake Markdown entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryReadResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry id that was read.
    pub id: String,
    /// Sirno Lake entry file path.
    pub path: String,
    /// Human-readable entry name.
    pub name: String,
    /// Short entry description.
    pub desc: String,
    /// Markdown body outside the metadata block.
    pub body: String,
    /// Full Markdown source as stored on disk.
    pub source: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of renaming one entry id.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryRenameResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry id before the rename.
    pub old_id: String,
    /// Entry id after the rename.
    pub new_id: String,
    /// Paths updated by the rename.
    pub updated_paths: Vec<String>,
    /// Concise human-readable summary.
    pub message: String,
}

/// Query result designed for JSON-first callers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResponse {
    /// Whether the query ran against a clean-enough lake.
    pub ok: bool,
    /// Selected output columns.
    pub columns: Vec<String>,
    /// Query records keyed by column label.
    pub records: Vec<IndexMap<String, String>>,
    /// Diagnostics when the lake prevents query execution.
    pub diagnostics: Vec<DiagnosticRecord>,
}

/// Ripgrep request shared by typed callers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgRequest {
    /// Include Sirno-owned generated-footer regions in the search.
    #[serde(default)]
    pub with_generated_footer: bool,
    /// Arguments forwarded to ripgrep before the lake path.
    pub args: Vec<String>,
}

/// Captured ripgrep result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgResult {
    /// Whether ripgrep exited successfully.
    pub ok: bool,
    /// Process exit code, or 1 when no ordinary code is available.
    pub exit_code: u8,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error.
    pub stderr: String,
}

/// One JSON-ready source span.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessSpanResult {
    /// One-based starting line.
    pub start_line: usize,
    /// One-based starting column.
    pub start_column: usize,
    /// One-based ending line.
    pub end_line: usize,
    /// One-based column after the span.
    pub end_column: usize,
}

/// One JSON-ready witness record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessRecordResult {
    /// Entry id captured by the witness block.
    pub entry: String,
    /// Repository file path containing the witness.
    pub path: String,
    /// Full matched block region.
    pub region: WitnessSpanResult,
    /// Opening delimiter span when verbose output is requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening: Option<WitnessSpanResult>,
    /// Closing delimiter span when verbose output is requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closing: Option<WitnessSpanResult>,
    /// Full matched witness block body.
    pub body: String,
}

impl WitnessRecordResult {
    pub(crate) fn from_record(record: &WitnessRecord, verbose: bool) -> Self {
        Self {
            entry: record.entry.to_string(),
            path: display_path(&record.path),
            region: WitnessSpanResult::from(record.region),
            opening: verbose.then(|| WitnessSpanResult::from(record.opening)),
            closing: verbose.then(|| WitnessSpanResult::from(record.closing)),
            body: record.body.clone(),
        }
    }
}

impl From<crate::witness::WitnessSpan> for WitnessSpanResult {
    fn from(value: crate::witness::WitnessSpan) -> Self {
        Self {
            start_line: value.start_line,
            start_column: value.start_column,
            end_line: value.end_line,
            end_column: value.end_column,
        }
    }
}

/// Repository witness lookup result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessResult {
    /// Whether any witness block was found.
    pub ok: bool,
    /// Entry id used for lookup.
    pub id: String,
    /// Matching witness records.
    pub records: Vec<WitnessRecordResult>,
    /// Concise human-readable summary.
    pub message: String,
}

/// Artifact listing result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactListResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry id whose artifacts were listed.
    pub id: String,
    /// Owner-relative artifact paths.
    pub artifacts: Vec<String>,
}

/// Artifact add request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactAddRequest {
    /// Entry id that will own the artifact.
    pub id: EntryId,
    /// Source file to copy.
    pub source: PathBuf,
    /// Owner-relative artifact path.
    pub artifact_path: Option<PathBuf>,
}

/// Artifact rename request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRenameRequest {
    /// Entry id that owns the artifact.
    pub id: EntryId,
    /// Existing owner-relative artifact path.
    pub old_path: PathBuf,
    /// New owner-relative artifact path.
    pub new_path: PathBuf,
}

/// Artifact removal request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRemoveRequest {
    /// Entry id that owns the artifact.
    pub id: EntryId,
    /// Owner-relative artifact path to remove.
    pub artifact_path: PathBuf,
}

/// Result of changing one artifact file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactChangeResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry id that owns the artifact.
    pub id: String,
    /// Owner-relative artifact path.
    pub artifact_path: String,
    /// Filesystem path affected by the command.
    pub path: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// One discovered Sirno skill wrapper, package target, or adjacent skill link.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillWrapperRecord {
    /// Discipline entry that owns the wrapper artifact.
    pub entry_id: String,
    /// Installed skill package name.
    pub name: String,
    /// Lake-owned wrapper artifact path or link source path.
    pub wrapper_path: String,
    /// Lake-owned full resource artifact path.
    pub full_path: String,
    /// Repository-relative package or link target path.
    pub target_path: String,
    /// Stable command status label.
    pub status: String,
    /// Whether the package target differs or was rewritten.
    pub changed: bool,
}

/// Result of listing, checking, or installing Sirno skill wrappers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillWrapperResult {
    /// Whether every wrapper package matched the requested state.
    pub ok: bool,
    /// Discovered wrapper records.
    pub records: Vec<SkillWrapperRecord>,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of moving a configured repository path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovePathResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Whether the filesystem path actually moved.
    pub moved: bool,
    /// Previous configured path.
    pub old_path: String,
    /// New configured path.
    pub new_path: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// One JSON-ready diagnostic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticRecord {
    /// Diagnostic severity.
    pub severity: String,
    /// Optional path responsible for the diagnostic.
    pub path: Option<String>,
    /// Human-readable diagnostic message.
    pub message: String,
}

/// JSON-ready lake check result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LakeCheckResult {
    /// Whether the selected check mode produced no errors.
    pub ok: bool,
    /// Lake root that was checked.
    pub root: String,
    /// Whether at least one error was reported.
    pub has_errors: bool,
    /// Diagnostics reported by the check.
    pub diagnostics: Vec<DiagnosticRecord>,
}

impl LakeCheckResult {
    pub(crate) fn from_report(report: &EntryDirectoryReport) -> Self {
        let has_errors = report.has_errors();
        Self {
            ok: !has_errors,
            root: display_path(report.root()),
            has_errors,
            diagnostics: diagnostics_from_entry_report(report),
        }
    }
}

/// JSON-ready rendered-footer result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderResult {
    /// Whether rendering or dry-checking completed without blocking diagnostics.
    pub ok: bool,
    /// Whether the render operation only checked for changes.
    pub dry: bool,
    /// Lake root that was processed.
    pub root: String,
    /// Number of entries processed.
    pub entry_count: usize,
    /// Entry files whose generated-link region changed.
    pub changed_paths: Vec<String>,
    /// Diagnostics that blocked rendering.
    pub diagnostics: Vec<DiagnosticRecord>,
    /// Concise human-readable summary.
    pub message: String,
}

impl RenderResult {
    pub(crate) fn from_report(report: &GenLinkDirectoryReport, dry: bool) -> Self {
        let changed_paths = display_paths(report.changed_paths());
        Self {
            ok: true,
            dry,
            root: display_path(report.root()),
            entry_count: report.entry_count(),
            changed_paths,
            diagnostics: Vec::new(),
            message: format_gen_link_report(
                report.root(),
                report.entry_count(),
                report.changed_paths(),
            ),
        }
    }

    pub(crate) fn blocked(report: &EntryDirectoryReport) -> Self {
        Self {
            ok: false,
            dry: false,
            root: display_path(report.root()),
            entry_count: report.entries().len(),
            changed_paths: Vec::new(),
            diagnostics: diagnostics_from_entry_report(report),
            message: format!("render blocked by check errors in {}", report.root().display()),
        }
    }
}

/// Structured edge policy for one structural field direction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralEdgeStatus {
    /// Whether generated footers render this edge direction.
    pub render: bool,
    /// Whether lake-side neighbors create tide workitems.
    pub ripple_lake: bool,
    /// Whether frost-side neighbors create tide workitems.
    pub ripple_frost: bool,
}

impl StructuralEdgeStatus {
    pub(crate) fn from_settings(settings: &StructuralEdgeSettings) -> Self {
        Self {
            render: settings.render,
            ripple_lake: settings.ripple.lake,
            ripple_frost: settings.ripple.frost,
        }
    }
}

/// Structural field status in one Sirno config.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralFieldStatus {
    /// Structural field name.
    pub field: String,
    /// Outgoing edge settings.
    pub to: StructuralEdgeStatus,
    /// Incoming edge settings.
    pub from: StructuralEdgeStatus,
    /// Shared-target edge settings.
    pub clique: StructuralEdgeStatus,
}

/// Current lock state of a configured Frost store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatusFrostState {
    /// No Sirno lock exists for the configured Frost path.
    Unlocked,
    /// The public lake is the current editable Frost version.
    Current,
    /// The public lake materializes a selected Frost version.
    CheckedOut,
}

/// Typed Frost status for the current project.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusFrost {
    /// Configured Frost path.
    pub path: String,
    /// Current public-lake state relative to Frost.
    pub state: StatusFrostState,
    /// Frost version when a lock names one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,
    /// Frost generation when a lock names one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
    /// Whether the public lake is writable as a Frost checkout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutable: Option<bool>,
}

/// Check policy used by project status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCheckPolicy {
    /// Check boundary used for status.
    pub mode: CheckMode,
    /// Whether generated-footer freshness is checked.
    pub render: bool,
}

/// Compact Tide summary for project status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusTide {
    /// Whether no open tide workitem remains.
    pub clear: bool,
    /// Number of open workitems.
    pub open_workitems: usize,
    /// Number of waves with at least one open workitem.
    pub open_waves: usize,
    /// Number of entries that still need review.
    pub review_entries: usize,
}

impl StatusTide {
    pub(crate) fn from_tide(tide: &Tide) -> Self {
        let open_statuses = tide.open_statuses().collect::<Vec<_>>();
        let open_waves = open_statuses
            .iter()
            .map(|status| &status.workitem.ripple)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        Self {
            clear: open_statuses.is_empty(),
            open_workitems: open_statuses.len(),
            open_waves,
            review_entries: tide.review_entries().len(),
        }
    }
}

/// Commit readiness for the configured project.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatusCommitState {
    /// A Frost commit can proceed.
    Ready,
    /// A Frost commit is blocked by one or more project states.
    Blocked,
    /// Frost commits are unavailable because Frost is not configured.
    Unavailable,
}

/// Specific project state that blocks a Frost commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatusCommitBlocker {
    /// Review-mode lake checks currently report errors.
    LakeCheck,
    /// The active Tide has open workitems.
    Tide,
    /// The public lake is an immutable Frost checkout.
    ImmutableCheckout,
}

/// Commit readiness summary for project status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCommit {
    /// Whether a Frost commit can proceed.
    pub ready: bool,
    /// Human-independent readiness state.
    pub state: StatusCommitState,
    /// States that block a commit.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<StatusCommitBlocker>,
}

/// JSON-ready project status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusResult {
    /// Whether the configured project has no status blockers.
    pub ok: bool,
    /// Config file used for the status command.
    pub config_path: String,
    /// Lake path.
    pub lake_path: String,
    /// Number of parsed entries.
    pub entry_count: usize,
    /// Optional typed Frost status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frost: Option<StatusFrost>,
    /// Status check policy.
    pub check_policy: StatusCheckPolicy,
    /// Configured structural field summaries.
    pub structural_fields: Vec<StructuralFieldStatus>,
    /// Tide summary when Frost is configured and the lake can be compared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tide: Option<StatusTide>,
    /// Frost commit readiness.
    pub commit: StatusCommit,
    /// Review-mode check result.
    pub check: LakeCheckResult,
}

/// Result of initializing frost.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostInitResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Configured frost path.
    pub frost_path: String,
    /// Current frost version after initialization.
    pub version: u64,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of committing a frost snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostCommitResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// New frost version.
    pub version: u64,
    /// Lake path committed to frost.
    pub lake_path: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of garbage-collecting frost storage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostGcResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Configured frost path.
    pub frost_path: String,
    /// GC generation before collection.
    pub before_generation: u64,
    /// Frost version before collection.
    pub before_version: u64,
    /// GC generation after collection.
    pub after_generation: u64,
    /// Frost version after collection.
    pub after_version: u64,
    /// Whether `eter` physically collected rows.
    pub collected: bool,
    /// Concise human-readable summary.
    pub message: String,
}

/// Frost checkout request.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostCheckoutRequest {
    /// Explicit frost version to check out.
    pub version: Option<u64>,
    /// Check out the latest frost version as mutable current lake.
    #[serde(default)]
    pub latest: bool,
    /// Leave an explicit version checkout writable.
    #[serde(default)]
    pub unsafe_mutable: bool,
}

/// Result of checking out a frost snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostCheckoutResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Checked-out frost version.
    pub version: u64,
    /// Lake path written by checkout.
    pub lake_path: String,
    /// Number of entries written.
    pub entry_count: usize,
    /// Mutable or immutable lake state after checkout.
    pub state: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// Tide workitem selection by exact workitems or neighbor ids.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TideSelectionRequest {
    /// Select all workitems whose neighbor matches one of these ids.
    #[serde(default)]
    pub neighbors: Vec<EntryId>,
    /// Select exact workitem objects.
    #[serde(default)]
    pub workitems: Vec<TideWorkitem>,
}

/// Tide resolution request.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TideResolveRequest {
    /// Resolve workitems whose neighbor appears in the current ripple set.
    #[serde(default)]
    pub infer: bool,
    /// Select all workitems whose neighbor matches one of these ids.
    #[serde(default)]
    pub neighbors: Vec<EntryId>,
    /// Select exact workitem objects.
    #[serde(default)]
    pub workitems: Vec<TideWorkitem>,
}

/// Result of changing tide resolutions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TideChangeResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Number of workitems changed.
    pub count: usize,
    /// Concise human-readable summary.
    pub message: String,
}

/// JSON-ready tide status result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TideStatusResult {
    /// Whether no entry still needs review.
    pub ok: bool,
    /// Entry ids that still need review.
    pub review_entries: Vec<EntryId>,
    /// Full workitem statuses when requested.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<TideStatus>,
}

/// Selected path classes for an entry path lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathSelection {
    pub(crate) entry: bool,
    pub(crate) artifact: bool,
    pub(crate) frost: bool,
}

impl PathSelection {
    /// Select entry, artifact, and frost paths.
    pub fn all() -> Self {
        Self { entry: true, artifact: true, frost: true }
    }

    /// Build an explicit path-class selection.
    pub fn new(entry: bool, artifact: bool, frost: bool) -> Self {
        Self { entry, artifact, frost }
    }
}

/// One filesystem path returned by an entry path lookup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PathRecord {
    /// Path class.
    pub kind: &'static str,
    /// Display-ready filesystem path.
    pub path: String,
}

impl PathRecord {
    pub(crate) fn new(kind: &'static str, path: PathBuf) -> Self {
        Self { kind, path: path.display().to_string() }
    }
}
