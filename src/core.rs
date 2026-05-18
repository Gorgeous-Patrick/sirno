//! Shared command surface for Sirno.
//!
//! This module owns command parsing, command execution, and presentation rendering.
//! The binary front door delegates here, and future MCP tools can call the typed helpers directly.

use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fmt};

use crate::{
    CONFIG_FILE_NAME, CheckMode, ConfigError, Entry, EntryArtifactPath, EntryArtifactPathError,
    EntryDirectory, EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryReport,
    EntryDirectoryWritePolicy, EntryId, EntryIdError, EntryMetadata, EntryParseError, EntryQuery,
    EntryStructuralMatcher, Eterator, FrostError, FrostLockStatus, GenLinkDirectoryReport,
    GeneratedLinkBody, GeneratedLinkError, LockError, SirnoConfig, SirnoFrost, SirnoLock,
    StructuralSettings, Tide, TideError, TideSource, TideStatus, TideWorkitem,
    TideWorkitemParseError, TutorialSettings, VagueEntryQuery, WitnessCheckSettings, WitnessError,
    WitnessRecord,
};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unicode_width::UnicodeWidthStr;

const RG_PREPROCESSOR_ARGV0_PREFIX: &str = "sirno-rg-preprocess-";

/// Sirno command-line entry point.
#[derive(Debug, Parser)]
#[command(name = "sirno")]
#[command(about = "Manage Sirno design entries")]
pub struct Cli {
    /// Sirno project config file.
    #[arg(short = 'C', long, global = true)]
    config: Option<PathBuf>,
    /// Public Markdown lake path override.
    #[arg(short = 'L', long = "lake-path", global = true)]
    lake_path: Option<PathBuf>,
    /// Sirno Frost path override for commands that inspect Frost directly.
    #[arg(short = 'F', long = "frost-path", global = true)]
    frost_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

/// Supported Sirno commands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Create a Sirno config, public lake, and Frost store.
    Init {
        /// Monograph path written to Sirno.toml.
        #[arg(long)]
        mono: Option<PathBuf>,
        /// Public Markdown entry lake path written to Sirno.toml.
        #[arg(long)]
        lake: Option<PathBuf>,
        /// Sirno Frost path written to Sirno.toml.
        #[arg(long)]
        frost: Option<PathBuf>,
    },
    /// Move an entry, the public lake path, or the Frost path.
    #[command(visible_alias = "mv")]
    Move {
        /// Move target.
        #[command(subcommand)]
        command: MoveCommand,
    },
    /// Manage public Markdown lake entries.
    Entry {
        /// Entry command.
        #[command(subcommand)]
        command: EntryCommand,
    },
    /// Manage public Markdown lake storage.
    Lake {
        /// Lake command.
        #[command(subcommand)]
        command: LakeCommand,
    },
    /// Manage optional Sirno Frost snapshots.
    Frost {
        /// Frost command.
        #[command(subcommand)]
        command: FrostCommand,
    },
    /// Manage dependency review worklists for lake edits.
    Tide {
        /// Tide command.
        #[command(subcommand)]
        command: TideCommand,
    },
    /// Run an entry operation at the top level.
    // sirno:witness:interfaces:begin
    #[command(flatten)]
    TopLevelEntry(TopLevelEntryCommand),
    /// Run a lake operation at the top level.
    #[command(flatten)]
    TopLevelLake(TopLevelLakeCommand),
    /// Run a Frost snapshot operation at the top level.
    #[command(flatten)]
    TopLevelFrost(TopLevelFrostCommand),
    /// Run a tide review operation at the top level.
    #[command(flatten)]
    TopLevelTide(TideReviewCommand),
    /// Utility commands.
    Util {
        /// Utility command.
        #[command(subcommand)]
        command: UtilCommand,
    },
    // sirno:witness:interfaces:end
}

/// Supported public entry commands.
#[derive(Debug, Subcommand)]
enum EntryCommand {
    /// Run a top-level entry operation under `sirno entry`.
    #[command(flatten)]
    TopLevel(TopLevelEntryCommand),
    /// Rename one entry id and its Sirno references.
    #[command(visible_aliases = ["mv", "move"])]
    Rename(EntryRenameArgs),
}

/// Supported top-level public entry commands.
#[derive(Debug, Subcommand)]
enum TopLevelEntryCommand {
    /// Create one Markdown entry.
    // sirno:witness:interfaces:begin
    New {
        /// Entry id and filename stem.
        id: String,
        /// Human-readable entry name.
        #[arg(short = 'n', long)]
        name: Option<String>,
        /// Short entry desc.
        #[arg(short = 'd', long)]
        desc: String,
        /// Structural metadata target as FIELD=ENTRY_ID.
        #[arg(long = "structural", value_name = "FIELD=ENTRY_ID")]
        structural: Vec<StructuralPredicate>,
        /// Initial Markdown body.
        #[arg(short = 'b', long)]
        body: Option<String>,
    },
    // sirno:witness:interfaces:end
    /// Freeze one current Frost entry and make its public file read-only.
    // sirno:witness:interfaces:begin
    Freeze {
        /// Entry id to freeze.
        id: String,
    },
    // sirno:witness:interfaces:end
    /// Melt one public Markdown entry and make its file writable.
    // sirno:witness:interfaces:begin
    #[command(visible_alias = "unfreeze")]
    Melt {
        /// Entry id to melt.
        id: String,
    },
    // sirno:witness:interfaces:end
    /// Show filesystem paths related to one entry.
    // sirno:witness:interfaces:begin
    Path(EntryPathArgs),
    // sirno:witness:interfaces:end
    /// Query public Markdown entries.
    // sirno:witness:interfaces:begin
    #[command(visible_alias = "q")]
    Query {
        /// Vague text terms matched against entries and structural target summaries.
        terms: Vec<String>,
        /// Exact text term matched against id, name, desc, and body.
        #[arg(long = "exact-term")]
        exact_terms: Vec<String>,
        /// Structural target filter as FIELD=ENTRY_ID[,ENTRY_ID].
        ///
        /// Different fields narrow results.
        /// Comma-separated values and repeated same-field filters are alternatives.
        #[arg(long = "has", value_name = "FIELD=ENTRY_ID[,ENTRY_ID]")]
        has: Vec<StructuralFilter>,
        /// Structural field state filter as FIELD=present, FIELD=empty, or FIELD=missing.
        ///
        /// Empty means the field is present with no targets.
        /// Same-field target filters and state filters are alternatives.
        #[arg(long = "is", value_name = "FIELD=STATE")]
        is: Vec<StructuralStateFilter>,
        /// Comma-separated output columns: id, name, path, desc.
        #[arg(long = "columns", value_name = "COLUMNS")]
        columns: Option<QueryColumns>,
        /// Output format.
        #[arg(short = 'o', long, value_enum)]
        format: Option<QueryOutputFormat>,
    },
    // sirno:witness:interfaces:end
    /// Run ripgrep in the configured public Markdown lake.
    // sirno:witness:interfaces:begin
    Rg {
        /// Include Sirno-owned generated-footer regions in the search.
        #[arg(long = "with-generated-footer")]
        with_generated_footer: bool,
        /// Arguments forwarded to ripgrep before the lake path.
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<OsString>,
    },
    // sirno:witness:interfaces:end
    /// Manage entry-owned artifact files.
    // sirno:witness:interfaces:begin
    Artifact {
        /// Artifact command.
        #[command(subcommand)]
        command: ArtifactCommand,
    },
    // sirno:witness:interfaces:end
    /// Show repository witness blocks for one entry id.
    // sirno:witness:interfaces:begin
    #[command(visible_aliases = ["w", "wit"])]
    Witness {
        /// Entry id used as the witness query key.
        id: String,
        /// Print full witness regions instead of only their locations.
        #[arg(short = 'f', long)]
        full: bool,
    },
    // sirno:witness:interfaces:end
}

/// Supported public lake commands.
#[derive(Debug, Subcommand)]
enum LakeCommand {
    /// Create a Sirno config and ordinary seed entries.
    // sirno:witness:interfaces:begin
    Init {
        /// Public Markdown entry lake path written to Sirno.toml.
        lake: Option<PathBuf>,
    },
    /// Move the configured public Markdown entry lake.
    #[command(visible_alias = "mv")]
    Move(LakeMoveArgs),
    /// Run a top-level lake operation under `sirno lake`.
    #[command(flatten)]
    TopLevel(TopLevelLakeCommand),
    // sirno:witness:interfaces:end
}

/// Supported top-level public lake commands.
#[derive(Debug, Subcommand)]
enum TopLevelLakeCommand {
    /// Check current entry structure.
    Check {
        /// Check boundary.
        #[arg(short = 'm', long, value_enum)]
        mode: Option<CheckModeArg>,
    },
    /// Render Markdown links in entry footers.
    Render {
        /// Report rendered-footer changes without writing files.
        #[arg(short = 'n', long, visible_alias = "dry-run")]
        dry: bool,
        /// Render command.
        #[command(subcommand)]
        command: Option<RenderCommand>,
    },
    /// Show the current Sirno project status.
    #[command(visible_alias = "st")]
    Status,
}

/// Supported top-level move wrappers.
// sirno:witness:interfaces:begin
#[derive(Debug, Subcommand)]
enum MoveCommand {
    /// Rename one entry id and its Sirno references.
    Entry(EntryRenameArgs),
    /// Move the configured public Markdown entry lake.
    Lake(LakeMoveArgs),
    /// Move the configured Sirno Frost path.
    Frost(FrostMoveArgs),
}

/// Arguments for renaming one entry id and its Sirno references.
#[derive(Debug, Args)]
struct EntryRenameArgs {
    /// Existing entry id.
    old_id: String,
    /// New entry id.
    new_id: String,
}

/// Arguments for moving the configured public Markdown entry lake.
#[derive(Debug, Args)]
struct LakeMoveArgs {
    /// New public Markdown entry lake path written to Sirno.toml.
    lake: PathBuf,
}

/// Arguments for moving the configured Sirno Frost path.
#[derive(Debug, Args)]
struct FrostMoveArgs {
    /// New Sirno Frost path written to Sirno.toml.
    frost: PathBuf,
}
// sirno:witness:interfaces:end

/// Arguments for entry path lookup.
// sirno:witness:interfaces:begin
#[derive(Clone, Debug, Args)]
struct EntryPathArgs {
    /// Entry id whose paths should be shown.
    id: String,
    /// Show the public Markdown entry file path.
    #[arg(long = "entry")]
    show_entry: bool,
    /// Show public entry artifact paths.
    #[arg(long = "artifact")]
    show_artifact: bool,
    /// Show private Frost backend paths when Frost is configured.
    #[arg(long = "frost")]
    show_frost: bool,
    /// Print absolute paths.
    #[arg(long)]
    absolute: bool,
    /// Output format.
    #[arg(short = 'o', long, value_enum)]
    format: Option<PathOutputFormat>,
}
// sirno:witness:interfaces:end

/// CLI path lookup output renderer.
// sirno:witness:interfaces:begin
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum PathOutputFormat {
    /// Print a JSON array of path records.
    Json,
    /// Print a Unicode table.
    #[default]
    Human,
    /// Print only paths, one per line.
    Paths,
}
// sirno:witness:interfaces:end

/// Supported entry artifact commands.
// sirno:witness:interfaces:begin
#[derive(Debug, Subcommand)]
enum ArtifactCommand {
    /// List artifacts owned by one entry.
    List {
        /// Entry id whose artifacts should be listed.
        id: String,
    },
    /// Copy a file into one entry's artifact tree.
    Add {
        /// Entry id that will own the artifact.
        id: String,
        /// Source file to copy.
        source: PathBuf,
        /// Owner-relative artifact path.
        artifact_path: Option<PathBuf>,
    },
    /// Rename one artifact path owned by an entry.
    #[command(visible_aliases = ["mv", "move"])]
    Rename {
        /// Entry id that owns the artifact.
        id: String,
        /// Existing owner-relative artifact path.
        old_path: PathBuf,
        /// New owner-relative artifact path.
        new_path: PathBuf,
    },
    /// Remove one artifact owned by an entry.
    #[command(visible_aliases = ["rm", "delete"])]
    Remove {
        /// Entry id that owns the artifact.
        id: String,
        /// Owner-relative artifact path to remove.
        artifact_path: PathBuf,
    },
}
// sirno:witness:interfaces:end

/// CLI representation of check boundaries.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CheckModeArg {
    /// Editing boundary: dangling references are warnings.
    Edit,
    /// Review boundary: dangling references are errors.
    Review,
}

impl From<CheckModeArg> for CheckMode {
    fn from(value: CheckModeArg) -> Self {
        match value {
            | CheckModeArg::Edit => CheckMode::Edit,
            | CheckModeArg::Review => CheckMode::Review,
        }
    }
}

/// Shared human-or-JSON output renderer.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum StructuredOutputFormat {
    /// Print JSON for machine-oriented callers.
    Json,
    /// Print terminal-oriented human text.
    #[default]
    Human,
}

type QueryOutputFormat = StructuredOutputFormat;

/// Query output column list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryColumns {
    columns: Vec<QueryColumn>,
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

/// Structural metadata predicate parsed from `FIELD=ENTRY_ID`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct StructuralPredicate {
    field: String,
    target: EntryId,
}

impl FromStr for StructuralPredicate {
    type Err = StructuralPredicateParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let Some((field, target)) = raw.split_once('=') else {
            return Err(StructuralPredicateParseError::MissingEquals);
        };
        if field.is_empty() {
            return Err(StructuralPredicateParseError::EmptyField);
        }
        let target = EntryId::new(target)?;
        Ok(Self { field: field.to_owned(), target })
    }
}

/// Error raised while parsing one structural `FIELD=ENTRY_ID` argument.
#[derive(Debug, Error)]
enum StructuralPredicateParseError {
    /// The argument does not contain the field-target separator.
    #[error("expected FIELD=ENTRY_ID")]
    MissingEquals,
    /// The structural field name is empty.
    #[error("structural field name must not be empty")]
    EmptyField,
    /// The target entry id is invalid.
    #[error(transparent)]
    EntryId(#[from] EntryIdError),
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

/// Supported Sirno Frost commands.
#[derive(Debug, Subcommand)]
enum FrostCommand {
    /// Configure Sirno Frost.
    Init {
        /// Sirno Frost path written to Sirno.toml.
        frost: Option<PathBuf>,
    },
    /// Move the configured Sirno Frost path.
    #[command(visible_alias = "mv")]
    Move(FrostMoveArgs),
    /// Run a Frost snapshot operation.
    #[command(flatten)]
    Snapshot(TopLevelFrostCommand),
}

/// Supported top-level Sirno Frost commands.
#[derive(Debug, Subcommand)]
enum TopLevelFrostCommand {
    /// Freeze the current public Markdown lake.
    Commit {
        /// Bypass open tide workitems for this commit without recording resolutions.
        #[arg(long = "unsafe-resolve-all")]
        unsafe_resolve_all: bool,
    },
    /// Check out the latest Frost version as the mutable current lake.
    Defrost,
    /// Check out Frost entries into the public Markdown lake.
    Checkout(CheckoutArgs),
}

/// Arguments for checking out Frost entries into the public Markdown lake.
#[derive(Debug, Args)]
struct CheckoutArgs {
    /// Version coordinate to materialize in the current Frost generation.
    #[arg(required_unless_present = "latest", conflicts_with = "latest")]
    version: Option<u64>,
    /// Check out the latest Frost version as the mutable current lake.
    #[arg(long, conflicts_with = "unsafe_mutable")]
    latest: bool,
    /// Leave an explicit version checkout writable.
    #[arg(long)]
    unsafe_mutable: bool,
}

/// Tide item selector parsed from one CLI argument.
#[derive(Clone, Debug, PartialEq, Eq)]
enum TideItemSelector {
    /// Select every open workitem whose neighbor matches this entry.
    Neighbor(EntryId),
    /// Select one full workitem tuple.
    Workitem(TideWorkitem),
}

impl FromStr for TideItemSelector {
    type Err = TideItemSelectorParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if raw.contains(',') {
            return Ok(Self::Workitem(raw.parse()?));
        }
        Ok(Self::Neighbor(EntryId::new(raw)?))
    }
}

/// Error raised while parsing one tide item selector.
#[derive(Debug, Error)]
enum TideItemSelectorParseError {
    /// Entry id parsing failed.
    #[error(transparent)]
    EntryId(#[from] EntryIdError),
    /// Full workitem parsing failed.
    #[error(transparent)]
    Workitem(#[from] TideWorkitemParseError),
}

/// Supported tide commands.
#[derive(Debug, Subcommand)]
// sirno:witness:tide:begin
enum TideCommand {
    /// Show tide workitems.
    Status {
        /// Include resolved workitems.
        #[arg(long)]
        all: bool,
        /// Output format.
        #[arg(short = 'o', long, value_enum)]
        format: Option<TideOutputFormat>,
    },
    /// Run a tide review operation.
    #[command(flatten)]
    Review(TideReviewCommand),
    /// Clear all tide resolutions from the lock.
    Reset,
}
// sirno:witness:tide:end

/// Supported tide review commands.
#[derive(Debug, Subcommand)]
enum TideReviewCommand {
    /// Resolve tide workitems.
    Resolve(ResolveArgs),
    /// Remove resolved marks from tide workitems.
    #[command(visible_alias = "reopen")]
    Unresolve(UnresolveArgs),
}

/// Arguments for resolving tide workitems.
#[derive(Debug, Args)]
struct ResolveArgs {
    /// Resolve workitems whose neighbor also appears in the current ripple set.
    #[arg(long, conflicts_with_all = ["items", "json"])]
    infer: bool,
    /// JSON array of full workitem tuples.
    #[arg(long, conflicts_with_all = ["infer", "items"])]
    json: Option<String>,
    /// Entry ids or full workitem tuples.
    #[arg(required_unless_present_any = ["infer", "json"])]
    items: Vec<TideItemSelector>,
}

/// Arguments for removing resolved marks from tide workitems.
#[derive(Debug, Args)]
struct UnresolveArgs {
    /// Entry ids or full workitem tuples.
    #[arg(required = true)]
    items: Vec<TideItemSelector>,
}

type TideOutputFormat = StructuredOutputFormat;

/// Supported rendered-footer commands.
#[derive(Debug, Subcommand)]
enum RenderCommand {
    /// Delete generated Markdown link footers.
    Delete,
}

/// CLI shell target for completion generation.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CompletionShell {
    /// Bash completion script.
    Bash,
    /// Elvish completion script.
    Elvish,
    /// Fish completion script.
    Fish,
    /// PowerShell completion script.
    #[value(name = "powershell", alias = "power-shell")]
    PowerShell,
    /// Zsh completion script.
    Zsh,
}

impl From<CompletionShell> for Shell {
    fn from(value: CompletionShell) -> Self {
        match value {
            | CompletionShell::Bash => Shell::Bash,
            | CompletionShell::Elvish => Shell::Elvish,
            | CompletionShell::Fish => Shell::Fish,
            | CompletionShell::PowerShell => Shell::PowerShell,
            | CompletionShell::Zsh => Shell::Zsh,
        }
    }
}

/// Supported utility commands.
#[derive(Debug, Subcommand)]
enum UtilCommand {
    /// Generate a shell completion script.
    Completion {
        /// Shell whose completion script should be generated.
        #[arg(value_enum)]
        shell: CompletionShell,
    },
    // sirno:witness:interfaces:begin
    /// Run the Sirno MCP server over stdio.
    Mcp,
    // sirno:witness:interfaces:end
}

/// Core command context shared by the CLI and future tool interfaces.
#[derive(Clone, Debug)]
pub struct CoreContext {
    config_path: PathBuf,
    lake_path: Option<PathBuf>,
}

impl CoreContext {
    /// Create a context rooted at one Sirno config path.
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self { config_path: config_path.into(), lake_path: None }
    }

    /// Override the public lake path used by lake-backed operations.
    pub fn with_lake_path(mut self, lake_path: impl Into<PathBuf>) -> Self {
        self.lake_path = Some(lake_path.into());
        self
    }

    fn from_cli_paths(config_path: &Path, lake_path: Option<&Path>) -> Self {
        let mut context = Self::new(config_path.to_path_buf());
        if let Some(lake_path) = lake_path {
            context = context.with_lake_path(lake_path.to_path_buf());
        }
        context
    }

