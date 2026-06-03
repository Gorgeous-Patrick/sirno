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
    CheckMode, EntryAddress, EntryAddressError, EntryAtom, EntryDirectoryReport,
    EntryStructuralMatcher, GenLinkDirectoryReport, StructuralEdgeSettings, Tide, TideStatus,
    TideWorkitem, UpstreamSettings, WitnessRecord,
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
pub(crate) type AnchorOutputFormat = StructuredOutputFormat;

/// Tide status detail selected by command callers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum TideStatusMode {
    /// Show only entry addresses that need review.
    #[default]
    Review,
    /// Show full open workitem statuses.
    Full,
    /// Show full open and resolved workitem statuses.
    All,
}

/// Anchor ripple class for one entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnchorRippleKind {
    /// The entry exists in the lake but not in the anchor.
    Added,
    /// The entry exists in both places and has a different fingerprint.
    Changed,
    /// The entry exists in the anchor but not in the lake.
    Deleted,
}

/// One entry-level difference between the current lake and the anchor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorRippleRecord {
    /// Entry address.
    pub id: EntryAddress,
    /// Ripple class.
    pub kind: AnchorRippleKind,
}

/// JSON-ready Anchor status result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorStatusResult {
    /// Whether the lake matches the stored anchor.
    pub ok: bool,
    /// Whether `.sirno/anchor.toml` exists.
    pub initialized: bool,
    /// Anchor file path.
    pub anchor_path: String,
    /// Lake path.
    pub lake_path: String,
    /// Number of current lake entries.
    pub entry_count: usize,
    /// Current entry-level ripples.
    pub ripples: Vec<AnchorRippleRecord>,
    /// Human-readable summary.
    pub message: String,
}

/// JSON-ready Anchor check result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorCheckResult {
    /// Whether the anchor exists, parses, and matches the current lake.
    pub ok: bool,
    /// Whether `.sirno/anchor.toml` exists.
    pub initialized: bool,
    /// Anchor file path.
    pub anchor_path: String,
    /// Lake path.
    pub lake_path: String,
    /// Number of current lake entries.
    pub entry_count: usize,
    /// Current entry-level ripples.
    pub ripples: Vec<AnchorRippleRecord>,
    /// Human-readable summary.
    pub message: String,
}

/// JSON-ready Anchor update result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorUpdateResult {
    /// Whether the anchor was written.
    pub ok: bool,
    /// Anchor file path.
    pub anchor_path: String,
    /// Lake path.
    pub lake_path: String,
    /// Number of entries written to the anchor.
    pub entry_count: usize,
    /// Number of tide resolutions removed after accepting the lake.
    pub cleared_tide_resolutions: usize,
    /// Human-readable summary.
    pub message: String,
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

/// One structural relation discovered from project-local entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralConfigRecord {
    /// Metadata relation name.
    pub field: String,
    /// Entry configured for the relation.
    pub entry: String,
    /// Whether this run changed the config row.
    pub changed: bool,
}

/// Result of syncing configured structural relations from local entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralConfigSyncResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Whether `Sirno.toml` changed.
    pub changed: bool,
    /// Checked config path.
    pub config_path: String,
    /// Discovered structural relation rows.
    pub relations: Vec<StructuralConfigRecord>,
    /// Human-readable summary.
    pub message: String,
}

/// Request to add or replace one upstream declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpstreamAddRequest {
    /// Glacier domain.
    pub domain: EntryAtom,
    /// Upstream settings to write.
    pub settings: UpstreamSettings,
}

/// Request to crystallize or update glacier domains.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpstreamCrystallizeRequest {
    /// Selected glacier domains. Empty means every upstream.
    pub domains: Vec<EntryAtom>,
    /// Use only the existing lock state and cache.
    pub locked: bool,
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

    /// Return selected structural link relation columns.
    pub(crate) fn structural_fields(&self) -> impl Iterator<Item = &str> {
        self.columns.iter().filter_map(QueryColumn::structural_field)
    }

    /// Build the default query output columns.
    pub fn default_output() -> Self {
        Self { columns: vec![QueryColumn::Id, QueryColumn::Name] }
    }
}

