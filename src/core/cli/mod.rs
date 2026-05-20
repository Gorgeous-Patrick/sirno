//! CLI grammar and terminal dispatch for the shared command surface.

mod config {
    pub(crate) mod tui;
}

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use serde::Deserialize;
use thiserror::Error;

use crate::core::CoreContext;
use crate::core::context::{default_config_path, default_lake_path};
use crate::core::dto::{
    ArtifactAddRequest, ArtifactRemoveRequest, ArtifactRenameRequest, EntryNewRequest,
    EntryPathRequest, FrostCheckoutRequest, LakeInitRequest, PathRecord, PathSelection,
    QueryColumns, QueryOutputFormat, QueryRequest, QueryRun, RgRequest, SkillWrapperResult,
    StructuralFilter, StructuralStateFilter, StructuralTarget, TideOutputFormat,
    TideResolveRequest, TideSelectionRequest, TideStatusMode,
};
use crate::core::error::CommandError;
use crate::core::output::{
    format_human_table_with_width, format_path_table, format_skill_wrapper_table,
    print_config_comment_result, print_entry_directory_report, print_json, print_lake_check_result,
    print_query_results, print_render_result, print_status_result, print_witness_records,
};
use crate::core::rg::{
    is_rg_preprocessor_invocation, rg_args_to_strings, run_rg_preprocessor_from_env,
};
use crate::{
    CheckMode, EntryId, EntryIdError, SirnoConfig, SirnoFrost, TideSource, TideStatus,
    TideWorkitem, TideWorkitemParseError,
};

#[cfg(test)]
use crate::core::context::entry_query_from_filters;
#[cfg(test)]
use crate::core::dto::{QueryColumn, StructuralFieldState};
#[cfg(test)]
use crate::core::error::OpenTideTutorial;
#[cfg(test)]
use crate::core::output::{
    format_config_comment_result, format_gen_link_report, format_json, format_lake_check_result,
    format_query_json, format_query_table, format_render_result, format_witness_record,
    format_witness_records,
};
#[cfg(test)]
use crate::core::rg::rg_args_include_preprocessor;