    /// Query entries and return structured rows before presentation rendering.
    pub fn query_entries(&self, request: QueryRequest) -> Result<QueryRun, CommandError> {
        let (lake, mut settings) =
            resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        settings.render = false;
        settings.witness = None;
        let report = EntryDirectory::new(&lake).check_with_settings(CheckMode::Edit, &settings)?;
        if report.has_errors() {
            return Ok(QueryRun::InvalidLake(report));
        }

        let vague_query = VagueEntryQuery::new().with_text_terms(request.terms);
        let filtered_query = entry_query_from_filters(
            EntryQuery::new().with_text_terms(request.exact_terms),
            request.has,
            request.is,
            &settings.structural,
        )?;
        let vague_matches = vague_query.select_entries(report.entries());
        let matches = filtered_query.select_entries(vague_matches);
        let rows = query_result_rows(&report, &matches, &request.columns)?;
        Ok(QueryRun::Results(QueryResults::new(request.columns, rows)))
    }

    /// Return filesystem paths related to one entry.
    pub fn entry_paths(&self, request: EntryPathRequest) -> Result<Vec<PathRecord>, CommandError> {
        let config = SirnoConfig::from_file(&self.config_path)?;
        let lake = resolve_lake_path(self.lake_path.as_deref(), &self.config_path, &config);
        let directory = EntryDirectory::new(&lake);
        directory.read_entry(&request.id)?;
        let artifacts = directory.read_entry_artifacts(&request.id)?;
        let mut records = Vec::new();

        if request.selection.entry {
            records.push(PathRecord::new(
                "entry",
                output_path(directory.entry_path(&request.id), request.absolute)?,
            ));
        }
        if request.selection.artifact {
            records.push(PathRecord::new(
                "artifact-root",
                output_path(directory.entry_artifact_root_path(&request.id), request.absolute)?,
            ));
            for artifact in &artifacts {
                records.push(PathRecord::new(
                    "artifact",
                    output_path(
                        directory.entry_artifact_path(&request.id, &artifact.path),
                        request.absolute,
                    )?,
                ));
            }
        }
        if request.selection.frost
            && let Some(frost) = config.resolve_frost(&self.config_path)
        {
            records.push(PathRecord::new(
                "frost-entry",
                output_path(
                    SirnoFrost::entry_storage_path(&frost, &request.id)?,
                    request.absolute,
                )?,
            ));
        }

        Ok(records)
    }

    /// Return tide statuses in structured form.
    pub fn tide_statuses(&self, all: bool) -> Result<Vec<TideStatus>, CommandError> {
        let context = TideContext::load(&self.config_path, self.lake_path.as_deref())?;
        let lock = context.load_lock_or_current()?;
        let tide = context.tide(&lock)?;
        Ok(tide_statuses_for_output(&tide, all))
    }

    /// Return tide statuses as a JSON-first command result.
    pub fn tide_status(&self, all: bool) -> Result<TideStatusResult, CommandError> {
        let statuses = self.tide_statuses(all)?;
        Ok(TideStatusResult { ok: statuses.iter().all(|status| status.resolved), statuses })
    }

    /// Return repository witness records for one entry.
    pub fn witness_records(&self, id: &EntryId) -> Result<Vec<WitnessRecord>, CommandError> {
        let config = SirnoConfig::from_file(&self.config_path)?;
        let lake = resolve_lake_path(self.lake_path.as_deref(), &self.config_path, &config);
        if !EntryDirectory::new(&lake).entry_exists(id)? {
            return Err(CommandError::MissingWitnessEntry(id.clone()));
        }
        let Some(settings) = witness_check_settings(&self.config_path, &config) else {
            return Err(CommandError::RepoMembersNotConfigured);
        };
        let index = settings.scan()?;
        Ok(index.records_for(id).to_vec())
    }

    /// Create a Sirno config and ordinary seed entries.
    pub fn lake_init(&self, request: LakeInitRequest) -> Result<LakeInitResult, CommandError> {
        let config = SirnoConfig::new(
            request
                .lake
                .or_else(|| self.lake_path.clone())
                .unwrap_or_else(|| default_lake_path(&self.config_path)),
        );
        let lake_path = config.resolve_lake(&self.config_path);
        config.write_new(&self.config_path)?;
        let paths = EntryDirectory::new(&lake_path).init()?;
        Ok(LakeInitResult {
            ok: true,
            config_path: display_path(&self.config_path),
            lake_path: display_path(&lake_path),
            entry_count: paths.len(),
            message: format!(
                "initialized {} with {} entries in {}",
                self.config_path.display(),
                paths.len(),
                lake_path.display()
            ),
        })
    }

    /// Create one Markdown entry.
    pub fn entry_new(&self, request: EntryNewRequest) -> Result<EntryPathResult, CommandError> {
        let (lake, settings) =
            resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        let mut metadata = EntryMetadata::new(
            request.name.unwrap_or_else(|| title_name_from_id(&request.id)),
            request.desc,
        )?;
        for (field, targets) in
            structural_targets_by_target(request.structural, &settings.structural)?
        {
            metadata.set_structural_targets(field, targets);
        }

        let entry = Entry::new(request.id.clone(), metadata, request.body.unwrap_or_default());
        let path = EntryDirectory::new(&lake).create_entry(&entry)?;
        Ok(EntryPathResult {
            ok: true,
            id: request.id.to_string(),
            path: display_path(&path),
            message: format!("created {}", path.display()),
        })
    }

    /// Rename one entry id and its Sirno references.
    pub fn entry_rename(
        &self, old_id: EntryId, new_id: EntryId,
    ) -> Result<EntryRenameResult, CommandError> {
        let (lake, settings) =
            resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        let report = EntryDirectory::new(&lake).rename_entry(&old_id, &new_id, &settings)?;
        let mut changed_paths = report.changed_paths().to_vec();
        if let Some(witness) = &settings.witness {
            changed_paths.extend(witness.rename_entry_references(&old_id, &new_id)?);
        }
        changed_paths.sort();
        changed_paths.dedup();
        let changed_paths = display_paths(&changed_paths);
        Ok(EntryRenameResult {
            ok: true,
            old_id: old_id.to_string(),
            new_id: new_id.to_string(),
            updated_paths: changed_paths,
            message: format!("renamed entry {old_id} to {new_id}"),
        })
    }

    /// Freeze one current Frost entry and make its public file read-only.
    pub fn entry_freeze(&self, id: EntryId) -> Result<EntryPathResult, CommandError> {
        let context = FrostContext::load(&self.config_path, self.lake_path.as_deref())?;
        context.reject_immutable_checkout()?;
        let directory = context.lake();
        let entry = directory.read_entry(&id)?;
        let artifacts = directory.read_entry_artifacts(&id)?;
        let frost = SirnoFrost::open(&context.frost_path)?;
        frost.ensure_entry_bundle_current(&entry, &artifacts)?;
        let path = directory.freeze_entry(&id)?;
        Ok(EntryPathResult {
            ok: true,
            id: id.to_string(),
            path: display_path(&path),
            message: format!("froze entry {id} at {}", path.display()),
        })
    }

    /// Melt one public Markdown entry and make its file writable.
    pub fn entry_melt(&self, id: EntryId) -> Result<EntryPathResult, CommandError> {
        let (lake, _) = resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        let path = EntryDirectory::new(&lake).melt_entry(&id)?;
        Ok(EntryPathResult {
            ok: true,
            id: id.to_string(),
            path: display_path(&path),
            message: format!("melted entry {id} at {}", path.display()),
        })
    }

    /// Query entries and return an MCP-friendly JSON result.
    pub fn entry_query(&self, request: QueryRequest) -> Result<QueryResponse, CommandError> {
        let columns = request.columns.clone();
        match self.query_entries(request)? {
            | QueryRun::InvalidLake(report) => Ok(QueryResponse {
                ok: false,
                columns: columns.labels(),
                records: Vec::new(),
                diagnostics: diagnostics_from_entry_report(&report),
            }),
            | QueryRun::Results(results) => Ok(QueryResponse {
                ok: true,
                columns: results.columns.labels(),
                records: results.records(),
                diagnostics: Vec::new(),
            }),
        }
    }

    /// Run ripgrep in the configured public Markdown lake and capture its output.
    pub fn entry_rg(&self, request: RgRequest) -> Result<RgResult, CommandError> {
        if !request.with_generated_footer
            && request.args.iter().any(|arg| arg == "--pre" || arg.starts_with("--pre="))
        {
            return Err(CommandError::RgPreprocessorConflict);
        }

        let lake = resolve_lake_path_for_rg(self.lake_path.as_deref(), &self.config_path)?;
        let preprocessor =
            if request.with_generated_footer { None } else { Some(RgPreprocessorLink::create()?) };

        let mut command = ProcessCommand::new("rg");
        if let Some(preprocessor) = &preprocessor {
            command.arg("--pre").arg(preprocessor.path()).arg("--pre-glob").arg("*.md");
        }
        let output = command.args(&request.args).arg(lake).output().map_err(CommandError::RunRg)?;
        let exit_code = output.status.code().and_then(|code| u8::try_from(code).ok()).unwrap_or(1);
        Ok(RgResult {
            ok: output.status.success(),
            exit_code,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    /// Return repository witness blocks for one entry.
    pub fn entry_witness(&self, id: EntryId, full: bool) -> Result<WitnessResult, CommandError> {
        let records = self.witness_records(&id)?;
        Ok(WitnessResult {
            ok: !records.is_empty(),
            id: id.to_string(),
            records: records
                .iter()
                .map(|record| WitnessRecordResult::from_record(record, full))
                .collect(),
            message: if records.is_empty() {
                format!("no witness found for {id}")
            } else {
                format!("found {} witness records for {id}", records.len())
            },
        })
    }

    /// List artifacts owned by one entry.
    pub fn entry_artifact_list(&self, id: EntryId) -> Result<ArtifactListResult, CommandError> {
        let (lake, _) = resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        let directory = EntryDirectory::new(&lake);
        directory.read_entry(&id)?;
        let artifacts = directory
            .read_entry_artifacts(&id)?
            .into_iter()
            .map(|artifact| artifact.path.to_string())
            .collect::<Vec<_>>();
        Ok(ArtifactListResult { ok: true, id: id.to_string(), artifacts })
    }

    /// Copy a file into one entry's artifact tree.
    pub fn entry_artifact_add(
        &self, request: ArtifactAddRequest,
    ) -> Result<ArtifactChangeResult, CommandError> {
        let (lake, _) = resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        let directory = EntryDirectory::new(&lake);
        let artifact_path = match request.artifact_path {
            | Some(path) => artifact_path_from_cli(&path)?,
            | None => default_artifact_path_from_source(&request.source)?,
        };
        let path = directory.add_entry_artifact(&request.id, &request.source, &artifact_path)?;
        Ok(ArtifactChangeResult {
            ok: true,
            id: request.id.to_string(),
            artifact_path: artifact_path.to_string(),
            path: display_path(&path),
            message: format!("added artifact {artifact_path} at {}", path.display()),
        })
    }

    /// Rename one artifact path owned by an entry.
    pub fn entry_artifact_rename(
        &self, request: ArtifactRenameRequest,
    ) -> Result<ArtifactChangeResult, CommandError> {
        let (lake, _) = resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        let directory = EntryDirectory::new(&lake);
        let old_path = artifact_path_from_cli(&request.old_path)?;
        let new_path = artifact_path_from_cli(&request.new_path)?;
        let path = directory.rename_entry_artifact(&request.id, &old_path, &new_path)?;
        Ok(ArtifactChangeResult {
            ok: true,
            id: request.id.to_string(),
            artifact_path: new_path.to_string(),
            path: display_path(&path),
            message: format!("renamed artifact {old_path} to {new_path} at {}", path.display()),
        })
    }

    /// Remove one artifact owned by an entry.
    pub fn entry_artifact_remove(
        &self, request: ArtifactRemoveRequest,
    ) -> Result<ArtifactChangeResult, CommandError> {
        let (lake, _) = resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        let directory = EntryDirectory::new(&lake);
        let artifact_path = artifact_path_from_cli(&request.artifact_path)?;
        let path = directory.remove_entry_artifact(&request.id, &artifact_path)?;
        Ok(ArtifactChangeResult {
            ok: true,
            id: request.id.to_string(),
            artifact_path: artifact_path.to_string(),
            path: display_path(&path),
            message: format!("removed artifact {artifact_path} at {}", path.display()),
        })
    }

    /// Move the configured public Markdown entry lake.
    pub fn lake_move(&self, lake: PathBuf) -> Result<MovePathResult, CommandError> {
        let config = SirnoConfig::from_file(&self.config_path)?;
        let old_lake = config.resolve_lake(&self.config_path);
        let config = config.with_lake(lake);
        config.validate_for_file(&self.config_path)?;
        let new_lake = config.resolve_lake(&self.config_path);
        let moved = move_configured_path_and_write_config(
            &old_lake,
            &new_lake,
            &config,
            &self.config_path,
        )?;
        Ok(MovePathResult {
            ok: true,
            moved,
            old_path: display_path(&old_lake),
            new_path: display_path(&new_lake),
            message: format!("moved lake {} to {}", old_lake.display(), new_lake.display()),
        })
    }

    /// Check current entry structure.
    pub fn lake_check(&self, mode: CheckMode) -> Result<LakeCheckResult, CommandError> {
        let (lake, settings) =
            resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        let report = EntryDirectory::new(lake).check_with_settings(mode, &settings)?;
        Ok(LakeCheckResult::from_report(&report))
    }

    /// Render Markdown links in entry footers.
    pub fn lake_render(&self, dry: bool) -> Result<RenderResult, CommandError> {
        let (lake, mut settings) =
            resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        settings.render = false;
        settings.witness = None;

        let directory = EntryDirectory::new(&lake);
        let check = directory.check_with_settings(CheckMode::Review, &settings)?;
        if check.has_errors() {
            return Ok(RenderResult::blocked(&check));
        }

        let report = if dry {
            directory.check_generated_links_with_ignored_paths(
                &settings.structural,
                settings.ignore.clone(),
            )?
        } else {
            directory
                .generate_links_with_ignored_paths(&settings.structural, settings.ignore.clone())?
        };
        Ok(RenderResult::from_report(&report, dry))
    }

    /// Delete generated Markdown link footers.
    pub fn lake_render_delete(&self) -> Result<RenderResult, CommandError> {
        let (lake, mut settings) =
            resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        settings.witness = None;
        let report = EntryDirectory::new(&lake)
            .delete_generated_links_with_ignored_paths(settings.ignore)?;
        Ok(RenderResult::from_report(&report, false))
    }

    /// Show the current Sirno project status.
    pub fn lake_status(&self) -> Result<StatusResult, CommandError> {
        let config = SirnoConfig::from_file(&self.config_path)?;
        let mono = config.resolve_mono(&self.config_path);
        let frost = config.resolve_frost(&self.config_path);
        let lock_path = SirnoLock::path_for_config(&self.config_path);
        let lock = if frost.is_some() { SirnoLock::from_file_if_exists(&lock_path)? } else { None };
        let (lake, settings) =
            resolve_lake_directory(self.lake_path.as_deref(), &self.config_path)?;
        let report =
            EntryDirectory::new(&lake).check_with_settings(CheckMode::Review, &settings)?;
        Ok(StatusResult {
            ok: !report.has_errors(),
            config_path: display_path(&self.config_path),
            mono_path: mono.as_ref().map(|path| display_path(path)),
            lake_path: display_path(report.root()),
            frost_path: frost.as_ref().map(|path| display_path(path)),
            frost_state: frost_state_label(lock.as_ref()),
            entry_count: report.entries().len(),
            check_render: config.check.render,
            structural_fields: config
                .structural
                .fields()
                .map(|(field, settings)| StructuralFieldStatus {
                    field: field.to_owned(),
                    to: settings.to.to_string(),
                    from: settings.from.to_string(),
                    clique: settings.clique.to_string(),
                })
                .collect(),
            check: LakeCheckResult::from_report(&report),
        })
    }

    /// Configure Sirno Frost.
    pub fn frost_init(&self, frost: Option<PathBuf>) -> Result<FrostInitResult, CommandError> {
        let config = SirnoConfig::from_file(&self.config_path)?;
        let existing_frost = config.frost.as_ref().map(|settings| settings.path.clone());
        let frost = frost
            .or_else(|| existing_frost.clone())
            .unwrap_or_else(|| default_frost_path(&self.config_path));
        if let Some(existing_frost) = existing_frost
            && existing_frost != frost
        {
            return Err(CommandError::FrostAlreadyConfigured(existing_frost));
        }

        let needs_config_write = config.frost.is_none();
        let config = if needs_config_write { config.with_frost(frost) } else { config };
        config.validate_for_file(&self.config_path)?;

        let frost_path = config.resolve_frost(&self.config_path).expect("frost path configured");
        let frost = SirnoFrost::open(&frost_path)?;
        let version = frost.current_snapshot()?;
        if needs_config_write {
            config.write(&self.config_path)?;
        }
        SirnoLock::current(version).write(SirnoLock::path_for_config(&self.config_path))?;
        Ok(FrostInitResult {
            ok: true,
            frost_path: display_path(&frost_path),
            version: version.version(),
            message: format!(
                "initialized frost {} at version {}",
                frost_path.display(),
                version.version()
            ),
        })
    }

    /// Move the configured Sirno Frost path.
    pub fn frost_move(&self, frost: PathBuf) -> Result<MovePathResult, CommandError> {
        let config = SirnoConfig::from_file(&self.config_path)?;
        let Some(old_frost) = config.resolve_frost(&self.config_path) else {
            return Err(CommandError::FrostNotConfigured);
        };
        let config = config.with_frost(frost);
        config.validate_for_file(&self.config_path)?;
        let new_frost = config.resolve_frost(&self.config_path).expect("frost path configured");
        let moved = move_configured_path_and_write_config(
            &old_frost,
            &new_frost,
            &config,
            &self.config_path,
        )?;
        Ok(MovePathResult {
            ok: true,
            moved,
            old_path: display_path(&old_frost),
            new_path: display_path(&new_frost),
            message: format!("moved frost {} to {}", old_frost.display(), new_frost.display()),
        })
    }

    /// Freeze the current public Markdown lake.
    pub fn frost_commit(
        &self, unsafe_resolve_all: bool,
    ) -> Result<FrostCommitResult, CommandError> {
        let context = FrostContext::load(&self.config_path, self.lake_path.as_deref())?;
        context.reject_immutable_checkout()?;
        if !unsafe_resolve_all {
            let tide_context = TideContext::load(&self.config_path, self.lake_path.as_deref())?;
            let lock = tide_context.load_lock_or_current()?;
            let tide = tide_context.tide(&lock)?;
            if !tide.is_clear() {
                return Err(CommandError::OpenTide {
                    count: tide.open_statuses().count(),
                    tutorial: OpenTideTutorial::new(
                        context.tutorial,
                        lock.frost.version == Eterator::EMPTY.version(),
                    ),
                });
            }
        }
        let mut frost = SirnoFrost::open(&context.frost_path)?;
        let version = frost.commit_entry_directory(&context.lake_path, &context.settings)?;
        context.lake().set_writable(&context.settings)?;
        let mut lock = SirnoLock::current(version);
        lock.tide.clear();
        lock.write(&context.lock_path)?;
        Ok(FrostCommitResult {
            ok: true,
            version: version.version(),
            lake_path: display_path(&context.lake_path),
            message: format!(
                "froze version {} from {}",
                version.version(),
                context.lake_path.display()
            ),
        })
    }

    /// Check out Frost entries into the public Markdown lake.
    pub fn frost_checkout(
        &self, request: FrostCheckoutRequest,
    ) -> Result<FrostCheckoutResult, CommandError> {
        let context = FrostContext::load(&self.config_path, self.lake_path.as_deref())?;
        let frost = SirnoFrost::open(&context.frost_path)?;
        let snapshot = if request.latest {
            frost.current_snapshot()?
        } else {
            let Some(version) = request.version else {
                return Err(CommandError::MissingFrostCheckoutTarget);
            };
            frost.snapshot_for_version(frost_version(version)?)?
        };
        if snapshot.version() == Eterator::EMPTY.version() {
            return Err(CommandError::InvalidFrostVersion(snapshot.version()));
        }
        let paths = frost.checkout_entry_directory(
            snapshot,
            &context.lake_path,
            EntryDirectoryWritePolicy::ReplaceDirectory { ignore: context.settings.ignore.clone() },
        )?;
        if request.latest || request.unsafe_mutable {
            context.lake().set_writable(&context.settings)?;
        } else {
            context.lake().add_readonly_checkout_warnings(&paths)?;
            context.lake().set_readonly(&context.settings)?;
        }
        if request.latest {
            SirnoLock::current(snapshot).write(&context.lock_path)?;
        } else {
            SirnoLock::checked_out(snapshot, request.unsafe_mutable).write(&context.lock_path)?;
        }
        let state = if request.latest {
            "mutable"
        } else if request.unsafe_mutable {
            "unsafe mutable"
        } else {
            "immutable"
        };
        Ok(FrostCheckoutResult {
            ok: true,
            version: snapshot.version(),
            lake_path: display_path(&context.lake_path),
            entry_count: paths.len(),
            state: state.to_owned(),
            message: format!(
                "checked out {}frost version {} into {} ({} entries, {})",
                if request.latest { "latest " } else { "" },
                snapshot.version(),
                context.lake_path.display(),
                paths.len(),
                state
            ),
        })
    }

    /// Check out the latest Frost version as the mutable current lake.
    pub fn frost_defrost(&self) -> Result<FrostCheckoutResult, CommandError> {
        self.frost_checkout(FrostCheckoutRequest {
            version: None,
            latest: true,
            unsafe_mutable: false,
        })
    }

    /// Resolve tide workitems.
    pub fn tide_resolve(
        &self, request: TideResolveRequest,
    ) -> Result<TideChangeResult, CommandError> {
        let context = TideContext::load(&self.config_path, self.lake_path.as_deref())?;
        let mut lock = context.load_lock_or_current()?;
        let tide = context.tide(&lock)?;
        let (resolutions, count) = if request.infer {
            tide.resolve_where(|status| tide.ripple_ids().contains(&status.workitem.neighbor))
        } else {
            tide.resolve_where(|status| tide_selection_matches(&request, status))
        };
        lock.tide.set_resolved(resolutions);
        lock.write(&context.lock_path)?;
        Ok(TideChangeResult {
            ok: true,
            count,
            message: format!("resolved {count} tide workitems"),
        })
    }

    /// Remove resolved marks from tide workitems.
    pub fn tide_unresolve(
        &self, request: TideSelectionRequest,
    ) -> Result<TideChangeResult, CommandError> {
        let context = TideContext::load(&self.config_path, self.lake_path.as_deref())?;
        let mut lock = context.load_lock_or_current()?;
        let tide = context.tide(&lock)?;
        let (resolutions, count) =
            tide.reopen_where(|status| tide_selection_request_matches(&request, status));
        lock.tide.set_resolved(resolutions);
        lock.write(&context.lock_path)?;
        Ok(TideChangeResult {
            ok: true,
            count,
            message: format!("unresolved {count} tide workitems"),
        })
    }

    /// Clear all tide resolutions from the lock.
    pub fn tide_reset(&self) -> Result<TideChangeResult, CommandError> {
        let context = TideContext::load(&self.config_path, self.lake_path.as_deref())?;
        let mut lock = context.load_lock_or_current()?;
        let count = lock.tide.resolved.len();
        lock.tide.clear();
        lock.write(&context.lock_path)?;
        Ok(TideChangeResult {
            ok: true,
            count,
            message: format!("cleared {count} tide resolutions"),
        })
    }
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
    columns: QueryColumns,
    rows: Vec<Vec<String>>,
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

    fn from_args(args: &EntryPathArgs) -> Result<Self, CommandError> {
        Ok(Self {
            id: EntryId::new(&args.id)?,
            selection: PathSelection::from_args(args),
            absolute: args.absolute,
        })
    }
}

/// Lake initialization request shared by non-CLI front ends.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LakeInitRequest {
    /// Public Markdown entry lake path written to `Sirno.toml`.
    pub lake: Option<PathBuf>,
}

/// Result of creating a public lake.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LakeInitResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Config file that was written.
    pub config_path: String,
    /// Public lake directory that was initialized.
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
    /// Public entry path affected by the command.
    pub path: String,
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
    /// Opening delimiter span.
    pub opening: WitnessSpanResult,
    /// Closing delimiter span.
    pub closing: WitnessSpanResult,
    /// Matched opening delimiter text.
    pub marker: String,
    /// Full witness body when requested.
    pub body: Option<String>,
}

impl WitnessRecordResult {
    fn from_record(record: &WitnessRecord, full: bool) -> Self {
        Self {
            entry: record.entry.to_string(),
            path: display_path(&record.path),
            region: WitnessSpanResult::from(record.region),
            opening: WitnessSpanResult::from(record.opening),
            closing: WitnessSpanResult::from(record.closing),
            marker: record.marker.clone(),
            body: full.then(|| record.body.clone()),
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
    fn from_report(report: &EntryDirectoryReport) -> Self {
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
    fn from_report(report: &GenLinkDirectoryReport, dry: bool) -> Self {
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

    fn blocked(report: &EntryDirectoryReport) -> Self {
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

/// Structural field status in one Sirno config.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralFieldStatus {
    /// Structural field name.
    pub field: String,
    /// Outgoing edge settings.
    pub to: String,
    /// Incoming edge settings.
    pub from: String,
    /// Shared-target edge settings.
    pub clique: String,
}

/// JSON-ready project status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusResult {
    /// Whether the configured public lake passes review checks.
    pub ok: bool,
    /// Config file used for the status command.
    pub config_path: String,
    /// Optional monograph path.
    pub mono_path: Option<String>,
    /// Public lake path.
    pub lake_path: String,
    /// Optional Frost path.
    pub frost_path: Option<String>,
    /// Current lock state summary.
    pub frost_state: String,
    /// Number of parsed entries.
    pub entry_count: usize,
    /// Whether generated-footer checks are enabled.
    pub check_render: bool,
    /// Configured structural field summaries.
    pub structural_fields: Vec<StructuralFieldStatus>,
    /// Review-mode check result.
    pub check: LakeCheckResult,
}

/// Result of initializing Frost.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostInitResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Configured Frost path.
    pub frost_path: String,
    /// Current Frost version after initialization.
    pub version: u64,
    /// Concise human-readable summary.
    pub message: String,
}

/// Result of committing a Frost snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostCommitResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// New Frost version.
    pub version: u64,
    /// Lake path committed to Frost.
    pub lake_path: String,
    /// Concise human-readable summary.
    pub message: String,
}

/// Frost checkout request.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostCheckoutRequest {
    /// Explicit Frost version to check out.
    pub version: Option<u64>,
    /// Check out the latest version as mutable current lake.
    #[serde(default)]
    pub latest: bool,
    /// Leave an explicit version checkout writable.
    #[serde(default)]
    pub unsafe_mutable: bool,
}

/// Result of checking out a Frost snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostCheckoutResult {
    /// Whether the command completed successfully.
    pub ok: bool,
    /// Checked-out Frost version.
    pub version: u64,
    /// Public lake path written by checkout.
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
    /// Whether no open workitem remains in the listed statuses.
    pub ok: bool,
    /// Tide workitem statuses.
    pub statuses: Vec<TideStatus>,
}

/// Render any serializable value as pretty JSON.
pub fn format_json<T: Serialize + ?Sized>(value: &T) -> Result<String, CommandError> {
    Ok(serde_json::to_string_pretty(value)?)
}

fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<(), CommandError> {
    println!("{}", format_json(value)?);
    Ok(())
}

/// Run Sirno from the current process environment.
///
/// This is the binary entry point extracted as a library function.
pub fn run_cli_from_env() -> ExitCode {
    if is_rg_preprocessor_invocation() {
        return match run_rg_preprocessor_from_env() {
            | Ok(code) => code,
            | Err(error) => {
                eprintln!("sirno: {error}");
                ExitCode::FAILURE
            }
        };
    }

    match Cli::parse().run() {
        | Ok(code) => code,
        | Err(error) => {
            eprintln!("sirno: {error}");
            ExitCode::FAILURE
        }
    }
}

impl Cli {
    /// Execute the parsed CLI command.
    pub fn run(self) -> Result<ExitCode, CommandError> {
        let config_path = self.config.unwrap_or_else(default_config_path);
        let lake_path = self.lake_path;
        let frost_path = self.frost_path;
        match self.command {
            | Command::Init { mono, lake, frost } => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                run_top_level_init(mono, lake, frost, &config_path, lake_path.as_deref())
            }
            | Command::Move { command } => {
                command.run(&config_path, lake_path.as_deref(), frost_path.as_deref())
            }
            | Command::Entry { command } => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::Lake { command } => {
                command.run(&config_path, lake_path.as_deref(), frost_path.as_deref())
            }
            | Command::Frost { command } => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::Tide { command } => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::TopLevelEntry(command) => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::TopLevelLake(command) => {
                command.run(&config_path, lake_path.as_deref(), frost_path.as_deref())
            }
            | Command::TopLevelFrost(command) => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::TopLevelTide(command) => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::Util { command } => {
                command.run(&config_path, lake_path.as_deref(), frost_path.as_deref())
            }
        }
    }
}

impl MoveCommand {
    fn run(
        self, config_path: &Path, lake_path: Option<&Path>, frost_path: Option<&Path>,
    ) -> Result<ExitCode, CommandError> {
        match self {
            | Self::Entry(args) => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                args.run(config_path, lake_path)
            }
            | Self::Lake(args) => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                args.run(config_path)
            }
            | Self::Frost(args) => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                args.run(config_path)
            }
        }
    }
}