/// Query column mode requested by a caller.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum QueryColumnSelection {
    /// Select the standard query output columns.
    #[default]
    Default,
    /// Print selectable column names without selecting entries.
    Options,
    /// Select explicit output columns.
    Selected(QueryColumns),
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
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:query:begin
pub enum QueryColumn {
    /// Entry address.
    Id,
    /// Human-readable entry name.
    Name,
    /// Markdown path.
    Path,
    /// Short entry desc.
    Desc,
    /// Configured structural link relation.
    Structural {
        /// Metadata relation to read from each entry.
        field: String,
    },
}
// sirno:witness:query:end

impl FromStr for QueryColumn {
    type Err = QueryColumnsParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            | "id" => Ok(Self::Id),
            | "name" => Ok(Self::Name),
            | "path" => Ok(Self::Path),
            | "desc" => Ok(Self::Desc),
            | column => Ok(Self::Structural { field: column.to_owned() }),
        }
    }
}

impl QueryColumn {
    /// Return the stable output field name for this column.
    pub fn label(&self) -> &str {
        match self {
            | Self::Id => "id",
            | Self::Name => "name",
            | Self::Path => "path",
            | Self::Desc => "desc",
            | Self::Structural { field } => field,
        }
    }

    /// Return the link relation name when this column selects structural metadata.
    pub fn structural_field(&self) -> Option<&str> {
        match self {
            | Self::Structural { field } => Some(field),
            | Self::Id | Self::Name | Self::Path | Self::Desc => None,
        }
    }
}

/// One materialized query cell value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
// sirno:witness:query:begin
pub enum QueryValue {
    /// Scalar entry field value.
    Text(String),
    /// Structural link targets for one configured relation.
    ///
    /// `None` means the relation is absent.
    /// `Some([])` means the relation is present and has no targets.
    Targets(Option<Vec<String>>),
}
// sirno:witness:query:end

impl QueryValue {
    /// Build a scalar query value.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Build a structural link target query value.
    pub fn targets(targets: Option<&[EntryAddress]>) -> Self {
        Self::Targets(
            targets.map(|targets| targets.iter().map(ToString::to_string).collect::<Vec<_>>()),
        )
    }

    /// Return the human table display string for this value.
    pub(crate) fn display(&self) -> String {
        match self {
            | Self::Text(value) => value.clone(),
            | Self::Targets(Some(targets)) => targets.join(", "),
            | Self::Targets(None) => String::new(),
        }
    }
}