/// Sirno command-line entry point.
#[derive(Debug, Parser)]
// sirno:witness:interfaces:begin
#[command(name = "sirno")]
#[command(about = "Manage Sirno design entries")]
#[command(version)]
// sirno:witness:interfaces:end
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
    /// Create a Sirno config, public lake, Frost store, and skill wrappers.
    Init {
        /// Public Markdown entry lake path written to Sirno.toml.
        #[arg(long)]
        lake: Option<PathBuf>,
        /// Sirno Frost path written to Sirno.toml.
        #[arg(long)]
        frost: Option<PathBuf>,
        /// Skip public lake initialization.
        #[arg(long = "no-lake", conflicts_with = "lake")]
        no_lake: bool,
        /// Skip Sirno Frost initialization.
        #[arg(long = "no-frost", conflicts_with = "frost")]
        no_frost: bool,
        /// Skip packaged skill wrapper initialization.
        #[arg(long = "no-skills")]
        no_skills: bool,
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
    // sirno:witness:interfaces:begin
    /// Render Markdown links in entry footers.
    Render {
        /// Report rendered-footer changes without writing files.
        #[arg(short = 'n', long, visible_alias = "dry-run")]
        dry: bool,
        /// JSON structural render settings used instead of the configured settings for this run.
        #[arg(long = "override-json", value_name = "JSON")]
        override_json: Option<String>,
        /// Render command.
        #[command(subcommand)]
        command: Option<RenderCommand>,
    },
    // sirno:witness:interfaces:end
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
    /// Show tide review status.
    Status {
        /// Select review entries, full open workitems, or all workitems.
        #[arg(long, value_enum, default_value_t = TideStatusMode::Review)]
        show: TideStatusMode,
        /// Group human output by wave or review entry.
        #[arg(long, value_enum, default_value_t = TideStatusGrouping::Entry)]
        by: TideStatusGrouping,
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

/// Human grouping for tide status output.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
enum TideStatusGrouping {
    /// Group by the changed ripple entry that caused review.
    Wave,
    /// Group by the entry that needs review.
    #[default]
    Entry,
}

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
    // sirno:witness:interfaces:begin
    /// Open the interactive Sirno.toml maintenance UI.
    Config(ConfigTuiArgs),
    // sirno:witness:interfaces:end
    /// Generate a shell completion script.
    Completion {
        /// Shell whose completion script should be generated.
        #[arg(value_enum)]
        shell: CompletionShell,
    },
    // sirno:witness:interfaces:begin
    /// Manage packaged Sirno skill wrappers.
    Skills {
        /// Skill wrapper command.
        #[command(subcommand)]
        command: SkillCommand,
    },
    // sirno:witness:interfaces:end
    // sirno:witness:interfaces:begin
    /// Run the Sirno MCP server over stdio.
    Mcp,
    // sirno:witness:interfaces:end
}

// sirno:witness:interfaces:begin
/// Arguments for interactive config maintenance.
#[derive(Debug, Args)]
struct ConfigTuiArgs {
    /// Print missing canonical comments without opening the TUI or writing the file.
    #[arg(long, conflicts_with = "fix")]
    dry: bool,
    /// Rewrite Sirno.toml with canonical comments without opening the TUI.
    #[arg(long)]
    fix: bool,
}
// sirno:witness:interfaces:end

/// Supported skill wrapper utility commands.
// sirno:witness:interfaces:begin
#[derive(Debug, Subcommand)]
enum SkillCommand {
    /// Install bundled wrappers into `.agents/skills/sirno-*`.
    Init,
    /// Check installed wrappers against bundled wrappers.
    Check,
    /// List bundled wrappers and package targets.
    List,
}
// sirno:witness:interfaces:end

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
            | Command::Init { lake, frost, no_lake, no_frost, no_skills } => {
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                let request = TopLevelInitRequest {
                    lake,
                    frost,
                    init_lake: !no_lake,
                    init_frost: !no_frost,
                    init_skills: !no_skills,
                };
                run_top_level_init(request, &config_path, lake_path.as_deref())
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

#[derive(Debug)]
struct TopLevelInitRequest {
    lake: Option<PathBuf>,
    frost: Option<PathBuf>,
    init_lake: bool,
    init_frost: bool,
    init_skills: bool,
}

fn run_top_level_init(
    request: TopLevelInitRequest, config_path: &Path, lake_path: Option<&Path>,
) -> Result<ExitCode, CommandError> {
    let mut initialized = false;
    if request.init_lake {
        run_lake_init(request.lake, config_path, lake_path)?;
        initialized = true;
    }
    if request.init_frost {
        if !request.init_lake {
            ensure_config_for_top_level_frost(config_path, lake_path)?;
        }
        FrostCommand::Init { frost: request.frost }.run(config_path, lake_path)?;
        initialized = true;
    }
    if request.init_skills {
        run_skill_wrappers_init(config_path)?;
        initialized = true;
    }
    if !initialized {
        println!("nothing initialized");
    }
    Ok(ExitCode::SUCCESS)
}

fn ensure_config_for_top_level_frost(
    config_path: &Path, lake_path: Option<&Path>,
) -> Result<(), CommandError> {
    if config_path.exists() {
        return Ok(());
    }
    let config = SirnoConfig::new(
        lake_path.map(Path::to_path_buf).unwrap_or_else(|| default_lake_path(config_path)),
    );
    config.write_new(config_path)?;
    Ok(())
}

fn run_lake_init(
    lake: Option<PathBuf>, config_path: &Path, lake_path: Option<&Path>,
) -> Result<ExitCode, CommandError> {
    let result =
        CoreContext::from_cli_paths(config_path, lake_path).lake_init(LakeInitRequest { lake })?;
    println!("{}", result.message);
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
            | LakeCommand::Init { lake } => run_lake_init(lake, config_path, lake_path),
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
                if report.has_errors() {
                    println!("check: failed in {}", frost.root().display());
                } else {
                    println!("check: warnings in {}", frost.root().display());
                }

                if report.has_errors() { Ok(ExitCode::FAILURE) } else { Ok(ExitCode::SUCCESS) }
            }
            | TopLevelLakeCommand::Render { .. } | TopLevelLakeCommand::Status
                if frost_path.is_some() =>
            {
                Err(CommandError::FrostPathRequiresCheck)
            }
            // sirno:witness:interfaces:begin
            | TopLevelLakeCommand::Render { command, dry, override_json } => match command {
                | None => {
                    let result = CoreContext::from_cli_paths(config_path, lake_path)
                        .lake_render_with_override_json(dry, override_json.as_deref())?;
                    print_render_result(&result);
                    if result.ok { Ok(ExitCode::SUCCESS) } else { Ok(ExitCode::FAILURE) }
                }
                | Some(RenderCommand::Delete) => {
                    if dry {
                        return Err(CommandError::DryWithRenderSubcommand);
                    }
                    if override_json.is_some() {
                        return Err(CommandError::OverrideJsonWithRenderSubcommand);
                    }
                    let result =
                        CoreContext::from_cli_paths(config_path, lake_path).lake_render_delete()?;
                    print_render_result(&result);
                    Ok(ExitCode::SUCCESS)
                }
            },
            // sirno:witness:interfaces:end
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
            | TideCommand::Status { show, by, format } => {
                let context = CoreContext::from_cli_paths(config_path, lake_path);
                let format = format.unwrap_or_default();
                if show.includes_workitems() {
                    let statuses = context.tide_statuses(show)?;
                    print_tide_statuses(&statuses, by, format)?;
                    Ok(if statuses.iter().all(|status| status.resolved) {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    })
                } else {
                    let statuses = context.tide_statuses(show)?;
                    print_tide_review_waves(&statuses, by, format)?;
                    Ok(if statuses.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
                }
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

fn print_tide_statuses(
    statuses: &[TideStatus], grouping: TideStatusGrouping, format: TideOutputFormat,
) -> Result<(), CommandError> {
    match format {
        | TideOutputFormat::Json => {
            print_json(statuses)?;
        }
        | TideOutputFormat::Human => {
            print!("{}", format_tide_statuses_grouped(statuses, grouping));
        }
    }
    Ok(())
}

fn print_tide_review_waves(
    statuses: &[TideStatus], grouping: TideStatusGrouping, format: TideOutputFormat,
) -> Result<(), CommandError> {
    match format {
        | TideOutputFormat::Json => {
            let entries = tide_review_entries_from_statuses(statuses);
            print_json(&entries)?;
        }
        | TideOutputFormat::Human => {
            print!("{}", format_tide_review_waves_grouped(statuses, grouping));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TideReviewWave {
    ripple: EntryId,
    entries: Vec<EntryId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TideReviewEntryGroup {
    entry: EntryId,
    ripples: Vec<EntryId>,
}

fn format_tide_review_waves_grouped(
    statuses: &[TideStatus], grouping: TideStatusGrouping,
) -> String {
    match grouping {
        | TideStatusGrouping::Wave => format_tide_review_waves(statuses),
        | TideStatusGrouping::Entry => format_tide_review_entries(statuses),
    }
}

fn format_tide_review_waves(statuses: &[TideStatus]) -> String {
    let waves = tide_review_waves(statuses);
    if waves.is_empty() {
        return "tide: clear\n".to_owned();
    }

    let open_count = statuses.iter().filter(|status| !status.resolved).count();
    let review_entry_count = tide_review_entries_from_statuses(statuses).len();
    let rows = waves
        .iter()
        .flat_map(|wave| {
            wave.entries.iter().enumerate().map(|(index, entry)| {
                let ripple = if index == 0 { wave.ripple.to_string() } else { String::new() };
                TideGroupedTableRow {
                    starts_group: index == 0,
                    cells: vec![ripple, entry.to_string()],
                }
            })
        })
        .collect::<Vec<_>>();
    let mut output = format_tide_grouped_table(vec!["wave".to_owned(), "entry".to_owned()], rows);
    output.push('\n');
    output.push_str(&tide_summary_sentence(open_count, 0, waves.len(), review_entry_count));
    output.push('\n');

    output
}

fn format_tide_review_entries(statuses: &[TideStatus]) -> String {
    let entries = tide_review_entry_groups(statuses);
    if entries.is_empty() {
        return "tide: clear\n".to_owned();
    }

    let waves = tide_review_waves(statuses);
    let open_count = statuses.iter().filter(|status| !status.resolved).count();
    let review_entry_count = entries.len();
    let rows = entries
        .iter()
        .flat_map(|group| {
            group.ripples.iter().enumerate().map(|(index, ripple)| {
                let entry = if index == 0 { group.entry.to_string() } else { String::new() };
                TideGroupedTableRow {
                    starts_group: index == 0,
                    cells: vec![entry, ripple.to_string()],
                }
            })
        })
        .collect::<Vec<_>>();
    let mut output = format_tide_grouped_table(vec!["entry".to_owned(), "reason".to_owned()], rows);
    output.push('\n');
    output.push_str(&tide_summary_sentence(open_count, 0, waves.len(), review_entry_count));
    output.push('\n');

    output
}

fn format_tide_statuses_grouped(statuses: &[TideStatus], grouping: TideStatusGrouping) -> String {
    match grouping {
        | TideStatusGrouping::Wave => format_tide_statuses(statuses),
        | TideStatusGrouping::Entry => format_tide_statuses_by_entry(statuses),
    }
}

fn format_tide_statuses(statuses: &[TideStatus]) -> String {
    let waves = tide_status_waves(statuses);
    if waves.is_empty() {
        return "tide: clear\n".to_owned();
    }

    let open_count = statuses.iter().filter(|status| !status.resolved).count();
    let resolved_count = statuses.len() - open_count;
    let review_entry_count = tide_review_entries_from_statuses(statuses).len();
    let rows = waves
        .iter()
        .flat_map(|wave| {
            wave.statuses.iter().enumerate().map(|(index, status)| {
                let ripple = if index == 0 { wave.ripple.to_string() } else { String::new() };
                TideGroupedTableRow {
                    starts_group: index == 0,
                    cells: vec![
                        ripple,
                        status.workitem.neighbor.to_string(),
                        tide_state_label(status).to_owned(),
                        status.workitem.field.clone(),
                        status.workitem.direction.to_string(),
                        tide_sources_label(status),
                    ],
                }
            })
        })
        .collect::<Vec<_>>();
    let mut output = format_tide_grouped_table(
        vec![
            "wave".to_owned(),
            "entry".to_owned(),
            "state".to_owned(),
            "field".to_owned(),
            "direction".to_owned(),
            "sources".to_owned(),
        ],
        rows,
    );
    output.push('\n');
    output.push_str(&tide_summary_sentence(
        open_count,
        resolved_count,
        waves.len(),
        review_entry_count,
    ));
    output.push('\n');

    output
}

fn format_tide_statuses_by_entry(statuses: &[TideStatus]) -> String {
    let entries = tide_status_entry_groups(statuses);
    if entries.is_empty() {
        return "tide: clear\n".to_owned();
    }

    let waves = tide_status_waves(statuses);
    let open_count = statuses.iter().filter(|status| !status.resolved).count();
    let resolved_count = statuses.len() - open_count;
    let review_entry_count = tide_review_entries_from_statuses(statuses).len();
    let rows = entries
        .iter()
        .flat_map(|group| {
            group.statuses.iter().enumerate().map(|(index, status)| {
                let entry = if index == 0 { group.entry.to_string() } else { String::new() };
                TideGroupedTableRow {
                    starts_group: index == 0,
                    cells: vec![
                        entry,
                        status.workitem.ripple.to_string(),
                        tide_state_label(status).to_owned(),
                        status.workitem.field.clone(),
                        status.workitem.direction.to_string(),
                        tide_sources_label(status),
                    ],
                }
            })
        })
        .collect::<Vec<_>>();
    let mut output = format_tide_grouped_table(
        vec![
            "entry".to_owned(),
            "reason".to_owned(),
            "state".to_owned(),
            "field".to_owned(),
            "direction".to_owned(),
            "sources".to_owned(),
        ],
        rows,
    );
    output.push('\n');
    output.push_str(&tide_summary_sentence(
        open_count,
        resolved_count,
        waves.len(),
        review_entry_count,
    ));
    output.push('\n');

    output
}

fn tide_summary_sentence(
    open_count: usize, resolved_count: usize, wave_count: usize, review_entry_count: usize,
) -> String {
    let resolved = if resolved_count == 0 {
        String::new()
    } else {
        format!(
            " and {resolved_count} resolved {}",
            plural(resolved_count, "workitem", "workitems"),
        )
    };
    format!(
        "The tide has {open_count} open {}{resolved} in {wave_count} {}, \
         with {review_entry_count} unique {}.",
        plural(open_count, "workitem", "workitems"),
        plural(wave_count, "wave", "waves"),
        plural(review_entry_count, "review entry", "review entries"),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TideGroupedTableRow {
    cells: Vec<String>,
    starts_group: bool,
}

fn format_tide_grouped_table(headers: Vec<String>, rows: Vec<TideGroupedTableRow>) -> String {
    let group_start_rows = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.starts_group.then_some(index))
        .filter(|index| *index > 0)
        .collect::<Vec<_>>();
    let rows = rows.into_iter().map(|row| row.cells).collect::<Vec<_>>();
    let table = format_human_table_with_width(headers, rows, None);
    strengthen_tide_group_separators(&table, &group_start_rows)
}

fn strengthen_tide_group_separators(table: &str, group_start_rows: &[usize]) -> String {
    if group_start_rows.is_empty() {
        return table.to_owned();
    }

    let mut lines = table.lines().map(str::to_owned).collect::<Vec<_>>();
    for row_index in group_start_rows {
        if let Some(separator) = lines.get_mut(tide_row_separator_index(*row_index)) {
            *separator = heavy_table_separator(separator);
        }
    }

    let mut output = lines.join("\n");
    output.push('\n');
    output
}

fn tide_row_separator_index(row_index: usize) -> usize {
    2 * row_index + 2
}

fn heavy_table_separator(separator: &str) -> String {
    let length = separator.chars().count();
    separator
        .chars()
        .enumerate()
        .map(|(index, character)| {
            if index == 0 {
                '╞'
            } else if index + 1 == length {
                '╡'
            } else if character == '┼' {
                '╪'
            } else {
                '═'
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TideStatusWave<'a> {
    ripple: EntryId,
    statuses: Vec<&'a TideStatus>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TideStatusEntryGroup<'a> {
    entry: EntryId,
    statuses: Vec<&'a TideStatus>,
}

fn tide_review_waves(statuses: &[TideStatus]) -> Vec<TideReviewWave> {
    let mut entries_by_ripple = BTreeMap::<EntryId, BTreeSet<EntryId>>::new();
    for status in statuses.iter().filter(|status| !status.resolved) {
        entries_by_ripple
            .entry(status.workitem.ripple.clone())
            .or_default()
            .insert(status.workitem.neighbor.clone());
    }

    entries_by_ripple
        .into_iter()
        .map(|(ripple, entries)| TideReviewWave { ripple, entries: entries.into_iter().collect() })
        .collect()
}

fn tide_review_entry_groups(statuses: &[TideStatus]) -> Vec<TideReviewEntryGroup> {
    let mut ripples_by_entry = BTreeMap::<EntryId, BTreeSet<EntryId>>::new();
    for status in statuses.iter().filter(|status| !status.resolved) {
        ripples_by_entry
            .entry(status.workitem.neighbor.clone())
            .or_default()
            .insert(status.workitem.ripple.clone());
    }

    ripples_by_entry
        .into_iter()
        .map(|(entry, ripples)| TideReviewEntryGroup {
            entry,
            ripples: ripples.into_iter().collect(),
        })
        .collect()
}

fn tide_status_waves(statuses: &[TideStatus]) -> Vec<TideStatusWave<'_>> {
    let mut statuses_by_ripple = BTreeMap::<EntryId, Vec<&TideStatus>>::new();
    for status in statuses {
        statuses_by_ripple.entry(status.workitem.ripple.clone()).or_default().push(status);
    }

    statuses_by_ripple
        .into_iter()
        .map(|(ripple, statuses)| TideStatusWave { ripple, statuses })
        .collect()
}

fn tide_status_entry_groups(statuses: &[TideStatus]) -> Vec<TideStatusEntryGroup<'_>> {
    let mut statuses_by_entry = BTreeMap::<EntryId, Vec<&TideStatus>>::new();
    for status in statuses {
        statuses_by_entry.entry(status.workitem.neighbor.clone()).or_default().push(status);
    }

    statuses_by_entry
        .into_iter()
        .map(|(entry, mut statuses)| {
            statuses.sort_by(|left, right| left.workitem.cmp(&right.workitem));
            TideStatusEntryGroup { entry, statuses }
        })
        .collect()
}

fn tide_review_entries_from_statuses(statuses: &[TideStatus]) -> Vec<EntryId> {
    statuses
        .iter()
        .filter(|status| !status.resolved)
        .map(|status| status.workitem.neighbor.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn tide_state_label(status: &TideStatus) -> &'static str {
    if status.resolved { "resolved" } else { "open" }
}

fn tide_sources_label(status: &TideStatus) -> String {
    status
        .sources
        .iter()
        .map(|source| match source {
            | TideSource::Lake => "lake",
            | TideSource::Frost => "frost",
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
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
            | UtilCommand::Config(args) => {
                if lake_path.is_some() {
                    return Err(CommandError::ConfigRejectsLakePath);
                }
                if frost_path.is_some() {
                    return Err(CommandError::ConfigRejectsFrostPath);
                }
                if args.dry {
                    let result =
                        CoreContext::new(config_path.to_path_buf()).config_comments_check()?;
                    print_config_comment_result(&result);
                    return if result.ok { Ok(ExitCode::SUCCESS) } else { Ok(ExitCode::FAILURE) };
                }
                if args.fix {
                    let result =
                        CoreContext::new(config_path.to_path_buf()).config_comments_fix()?;
                    print_config_comment_result(&result);
                    return Ok(ExitCode::SUCCESS);
                }
                config::tui::run(config_path)
            }
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
            | UtilCommand::Skills { command } => {
                if lake_path.is_some() {
                    return Err(CommandError::SkillsRejectsLakePath);
                }
                if frost_path.is_some() {
                    return Err(CommandError::FrostPathRequiresCheck);
                }
                command.run(config_path)
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

impl SkillCommand {
    fn run(self, config_path: &Path) -> Result<ExitCode, CommandError> {
        let context = CoreContext::new(config_path.to_path_buf());
        let result = match self {
            | SkillCommand::Init => context.skill_wrappers_init()?,
            | SkillCommand::Check => context.skill_wrappers_check()?,
            | SkillCommand::List => context.skill_wrappers_list()?,
        };
        Ok(print_skill_wrapper_result(result))
    }
}

fn run_skill_wrappers_init(config_path: &Path) -> Result<(), CommandError> {
    let result = CoreContext::new(config_path.to_path_buf()).skill_wrappers_init()?;
    print_skill_wrapper_result(result);
    Ok(())
}

fn print_skill_wrapper_result(result: SkillWrapperResult) -> ExitCode {
    print!("{}", format_skill_wrapper_table(&result.records));
    println!("{}", result.message);
    if result.ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
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

fn entry_path_records(
    config_path: &Path, lake_path: Option<&Path>, args: &EntryPathArgs,
) -> Result<Vec<PathRecord>, CommandError> {
    let request = EntryPathRequest::new(
        EntryId::new(&args.id)?,
        path_selection_from_args(args),
        args.absolute,
    );
    CoreContext::from_cli_paths(config_path, lake_path).entry_paths(request)
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

fn path_selection_from_args(args: &EntryPathArgs) -> PathSelection {
    let all = !args.show_entry && !args.show_artifact && !args.show_frost;
    PathSelection::new(all || args.show_entry, all || args.show_artifact, all || args.show_frost)
}

#[cfg(test)]
mod tests;