fn run_top_level_init(
    mono: Option<PathBuf>, lake: Option<PathBuf>, frost: Option<PathBuf>, config_path: &Path,
    lake_path: Option<&Path>,
) -> Result<ExitCode, CommandError> {
    run_lake_init(mono, lake, config_path, lake_path)?;
    FrostCommand::Init { frost }.run(config_path, lake_path)
}

fn run_lake_init(
    mono: Option<PathBuf>, lake: Option<PathBuf>, config_path: &Path, lake_path: Option<&Path>,
) -> Result<ExitCode, CommandError> {
    if mono.is_none() {
        let result = CoreContext::from_cli_paths(config_path, lake_path)
            .lake_init(LakeInitRequest { lake })?;
        println!("{}", result.message);
        return Ok(ExitCode::SUCCESS);
    }

    let mut config = SirnoConfig::new(
        lake.or_else(|| lake_path.map(Path::to_path_buf))
            .unwrap_or_else(|| default_lake_path(config_path)),
    );
    if let Some(mono) = mono {
        config = config.with_mono(mono);
    }
    let lake_path = config.resolve_lake(config_path);
    config.write_new(config_path)?;
    let paths = EntryDirectory::new(&lake_path).init()?;
    println!(
        "initialized {} with {} entries in {}",
        config_path.display(),
        paths.len(),
        lake_path.display()
    );
    Ok(ExitCode::SUCCESS)
}

impl EntryCommand {
    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
        match self {
            | EntryCommand::TopLevel(command) => command.run(config_path, lake_path),
            | EntryCommand::Rename(args) => args.run(config_path, lake_path),
        }
    }
}

impl EntryRenameArgs {
    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
        let old_id = EntryId::new(&self.old_id)?;
        let new_id = EntryId::new(&self.new_id)?;
        let result =
            CoreContext::from_cli_paths(config_path, lake_path).entry_rename(old_id, new_id)?;
        println!("{}", result.message);
        println!("updated {} paths", result.updated_paths.len());
        Ok(ExitCode::SUCCESS)
    }
}

impl TopLevelEntryCommand {
    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
        match self {
            | TopLevelEntryCommand::New { id, name, desc, structural, body } => {
                let id = EntryId::new(&id)?;
                let structural = structural
                    .into_iter()
                    .map(|target| StructuralTarget { field: target.field, target: target.target })
                    .collect();
                let result = CoreContext::from_cli_paths(config_path, lake_path)
                    .entry_new(EntryNewRequest { id, name, desc, structural, body })?;
                println!("{}", result.message);
                Ok(ExitCode::SUCCESS)
            }
            | TopLevelEntryCommand::Freeze { id } => {
                let id = EntryId::new(&id)?;
                let result =
                    CoreContext::from_cli_paths(config_path, lake_path).entry_freeze(id)?;
                println!("{}", result.message);
                Ok(ExitCode::SUCCESS)
            }
            | TopLevelEntryCommand::Melt { id } => {
                let id = EntryId::new(&id)?;
                let result = CoreContext::from_cli_paths(config_path, lake_path).entry_melt(id)?;
                println!("{}", result.message);
                Ok(ExitCode::SUCCESS)
            }
            | TopLevelEntryCommand::Path(args) => {
                let records = entry_path_records(config_path, lake_path, &args)?;
                print_path_records(&records, args.format.unwrap_or_default())?;
                Ok(ExitCode::SUCCESS)
            }
            | TopLevelEntryCommand::Query { terms, exact_terms, has, is, columns, format } => {
                let request = QueryRequest {
                    terms,
                    exact_terms,
                    has,
                    is,
                    columns: columns.unwrap_or_default(),
                };
                let results = match CoreContext::from_cli_paths(config_path, lake_path)
                    .query_entries(request)?
                {
                    | QueryRun::InvalidLake(report) => {
                        print_entry_directory_report(&report);
                        return Ok(ExitCode::FAILURE);
                    }
                    | QueryRun::Results(results) => results,
                };
                let format = format.unwrap_or_default();
                print_query_results(&results, format)?;
                Ok(ExitCode::SUCCESS)
            }
            | TopLevelEntryCommand::Rg { with_generated_footer, args } => {
                let args = rg_args_to_strings(args)?;
                let result = CoreContext::from_cli_paths(config_path, lake_path)
                    .entry_rg(RgRequest { with_generated_footer, args })?;
                print!("{}", result.stdout);
                eprint!("{}", result.stderr);
                Ok(ExitCode::from(result.exit_code))
            }
            | TopLevelEntryCommand::Artifact { command } => command.run(config_path, lake_path),
            | TopLevelEntryCommand::Witness { id, full } => {
                run_witness_command(config_path, lake_path, &id, full)
            }
        }
    }
}

impl LakeCommand {
    fn run(
        self, config_path: &Path, lake_path: Option<&Path>, frost_path: Option<&Path>,
    ) -> Result<ExitCode, CommandError> {
        match self {
            | LakeCommand::Init { .. } | LakeCommand::Move(_) if frost_path.is_some() => {
                Err(CommandError::FrostPathRequiresCheck)
            }
            | LakeCommand::Init { lake } => run_lake_init(None, lake, config_path, lake_path),
            | LakeCommand::Move(args) => args.run(config_path),
            | LakeCommand::TopLevel(command) => command.run(config_path, lake_path, frost_path),
        }
    }
}

impl LakeMoveArgs {
    fn run(self, config_path: &Path) -> Result<ExitCode, CommandError> {
        let result = CoreContext::new(config_path.to_path_buf()).lake_move(self.lake)?;
        println!("{}", result.message);
        Ok(ExitCode::SUCCESS)
    }
}

impl TopLevelLakeCommand {
    fn run(
        self, config_path: &Path, lake_path: Option<&Path>, frost_path: Option<&Path>,
    ) -> Result<ExitCode, CommandError> {
        match self {
            | TopLevelLakeCommand::Check { mode } => {
                if lake_path.is_some() && frost_path.is_some() {
                    return Err(CommandError::LakePathWithFrostPath);
                }
                let mode = mode.unwrap_or(CheckModeArg::Review);
                if lake_path.is_some() {
                    let result = CoreContext::from_cli_paths(config_path, lake_path)
                        .lake_check(mode.into())?;
                    print_lake_check_result(&result);
                    return if result.has_errors {
                        Ok(ExitCode::FAILURE)
                    } else {
                        Ok(ExitCode::SUCCESS)
                    };
                }

                let Some(frost_path) = frost_path else {
                    let result =
                        CoreContext::new(config_path.to_path_buf()).lake_check(mode.into())?;
                    print_lake_check_result(&result);
                    return if result.has_errors {
                        Ok(ExitCode::FAILURE)
                    } else {
                        Ok(ExitCode::SUCCESS)
                    };
                };

                let frost = SirnoFrost::open(frost_path)?;
                let report = frost.check_current(mode.into())?;
                if report.is_clean() {
                    println!("ok: {}", frost.root().display());
                    return Ok(ExitCode::SUCCESS);
                }

                for diagnostic in report.diagnostics() {
                    println!("{}: {}", diagnostic.severity.label(), diagnostic.message());
                }

                if report.has_errors() { Ok(ExitCode::FAILURE) } else { Ok(ExitCode::SUCCESS) }
            }
            | TopLevelLakeCommand::Render { .. } | TopLevelLakeCommand::Status
                if frost_path.is_some() =>
            {
                Err(CommandError::FrostPathRequiresCheck)
            }
            | TopLevelLakeCommand::Render { command, dry } => match command {
                | None => {
                    let result =
                        CoreContext::from_cli_paths(config_path, lake_path).lake_render(dry)?;
                    print_render_result(&result);
                    if result.ok { Ok(ExitCode::SUCCESS) } else { Ok(ExitCode::FAILURE) }
                }
                | Some(RenderCommand::Delete) => {
                    if dry {
                        return Err(CommandError::DryWithRenderSubcommand);
                    }
                    let result =
                        CoreContext::from_cli_paths(config_path, lake_path).lake_render_delete()?;
                    print_render_result(&result);
                    Ok(ExitCode::SUCCESS)
                }
            },
            | TopLevelLakeCommand::Status => {
                let result = CoreContext::from_cli_paths(config_path, lake_path).lake_status()?;
                print_status_result(&result);
                if result.ok { Ok(ExitCode::SUCCESS) } else { Ok(ExitCode::FAILURE) }
            }
        }
    }
}

impl TopLevelFrostCommand {
    fn run(
        self, config_path: &std::path::Path, lake_path: Option<&Path>,
    ) -> Result<ExitCode, CommandError> {
        match self {
            | TopLevelFrostCommand::Commit { unsafe_resolve_all } => {
                let result = CoreContext::from_cli_paths(config_path, lake_path)
                    .frost_commit(unsafe_resolve_all)?;
                println!("{}", result.message);
                Ok(ExitCode::SUCCESS)
            }
            | TopLevelFrostCommand::Defrost => CheckoutArgs::latest().run(config_path, lake_path),
            | TopLevelFrostCommand::Checkout(args) => args.run(config_path, lake_path),
        }
    }
}

impl CheckoutArgs {
    fn latest() -> Self {
        Self { version: None, latest: true, unsafe_mutable: false }
    }

    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
        let result = CoreContext::from_cli_paths(config_path, lake_path).frost_checkout(
            FrostCheckoutRequest {
                version: self.version,
                latest: self.latest,
                unsafe_mutable: self.unsafe_mutable,
            },
        )?;
        println!("{}", result.message);
        Ok(ExitCode::SUCCESS)
    }
}

impl FrostCommand {
    fn run(
        self, config_path: &std::path::Path, lake_path: Option<&Path>,
    ) -> Result<ExitCode, CommandError> {
        match self {
            | FrostCommand::Init { frost } => {
                let result =
                    CoreContext::from_cli_paths(config_path, lake_path).frost_init(frost)?;
                println!("{}", result.message);
                Ok(ExitCode::SUCCESS)
            }
            | FrostCommand::Move(args) => args.run(config_path),
            | FrostCommand::Snapshot(command) => command.run(config_path, lake_path),
        }
    }
}

impl FrostMoveArgs {
    fn run(self, config_path: &Path) -> Result<ExitCode, CommandError> {
        let result = CoreContext::new(config_path.to_path_buf()).frost_move(self.frost)?;
        println!("{}", result.message);
        Ok(ExitCode::SUCCESS)
    }
}