impl From<String> for QueryValue {
    fn from(value: String) -> Self {
        Self::Text(value)
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
}

/// Structural link query filter parsed from `FIELD=ENTRY_ADDRESS[,ENTRY_ADDRESS]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructuralFilter {
    /// Link relation name.
    pub field: String,
    /// Accepted target entry addresses for this relation.
    pub targets: Vec<EntryAddress>,
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

fn parse_structural_filter_targets(
    raw: &str,
) -> Result<Vec<EntryAddress>, StructuralFilterParseError> {
    let mut targets = Vec::new();
    for raw_target in raw.split(',') {
        let target = raw_target.trim();
        if target.is_empty() {
            return Err(StructuralFilterParseError::EmptyTarget);
        }
        targets.push(EntryAddress::new(target)?);
    }
    Ok(targets)
}

/// Error raised while parsing one structural link query filter.
#[derive(Debug, Error)]
pub enum StructuralFilterParseError {
    /// The argument does not contain the field-target separator.
    #[error("expected FIELD=ENTRY_ADDRESS[,ENTRY_ADDRESS]")]
    MissingEquals,
    /// The link relation name is empty.
    #[error("link relation name must not be empty")]
    EmptyField,
    /// The target entry address list contains a separator without a target.
    #[error("structural filter contains an empty target")]
    EmptyTarget,
    /// A target entry address is invalid.
    #[error(transparent)]
    EntryAddress(#[from] EntryAddressError),
}

/// Structural link state filter parsed from `FIELD=STATE`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructuralStateFilter {
    /// Link relation name.
    pub field: String,
    /// Accepted state for this relation.
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

/// Structural link state matched by `sirno query --is`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralFieldState {
    /// The relation is present with any target count.
    Present,
    /// The relation is present with no targets.
    Empty,
    /// The relation is absent.
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
    /// The link relation name is empty.
    #[error("link relation name must not be empty")]
    EmptyField,
    /// The structural link state is not recognized.
    #[error("unknown structural link state `{0}`; expected present, empty, or missing")]
    UnknownState(String),
}

/// Entry query request shared by CLI and tool callers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QueryRequest {
    /// Vague text terms matched against expanded entry text.
    pub terms: Vec<String>,
    /// Exact text terms matched against entry-local text.
    pub exact_terms: Vec<String>,
    /// Structural link target filters.
    pub has: Vec<StructuralFilter>,
    /// Structural link state filters.
    pub is: Vec<StructuralStateFilter>,
    /// Output columns to materialize.
    pub columns: QueryColumnSelection,
}

/// Query execution result before presentation rendering.
#[derive(Debug)]
pub enum QueryRun {
    /// The caller requested selectable column names without selecting entries.
    ColumnOptions(QueryColumns),
    /// The lake did not pass the edit-mode checks needed for query.
    InvalidLake {
        /// Columns selected for the attempted query.
        columns: QueryColumns,
        /// Lake report that blocked query execution.
        report: EntryDirectoryReport,
    },
    /// The query completed and produced rows.
    Results(QueryResults),
}

/// Structured query rows plus the selected column order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryResults {
    pub(crate) columns: QueryColumns,
    pub(crate) rows: Vec<Vec<QueryValue>>,
}

impl QueryResults {
    /// Build query results from selected columns and materialized rows.
    pub fn new(columns: QueryColumns, rows: Vec<Vec<QueryValue>>) -> Self {
        Self { columns, rows }
    }

    /// Return selected columns in display order.
    pub fn columns(&self) -> &QueryColumns {
        &self.columns
    }

    /// Return raw row values in selected column order.
    pub fn rows(&self) -> &[Vec<QueryValue>] {
        &self.rows
    }

    /// Return JSON-ready records keyed by selected column labels.
    pub fn records(&self) -> Vec<IndexMap<String, QueryValue>> {
        query_result_records(&self.columns, &self.rows)
    }

    /// Render the result rows as pretty JSON.
    pub fn to_json(&self) -> Result<String, CommandError> {
        format_query_json(&self.columns, &self.rows)
    }
}

/// Entry address lookup request shared by CLI and tool callers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryPathsRequest {
    /// Entry address whose paths should be returned.
    pub id: EntryAddress,
    /// Selected path classes.
    pub selection: PathSelection,
    /// Whether returned paths should be absolute.
    pub absolute: bool,
}

impl EntryPathsRequest {
    /// Build a path lookup request from explicit typed fields.
    pub fn new(id: EntryAddress, selection: PathSelection, absolute: bool) -> Self {
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

/// Structural link target for typed command callers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralTarget {
    /// Link relation name.
    pub field: String,
    /// Target entry address.
    pub target: EntryAddress,
}

/// Entry creation request shared by the CLI and tool callers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryNewRequest {
    /// Entry address.
    pub id: EntryAddress,
    /// Human-readable entry name.
    pub name: Option<String>,
    /// Short entry description.
    pub desc: String,
    /// Structural link targets.
    #[serde(default)]
    pub structural: Vec<StructuralTarget>,
    /// Initial Markdown body.
    pub body: Option<String>,
}

