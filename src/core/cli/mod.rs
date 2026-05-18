//! CLI grammar and terminal dispatch for the shared command surface.

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
    QueryColumns, QueryOutputFormat, QueryRequest, QueryRun, RgRequest, StructuralFilter,
    StructuralStateFilter, StructuralTarget, TideOutputFormat, TideResolveRequest,
    TideSelectionRequest,
};
use crate::core::error::CommandError;
use crate::core::output::{
    format_path_table, print_entry_directory_report, print_json, print_lake_check_result,
    print_query_results, print_render_result, print_status_result, print_witness_records,
};
use crate::core::rg::{
    is_rg_preprocessor_invocation, rg_args_to_strings, run_rg_preprocessor_from_env,
};
use crate::{
    CheckMode, EntryDirectory, EntryId, EntryIdError, SirnoConfig, SirnoFrost, TideSource,
    TideStatus, TideWorkitem, TideWorkitemParseError,
};

#[cfg(test)]
use crate::core::context::entry_query_from_filters;
#[cfg(test)]
use crate::core::dto::{QueryColumn, StructuralFieldState};
#[cfg(test)]
use crate::core::error::OpenTideTutorial;
#[cfg(test)]
use crate::core::output::{
    format_gen_link_report, format_human_table_with_width, format_json, format_query_json,
    format_query_table, format_witness_record, format_witness_records,
};
#[cfg(test)]
use crate::core::rg::rg_args_include_preprocessor;

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