impl TideCommand {
    fn run(
        self, config_path: &std::path::Path, lake_path: Option<&Path>,
    ) -> Result<ExitCode, CommandError> {
        match self {
            | TideCommand::Status { all, format } => {
                let statuses =
                    CoreContext::from_cli_paths(config_path, lake_path).tide_statuses(all)?;
                let format = format.unwrap_or_default();
                print_tide_statuses(&statuses, format)?;
                Ok(if statuses.iter().all(|status| status.resolved) {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                })
            }
            | TideCommand::Review(command) => command.run(config_path, lake_path),
            | TideCommand::Reset => {
                let result = CoreContext::from_cli_paths(config_path, lake_path).tide_reset()?;
                println!("{}", result.message);
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

impl TideReviewCommand {
    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
        match self {
            | Self::Resolve(args) => args.run(config_path, lake_path),
            | Self::Unresolve(args) => args.run(config_path, lake_path),
        }
    }
}

impl ResolveArgs {
    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
        let request = if self.infer {
            TideResolveRequest { infer: true, ..TideResolveRequest::default() }
        } else if let Some(json) = self.json {
            TideResolveRequest {
                workitems: tide_workitems_from_json(&json)?,
                ..TideResolveRequest::default()
            }
        } else {
            let selection = tide_selection_from_items(self.items);
            TideResolveRequest {
                neighbors: selection.neighbors,
                workitems: selection.workitems,
                ..TideResolveRequest::default()
            }
        };
        let result = CoreContext::from_cli_paths(config_path, lake_path).tide_resolve(request)?;
        println!("{}", result.message);
        Ok(ExitCode::SUCCESS)
    }
}

impl UnresolveArgs {
    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
        let request = tide_selection_from_items(self.items);
        let result = CoreContext::from_cli_paths(config_path, lake_path).tide_unresolve(request)?;
        println!("{}", result.message);
        Ok(ExitCode::SUCCESS)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TideJsonWorkitems {
    One(TideWorkitem),
    Many(Vec<TideWorkitem>),
}

fn tide_workitems_from_json(source: &str) -> Result<Vec<TideWorkitem>, CommandError> {
    Ok(match serde_json::from_str::<TideJsonWorkitems>(source)? {
        | TideJsonWorkitems::One(workitem) => vec![workitem],
        | TideJsonWorkitems::Many(workitems) => workitems,
    })
}

fn tide_selection_from_items(items: Vec<TideItemSelector>) -> TideSelectionRequest {
    let mut request = TideSelectionRequest::default();
    for item in items {
        match item {
            | TideItemSelector::Neighbor(id) => request.neighbors.push(id),
            | TideItemSelector::Workitem(workitem) => request.workitems.push(workitem),
        }
    }
    request
}

fn tide_statuses_for_output(tide: &Tide, all: bool) -> Vec<TideStatus> {
    tide.statuses().iter().filter(|status| all || !status.resolved).cloned().collect()
}

fn print_tide_statuses(
    statuses: &[TideStatus], format: TideOutputFormat,
) -> Result<(), CommandError> {
    match format {
        | TideOutputFormat::Json => {
            print_json(statuses)?;
        }
        | TideOutputFormat::Human => {
            if statuses.is_empty() {
                println!("tide: clear");
            } else {
                for status in statuses {
                    let state = if status.resolved { "resolved" } else { "open" };
                    let sources = status
                        .sources
                        .iter()
                        .map(|source| match source {
                            | TideSource::Lake => "lake",
                            | TideSource::Frost => "frost",
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    println!("{state}: {} [{sources}]", status.workitem);
                }
            }
        }
    }
    Ok(())
}

impl ArtifactCommand {
    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
        let context = CoreContext::from_cli_paths(config_path, lake_path);
        match self {
            | ArtifactCommand::List { id } => {
                let id = EntryId::new(&id)?;
                for artifact in context.entry_artifact_list(id)?.artifacts {
                    println!("{artifact}");
                }
                Ok(ExitCode::SUCCESS)
            }
            | ArtifactCommand::Add { id, source, artifact_path } => {
                let id = EntryId::new(&id)?;
                let result =
                    context.entry_artifact_add(ArtifactAddRequest { id, source, artifact_path })?;
                println!("{}", result.message);
                Ok(ExitCode::SUCCESS)
            }
            | ArtifactCommand::Rename { id, old_path, new_path } => {
                let id = EntryId::new(&id)?;
                let result = context.entry_artifact_rename(ArtifactRenameRequest {
                    id,
                    old_path,
                    new_path,
                })?;
                println!("{}", result.message);
                Ok(ExitCode::SUCCESS)
            }
            | ArtifactCommand::Remove { id, artifact_path } => {
                let id = EntryId::new(&id)?;
                let result =
                    context.entry_artifact_remove(ArtifactRemoveRequest { id, artifact_path })?;
                println!("{}", result.message);
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

impl UtilCommand {
    fn run(
        self, config_path: &Path, lake_path: Option<&Path>, frost_path: Option<&Path>,
    ) -> Result<ExitCode, CommandError> {
        match self {
            | UtilCommand::Completion { shell } => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                let shell = Shell::from(shell);
                let mut command = Cli::command();
                let mut stdout = std::io::stdout();
                generate(shell, &mut command, "sirno", &mut stdout);
                Ok(ExitCode::SUCCESS)
            }
            | UtilCommand::Mcp => {
                if lake_path.is_some() {
                    return Err(CommandError::McpRejectsLakePath);
                }
                if frost_path.is_some() {
                    return Err(CommandError::McpRejectsFrostPath);
                }
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .map_err(CommandError::CreateMcpRuntime)?;
                runtime
                    .block_on(crate::mcp::run_stdio(CoreContext::new(config_path.to_path_buf())))
                    .map_err(|error| CommandError::McpServer(error.to_string()))?;
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

fn move_configured_path_and_write_config(
    source: &Path, destination: &Path, config: &SirnoConfig, config_path: &Path,
) -> Result<bool, CommandError> {
    let moved = move_configured_path(source, destination)?;
    if let Err(config_error) = config.write(config_path) {
        if moved && let Err(rollback) = fs::rename(destination, source) {
            return Err(CommandError::MoveConfigWriteRollback {
                source_path: source.to_path_buf(),
                destination_path: destination.to_path_buf(),
                source: Box::new(config_error),
                rollback,
            });
        }
        return Err(CommandError::Config(config_error));
    }
    Ok(moved)
}

fn move_configured_path(source: &Path, destination: &Path) -> Result<bool, CommandError> {
    if source == destination {
        return Ok(false);
    }
    match fs::symlink_metadata(destination) {
        | Ok(_) => return Err(CommandError::MoveDestinationExists(destination.to_path_buf())),
        | Err(source) if source.kind() == ErrorKind::NotFound => {}
        | Err(source) => {
            return Err(CommandError::ReadMoveDestination {
                path: destination.to_path_buf(),
                source,
            });
        }
    }
    fs::rename(source, destination).map_err(|error| CommandError::MovePath {
        source_path: source.to_path_buf(),
        destination_path: destination.to_path_buf(),
        source: error,
    })?;
    Ok(true)
}

struct FrostContext {
    frost_path: PathBuf,
    lock_path: PathBuf,
    settings: EntryDirectoryCheckSettings,
    lake_path: PathBuf,
    tutorial: Option<TutorialSettings>,
}

struct TideContext {
    frost_path: PathBuf,
    lock_path: PathBuf,
    settings: EntryDirectoryCheckSettings,
    lake_path: PathBuf,
}

impl FrostContext {
    fn load(config_path: &Path, lake_path: Option<&Path>) -> Result<Self, CommandError> {
        let config = SirnoConfig::from_file(config_path)?;
        let Some(frost_path) = config.resolve_frost(config_path) else {
            return Err(CommandError::FrostNotConfigured);
        };
        Ok(Self {
            frost_path,
            lock_path: SirnoLock::path_for_config(config_path),
            settings: entry_directory_check_settings(config_path, &config),
            lake_path: resolve_lake_path(lake_path, config_path, &config),
            tutorial: config.tutorial,
        })
    }

    fn lake(&self) -> EntryDirectory {
        EntryDirectory::new(&self.lake_path)
    }

    fn reject_immutable_checkout(&self) -> Result<(), CommandError> {
        let Some(lock) = SirnoLock::from_file_if_exists(&self.lock_path)? else {
            return Ok(());
        };
        if lock.frost.is_checked_out() && !lock.frost.is_unsafe_mutable_checkout() {
            return Err(CommandError::ImmutableFrostCheckout(lock.frost.version));
        }
        Ok(())
    }
}

impl TideContext {
    fn load(config_path: &Path, lake_path: Option<&Path>) -> Result<Self, CommandError> {
        let config = SirnoConfig::from_file(config_path)?;
        let Some(frost_path) = config.resolve_frost(config_path) else {
            return Err(CommandError::FrostNotConfigured);
        };
        Ok(Self {
            frost_path,
            lock_path: SirnoLock::path_for_config(config_path),
            settings: entry_directory_check_settings(config_path, &config),
            lake_path: resolve_lake_path(lake_path, config_path, &config),
        })
    }

    fn load_lock_or_current(&self) -> Result<SirnoLock, CommandError> {
        let Some(lock) = SirnoLock::from_file_if_exists(&self.lock_path)? else {
            let frost = SirnoFrost::open(&self.frost_path)?;
            return Ok(SirnoLock::current(frost.current_snapshot()?));
        };
        Ok(lock)
    }

    fn tide(&self, lock: &SirnoLock) -> Result<Tide, CommandError> {
        let frost = SirnoFrost::open(&self.frost_path)?;
        let frostline = frost.read_all_entries_at_snapshot(frost.current_snapshot()?)?;
        let mut settings = self.settings.clone();
        settings.render = false;
        settings.witness = None;
        let report =
            EntryDirectory::new(&self.lake_path).check_with_settings(CheckMode::Edit, &settings)?;
        if report.has_errors() {
            return Err(EntryDirectoryError::InvalidEntryDirectory(self.lake_path.clone()).into());
        }
        Ok(Tide::from_entries(
            &frostline,
            report.entries(),
            &settings.structural,
            &lock.tide.resolved,
        )?)
    }
}

fn frost_version(version: u64) -> Result<Eterator, CommandError> {
    if version == Eterator::EMPTY.version() {
        return Err(CommandError::InvalidFrostVersion(version));
    }
    Ok(Eterator(version))
}

fn run_witness_command(
    config_path: &Path, lake_path: Option<&Path>, raw_id: &str, full: bool,
) -> Result<ExitCode, CommandError> {
    let id = EntryId::new(raw_id)?;
    let records = CoreContext::from_cli_paths(config_path, lake_path).witness_records(&id)?;
    if records.is_empty() {
        println!("no witness found for {id}");
        return Ok(ExitCode::FAILURE);
    }
    print_witness_records(&records, full);
    Ok(ExitCode::SUCCESS)
}

fn print_witness_records(records: &[WitnessRecord], full: bool) {
    print!("{}", format_witness_records(records, full));
}

fn rg_args_to_strings(args: Vec<OsString>) -> Result<Vec<String>, CommandError> {
    args.into_iter().map(|arg| arg.into_string().map_err(CommandError::RgArgumentNotUtf8)).collect()
}

#[cfg(test)]
fn rg_args_include_preprocessor(args: &[OsString]) -> bool {
    args.iter()
        .filter_map(|arg| arg.to_str())
        .any(|arg| arg == "--pre" || arg.starts_with("--pre="))
}

fn resolve_lake_path_for_rg(
    lake_path: Option<&Path>, config_path: &Path,
) -> Result<PathBuf, CommandError> {
    if let Some(lake_path) = lake_path {
        return Ok(lake_path.to_path_buf());
    }

    let config = SirnoConfig::from_file(config_path)?;
    Ok(config.resolve_lake(config_path))
}

fn is_rg_preprocessor_invocation() -> bool {
    env::args_os()
        .next()
        .and_then(|arg| PathBuf::from(arg).file_name().map(|name| name.to_os_string()))
        .is_some_and(|name| name.to_string_lossy().starts_with(RG_PREPROCESSOR_ARGV0_PREFIX))
}

fn run_rg_preprocessor_from_env() -> Result<ExitCode, CommandError> {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        return Err(CommandError::RgPreprocessorArgumentCount);
    };
    if args.next().is_some() {
        return Err(CommandError::RgPreprocessorArgumentCount);
    }

    run_rg_preprocessor(&PathBuf::from(path))
}

fn run_rg_preprocessor(path: &Path) -> Result<ExitCode, CommandError> {
    let body = fs::read_to_string(path).map_err(|source| {
        CommandError::ReadRgPreprocessorInput { path: path.to_path_buf(), source }
    })?;
    let masked = GeneratedLinkBody::new(&body).mask()?;
    io::stdout().write_all(masked.as_bytes()).map_err(CommandError::WriteRgPreprocessorOutput)?;
    Ok(ExitCode::SUCCESS)
}

#[derive(Debug)]
struct RgPreprocessorLink {
    path: PathBuf,
}

impl RgPreprocessorLink {
    fn create() -> Result<Self, CommandError> {
        let current_exe = env::current_exe().map_err(CommandError::LocateCurrentExe)?;
        let mut path = env::temp_dir();
        path.push(format!(
            "{RG_PREPROCESSOR_ARGV0_PREFIX}{}-{}",
            std::process::id(),
            current_time_nanos()
        ));
        #[cfg(not(unix))]
        if let Some(extension) = current_exe.extension() {
            path.set_extension(extension);
        }

        create_rg_preprocessor_invoker(&current_exe, &path).map_err(|source| {
            CommandError::CreateRgPreprocessorInvoker { path: path.clone(), source }
        })?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for RgPreprocessorLink {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn current_time_nanos() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()
}

#[cfg(unix)]
fn create_rg_preprocessor_invoker(current_exe: &Path, path: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(current_exe, path)
}

#[cfg(not(unix))]
fn create_rg_preprocessor_invoker(current_exe: &Path, path: &Path) -> io::Result<()> {
    fs::copy(current_exe, path).map(|_| ())
}

fn format_witness_records(records: &[WitnessRecord], full: bool) -> String {
    let mut out = String::new();
    for (index, record) in records.iter().enumerate() {
        if full && index > 0 {
            out.push_str("---\n\n");
        }
        out.push_str(&format_witness_record(record, full));
    }
    out
}

fn format_witness_record(record: &WitnessRecord, full: bool) -> String {
    let range = format_witness_summary(record);
    if !full {
        let marker =
            record.body.lines().next().map(str::to_owned).unwrap_or_else(|| record.marker.clone());
        return format!("{range}\t{marker}\n");
    }

    let mut out = format!("{range}\n\n");
    out.push_str(&record.body);
    if !record.body.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out
}

fn format_witness_summary(record: &WitnessRecord) -> String {
    format!(
        "{}:{}:{}-{} :: {}:{}-{}",
        record.path.display(),
        record.opening.start_line,
        record.opening.start_column,
        record.opening.end_column,
        record.closing.start_line,
        record.closing.start_column,
        record.closing.end_column
    )
}

fn default_config_path() -> PathBuf {
    PathBuf::from(CONFIG_FILE_NAME)
}

fn default_lake_path(config_path: &Path) -> PathBuf {
    default_repo_path(config_path, "lake")
}

fn default_frost_path(config_path: &Path) -> PathBuf {
    default_repo_path(config_path, "frost")
}

fn default_repo_path(config_path: &Path, suffix: &str) -> PathBuf {
    let mut name = default_repo_name(config_path);
    name.push("-");
    name.push(suffix);
    PathBuf::from(name)
}

fn default_repo_name(config_path: &Path) -> OsString {
    let config_dir = match config_path.parent().filter(|path| !path.as_os_str().is_empty()) {
        | Some(path) if path == Path::new(".") => env::current_dir().ok(),
        | Some(path) => Some(path.to_path_buf()),
        | None => env::current_dir().ok(),
    };
    config_dir
        .and_then(|path| path.file_name().map(OsString::from))
        .unwrap_or_else(|| OsString::from("sirno"))
}

fn artifact_path_from_cli(path: &Path) -> Result<EntryArtifactPath, CommandError> {
    Ok(EntryArtifactPath::new(path)?)
}

fn default_artifact_path_from_source(source: &Path) -> Result<EntryArtifactPath, CommandError> {
    let Some(file_name) = source.file_name() else {
        return Err(CommandError::ArtifactSourceHasNoFileName(source.to_path_buf()));
    };
    Ok(EntryArtifactPath::new(Path::new(file_name))?)
}

fn explicit_lake_check_settings(
    config_path: &std::path::Path,
) -> Result<EntryDirectoryCheckSettings, CommandError> {
    if config_path.exists() {
        let config = SirnoConfig::from_file(config_path)?;
        Ok(entry_directory_check_settings(config_path, &config))
    } else {
        Ok(EntryDirectoryCheckSettings::default())
    }
}

fn entry_directory_check_settings(
    config_path: &Path, config: &SirnoConfig,
) -> EntryDirectoryCheckSettings {
    EntryDirectoryCheckSettings {
        render: config.check.render,
        structural: config.structural.clone(),
        ignore: config.lake.ignore.clone(),
        witness: witness_check_settings(config_path, config),
    }
}

fn witness_check_settings(
    config_path: &Path, config: &SirnoConfig,
) -> Option<WitnessCheckSettings> {
    let repo = config.repo.as_ref()?;
    if repo.members.is_empty() {
        return None;
    }
    Some(WitnessCheckSettings::new(
        config_path.parent().unwrap_or_else(|| Path::new(".")),
        repo.members.clone(),
        config.witness.clone(),
    ))
}

fn resolve_lake_path(
    lake_path: Option<&Path>, config_path: &Path, config: &SirnoConfig,
) -> PathBuf {
    lake_path.map(Path::to_path_buf).unwrap_or_else(|| config.resolve_lake(config_path))
}

fn resolve_lake_directory(
    lake_path: Option<&Path>, config_path: &std::path::Path,
) -> Result<(PathBuf, EntryDirectoryCheckSettings), CommandError> {
    if let Some(lake_path) = lake_path {
        return Ok((lake_path.to_path_buf(), explicit_lake_check_settings(config_path)?));
    }

    let config = SirnoConfig::from_file(config_path)?;
    Ok((config.resolve_lake(config_path), entry_directory_check_settings(config_path, &config)))
}

fn entry_query_from_filters(
    mut query: EntryQuery, filters: Vec<StructuralFilter>, states: Vec<StructuralStateFilter>,
    structural: &StructuralSettings,
) -> Result<EntryQuery, CommandError> {
    for (field, matchers) in structural_matchers_by_field(filters, states, structural)? {
        for matcher in matchers {
            query = query.with_structural_matcher(field.clone(), matcher);
        }
    }
    Ok(query)
}

fn structural_matchers_by_field(
    filters: Vec<StructuralFilter>, states: Vec<StructuralStateFilter>,
    structural: &StructuralSettings,
) -> Result<IndexMap<String, Vec<EntryStructuralMatcher>>, CommandError> {
    let mut matchers_by_field = IndexMap::<String, Vec<EntryStructuralMatcher>>::new();
    for filter in filters {
        if !structural.contains_field(&filter.field) {
            return Err(CommandError::UnconfiguredStructuralField(filter.field));
        }
        matchers_by_field
            .entry(filter.field)
            .or_default()
            .push(EntryStructuralMatcher::Targets(filter.targets));
    }
    for state in states {
        if !structural.contains_field(&state.field) {
            return Err(CommandError::UnconfiguredStructuralField(state.field));
        }
        matchers_by_field.entry(state.field).or_default().push(state.state.into());
    }
    Ok(matchers_by_field)
}

fn structural_targets_by_target(
    targets: Vec<StructuralTarget>, structural: &StructuralSettings,
) -> Result<IndexMap<String, Vec<EntryId>>, CommandError> {
    let mut targets_by_field = IndexMap::<String, Vec<EntryId>>::new();
    for target in targets {
        if !structural.contains_field(&target.field) {
            return Err(CommandError::UnconfiguredStructuralField(target.field));
        }
        targets_by_field.entry(target.field).or_default().push(target.target);
    }
    Ok(targets_by_field)
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

fn display_paths(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().map(|path| display_path(path)).collect()
}

fn diagnostics_from_entry_report(report: &EntryDirectoryReport) -> Vec<DiagnosticRecord> {
    let mut diagnostics = Vec::new();
    for diagnostic in report.file_diagnostics() {
        diagnostics.push(DiagnosticRecord {
            severity: diagnostic.severity.label().to_owned(),
            path: Some(display_path(&diagnostic.path)),
            message: diagnostic.message.clone(),
        });
    }
    for diagnostic in report.structural_report().diagnostics() {
        diagnostics.push(DiagnosticRecord {
            severity: diagnostic.severity.label().to_owned(),
            path: report.entry_path(&diagnostic.entry).map(display_path),
            message: diagnostic.message(),
        });
    }
    diagnostics
}

fn tide_selection_matches(request: &TideResolveRequest, status: &TideStatus) -> bool {
    request.neighbors.iter().any(|id| &status.workitem.neighbor == id)
        || request.workitems.iter().any(|workitem| &status.workitem == workitem)
}

fn tide_selection_request_matches(request: &TideSelectionRequest, status: &TideStatus) -> bool {
    request.neighbors.iter().any(|id| &status.workitem.neighbor == id)
        || request.workitems.iter().any(|workitem| &status.workitem == workitem)
}

fn title_name_from_id(id: &EntryId) -> String {
    id.as_str()
        .split('-')
        .map(|segment| {
            let mut chars = segment.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut word = first.to_uppercase().to_string();
            word.push_str(chars.as_str());
            word
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn print_status_result(result: &StatusResult) {
    println!("config: {}", result.config_path);
    if let Some(mono) = &result.mono_path {
        println!("mono: {mono}");
    } else {
        println!("mono: (not configured)");
    }
    println!("lake: {}", result.lake_path);
    if let Some(frost) = &result.frost_path {
        println!("frost: {frost}");
        println!("frost-state: {}", result.frost_state);
    } else {
        println!("frost: (not configured)");
    }
    println!("entries: {}", result.entry_count);
    println!("checks:");
    println!("  render: {}", result.check_render);
    println!("structural:");
    for field in &result.structural_fields {
        println!("  {}.to: {}", field.field, field.to);
        println!("  {}.from: {}", field.field, field.from);
        println!("  {}.clique: {}", field.field, field.clique);
    }
    if result.ok {
        println!("check: ok");
    } else {
        println!("check: failed");
        print_diagnostics(&result.check.diagnostics);
    }
}

fn print_lake_check_result(result: &LakeCheckResult) {
    if result.diagnostics.is_empty() {
        println!("ok: {}", result.root);
    } else {
        print_diagnostics(&result.diagnostics);
    }
}

fn print_render_result(result: &RenderResult) {
    if result.diagnostics.is_empty() {
        println!("{}", result.message);
    } else {
        print_diagnostics(&result.diagnostics);
    }
}

fn print_diagnostics(diagnostics: &[DiagnosticRecord]) {
    for diagnostic in diagnostics {
        if let Some(path) = &diagnostic.path {
            println!("{}: {}: {}", diagnostic.severity, path, diagnostic.message);
        } else {
            println!("{}: {}", diagnostic.severity, diagnostic.message);
        }
    }
}

fn frost_state_label(lock: Option<&SirnoLock>) -> String {
    let Some(lock) = lock else {
        return "(unlocked)".to_owned();
    };
    match lock.frost.status {
        | FrostLockStatus::Current => {
            format!(
                "current version {} (generation {}, mutable)",
                lock.frost.version, lock.frost.generation
            )
        }
        | FrostLockStatus::CheckedOut if lock.frost.mutable => {
            format!(
                "checked-out version {} (generation {}, unsafe mutable)",
                lock.frost.version, lock.frost.generation
            )
        }
        | FrostLockStatus::CheckedOut => {
            format!(
                "checked-out version {} (generation {}, immutable)",
                lock.frost.version, lock.frost.generation
            )
        }
    }
}

fn format_gen_link_report(root: &Path, entry_count: usize, changed_paths: &[PathBuf]) -> String {
    if changed_paths.is_empty() {
        return format!("No changes in {}", root.display());
    }

    let mut report = format!("Changes in {}:", root.display());
    for path in changed_paths {
        report.push_str("\n- ");
        report.push_str(&path.display().to_string());
    }
    report.push_str("\nTotal changes: ");
    report.push_str(&changed_paths.len().to_string());
    report.push('/');
    report.push_str(&entry_count.to_string());
    report
}

fn print_query_results(
    results: &QueryResults, format: QueryOutputFormat,
) -> Result<(), CommandError> {
    match format {
        | QueryOutputFormat::Json => {
            println!("{}", results.to_json()?);
        }
        | QueryOutputFormat::Human => {
            print!("{}", format_query_table(&results.columns, &results.rows));
        }
    }
    Ok(())
}

fn entry_path_records(
    config_path: &Path, lake_path: Option<&Path>, args: &EntryPathArgs,
) -> Result<Vec<PathRecord>, CommandError> {
    CoreContext::from_cli_paths(config_path, lake_path)
        .entry_paths(EntryPathRequest::from_args(args)?)
}

/// Selected path classes for an entry path lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathSelection {
    entry: bool,
    artifact: bool,
    frost: bool,
}

impl PathSelection {
    /// Select entry, artifact, and Frost paths.
    pub fn all() -> Self {
        Self { entry: true, artifact: true, frost: true }
    }

    /// Build an explicit path-class selection.
    pub fn new(entry: bool, artifact: bool, frost: bool) -> Self {
        Self { entry, artifact, frost }
    }

    fn from_args(args: &EntryPathArgs) -> Self {
        let all = !args.show_entry && !args.show_artifact && !args.show_frost;
        Self {
            entry: all || args.show_entry,
            artifact: all || args.show_artifact,
            frost: all || args.show_frost,
        }
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
    fn new(kind: &'static str, path: PathBuf) -> Self {
        Self { kind, path: path.display().to_string() }
    }
}

fn output_path(path: PathBuf, absolute: bool) -> Result<PathBuf, CommandError> {
    if !absolute || path.is_absolute() {
        return Ok(path);
    }
    Ok(env::current_dir().map_err(CommandError::CurrentDirectory)?.join(path))
}

fn print_path_records(
    records: &[PathRecord], format: PathOutputFormat,
) -> Result<(), CommandError> {
    match format {
        | PathOutputFormat::Json => print_json(records)?,
        | PathOutputFormat::Human => print!("{}", format_path_table(records)),
        | PathOutputFormat::Paths => {
            for record in records {
                println!("{}", record.path);
            }
        }
    }
    Ok(())
}

fn format_path_table(records: &[PathRecord]) -> String {
    let headers = ["kind", "path"];
    let rows = records.iter().map(|record| [record.kind, record.path.as_str()]);
    format_human_table(headers, rows)
}

fn query_result_rows(
    report: &EntryDirectoryReport, entries: &[&Entry], columns: &QueryColumns,
) -> Result<Vec<Vec<String>>, CommandError> {
    entries
        .iter()
        .map(|entry| {
            columns
                .columns
                .iter()
                .map(|column| format_query_column(report, entry, *column))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect()
}

fn format_query_column(
    report: &EntryDirectoryReport, entry: &Entry, column: QueryColumn,
) -> Result<String, CommandError> {
    match column {
        | QueryColumn::Id => Ok(entry.id.to_string()),
        | QueryColumn::Name => Ok(entry.metadata.name.clone()),
        | QueryColumn::Path => {
            let path = report
                .entry_path(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
            Ok(path.display().to_string())
        }
        | QueryColumn::Desc => Ok(entry.metadata.desc.clone()),
    }
}

fn format_query_json(columns: &QueryColumns, rows: &[Vec<String>]) -> Result<String, CommandError> {
    format_json(&query_result_records(columns, rows))
}

fn query_result_records(
    columns: &QueryColumns, rows: &[Vec<String>],
) -> Vec<IndexMap<String, String>> {
    rows.iter()
        .map(|row| {
            columns
                .columns
                .iter()
                .zip(row)
                .map(|(column, value)| (column.label().to_owned(), value.clone()))
                .collect()
        })
        .collect()
}

fn format_query_table(columns: &QueryColumns, rows: &[Vec<String>]) -> String {
    let headers = columns.columns.iter().map(|column| column.label()).collect::<Vec<_>>();
    format_human_table(headers, rows.iter().map(|row| row.iter().map(String::as_str)))
}

fn format_human_table<'a>(
    headers: impl IntoIterator<Item = &'a str>,
    rows: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
) -> String {
    let headers = headers.into_iter().map(str::to_owned).collect::<Vec<_>>();
    let rows = rows
        .into_iter()
        .map(|row| row.into_iter().map(str::to_owned).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    format_human_table_with_width(headers, rows, None)
}

fn format_human_table_with_width(
    headers: Vec<String>, rows: Vec<Vec<String>>, width: Option<u16>,
) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    if let Some(width) = width {
        table.set_width(width);
    }
    let (headers, rows) = elide_human_table_columns(headers, rows, table.width());
    table.set_header(headers);
    table.add_rows(rows);
    let mut output = table.to_string();
    output.push('\n');
    output
}

fn elide_human_table_columns(
    headers: Vec<String>, rows: Vec<Vec<String>>, width: Option<u16>,
) -> (Vec<String>, Vec<Vec<String>>) {
    let Some(width) = width.map(usize::from) else {
        return (headers, rows);
    };
    if headers.len() <= 2 || min_table_width(&headers) <= width {
        return (headers, rows);
    }

    for visible in (1..headers.len()).rev() {
        let mut candidate_headers = headers.iter().take(visible).cloned().collect::<Vec<_>>();
        candidate_headers.push("...".to_owned());
        if min_table_width(&candidate_headers) <= width {
            let candidate_rows = rows
                .into_iter()
                .map(|row| {
                    let mut cells = row.into_iter().take(visible).collect::<Vec<_>>();
                    cells.push("...".to_owned());
                    cells
                })
                .collect();
            return (candidate_headers, candidate_rows);
        }
    }

    (
        headers.into_iter().take(1).collect(),
        rows.into_iter().map(|row| row.into_iter().take(1).collect()).collect(),
    )
}

fn min_table_width(headers: &[String]) -> usize {
    headers.iter().map(|header| UnicodeWidthStr::width(header.as_str()).max(1)).sum::<usize>()
        + headers.len() * 3
        + 1
}

fn print_entry_directory_report(report: &EntryDirectoryReport) {
    if report.is_clean() {
        println!("ok: {}", report.root().display());
        return;
    }

    for diagnostic in report.file_diagnostics() {
        println!(
            "{}: {}: {}",
            diagnostic.severity.label(),
            diagnostic.path.display(),
            diagnostic.message
        );
    }

    for diagnostic in report.structural_report().diagnostics() {
        if let Some(path) = report.entry_path(&diagnostic.entry) {
            println!(
                "{}: {}: {}",
                diagnostic.severity.label(),
                path.display(),
                diagnostic.message()
            );
        } else {
            println!("{}: {}", diagnostic.severity.label(), diagnostic.message());
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenTideTutorial {
    frost_commit_tide: bool,
    frost_bootstrap_tide: bool,
    bootstrap: bool,
}

impl OpenTideTutorial {
    fn new(settings: Option<TutorialSettings>, bootstrap: bool) -> Self {
        let Some(settings) = settings else {
            return Self { frost_commit_tide: false, frost_bootstrap_tide: false, bootstrap };
        };
        Self {
            frost_commit_tide: settings.frost_commit_tide,
            frost_bootstrap_tide: settings.frost_bootstrap_tide,
            bootstrap,
        }
    }
}

impl fmt::Display for OpenTideTutorial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.frost_commit_tide {
            return Ok(());
        }

        writeln!(formatter)?;
        writeln!(formatter)?;
        writeln!(formatter, "Tutorial:")?;
        writeln!(
            formatter,
            "A tide is the review worklist for differences between the waterline and frostline.",
        )?;
        if self.bootstrap && self.frost_bootstrap_tide {
            writeln!(
                formatter,
                "This Frost path is still at empty version 0, so the first commit compares",
            )?;
            writeln!(formatter, "the full lake to an empty frostline.")?;
        }
        writeln!(formatter, "Inspect the work with `sirno tide status`.")?;
        writeln!(formatter, "Resolve reviewed work with `sirno resolve ...`,",)?;
        writeln!(
            formatter,
            "or choose the current lake as the baseline with `sirno commit --unsafe-resolve-all`.",
        )?;
        write!(formatter, "Remove `[tutorial]` from Sirno.toml, or set tutorial knobs to false,",)?;
        write!(formatter, " to silence tutorial text.")
    }
}

/// Error raised while running the CLI.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Sirno Frost has already been configured at another path.
    #[error("frost is already configured at {0}")]
    FrostAlreadyConfigured(PathBuf),
    /// Sirno Frost is required for a frost command but is not configured.
    #[error("frost is not configured; run `sirno frost init` first")]
    FrostNotConfigured,
    /// Immutable Frost checkouts cannot be committed.
    #[error("frost version {0} is checked out immutably; use checkout --unsafe-mutable first")]
    ImmutableFrostCheckout(u64),
    /// Frost commit requires all tide workitems to be resolved.
    #[error("tide has {count} open workitems; run `sirno tide status`{tutorial}")]
    OpenTide {
        /// Number of open tide workitems.
        count: usize,
        /// Optional tutorial text controlled by Sirno.toml.
        tutorial: OpenTideTutorial,
    },
    /// Empty Frost cannot be checked out as a version.
    #[error("frost version {0} is not a check-outable snapshot")]
    InvalidFrostVersion(u64),
    /// Frost checkout needs one target selector.
    #[error("frost checkout requires `latest` or `version`")]
    MissingFrostCheckoutTarget,
    /// An artifact source path did not have a file name for the default artifact path.
    #[error("artifact source has no file name: {0}")]
    ArtifactSourceHasNoFileName(PathBuf),
    /// A configured lake move cannot replace an existing destination.
    #[error("move destination already exists: {0}")]
    MoveDestinationExists(PathBuf),
    /// A configured lake move could not inspect its destination.
    #[error("failed to inspect move destination {path}")]
    ReadMoveDestination {
        /// Destination path that could not be inspected.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A configured lake path could not be moved.
    #[error("failed to move {source_path} to {destination_path}")]
    MovePath {
        /// Source path configured before the move.
        source_path: PathBuf,
        /// Destination path configured by the move.
        destination_path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A config write failed after a configured path was moved, and the rollback also failed.
    #[error(
        "failed to write config after moving {source_path} to {destination_path}; rollback failed: {rollback}"
    )]
    MoveConfigWriteRollback {
        /// Source path configured before the move.
        source_path: PathBuf,
        /// Destination path already moved into place.
        destination_path: PathBuf,
        /// Config write error.
        #[source]
        source: Box<ConfigError>,
        /// Rollback rename error.
        rollback: std::io::Error,
    },
    /// Witness lookup requires configured repo members.
    #[error("repo members are not configured; add [repo].members to Sirno.toml")]
    RepoMembersNotConfigured,
    /// Witness lookup requires an existing entry id.
    #[error("entry `{0}` does not exist")]
    MissingWitnessEntry(EntryId),
    /// Lake path override does not apply to checking a Frost path directly.
    #[error("`--lake-path` cannot be used with `check --frost-path`")]
    LakePathWithFrostPath,
    /// Frost path override applies only to direct Frost checks.
    #[error("`--frost-path` only applies to `sirno check`")]
    FrostPathRequiresCheck,
    /// The MCP server selects its project only through the config path.
    #[error("`--lake-path` cannot be used with `sirno util mcp`; configure the lake in Sirno.toml")]
    McpRejectsLakePath,
    /// The MCP server selects its project only through the config path.
    #[error("`--frost-path` cannot be used with `sirno util mcp`; use `--config` only")]
    McpRejectsFrostPath,
    /// The async MCP runtime could not be created.
    #[error("failed to create MCP runtime")]
    CreateMcpRuntime(#[source] std::io::Error),
    /// The MCP server failed.
    #[error("MCP server failed: {0}")]
    McpServer(String),
    /// Dry-run mode applies only to render writing.
    #[error("`--dry` only applies to `sirno render` without a subcommand")]
    DryWithRenderSubcommand,
    /// A command named a structural field not configured for this project.
    #[error("structural field `{0}` is not configured; add [structural.{0}] to Sirno.toml")]
    UnconfiguredStructuralField(String),
    /// Generated-footer masking cannot compose with another ripgrep preprocessor.
    #[error(
        "generated-footer filtering cannot be combined with `rg --pre`; use `--with-generated-footer`"
    )]
    RgPreprocessorConflict,
    /// Ripgrep generated-footer preprocessor received an unexpected argument shape.
    #[error("rg generated-footer preprocessor expects one path argument")]
    RgPreprocessorArgumentCount,
    /// The current executable path could not be resolved.
    #[error("failed to locate current executable for rg preprocessor")]
    LocateCurrentExe(#[source] std::io::Error),
    /// The current working directory could not be resolved.
    #[error("failed to locate current working directory")]
    CurrentDirectory(#[source] std::io::Error),
    /// A temporary ripgrep preprocessor invoker could not be created.
    #[error("failed to create rg preprocessor invoker at {path}")]
    CreateRgPreprocessorInvoker {
        /// Invoker path that could not be created.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The generated-footer preprocessor could not read one file.
    #[error("failed to read rg preprocessor input {path}")]
    ReadRgPreprocessorInput {
        /// Path passed by ripgrep.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The generated-footer preprocessor could not write masked output.
    #[error("failed to write rg preprocessor output")]
    WriteRgPreprocessorOutput(#[source] std::io::Error),
    /// Config-backed command failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Lock-backed command failed.
    #[error(transparent)]
    Lock(#[from] LockError),
    /// Sirno-Frost-backed command failed.
    #[error(transparent)]
    Frost(#[from] FrostError),
    /// Witness lookup failed.
    #[error(transparent)]
    Witness(#[from] WitnessError),
    /// Public Markdown entry directory command failed.
    #[error(transparent)]
    EntryDirectory(#[from] EntryDirectoryError),
    /// Entry id parsing failed.
    #[error(transparent)]
    EntryId(#[from] EntryIdError),
    /// Entry artifact path parsing failed.
    #[error(transparent)]
    ArtifactPath(#[from] EntryArtifactPathError),
    /// Entry metadata construction failed.
    #[error(transparent)]
    EntryParse(#[from] EntryParseError),
    /// Generated-link footer handling failed.
    #[error(transparent)]
    GeneratedLink(#[from] GeneratedLinkError),
    /// Tide operation failed.
    #[error(transparent)]
    Tide(#[from] TideError),
    /// Ripgrep could not be started.
    #[error("failed to run rg")]
    RunRg(#[source] std::io::Error),
    /// JSON-oriented ripgrep execution needs UTF-8 arguments.
    #[error("rg argument is not valid UTF-8: {0:?}")]
    RgArgumentNotUtf8(OsString),
    /// Query JSON rendering failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};

    use clap::{CommandFactory, Parser};

    use super::OpenTideTutorial;

    use crate::{
        CONFIG_FILE_NAME, Entry, EntryId, EntryMetadata, EntryQuery, Eterator, FrostError,
        FrostLockStatus, FrostSettings, LOCK_FILE_NAME, RepoMember, RepoSettings, SirnoConfig,
        SirnoFrost, SirnoLock, StructuralEdgeSettings, StructuralFieldSettings,
        StructuralRippleSettings, StructuralSettings, TutorialSettings, WitnessRecord, WitnessSpan,
    };

    use super::{
        ArtifactCommand, CheckModeArg, CheckoutArgs, Cli, Command, CommandError, CoreContext,
        EntryCommand, EntryNewRequest, EntryPathArgs, EntryRenameArgs, FrostCommand, FrostMoveArgs,
        LakeCommand, LakeInitRequest, LakeMoveArgs, MoveCommand, PathOutputFormat, QueryColumn,
        QueryColumns, QueryOutputFormat, ResolveArgs, StructuralFieldState, StructuralFilter,
        StructuralPredicate, StructuralStateFilter, TideCommand, TideItemSelector,
        TideOutputFormat, TideReviewCommand, TopLevelEntryCommand, TopLevelFrostCommand,
        TopLevelLakeCommand, UnresolveArgs, UtilCommand, entry_path_records,
        entry_query_from_filters, format_gen_link_report, format_human_table_with_width,
        format_json, format_path_table, format_query_json, format_query_table,
        format_witness_record, format_witness_records, rg_args_include_preprocessor,
    };

    fn assert_before(source: &str, before: &str, after: &str) {
        assert!(source.find(before).unwrap() < source.find(after).unwrap());
    }

    fn run_configured(config_path: &Path, args: &[&str]) {
        let mut command = vec!["sirno", "--config", config_path.to_str().unwrap()];
        command.extend_from_slice(args);
        Cli::parse_from(command).run().unwrap();
    }

    fn committed_alpha_frost_project() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        SirnoConfig::new("docs").with_frost("sirno-frost").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        )
        .unwrap();

        run_configured(&config_path, &["frost", "commit"]);

        (temp, config_path, docs)
    }

    fn assert_mutable_current_frost_lake(root: &Path, docs: &Path) {
        let lock = SirnoLock::from_file(root.join(LOCK_FILE_NAME)).unwrap();
        let source = fs::read_to_string(docs.join("alpha.md")).unwrap();
        assert_eq!(lock.frost.status, FrostLockStatus::Current);
        assert_eq!(lock.frost.version, 1);
        assert!(!lock.frost.mutable);
        assert!(!source.contains("read-only Sirno Frost checkout"));
        assert!(!fs::metadata(docs).unwrap().permissions().readonly());
        assert!(!fs::metadata(docs.join("alpha.md")).unwrap().permissions().readonly());
    }

    #[test]
    fn top_level_init_initializes_lake_and_frost() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("alpha-project");
        fs::create_dir(&repo).unwrap();
        let config_path = repo.join(CONFIG_FILE_NAME);

        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "init"])
            .run()
            .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        let lock = SirnoLock::from_file(repo.join(LOCK_FILE_NAME)).unwrap();
        assert_eq!(config.lake.path, PathBuf::from("alpha-project-lake"));
        assert_eq!(
            config.frost,
            Some(FrostSettings { path: PathBuf::from("alpha-project-frost") })
        );
        assert!(repo.join("alpha-project-lake").join("concept.md").exists());
        assert!(repo.join("alpha-project-frost").join("Eter.lock.toml").exists());
        assert_eq!(lock.frost.status, FrostLockStatus::Current);
        assert_eq!(lock.frost.version, Eterator::EMPTY.version());
    }

    #[test]
    fn top_level_init_accepts_explicit_paths() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "init",
            "--mono",
            "DESIGN.md",
            "--lake",
            "custom-lake",
            "--frost",
            "custom-frost",
        ])
        .run()
        .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.mono.unwrap().path, PathBuf::from("DESIGN.md"));
        assert_eq!(config.lake.path, PathBuf::from("custom-lake"));
        assert_eq!(config.frost.unwrap().path, PathBuf::from("custom-frost"));
        assert!(temp.path().join("custom-lake").join("concept.md").exists());
        assert!(temp.path().join("custom-frost").join("Eter.lock.toml").exists());
    }

    #[test]
    fn lake_init_uses_global_lake_path() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("sirno-docs");

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "--lake-path",
            "sirno-docs",
            "lake",
            "init",
        ])
        .run()
        .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.lake.path, PathBuf::from("sirno-docs"));
        assert!(docs.join("concept.md").exists());
    }

    #[test]
    fn lake_init_accepts_lake_path() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "lake",
            "init",
            "custom-lake",
        ])
        .run()
        .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.lake.path, PathBuf::from("custom-lake"));
        assert!(temp.path().join("custom-lake").join("concept.md").exists());
    }

    #[test]
    fn core_context_lake_init_and_entry_new_return_json_dtos() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let context = CoreContext::new(&config_path);

        let init =
            context.lake_init(LakeInitRequest { lake: Some(PathBuf::from("docs")) }).unwrap();
        let entry = context
            .entry_new(EntryNewRequest {
                id: EntryId::new("alpha").unwrap(),
                name: None,
                desc: "Alpha entry.".to_owned(),
                structural: Vec::new(),
                body: Some("Body.".to_owned()),
            })
            .unwrap();
        let json = format_json(&entry).unwrap();

        assert!(init.ok);
        assert!(init.entry_count > 0);
        assert!(entry.ok);
        assert!(entry.path.ends_with("docs/alpha.md"));
        assert!(json.contains("\"ok\": true"));
    }

    #[test]
    fn lake_init_rejects_mono_option() {
        let error =
            Cli::try_parse_from(["sirno", "lake", "init", "--mono", "DESIGN.md"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn short_config_matches_global_config() {
        let cli = Cli::parse_from(["sirno", "-C", "Sirno.alt.toml", "status"]);

        assert_eq!(cli.config, Some(PathBuf::from("Sirno.alt.toml")));
        assert!(matches!(cli.command, Command::TopLevelLake(TopLevelLakeCommand::Status)));
    }

    #[test]
    fn short_lake_path_matches_global_lake_path() {
        let cli = Cli::parse_from(["sirno", "-L", "scratch-docs", "status"]);

        assert_eq!(cli.lake_path.as_deref(), Some(Path::new("scratch-docs")));
        assert!(matches!(cli.command, Command::TopLevelLake(TopLevelLakeCommand::Status)));
    }

    #[test]
    fn short_frost_path_matches_global_frost_path() {
        let cli = Cli::parse_from(["sirno", "-F", "sirno-frost", "check"]);

        assert_eq!(cli.frost_path.as_deref(), Some(Path::new("sirno-frost")));
        assert!(matches!(cli.command, Command::TopLevelLake(TopLevelLakeCommand::Check { .. })));
    }

    #[test]
    fn frost_init_accepts_frost_path() {
        let cli = Cli::parse_from(["sirno", "frost", "init", "sirno-frost"]);

        assert!(matches!(
            cli.command,
            Command::Frost { command: FrostCommand::Init { frost: Some(_) } }
        ));
    }

    #[test]
    fn frost_init_rejects_frost_option() {
        let error =
            Cli::try_parse_from(["sirno", "frost", "init", "--frost", "sirno-frost"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn top_level_help_orders_grouped_commands_before_shortcuts() {
        let help = Cli::command().render_help().to_string();

        assert_before(&help, "  init", "  new");
        assert_before(&help, "  tide", "  new");
        assert_before(&help, "  entry", "  lake");
        assert_before(&help, "  lake", "  frost");
        assert_before(&help, "  frost", "  tide");
        assert_before(&help, "  new", "  check");
    }

    #[test]
    fn frost_commit_accepts_top_level_form() {
        let cli = Cli::parse_from(["sirno", "commit", "--unsafe-resolve-all"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelFrost(TopLevelFrostCommand::Commit { unsafe_resolve_all: true })
        ));
    }

    #[test]
    fn frost_checkout_accepts_top_level_form_and_defrost_shortcut() {
        let checkout = Cli::parse_from(["sirno", "checkout", "--latest"]);
        let defrost = Cli::parse_from(["sirno", "defrost"]);

        assert!(matches!(
            checkout.command,
            Command::TopLevelFrost(TopLevelFrostCommand::Checkout(CheckoutArgs {
                version: None,
                latest: true,
                unsafe_mutable: false,
            }))
        ));
        assert!(matches!(defrost.command, Command::TopLevelFrost(TopLevelFrostCommand::Defrost)));
    }

    #[test]
    fn frost_init_rejects_global_frost_path() {
        let error = Cli::parse_from(["sirno", "frost", "init", "--frost-path", "sirno-frost"])
            .run()
            .unwrap_err();

        assert!(matches!(error, CommandError::FrostPathRequiresCheck));
    }

    #[test]
    fn util_mcp_accepts_config_launch_form() {
        let cli = Cli::parse_from(["sirno", "--config", "Sirno.toml", "util", "mcp"]);

        assert!(matches!(cli.command, Command::Util { command: UtilCommand::Mcp }));
    }

    #[test]
    fn top_level_mcp_is_not_a_command() {
        let error = Cli::try_parse_from(["sirno", "mcp"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn util_mcp_rejects_global_lake_path() {
        let error =
            Cli::parse_from(["sirno", "--lake-path", "docs", "util", "mcp"]).run().unwrap_err();

        assert!(matches!(error, CommandError::McpRejectsLakePath));
    }

    #[test]
    fn util_mcp_rejects_global_frost_path() {
        let error = Cli::parse_from(["sirno", "--frost-path", "sirno-frost", "util", "mcp"])
            .run()
            .unwrap_err();

        assert!(matches!(error, CommandError::McpRejectsFrostPath));
    }

    #[test]
    fn frost_init_creates_empty_version_zero_store() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("frost-project");
        fs::create_dir(&repo).unwrap();
        let config_path = repo.join(CONFIG_FILE_NAME);
        let docs = repo.join("docs");
        let frost_path = repo.join("frost-project-frost");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        )
        .unwrap();

        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "init"])
            .run()
            .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        let lock = SirnoLock::from_file(repo.join(LOCK_FILE_NAME)).unwrap();
        let frost = SirnoFrost::open(&frost_path).unwrap();
        let mut frost_paths = fs::read_dir(&frost_path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        frost_paths.sort();

        assert_eq!(
            config.frost,
            Some(FrostSettings { path: PathBuf::from("frost-project-frost") })
        );
        assert_eq!(lock.frost.status, FrostLockStatus::Current);
        assert_eq!(lock.frost.version, Eterator::EMPTY.version());
        assert_eq!(frost.current_version().unwrap(), Eterator::EMPTY);
        assert!(frost.read_all_entries().unwrap().is_empty());
        assert_eq!(frost_paths, [OsString::from("Eter.lock.toml")]);
    }

    #[test]
    fn frost_checkout_latest_writes_mutable_current_lake() {
        let (temp, config_path, docs) = committed_alpha_frost_project();

        run_configured(&config_path, &["frost", "checkout", "1"]);
        assert!(fs::metadata(docs.join("alpha.md")).unwrap().permissions().readonly());

        run_configured(&config_path, &["frost", "checkout", "--latest"]);

        assert_mutable_current_frost_lake(temp.path(), &docs);
    }

    #[test]
    fn frost_defrost_writes_mutable_current_lake() {
        let (temp, config_path, docs) = committed_alpha_frost_project();

        run_configured(&config_path, &["frost", "checkout", "1"]);
        assert!(fs::metadata(docs.join("alpha.md")).unwrap().permissions().readonly());

        run_configured(&config_path, &["frost", "defrost"]);

        assert_mutable_current_frost_lake(temp.path(), &docs);
    }

    #[test]
    fn frost_commit_requires_clear_tide() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        let config = SirnoConfig {
            structural: StructuralSettings::from_fields([(
                "belongs",
                StructuralFieldSettings::new(
                    StructuralEdgeSettings::new(false, StructuralRippleSettings::new(true, false)),
                    StructuralEdgeSettings::default(),
                    StructuralEdgeSettings::default(),
                ),
            )]),
            ..SirnoConfig::new("docs").with_frost("sirno-frost")
        };
        config.write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
belongs:
  - beta
---

Body.
",
        )
        .unwrap();
        fs::write(
            docs.join("beta.md"),
            "\
---
name: Beta
desc: Beta entry.
---

Body.
",
        )
        .unwrap();
        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "frost",
            "commit",
            "--unsafe-resolve-all",
        ])
        .run()
        .unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
belongs:
  - beta
---

Changed body.
",
        )
        .unwrap();

        let error = Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "frost",
            "commit",
        ])
        .run()
        .unwrap_err();
        assert!(matches!(
            &error,
            CommandError::OpenTide { count, tutorial }
                if *count == 1 && !tutorial.frost_commit_tide
        ));
        assert_eq!(error.to_string(), "tide has 1 open workitems; run `sirno tide status`");

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "tide",
            "resolve",
            "beta",
        ])
        .run()
        .unwrap();
        assert_eq!(
            SirnoLock::from_file(temp.path().join(LOCK_FILE_NAME)).unwrap().tide.resolved.len(),
            1
        );

        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "commit"])
            .run()
            .unwrap();
        let lock = SirnoLock::from_file(temp.path().join(LOCK_FILE_NAME)).unwrap();
        assert!(lock.tide.resolved.is_empty());
        assert_eq!(lock.frost.version, 2);
    }

    #[test]
    fn frost_commit_open_tide_tutorial_explains_bootstrap_when_enabled() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        let config = SirnoConfig {
            structural: StructuralSettings::from_fields([(
                "belongs",
                StructuralFieldSettings::new(
                    StructuralEdgeSettings::new(false, StructuralRippleSettings::new(true, false)),
                    StructuralEdgeSettings::default(),
                    StructuralEdgeSettings::default(),
                ),
            )]),
            tutorial: Some(TutorialSettings::all()),
            ..SirnoConfig::new("docs").with_frost("sirno-frost")
        };
        config.write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
belongs:
  - beta
---

Body.
",
        )
        .unwrap();
        fs::write(
            docs.join("beta.md"),
            "\
---
name: Beta
desc: Beta entry.
---

Body.
",
        )
        .unwrap();

        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "init"])
            .run()
            .unwrap();
        let error = Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "frost",
            "commit",
        ])
        .run()
        .unwrap_err();
        let message = error.to_string();

        assert!(matches!(&error, CommandError::OpenTide { count, .. } if *count == 1));
        assert!(message.contains("Tutorial:"));
        assert!(message.contains("empty version 0"));
        assert!(message.contains("sirno commit --unsafe-resolve-all"));
        assert!(message.contains("Remove `[tutorial]` from Sirno.toml"));
    }

    #[test]
    fn open_tide_tutorial_knobs_control_message_parts() {
        let no_tutorial = OpenTideTutorial::new(
            Some(TutorialSettings { frost_commit_tide: false, frost_bootstrap_tide: true }),
            true,
        )
        .to_string();
        let generic_tutorial = OpenTideTutorial::new(
            Some(TutorialSettings { frost_commit_tide: true, frost_bootstrap_tide: false }),
            true,
        )
        .to_string();

        assert!(no_tutorial.is_empty());
        assert!(generic_tutorial.contains("Tutorial:"));
        assert!(!generic_tutorial.contains("empty version 0"));
    }

    #[test]
    fn move_accepts_entry_lake_and_frost_subcommands() {
        let entry = Cli::parse_from(["sirno", "move", "entry", "old-entry", "new-entry"]);
        let lake = Cli::parse_from(["sirno", "move", "lake", "sirno-docs"]);
        let frost = Cli::parse_from(["sirno", "move", "frost", "sirno-frost-2"]);

        assert!(matches!(
            entry.command,
            Command::Move {
                command: MoveCommand::Entry(EntryRenameArgs { old_id, new_id })
            }
                if old_id == "old-entry" && new_id == "new-entry"
        ));
        assert!(matches!(
            lake.command,
            Command::Move { command: MoveCommand::Lake(LakeMoveArgs { lake }) }
                if lake == Path::new("sirno-docs")
        ));
        assert!(matches!(
            frost.command,
            Command::Move { command: MoveCommand::Frost(FrostMoveArgs { frost }) }
                if frost == Path::new("sirno-frost-2")
        ));
    }

    #[test]
    fn mv_alias_accepts_move_subcommands() {
        let cli = Cli::parse_from(["sirno", "mv", "entry", "old-entry", "new-entry"]);

        assert!(matches!(
            cli.command,
            Command::Move {
                command: MoveCommand::Entry(EntryRenameArgs { old_id, new_id })
            }
                if old_id == "old-entry" && new_id == "new-entry"
        ));
    }

    #[test]
    fn lake_move_accepts_mv_alias() {
        let cli = Cli::parse_from(["sirno", "lake", "mv", "sirno-docs"]);

        assert!(matches!(
            cli.command,
            Command::Lake { command: LakeCommand::Move(LakeMoveArgs { lake }) }
                if lake == Path::new("sirno-docs")
        ));
    }

    #[test]
    fn frost_move_accepts_frost_path() {
        let cli = Cli::parse_from(["sirno", "frost", "move", "sirno-frost-2"]);

        assert!(matches!(
            cli.command,
            Command::Frost { command: FrostCommand::Move(FrostMoveArgs { frost }) }
                if frost == Path::new("sirno-frost-2")
        ));
    }

    #[test]
    fn frost_mv_alias_accepts_frost_path() {
        let cli = Cli::parse_from(["sirno", "frost", "mv", "sirno-frost-2"]);

        assert!(matches!(
            cli.command,
            Command::Frost { command: FrostCommand::Move(FrostMoveArgs { frost }) }
                if frost == Path::new("sirno-frost-2")
        ));
    }

    #[test]
    fn frost_checkout_accepts_unsafe_mutable_flag() {
        let cli = Cli::parse_from(["sirno", "frost", "checkout", "3", "--unsafe-mutable"]);

        assert!(matches!(
            cli.command,
            Command::Frost {
                command: FrostCommand::Snapshot(TopLevelFrostCommand::Checkout(CheckoutArgs {
                    version: Some(3),
                    latest: false,
                    unsafe_mutable: true
                }))
            }
        ));
    }

    #[test]
    fn frost_checkout_accepts_latest_flag() {
        let cli = Cli::parse_from(["sirno", "frost", "checkout", "--latest"]);

        assert!(matches!(
            cli.command,
            Command::Frost {
                command: FrostCommand::Snapshot(TopLevelFrostCommand::Checkout(CheckoutArgs {
                    version: None,
                    latest: true,
                    unsafe_mutable: false
                }))
            }
        ));
    }

    #[test]
    fn frost_defrost_accepts_grouped_latest_shortcut() {
        let cli = Cli::parse_from(["sirno", "frost", "defrost"]);

        assert!(matches!(
            cli.command,
            Command::Frost { command: FrostCommand::Snapshot(TopLevelFrostCommand::Defrost) }
        ));
    }

    #[test]
    fn frost_checkout_rejects_latest_with_version() {
        let error =
            Cli::try_parse_from(["sirno", "frost", "checkout", "3", "--latest"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn frost_defrost_rejects_checkout_arguments() {
        let cases: &[&[&str]] = &[
            &["sirno", "defrost", "1"],
            &["sirno", "defrost", "--latest"],
            &["sirno", "defrost", "--unsafe-mutable"],
            &["sirno", "frost", "defrost", "1"],
            &["sirno", "frost", "defrost", "--latest"],
            &["sirno", "frost", "defrost", "--unsafe-mutable"],
        ];

        for args in cases {
            let error = Cli::try_parse_from(args.iter().copied()).unwrap_err();

            assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
        }
    }

    #[test]
    fn tide_resolve_accepts_neighbor_and_tuple_selectors() {
        let neighbor = Cli::parse_from(["sirno", "tide", "resolve", "beta"]);
        let tuple = Cli::parse_from(["sirno", "tide", "resolve", "alpha,belongs,to,beta"]);

        assert!(matches!(
            neighbor.command,
            Command::Tide {
                command: TideCommand::Review(TideReviewCommand::Resolve(ResolveArgs {
                    items,
                    infer: false,
                    json: None
                }))
            } if items == vec![TideItemSelector::Neighbor(EntryId::new("beta").unwrap())]
        ));
        assert!(matches!(
            tuple.command,
            Command::Tide {
                command: TideCommand::Review(TideReviewCommand::Resolve(ResolveArgs {
                    items,
                    infer: false,
                    json: None
                }))
            } if matches!(&items[..], [TideItemSelector::Workitem(workitem)]
                if workitem.to_string() == "alpha,belongs,to,beta")
        ));
    }

    #[test]
    fn tide_resolve_accepts_infer_and_json() {
        let infer = Cli::parse_from(["sirno", "tide", "resolve", "--infer"]);
        let json = Cli::parse_from([
            "sirno",
            "tide",
            "resolve",
            "--json",
            r#"{"ripple":"alpha","field":"belongs","direction":"to","neighbor":"beta"}"#,
        ]);

        assert!(matches!(
            infer.command,
            Command::Tide {
                command: TideCommand::Review(TideReviewCommand::Resolve(ResolveArgs {
                    infer: true,
                    ..
                }))
            }
        ));
        assert!(matches!(
            json.command,
            Command::Tide {
                command: TideCommand::Review(TideReviewCommand::Resolve(ResolveArgs {
                    json: Some(_),
                    infer: false,
                    ..
                }))
            }
        ));
    }

    #[test]
    fn tide_resolve_requires_selector_json_or_infer() {
        let error = Cli::try_parse_from(["sirno", "tide", "resolve"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn top_level_resolve_accepts_tide_resolve_args() {
        let neighbor = Cli::parse_from(["sirno", "resolve", "beta"]);
        let tuple = Cli::parse_from(["sirno", "resolve", "alpha,belongs,to,beta"]);
        let infer = Cli::parse_from(["sirno", "resolve", "--infer"]);
        let json = Cli::parse_from([
            "sirno",
            "resolve",
            "--json",
            r#"{"ripple":"alpha","field":"belongs","direction":"to","neighbor":"beta"}"#,
        ]);

        assert!(matches!(
            neighbor.command,
            Command::TopLevelTide(TideReviewCommand::Resolve(ResolveArgs {
                items,
                infer: false,
                json: None
            })) if items == vec![TideItemSelector::Neighbor(EntryId::new("beta").unwrap())]
        ));
        assert!(matches!(
            tuple.command,
            Command::TopLevelTide(TideReviewCommand::Resolve(ResolveArgs {
                items,
                infer: false,
                json: None
            })) if matches!(&items[..], [TideItemSelector::Workitem(workitem)]
                if workitem.to_string() == "alpha,belongs,to,beta")
        ));
        assert!(matches!(
            infer.command,
            Command::TopLevelTide(TideReviewCommand::Resolve(ResolveArgs { infer: true, .. }))
        ));
        assert!(matches!(
            json.command,
            Command::TopLevelTide(TideReviewCommand::Resolve(ResolveArgs {
                json: Some(_),
                infer: false,
                ..
            }))
        ));
    }

    #[test]
    fn top_level_resolve_requires_selector_json_or_infer() {
        let error = Cli::try_parse_from(["sirno", "resolve"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn unresolve_accepts_top_level_grouped_and_reopen_alias() {
        let top_level = Cli::parse_from(["sirno", "unresolve", "beta"]);
        let top_level_alias = Cli::parse_from(["sirno", "reopen", "beta"]);
        let grouped = Cli::parse_from(["sirno", "tide", "unresolve", "beta"]);
        let alias = Cli::parse_from(["sirno", "tide", "reopen", "beta"]);

        assert!(matches!(
            top_level.command,
            Command::TopLevelTide(TideReviewCommand::Unresolve(UnresolveArgs { items }))
                if items == vec![TideItemSelector::Neighbor(EntryId::new("beta").unwrap())]
        ));
        assert!(matches!(
            top_level_alias.command,
            Command::TopLevelTide(TideReviewCommand::Unresolve(UnresolveArgs { items }))
                if items == vec![TideItemSelector::Neighbor(EntryId::new("beta").unwrap())]
        ));
        assert!(matches!(
            grouped.command,
            Command::Tide {
                command: TideCommand::Review(TideReviewCommand::Unresolve(UnresolveArgs { items }))
            }
                if items == vec![TideItemSelector::Neighbor(EntryId::new("beta").unwrap())]
        ));
        assert!(matches!(
            alias.command,
            Command::Tide {
                command: TideCommand::Review(TideReviewCommand::Unresolve(UnresolveArgs { items }))
            }
                if items == vec![TideItemSelector::Neighbor(EntryId::new("beta").unwrap())]
        ));
    }

    #[test]
    fn frost_checkout_rejects_latest_with_unsafe_mutable() {
        let error =
            Cli::try_parse_from(["sirno", "frost", "checkout", "--latest", "--unsafe-mutable"])
                .unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn freeze_accepts_entry_id() {
        let cli = Cli::parse_from(["sirno", "freeze", "alpha"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Freeze { id, .. }) if id == "alpha"
        ));
    }

    #[test]
    fn new_accepts_short_metadata_flags() {
        let cli = Cli::parse_from([
            "sirno",
            "new",
            "alpha",
            "-n",
            "Alpha",
            "-d",
            "Alpha desc.",
            "-b",
            "Alpha body.",
        ]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::New {
                id,
                name: Some(name),
                desc,
                body: Some(body),
                ..
            })
                if id == "alpha"
                    && name == "Alpha"
                    && desc == "Alpha desc."
                    && body == "Alpha body."
        ));
    }

    #[test]
    fn new_accepts_structural_targets() {
        let cli = Cli::parse_from([
            "sirno",
            "new",
            "alpha",
            "-d",
            "Alpha desc.",
            "--structural",
            "topic=concept",
            "--structural",
            "topic=methodology",
        ]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::New { structural, .. })
                if structural == vec![
                    StructuralPredicate {
                        field: "topic".to_owned(),
                        target: EntryId::new("concept").unwrap(),
                    },
                    StructuralPredicate {
                        field: "topic".to_owned(),
                        target: EntryId::new("methodology").unwrap(),
                    },
            ]
        ));
    }

    #[test]
    fn rename_accepts_entry_ids_and_aliases() {
        let entry = Cli::parse_from(["sirno", "entry", "rename", "old-entry", "new-entry"]);
        let short = Cli::parse_from(["sirno", "entry", "mv", "old-entry", "new-entry"]);
        let mnemonic = Cli::parse_from(["sirno", "entry", "move", "old-entry", "new-entry"]);

        assert!(matches!(
            entry.command,
            Command::Entry {
                command: EntryCommand::Rename(EntryRenameArgs { old_id, new_id })
            }
                if old_id == "old-entry" && new_id == "new-entry"
        ));
        assert!(matches!(
            short.command,
            Command::Entry {
                command: EntryCommand::Rename(EntryRenameArgs { old_id, new_id })
            }
                if old_id == "old-entry" && new_id == "new-entry"
        ));
        assert!(matches!(
            mnemonic.command,
            Command::Entry {
                command: EntryCommand::Rename(EntryRenameArgs { old_id, new_id })
            }
                if old_id == "old-entry" && new_id == "new-entry"
        ));
    }

    #[test]
    fn path_accepts_filters_and_entry_form() {
        let top_level =
            Cli::parse_from(["sirno", "path", "alpha", "--artifact", "--frost", "-o", "paths"]);
        let entry = Cli::parse_from(["sirno", "entry", "path", "alpha", "--entry"]);

        assert!(matches!(
            top_level.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Path(EntryPathArgs {
                id,
                show_entry: false,
                show_artifact: true,
                show_frost: true,
                absolute: false,
                format: Some(PathOutputFormat::Paths),
            })) if id == "alpha"
        ));
        assert!(matches!(
            entry.command,
            Command::Entry { command: EntryCommand::TopLevel(TopLevelEntryCommand::Path(EntryPathArgs {
                id,
                show_entry: true,
                show_artifact: false,
                show_frost: false,
                absolute: false,
                format: None,
            })) } if id == "alpha"
        ));
    }

    #[test]
    fn rename_rejects_top_level_form() {
        let error = Cli::try_parse_from(["sirno", "rename", "old-entry", "new-entry"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn artifact_commands_accept_top_level_and_entry_form() {
        let list = Cli::parse_from(["sirno", "artifact", "list", "alpha"]);
        let add = Cli::parse_from([
            "sirno",
            "entry",
            "artifact",
            "add",
            "alpha",
            "logo.png",
            "images/logo.png",
        ]);
        let rename = Cli::parse_from([
            "sirno",
            "artifact",
            "mv",
            "alpha",
            "images/logo.png",
            "images/wordmark.png",
        ]);
        let remove = Cli::parse_from(["sirno", "entry", "artifact", "rm", "alpha", "logo.png"]);

        assert!(matches!(
            list.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Artifact {
                command: ArtifactCommand::List { id },
            }) if id == "alpha"
        ));
        assert!(matches!(
            add.command,
            Command::Entry {
                command: EntryCommand::TopLevel(TopLevelEntryCommand::Artifact {
                    command: ArtifactCommand::Add { id, source, artifact_path: Some(path) },
                }),
            } if id == "alpha" && source == Path::new("logo.png") && path == Path::new("images/logo.png")
        ));
        assert!(matches!(
            rename.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Artifact {
                command: ArtifactCommand::Rename { id, old_path, new_path },
            }) if id == "alpha"
                && old_path == Path::new("images/logo.png")
                && new_path == Path::new("images/wordmark.png")
        ));
        assert!(matches!(
            remove.command,
            Command::Entry {
                command: EntryCommand::TopLevel(TopLevelEntryCommand::Artifact {
                    command: ArtifactCommand::Remove { id, artifact_path },
                }),
            } if id == "alpha" && artifact_path == Path::new("logo.png")
        ));
    }

    #[test]
    fn artifact_entry_form_matches_top_level_form() {
        let list = Cli::parse_from(["sirno", "entry", "artifact", "list", "alpha"]);
        let rename = Cli::parse_from([
            "sirno",
            "entry",
            "artifact",
            "mv",
            "alpha",
            "images/logo.png",
            "images/wordmark.png",
        ]);

        assert!(matches!(
            list.command,
            Command::Entry {
                command: EntryCommand::TopLevel(TopLevelEntryCommand::Artifact {
                    command: ArtifactCommand::List { id },
                }),
            } if id == "alpha"
        ));
        assert!(matches!(
            rename.command,
            Command::Entry {
                command: EntryCommand::TopLevel(TopLevelEntryCommand::Artifact {
                    command: ArtifactCommand::Rename { id, old_path, new_path },
                }),
            } if id == "alpha"
                && old_path == Path::new("images/logo.png")
                && new_path == Path::new("images/wordmark.png")
        ));
    }

    #[test]
    fn entry_new_creates_entry() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "entry",
            "new",
            "alpha",
            "--desc",
            "Alpha entry.",
        ])
        .run()
        .unwrap();

        assert!(docs.join("alpha.md").exists());
    }

    #[test]
    fn artifact_commands_manage_entry_artifact_paths() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        let source = temp.path().join("logo.bin");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        )
        .unwrap();
        fs::write(&source, b"logo").unwrap();

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "entry",
            "artifact",
            "add",
            "alpha",
            source.to_str().unwrap(),
            "images/logo.bin",
        ])
        .run()
        .unwrap();
        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "entry",
            "artifact",
            "mv",
            "alpha",
            "images/logo.bin",
            "images/wordmark.bin",
        ])
        .run()
        .unwrap();
        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "entry",
            "artifact",
            "rm",
            "alpha",
            "images/wordmark.bin",
        ])
        .run()
        .unwrap();

        assert!(!docs.join(".artifacts").join("alpha").join("images").exists());
    }

    #[test]
    fn path_records_include_frost_and_exclude_witness_by_default() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        SirnoConfig::new("docs").with_frost("sirno-frost").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        )
        .unwrap();
        fs::create_dir_all(docs.join(".artifacts").join("alpha")).unwrap();
        fs::write(docs.join(".artifacts").join("alpha").join("note.bin"), b"note").unwrap();
        let args = EntryPathArgs {
            id: "alpha".to_owned(),
            show_entry: false,
            show_artifact: false,
            show_frost: false,
            absolute: false,
            format: None,
        };

        let records = entry_path_records(&config_path, None, &args).unwrap();
        let kinds = records.iter().map(|record| record.kind).collect::<Vec<_>>();
        let table = format_path_table(&records);

        assert_eq!(kinds, ["entry", "artifact-root", "artifact", "frost-entry"]);
        assert!(!table.contains("witness"));
        assert!(table.contains(".artifacts"));
        assert!(table.contains("sirno-frost"));
    }

    #[test]
    fn new_rejects_exact_short_alias() {
        let error = Cli::try_parse_from([
            "sirno",
            "new",
            "alpha",
            "-d",
            "Alpha desc.",
            "-x",
            "topic=concept",
        ])
        .unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn lake_path_is_global() {
        let cli = Cli::parse_from(["sirno", "freeze", "alpha", "--lake-path", "scratch-docs"]);

        assert_eq!(cli.lake_path.as_deref(), Some(Path::new("scratch-docs")));
        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Freeze { id }) if id == "alpha"
        ));
    }

    #[test]
    fn lake_path_conflicts_with_frost_path_check() {
        let error = Cli::parse_from([
            "sirno",
            "--lake-path",
            "scratch-docs",
            "check",
            "--frost-path",
            "sirno-frost",
        ])
        .run()
        .unwrap_err();

        assert!(matches!(error, CommandError::LakePathWithFrostPath));
    }

    #[test]
    fn check_rejects_old_frost_root_flag() {
        let error =
            Cli::try_parse_from(["sirno", "check", "--frost-root", "sirno-frost"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn query_accepts_structural_filter() {
        let cli = Cli::parse_from(["sirno", "query", "--has", "topic=concept,methodology"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Query { has, .. })
                if has == vec![StructuralFilter {
                    field: "topic".to_owned(),
                    targets: vec![
                        EntryId::new("concept").unwrap(),
                        EntryId::new("methodology").unwrap(),
                    ],
                }]
        ));
    }

    #[test]
    fn query_accepts_structural_state_filter() {
        let cli = Cli::parse_from(["sirno", "query", "--is", "topic=empty"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Query { is, .. })
                if is == vec![StructuralStateFilter {
                    field: "topic".to_owned(),
                    state: StructuralFieldState::Empty,
                }]
        ));
    }

    #[test]
    fn query_accepts_short_alias_and_options() {
        let cli = Cli::parse_from([
            "sirno",
            "q",
            "--has",
            "topic=concept",
            "--columns",
            "id,path",
            "-o",
            "human",
        ]);
        let Command::TopLevelEntry(TopLevelEntryCommand::Query {
            has,
            columns: Some(columns),
            format: Some(format),
            ..
        }) = cli.command
        else {
            panic!("expected query command with short options");
        };

        assert_eq!(
            has,
            vec![StructuralFilter {
                field: "topic".to_owned(),
                targets: vec![EntryId::new("concept").unwrap()],
            }]
        );
        assert_eq!(columns.columns, vec![QueryColumn::Id, QueryColumn::Path]);
        assert!(matches!(format, QueryOutputFormat::Human));
    }

    #[test]
    fn entry_query_accepts_short_alias_and_options() {
        let cli = Cli::parse_from([
            "sirno",
            "entry",
            "q",
            "--has",
            "topic=concept",
            "--columns",
            "id,path",
            "-o",
            "human",
        ]);
        let Command::Entry {
            command:
                EntryCommand::TopLevel(TopLevelEntryCommand::Query {
                    has,
                    columns: Some(columns),
                    format: Some(format),
                    ..
                }),
        } = cli.command
        else {
            panic!("expected entry query command with short options");
        };

        assert_eq!(
            has,
            vec![StructuralFilter {
                field: "topic".to_owned(),
                targets: vec![EntryId::new("concept").unwrap()],
            }]
        );
        assert_eq!(columns.columns, vec![QueryColumn::Id, QueryColumn::Path]);
        assert!(matches!(format, QueryOutputFormat::Human));
    }

    #[test]
    fn query_accepts_comma_separated_columns() {
        let cli = Cli::parse_from(["sirno", "query", "--columns", "id,name,path,desc"]);
        let Command::TopLevelEntry(TopLevelEntryCommand::Query { columns: Some(columns), .. }) =
            cli.command
        else {
            panic!("expected query command with columns");
        };

        assert_eq!(
            columns.columns,
            vec![QueryColumn::Id, QueryColumn::Name, QueryColumn::Path, QueryColumn::Desc,]
        );
    }

    #[test]
    fn query_accepts_json_format() {
        let cli = Cli::parse_from(["sirno", "query", "--format", "json"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Query {
                format: Some(QueryOutputFormat::Json),
                ..
            })
        ));
    }

    #[test]
    fn query_accepts_human_format() {
        let cli = Cli::parse_from(["sirno", "query", "--format", "human"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Query {
                format: Some(QueryOutputFormat::Human),
                ..
            })
        ));
    }

    #[test]
    fn table_output_formats_default_to_human() {
        assert!(matches!(PathOutputFormat::default(), PathOutputFormat::Human));
        assert!(matches!(QueryOutputFormat::default(), QueryOutputFormat::Human));
        assert!(matches!(TideOutputFormat::default(), TideOutputFormat::Human));
    }

    #[test]
    fn query_rejects_old_human_flag() {
        let error = Cli::try_parse_from(["sirno", "query", "--human"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn query_rejects_old_format_field_list() {
        let error = Cli::try_parse_from(["sirno", "query", "--format", "id,desc"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
    }

    #[test]
    fn query_rejects_old_fields_flag() {
        let error = Cli::try_parse_from(["sirno", "query", "--fields", "id,desc"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn query_rejects_old_fields_short_flag() {
        let error = Cli::try_parse_from(["sirno", "query", "-f", "id,desc"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn query_rejects_old_output_flag() {
        let error = Cli::try_parse_from(["sirno", "query", "--output", "id,desc"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn query_rejects_unknown_column() {
        let error = Cli::try_parse_from(["sirno", "query", "--columns", "id,summary"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn query_rejects_empty_column() {
        let error = Cli::try_parse_from(["sirno", "query", "--columns", "id,,desc"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn query_json_uses_selected_column_names() {
        let columns = "id,desc".parse::<QueryColumns>().unwrap();
        let json = format_query_json(&columns, &[vec!["query".to_owned(), "Selection".to_owned()]])
            .unwrap();
        let parsed = serde_json::from_str::<serde_json::Value>(&json).unwrap();

        assert_eq!(
            json,
            "\
[
  {
    \"id\": \"query\",
    \"desc\": \"Selection\"
  }
]"
        );
        assert_eq!(parsed, serde_json::json!([{ "id": "query", "desc": "Selection" }]));
    }

    #[test]
    fn query_table_uses_selected_column_headers_and_widths() {
        let columns = "id,desc".parse::<QueryColumns>().unwrap();
        let table =
            format_query_table(&columns, &[vec!["query".to_owned(), "Selection".to_owned()]]);

        assert_eq!(
            table,
            "\
┌───────┬───────────┐
│ id    ┆ desc      │
╞═══════╪═══════════╡
│ query ┆ Selection │
└───────┴───────────┘
"
        );
    }

    #[test]
    fn query_table_uses_unicode_display_width() {
        let columns = "id".parse::<QueryColumns>().unwrap();
        let table =
            format_query_table(&columns, &[vec!["界界".to_owned()], vec!["aaa".to_owned()]]);

        assert_eq!(
            table,
            "\
┌──────┐
│ id   │
╞══════╡
│ 界界 │
├╌╌╌╌╌╌┤
│ aaa  │
└──────┘
"
        );
    }

    #[test]
    fn human_table_wraps_to_explicit_width() {
        let table = format_human_table_with_width(
            vec!["id".to_owned(), "desc".to_owned()],
            vec![vec!["query".to_owned(), "one two three".to_owned()]],
            Some(18),
        );

        assert_eq!(
            table,
            "\
┌───────┬────────┐
│ id    ┆ desc   │
╞═══════╪════════╡
│ query ┆ one    │
│       ┆ two    │
│       ┆ three  │
└───────┴────────┘
"
        );
    }

    #[test]
    fn human_table_elides_columns_when_width_is_too_small() {
        let table = format_human_table_with_width(
            vec!["id".to_owned(), "name".to_owned(), "path".to_owned(), "desc".to_owned()],
            vec![vec![
                "a".to_owned(),
                "Beta".to_owned(),
                "sirno-docs/a.md".to_owned(),
                "A compact entry.".to_owned(),
            ]],
            Some(19),
        );

        assert_eq!(
            table,
            "\
┌────┬──────┬─────┐
│ id ┆ name ┆ ... │
╞════╪══════╪═════╡
│ a  ┆ Beta ┆ ... │
└────┴──────┴─────┘
"
        );
    }

    #[test]
    fn query_rejects_old_exact_structural_flags() {
        let error =
            Cli::try_parse_from(["sirno", "query", "--exact", "topic=concept"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);

        let error = Cli::try_parse_from(["sirno", "query", "-x", "topic=concept"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);

        let error =
            Cli::try_parse_from(["sirno", "query", "--exact-topic", "concept"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn query_rejects_empty_has_target() {
        let error = Cli::try_parse_from(["sirno", "query", "--has", "topic=concept,"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn query_rejects_unknown_structural_state_filter() {
        let error = Cli::try_parse_from(["sirno", "query", "--is", "topic=blank"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn check_accepts_short_mode() {
        let cli = Cli::parse_from(["sirno", "check", "-m", "review"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelLake(TopLevelLakeCommand::Check {
                mode: Some(CheckModeArg::Review),
                ..
            })
        ));
    }

    #[test]
    fn rg_accepts_forwarded_arguments() {
        let cli = Cli::parse_from(["sirno", "rg", "--json", "metadata"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Rg { with_generated_footer: false, args })
                if args == vec![OsString::from("--json"), OsString::from("metadata")]
        ));
    }

    #[test]
    fn rg_accepts_generated_footer_inclusion_flag() {
        let cli = Cli::parse_from(["sirno", "rg", "--with-generated-footer", "metadata"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Rg { with_generated_footer: true, args })
                if args == vec![OsString::from("metadata")]
        ));
    }

    #[test]
    fn rg_detects_forwarded_preprocessor_arguments() {
        assert!(rg_args_include_preprocessor(&[OsString::from("--pre"), OsString::from("cat")]));
        assert!(rg_args_include_preprocessor(&[OsString::from("--pre=cat")]));
        assert!(!rg_args_include_preprocessor(&[
            OsString::from("--pre-glob"),
            OsString::from("*.md")
        ]));
    }

    #[test]
    fn rg_requires_forwarded_arguments() {
        let error = Cli::try_parse_from(["sirno", "rg"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn query_filter_rejects_unconfigured_structural_field() {
        let error = entry_query_from_filters(
            EntryQuery::new(),
            vec!["topic=concept".parse::<StructuralFilter>().unwrap()],
            Vec::new(),
            &StructuralSettings::default(),
        )
        .unwrap_err();

        assert!(
            matches!(error, CommandError::UnconfiguredStructuralField(field) if field == "topic")
        );
    }

    #[test]
    fn query_filter_keeps_comma_separated_targets_disjunctive() {
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target("topic", EntryId::new("meta").unwrap());
        let entry = Entry::new(EntryId::new("concept").unwrap(), metadata, "");
        let settings =
            StructuralSettings::from_fields([("topic", StructuralFieldSettings::default())]);
        let query = entry_query_from_filters(
            EntryQuery::new(),
            vec!["topic=concept,meta".parse::<StructuralFilter>().unwrap()],
            Vec::new(),
            &settings,
        )
        .unwrap();

        assert!(query.matches(&entry));
    }

    #[test]
    fn query_filter_keeps_repeated_field_targets_disjunctive() {
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target("topic", EntryId::new("meta").unwrap());
        let entry = Entry::new(EntryId::new("concept").unwrap(), metadata, "");
        let settings =
            StructuralSettings::from_fields([("topic", StructuralFieldSettings::default())]);
        let query = entry_query_from_filters(
            EntryQuery::new(),
            vec![
                "topic=concept".parse::<StructuralFilter>().unwrap(),
                "topic=meta".parse::<StructuralFilter>().unwrap(),
            ],
            Vec::new(),
            &settings,
        )
        .unwrap();

        assert!(query.matches(&entry));
    }

    #[test]
    fn query_filter_matches_present_empty_structural_field() {
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.set_structural_targets("topic", Vec::<EntryId>::new());
        let entry = Entry::new(EntryId::new("concept").unwrap(), metadata, "");
        let settings =
            StructuralSettings::from_fields([("topic", StructuralFieldSettings::default())]);
        let query = entry_query_from_filters(
            EntryQuery::new(),
            Vec::new(),
            vec!["topic=empty".parse::<StructuralStateFilter>().unwrap()],
            &settings,
        )
        .unwrap();

        assert!(query.matches(&entry));
    }

    #[test]
    fn query_filter_keeps_target_and_state_matchers_disjunctive() {
        let mut empty_metadata = EntryMetadata::new("Empty", "A present empty field.").unwrap();
        empty_metadata.set_structural_targets("topic", Vec::<EntryId>::new());
        let empty = Entry::new(EntryId::new("empty").unwrap(), empty_metadata, "");
        let mut targeted_metadata = EntryMetadata::new("Targeted", "A targeted field.").unwrap();
        targeted_metadata.push_structural_target("topic", EntryId::new("meta").unwrap());
        let targeted = Entry::new(EntryId::new("targeted").unwrap(), targeted_metadata, "");
        let settings =
            StructuralSettings::from_fields([("topic", StructuralFieldSettings::default())]);
        let query = entry_query_from_filters(
            EntryQuery::new(),
            vec!["topic=meta".parse::<StructuralFilter>().unwrap()],
            vec!["topic=empty".parse::<StructuralStateFilter>().unwrap()],
            &settings,
        )
        .unwrap();

        assert!(query.matches(&empty));
        assert!(query.matches(&targeted));
    }

    #[test]
    fn subcommands_reject_entries_flag() {
        let error = Cli::try_parse_from(["sirno", "freeze", "alpha", "--entries", "scratch-docs"])
            .unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn melt_accepts_entry_id_and_unfreeze_alias() {
        let melt = Cli::parse_from(["sirno", "melt", "alpha"]);
        let unfreeze = Cli::parse_from(["sirno", "unfreeze", "alpha"]);

        assert!(matches!(
            melt.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Melt { id, .. }) if id == "alpha"
        ));
        assert!(matches!(
            unfreeze.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Melt { id, .. }) if id == "alpha"
        ));
    }

    #[test]
    fn lake_move_moves_lake_and_rewrites_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let old_lake = temp.path().join("docs");
        let new_lake = temp.path().join("sirno-docs");
        let config = SirnoConfig {
            structural: StructuralSettings::from_fields([
                ("zeta", StructuralFieldSettings::default()),
                ("area", StructuralFieldSettings::default()),
            ]),
            ..SirnoConfig::new("docs")
        };
        config.write_new(&config_path).unwrap();
        fs::create_dir(&old_lake).unwrap();
        fs::write(old_lake.join("entry.md"), "entry").unwrap();

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "lake",
            "move",
            "sirno-docs",
        ])
        .run()
        .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        let source = fs::read_to_string(&config_path).unwrap();
        assert_eq!(config.lake.path, PathBuf::from("sirno-docs"));
        assert_before(&source, "[structural.zeta]", "[structural.area]");
        assert!(!old_lake.exists());
        assert!(new_lake.join("entry.md").exists());
    }

    #[test]
    fn lake_move_refuses_existing_destination() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let old_lake = temp.path().join("docs");
        let new_lake = temp.path().join("sirno-docs");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&old_lake).unwrap();
        fs::create_dir(&new_lake).unwrap();

        let error = Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "lake",
            "move",
            "sirno-docs",
        ])
        .run()
        .unwrap_err();

        assert!(matches!(error, CommandError::MoveDestinationExists(_)));
        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.lake.path, PathBuf::from("docs"));
        assert!(old_lake.exists());
    }

    #[test]
    fn frost_move_moves_frost_and_rewrites_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let old_frost = temp.path().join("sirno-frost");
        let new_frost = temp.path().join("frost");
        let config = SirnoConfig {
            structural: StructuralSettings::from_fields([
                ("zeta", StructuralFieldSettings::default()),
                ("area", StructuralFieldSettings::default()),
            ]),
            ..SirnoConfig::new("docs").with_frost("sirno-frost")
        };
        config.write_new(&config_path).unwrap();
        fs::create_dir(&old_frost).unwrap();
        fs::write(old_frost.join("row"), "frost").unwrap();

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "frost",
            "move",
            "frost",
        ])
        .run()
        .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        let source = fs::read_to_string(&config_path).unwrap();
        assert_eq!(config.frost, Some(FrostSettings { path: PathBuf::from("frost") }));
        assert_before(&source, "[structural.zeta]", "[structural.area]");
        assert!(!old_frost.exists());
        assert!(new_frost.join("row").exists());
    }

    #[test]
    fn move_lake_wrapper_moves_lake_and_rewrites_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let old_lake = temp.path().join("docs");
        let new_lake = temp.path().join("sirno-docs");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&old_lake).unwrap();
        fs::write(old_lake.join("entry.md"), "entry").unwrap();

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "move",
            "lake",
            "sirno-docs",
        ])
        .run()
        .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.lake.path, PathBuf::from("sirno-docs"));
        assert!(!old_lake.exists());
        assert!(new_lake.join("entry.md").exists());
    }

    #[test]
    fn move_frost_wrapper_moves_frost_and_rewrites_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let old_frost = temp.path().join("sirno-frost");
        let new_frost = temp.path().join("frost");
        SirnoConfig::new("docs").with_frost("sirno-frost").write_new(&config_path).unwrap();
        fs::create_dir(&old_frost).unwrap();
        fs::write(old_frost.join("row"), "frost").unwrap();

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "move",
            "frost",
            "frost",
        ])
        .run()
        .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.frost, Some(FrostSettings { path: PathBuf::from("frost") }));
        assert!(!old_frost.exists());
        assert!(new_frost.join("row").exists());
    }

    #[test]
    fn freeze_and_melt_commands_toggle_marker_and_permissions() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        SirnoConfig::new("docs").with_frost("sirno-frost").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        )
        .unwrap();

        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "commit"])
            .run()
            .unwrap();
        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "freeze", "alpha"])
            .run()
            .unwrap();
        let source = fs::read_to_string(docs.join("alpha.md")).unwrap();
        assert!(source.contains("frozen:\n"));
        assert!(fs::metadata(docs.join("alpha.md")).unwrap().permissions().readonly());

        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "melt", "alpha"])
            .run()
            .unwrap();
        let source = fs::read_to_string(docs.join("alpha.md")).unwrap();
        assert!(!source.contains("frozen:\n"));
        assert!(!fs::metadata(docs.join("alpha.md")).unwrap().permissions().readonly());
    }

    #[test]
    fn frost_commit_preserves_frozen_entry_permissions() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        SirnoConfig::new("docs").with_frost("sirno-frost").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        )
        .unwrap();

        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "commit"])
            .run()
            .unwrap();
        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "freeze", "alpha"])
            .run()
            .unwrap();
        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "commit"])
            .run()
            .unwrap();

        assert!(fs::metadata(docs.join("alpha.md")).unwrap().permissions().readonly());

        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "melt", "alpha"])
            .run()
            .unwrap();
    }

    #[test]
    fn freeze_command_requires_current_frost_entry() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        SirnoConfig::new("docs").with_frost("sirno-frost").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        )
        .unwrap();
        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "commit"])
            .run()
            .unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