/// Result that points at one entry file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryFileResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry address affected by the command.
    pub id: String,
    /// Sirno Lake entry file path affected by the command.
    pub path: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of clearing or repairing local filesystem protection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalProtectionResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Whether this run only reported selected paths.
    pub dry_run: bool,
    /// Sirno Lake directory inspected by the command.
    pub lake_path: String,
    /// Paths selected by the local protection operation.
    pub paths: Vec<String>,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of reading one Sirno Lake Markdown entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryReadResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry address that was read.
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

/// Result of renaming one entry address.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryRenameResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry address before the rename.
    pub old_id: String,
    /// Entry address after the rename.
    pub new_id: String,
    /// Paths updated by the rename.
    pub updated_paths: Vec<String>,
    /// Concise human-readable summary.
    pub message: String,
}

/// Query result designed for JSON-first callers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResponse {
    /// Whether the request completed against a clean-enough lake.
    pub ok: bool,
    /// Available or selected output columns.
    pub columns: Vec<String>,
    /// Query records keyed by column label.
    pub records: Vec<IndexMap<String, QueryValue>>,
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

// sirno:witness:mcp-interface:begin
/// One JSON-ready witness record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WitnessRecordResult {
    /// Compact MCP witness record for ordinary agent use.
    Compact(CompactWitnessRecordResult),
    /// Verbose MCP witness record for callers that need structured coordinates.
    Verbose(VerboseWitnessRecordResult),
}

/// Compact JSON-ready witness record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactWitnessRecordResult {
    /// Entry address captured by the witness block.
    pub entry: String,
    /// Repository location of the matched witness block.
    pub location: String,
    /// Full matched witness block body.
    pub body: String,
}

/// Verbose JSON-ready witness record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerboseWitnessRecordResult {
    /// Entry address captured by the witness block.
    pub entry: String,
    /// Repository file path containing the witness.
    pub path: String,
    /// Full matched block region.
    pub region: WitnessSpanResult,
    /// Full matched witness block body.
    pub body: String,
}

impl WitnessRecordResult {
    pub(crate) fn from_record(record: &WitnessRecord, verbose_json: bool) -> Self {
        if verbose_json {
            return Self::Verbose(VerboseWitnessRecordResult {
                entry: record.entry.to_string(),
                path: display_path(&record.path),
                region: WitnessSpanResult::from(record.region),
                body: record.body.clone(),
            });
        }

        Self::Compact(CompactWitnessRecordResult {
            entry: record.entry.to_string(),
            location: format_witness_location(record),
            body: record.body.clone(),
        })
    }
}

fn format_witness_location(record: &WitnessRecord) -> String {
    format!(
        "{}:{}:{}-{}:{}",
        display_path(&record.path),
        record.region.start_line,
        record.region.start_column,
        record.region.end_line,
        record.region.end_column
    )
}
// sirno:witness:mcp-interface:end

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
    /// Entry address used for lookup.
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
    /// Entry address whose artifacts were listed.
    pub id: String,
    /// Owner-relative artifact paths.
    pub artifacts: Vec<String>,
}

/// Artifact add request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactAddRequest {
    /// Entry address that will own the artifact.
    pub id: EntryAddress,
    /// Source file to copy.
    pub source: PathBuf,
    /// Owner-relative artifact path.
    pub artifact_path: Option<PathBuf>,
}

/// Artifact rename request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRenameRequest {
    /// Entry address that owns the artifact.
    pub id: EntryAddress,
    /// Existing owner-relative artifact path.
    pub old_path: PathBuf,
    /// New owner-relative artifact path.
    pub new_path: PathBuf,
}

/// Artifact removal request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRemoveRequest {
    /// Entry address that owns the artifact.
    pub id: EntryAddress,
    /// Owner-relative artifact path to remove.
    pub artifact_path: PathBuf,
}