---

Changed body.
",
        )
        .unwrap();

        let error = Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "freeze",
            "alpha",
        ])
        .run()
        .unwrap_err();

        assert!(
            matches!(error, CommandError::Frost(FrostError::FrozenEntryChanged(id)) if id.as_str() == "alpha")
        );
    }

    #[test]
    fn rename_command_updates_lake_and_witness_references() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        let src = temp.path().join("src");
        SirnoConfig {
            repo: Some(RepoSettings { members: vec![RepoMember::new("src").unwrap()] }),
            structural: StructuralSettings::from_fields([(
                "area",
                StructuralFieldSettings::default(),
            )]),
            ..SirnoConfig::new("docs")
        }
        .write_new(&config_path)
        .unwrap();
        fs::create_dir(&docs).unwrap();
        fs::create_dir(&src).unwrap();
        fs::write(
            docs.join("old-entry.md"),
            "\
---
name: Old
desc: Old entry.
---

Body.
",
        )
        .unwrap();
        fs::write(
            docs.join("reader.md"),
            "\
---
name: Reader
desc: Reader entry.
area:
  - old-entry
---

Body.
",
        )
        .unwrap();
        let witness_source = format!(
            "\
// sirno{}old-entry:begin
fn sample() {{}}
// sirno{}old-entry:end
",
            ":witness:", ":witness:"
        );
        fs::write(src.join("lib.rs"), witness_source).unwrap();

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "entry",
            "rename",
            "old-entry",
            "new-entry",
        ])
        .run()
        .unwrap();

        let reader_source = fs::read_to_string(docs.join("reader.md")).unwrap();
        let witness_source = fs::read_to_string(src.join("lib.rs")).unwrap();
        assert!(!docs.join("old-entry.md").exists());
        assert!(docs.join("new-entry.md").exists());
        assert!(reader_source.contains("area:\n  - new-entry\n"));
        assert!(witness_source.contains("sirno:witness:new-entry:begin"));
        assert!(witness_source.contains("sirno:witness:new-entry:end"));
    }

    #[test]
    fn move_entry_wrapper_renames_entry() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("docs");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("old-entry.md"),
            "\
---
name: Old
desc: Old entry.
---