/// Result of changing one artifact file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactChangeResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry address that owns the artifact.
    pub id: String,
    /// Owner-relative artifact path.
    pub artifact_path: String,
    /// Filesystem path affected by the command.
    pub path: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// One discovered charm entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharmRecord {
    /// Entry address that owns the charm.
    pub id: String,
    /// Human-readable entry name.
    pub name: String,
    /// Whether the charm is enabled in project config.
    pub enabled: bool,
    /// Charm kind: direct or source.
    pub kind: String,
    /// Charm manifest path.
    pub manifest_path: String,
}

/// Result of listing charm entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharmListResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Discovered charms.
    pub charms: Vec<CharmRecord>,
    /// Concise human-readable summary.
    pub message: String,
}

/// Detailed charm record for one entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharmShowResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Entry address that owns the charm.
    pub id: String,
    /// Human-readable entry name.
    pub name: String,
    /// Whether the charm is enabled in project config.
    pub enabled: bool,
    /// Charm kind: direct or source.
    pub kind: String,
    /// Charm manifest path.
    pub manifest_path: String,
    /// Artifact root path.
    pub artifact_root: String,
    /// Spell cache directory for this charm fingerprint.
    pub spell_cache_path: String,
    /// Declared spell command.
    pub spell_command: Vec<String>,
    /// Whether a setup command exists.
    pub has_setup: bool,
    /// Whether a check command exists.
    pub has_check: bool,
    /// Whether a build command exists.
    pub has_build: bool,
    /// Declared hook ids.
    pub hooks: Vec<String>,
}

/// Result of enabling or disabling one charm.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharmEnablementResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Whether project config changed.
    pub changed: bool,
    /// Entry address.
    pub id: String,
    /// Project config path.
    pub config_path: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of running a charm or spell command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharmProcessResult {
    /// Whether the process was skipped or exited successfully.
    pub ok: bool,
    /// Entry address.
    pub id: String,
    /// Operation phase.
    pub phase: String,
    /// Whether no command was declared for this phase.
    pub skipped: bool,
    /// Process exit code.
    pub exit_code: Option<i32>,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error.
    pub stderr: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of cleaning spell cache state for one charm.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharmCleanResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Whether any cache path was removed.
    pub removed: bool,
    /// Entry address.
    pub id: String,
    /// Cache root selected by the command.
    pub path: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// One spell resolved from an enabled charm.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellRecord {
    /// Entry address that owns the charm.
    pub id: String,
    /// Human-readable entry name.
    pub name: String,
    /// Spell kind: direct or source.
    pub kind: String,
    /// Spell cache directory for this charm fingerprint.
    pub spell_cache_path: String,
}

/// Result of listing spells.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellListResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Resolved spells.
    pub spells: Vec<SpellRecord>,
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
        Self::from_report_with_extra_changed_paths(report, dry, &[])
    }

    pub(crate) fn from_report_with_extra_changed_paths(
        report: &GenLinkDirectoryReport, dry: bool, extra_changed_paths: &[PathBuf],
    ) -> Self {
        let mut changed_path_bufs = report.changed_paths().to_vec();
        changed_path_bufs.extend_from_slice(extra_changed_paths);
        changed_path_bufs.sort();
        changed_path_bufs.dedup();
        let changed_paths = display_paths(&changed_path_bufs);
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
                &changed_path_bufs,
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

/// JSON-ready mist projection status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MistStatusResult {
    /// Whether the misty lake has no intake blockers or pending ripples.
    pub ok: bool,
    /// Whether a projection manifest was found.
    pub manifest_present: bool,
    /// Mist name that was inspected.
    pub mist: String,
    /// Mist spec path used for the projection.
    pub spec_path: String,
    /// Reservoir path used for comparison.
    pub reservoir_path: String,
    /// Misty lake projection path.
    pub projection_path: String,
    /// Whether projection edits can be intaken.
    pub editable: bool,
    /// Number of entries recorded in the current manifest.
    pub entry_count: usize,
    /// Entries whose projected source differs from the reservoir source.
    pub changed_entries: Vec<String>,
    /// Entries whose reservoir fingerprint no longer matches the manifest.
    pub stale_entries: Vec<String>,
    /// Entries recorded in the manifest but missing from the projection.
    pub missing_entries: Vec<String>,
    /// Git-staged paths below the misty lake projection.
    pub staged_paths: Vec<String>,
    /// Concise human-readable summary.
    pub message: String,
}