Body.
",
        )
        .unwrap();

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "move",
            "entry",
            "old-entry",
            "new-entry",
        ])
        .run()
        .unwrap();

        assert!(!docs.join("old-entry.md").exists());
        assert!(docs.join("new-entry.md").exists());
    }

    #[test]
    fn lake_path_override_targets_public_lake_commands() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let configured_docs = temp.path().join("docs");
        let override_docs = temp.path().join("scratch-docs");
        SirnoConfig::new("docs").with_frost("sirno-frost").write_new(&config_path).unwrap();
        fs::create_dir(&configured_docs).unwrap();
        fs::create_dir(&override_docs).unwrap();
        let entry = "\
---
name: Alpha
desc: Alpha entry.
---

Body.
";
        fs::write(configured_docs.join("alpha.md"), entry).unwrap();
        fs::write(override_docs.join("alpha.md"), entry).unwrap();
        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "frost",
            "commit",
            "--lake-path",
            override_docs.to_str().unwrap(),
        ])
        .run()
        .unwrap();

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "freeze",
            "alpha",
            "--lake-path",
            override_docs.to_str().unwrap(),
        ])
        .run()
        .unwrap();

        assert!(!fs::read_to_string(configured_docs.join("alpha.md")).unwrap().contains("frozen:"));
        assert!(fs::read_to_string(override_docs.join("alpha.md")).unwrap().contains("frozen:"));
    }

    #[test]
    fn new_rejects_witness_flag() {
        let error = Cli::try_parse_from(["sirno", "new", "alpha", "--desc", "Alpha.", "--witness"])
            .unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn new_rejects_old_description_flag() {
        let error =
            Cli::try_parse_from(["sirno", "new", "alpha", "--description", "Alpha."]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn witness_accepts_entry_id() {
        let cli = Cli::parse_from(["sirno", "witness", "witness"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Witness { id, full: false }) if id == "witness"
        ));
    }

    #[test]
    fn status_accepts_short_alias() {
        let cli = Cli::parse_from(["sirno", "st"]);

        assert!(matches!(cli.command, Command::TopLevelLake(TopLevelLakeCommand::Status)));
    }

    #[test]
    fn witness_accepts_short_aliases() {
        let short = Cli::parse_from(["sirno", "w", "alpha"]);
        let mnemonic = Cli::parse_from(["sirno", "wit", "beta"]);

        assert!(matches!(
            short.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Witness { id, full: false }) if id == "alpha"
        ));
        assert!(matches!(
            mnemonic.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Witness { id, full: false }) if id == "beta"
        ));
    }

    #[test]
    fn lake_subcommand_accepts_status_alias() {
        let status = Cli::parse_from(["sirno", "lake", "st"]);

        assert!(matches!(
            status.command,
            Command::Lake { command: LakeCommand::TopLevel(TopLevelLakeCommand::Status) }
        ));
    }

    #[test]
    fn lake_subcommand_rejects_entry_aliases() {
        let error = Cli::try_parse_from(["sirno", "lake", "q"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn entry_subcommand_accepts_common_aliases() {
        let short_query = Cli::parse_from(["sirno", "entry", "q", "alpha"]);
        let short_witness = Cli::parse_from(["sirno", "entry", "w", "alpha"]);
        let mnemonic_witness = Cli::parse_from(["sirno", "entry", "wit", "beta"]);

        assert!(matches!(
            short_query.command,
            Command::Entry {
                command: EntryCommand::TopLevel(TopLevelEntryCommand::Query { terms, .. })
            }
                if terms == vec!["alpha"]
        ));
        assert!(matches!(
            short_witness.command,
            Command::Entry {
                command: EntryCommand::TopLevel(TopLevelEntryCommand::Witness {
                    id,
                    full: false,
                })
            }
                if id == "alpha"
        ));
        assert!(matches!(
            mnemonic_witness.command,
            Command::Entry {
                command: EntryCommand::TopLevel(TopLevelEntryCommand::Witness {
                    id,
                    full: false,
                })
            }
                if id == "beta"
        ));
    }

    #[test]
    fn witness_accepts_full_flag() {
        let cli = Cli::parse_from(["sirno", "witness", "witness", "--full"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Witness { id, full: true }) if id == "witness"
        ));
    }

    #[test]
    fn witness_accepts_short_full_flag() {
        let cli = Cli::parse_from(["sirno", "witness", "witness", "-f"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Witness { id, full: true }) if id == "witness"
        ));
    }

    #[test]
    fn witness_rejects_missing_entry_before_repo_scan() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        fs::create_dir(temp.path().join("docs")).unwrap();
        SirnoConfig {
            repo: Some(RepoSettings { members: vec![RepoMember::new("missing-src").unwrap()] }),
            ..SirnoConfig::new("docs")
        }
        .write_new(&config_path)
        .unwrap();

        let error = Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "witness",
            "missing-entry",
        ])
        .run()
        .unwrap_err();

        assert!(
            matches!(error, CommandError::MissingWitnessEntry(id) if id.as_str() == "missing-entry")
        );
    }

    // sirno:witness:witness-fixture-isolation:begin
    #[test]
    fn format_witness_record_prints_range_and_preserves_body() {
        let record = WitnessRecord {
            entry: EntryId::new("entry").unwrap(),
            path: PathBuf::from("src/lib.rs"),
            region: witness_span(10, 5, 14, 25),
            opening: witness_span(10, 5, 10, 33),
            closing: witness_span(14, 5, 14, 25),
            marker: "    // sample:start entry".to_owned(),
            body: concat!(
                "    // sample:start entry\n",
                "        fn main() {}\n",
                "    // sample:end"
            )
            .to_owned(),
        };

        assert_eq!(
            format_witness_record(&record, false),
            "src/lib.rs:10:5-33 :: 14:5-25\t    // sample:start entry\n"
        );
        assert_eq!(
            format_witness_record(&record, true),
            concat!(
                "src/lib.rs:10:5-33 :: 14:5-25\n",
                "\n",
                "    // sample:start entry\n",
                "        fn main() {}\n",
                "    // sample:end\n",
                "\n",
            )
        );
    }

    #[test]
    fn format_witness_records_adds_full_region_spacing() {
        let first = WitnessRecord {
            entry: EntryId::new("entry").unwrap(),
            path: PathBuf::from("src/lib.rs"),
            region: witness_span(10, 5, 14, 25),
            opening: witness_span(10, 5, 10, 33),
            closing: witness_span(14, 5, 14, 25),
            marker: "    // sample:start entry".to_owned(),
            body: concat!(
                "    // sample:start entry\n",
                "        fn main() {}\n",
                "    // sample:end"
            )
            .to_owned(),
        };
        let mut second = first.clone();
        second.region = witness_span(20, 5, 24, 25);
        second.opening = witness_span(20, 5, 20, 33);
        second.closing = witness_span(24, 5, 24, 25);

        assert!(format_witness_records(&[first, second], true).contains(concat!(
            "    // sample:end\n",
            "\n",
            "---\n",
            "\n",
            "src/lib.rs:20:5-33 :: 24:5-25\n",
        )));
    }
    // sirno:witness:witness-fixture-isolation:end

    fn witness_span(
        start_line: usize, start_column: usize, end_line: usize, end_column: usize,
    ) -> WitnessSpan {
        WitnessSpan { start_line, start_column, end_line, end_column }
    }

    #[test]
    fn render_rejects_no_check_flag() {
        let error = Cli::try_parse_from(["sirno", "render", "--no-check"]).unwrap_err();

        assert!(error.to_string().contains("unexpected argument"));
    }

    #[test]
    fn render_accepts_dry_flag() {
        let cli = Cli::parse_from(["sirno", "render", "--dry"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelLake(TopLevelLakeCommand::Render { dry: true, command: None, .. })
        ));
    }

    #[test]
    fn render_accepts_dry_run_aliases() {
        let short = Cli::parse_from(["sirno", "render", "-n"]);
        let long = Cli::parse_from(["sirno", "render", "--dry-run"]);

        assert!(matches!(
            short.command,
            Command::TopLevelLake(TopLevelLakeCommand::Render { dry: true, command: None, .. })
        ));
        assert!(matches!(
            long.command,
            Command::TopLevelLake(TopLevelLakeCommand::Render { dry: true, command: None, .. })
        ));
    }

    #[test]
    fn format_gen_link_report_lists_changed_paths() {
        let report = format_gen_link_report(
            Path::new("sirno-docs"),
            31,
            &[PathBuf::from("sirno-docs/concept.md"), PathBuf::from("sirno-docs/entry.md")],
        );

        assert_eq!(
            report,
            "Changes in sirno-docs:\n- sirno-docs/concept.md\n- sirno-docs/entry.md\nTotal changes: 2/31"
        );
    }

    #[test]
    fn format_gen_link_report_summarizes_no_changes() {
        let report = format_gen_link_report(Path::new("sirno-docs"), 31, &[]);

        assert_eq!(report, "No changes in sirno-docs");
    }
}