impl MistStatusResult {
    /// Return true when the projection has changed or blocked state.
    pub fn has_ripples_or_blockers(&self) -> bool {
        !self.changed_entries.is_empty()
            || !self.stale_entries.is_empty()
            || !self.missing_entries.is_empty()
            || !self.staged_paths.is_empty()
            || !self.manifest_present
    }
}

/// JSON-ready mist intake result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MistIntakeResult {
    /// Whether the intake completed.
    pub ok: bool,
    /// Mist name that was intaken.
    pub mist: String,
    /// Reservoir path that received accepted edits.
    pub reservoir_path: String,
    /// Misty lake projection path that supplied edits.
    pub projection_path: String,
    /// Entry addresses written back to the reservoir.
    pub updated_entries: Vec<String>,
    /// Reservoir paths changed by intake and projection rerendering.
    pub changed_paths: Vec<String>,
    /// Concise human-readable summary.
    pub message: String,
}

/// Structured edge policy for one structural link direction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralEdgeStatus {
    /// Whether generated footers render this edge direction.
    pub render: bool,
    /// Whether lake-side neighbors create tide workitems.
    pub ripple_lake: bool,
    /// Whether accepted-baseline neighbors create tide workitems.
    pub ripple_anchor: bool,
}

impl StructuralEdgeStatus {
    pub(crate) fn from_settings(settings: &StructuralEdgeSettings) -> Self {
        Self {
            render: settings.render,
            ripple_lake: settings.ripple.lake,
            ripple_anchor: settings.ripple.anchor,
        }
    }
}

/// Link relation status in one Sirno config.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralFieldStatus {
    /// Link relation name.
    pub field: String,
    /// Entry that defines the relation.
    pub entry: String,
    /// Outgoing edge settings.
    pub to: StructuralEdgeStatus,
    /// Incoming edge settings.
    pub from: StructuralEdgeStatus,
    /// Shared-target edge settings.
    pub clique: StructuralEdgeStatus,
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
    /// Status check policy.
    pub check_policy: StatusCheckPolicy,
    /// Configured link relation summaries.
    pub structural_fields: Vec<StructuralFieldStatus>,
    /// Tide summary when the lake can be compared against the active review baseline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tide: Option<StatusTide>,
    /// Default mist projection status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mist: Option<MistStatusResult>,
    /// Review-mode check result.
    pub check: LakeCheckResult,
}

/// Tide workitem selection by exact workitems or neighbor ids.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TideSelectionRequest {
    /// Select all workitems whose neighbor matches one of these entry addresses.
    #[serde(default)]
    pub neighbors: Vec<EntryAddress>,
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
    /// Select all workitems whose neighbor matches one of these entry addresses.
    #[serde(default)]
    pub neighbors: Vec<EntryAddress>,
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
    /// Entry addresses that still need review.
    pub review_entries: Vec<EntryAddress>,
    /// Full workitem statuses when requested.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<TideStatus>,
}

/// Selected path classes for an entry address lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathSelection {
    pub(crate) entry: bool,
    pub(crate) artifact: bool,
}

impl PathSelection {
    /// Select entry and artifact paths.
    pub fn all() -> Self {
        Self { entry: true, artifact: true }
    }

    /// Build an explicit path-class selection.
    pub fn new(entry: bool, artifact: bool) -> Self {
        Self { entry, artifact }
    }
}

/// One filesystem path returned by an entry address lookup.
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
