//! Command-line interface for Sirno.

use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode, ExitStatus};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fmt};

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use indexmap::IndexMap;
use serde::{Deserialize, ser::SerializeMap};
use sirno::{
    CONFIG_FILE_NAME, CheckMode, ConfigError, Entry, EntryArtifactPath, EntryArtifactPathError,
    EntryDirectory, EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryReport,
    EntryDirectoryWritePolicy, EntryId, EntryIdError, EntryMetadata, EntryParseError, EntryQuery,
    Eterator, FrostError, FrostLockStatus, GenLinkDirectoryReport, GeneratedLinkBody,
    GeneratedLinkError, LockError, SirnoConfig, SirnoFrost, SirnoLock, StructuralSettings, Tide,
    TideError, TideWorkitem, TutorialSettings, VagueEntryQuery, WitnessCheckSettings, WitnessError,
    WitnessRecord,
};
use thiserror::Error;

const RG_PREPROCESSOR_ARGV0_PREFIX: &str = "sirno-rg-preprocess-";

/// Sirno command-line entry point.
#[derive(Debug, Parser)]
#[command(name = "sirno")]
#[command(about = "Manage Sirno design entries")]
struct Cli {
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
    /// Reserved top-level move command.
    #[command(visible_alias = "mv")]
    Move {
        /// Reserved arguments.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
        args: Vec<OsString>,
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
    /// Utility commands.
    Util {
        /// Utility command.
        #[command(subcommand)]
        command: UtilCommand,
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
    Move {
        /// New public Markdown entry lake path written to Sirno.toml.
        lake: PathBuf,
    },
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
        mode: Option<CliCheckMode>,
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
        structural: Vec<CliStructuralPredicate>,
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
    Path(CliPathArgs),
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
        /// Exact structural predicate as FIELD=ENTRY_ID.
        #[arg(short = 'x', long, value_name = "FIELD=ENTRY_ID")]
        exact: Vec<CliStructuralPredicate>,
        /// Comma-separated output fields: id, name, path, desc.
        #[arg(short = 'f', long, value_name = "FIELDS")]
        fields: Option<CliQueryFields>,
        /// Output format.
        #[arg(short = 'o', long, value_enum)]
        format: Option<CliQueryOutputFormat>,
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

/// Supported public entry commands.
#[derive(Debug, Subcommand)]
enum EntryCommand {
    /// Create one Markdown entry.
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
        structural: Vec<CliStructuralPredicate>,
        /// Initial Markdown body.
        #[arg(short = 'b', long)]
        body: Option<String>,
    },
    /// Rename one entry id and its Sirno references.
    #[command(visible_aliases = ["mv", "move"])]
    Rename {
        /// Existing entry id.
        old_id: String,
        /// New entry id.
        new_id: String,
    },
    /// Freeze one current Frost entry and make its public file read-only.
    Freeze {
        /// Entry id to freeze.
        id: String,
    },
    /// Melt one public Markdown entry and make its file writable.
    #[command(visible_alias = "unfreeze")]
    Melt {
        /// Entry id to melt.
        id: String,
    },
    /// Show filesystem paths related to one entry.
    Path(CliPathArgs),
    /// Query public Markdown entries.
    #[command(visible_alias = "q")]
    Query {
        /// Vague text terms matched against entries and structural target summaries.
        terms: Vec<String>,
        /// Exact text term matched against id, name, desc, and body.
        #[arg(long = "exact-term")]
        exact_terms: Vec<String>,
        /// Exact structural predicate as FIELD=ENTRY_ID.
        #[arg(short = 'x', long, value_name = "FIELD=ENTRY_ID")]
        exact: Vec<CliStructuralPredicate>,
        /// Comma-separated output fields: id, name, path, desc.
        #[arg(short = 'f', long, value_name = "FIELDS")]
        fields: Option<CliQueryFields>,
        /// Output format.
        #[arg(short = 'o', long, value_enum)]
        format: Option<CliQueryOutputFormat>,
    },
    /// Run ripgrep in the configured public Markdown lake.
    Rg {
        /// Include Sirno-owned generated-footer regions in the search.
        #[arg(long = "with-generated-footer")]
        with_generated_footer: bool,
        /// Arguments forwarded to ripgrep before the lake path.
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<OsString>,
    },
    /// Manage entry-owned artifact files.
    Artifact {
        /// Artifact command.
        #[command(subcommand)]
        command: ArtifactCommand,
    },
    /// Show repository witness blocks for one entry id.
    #[command(visible_aliases = ["w", "wit"])]
    Witness {
        /// Entry id used as the witness query key.
        id: String,
        /// Print full witness regions instead of only their locations.
        #[arg(short = 'f', long)]
        full: bool,
    },
}

impl From<TopLevelEntryCommand> for EntryCommand {
    fn from(value: TopLevelEntryCommand) -> Self {
        match value {
            | TopLevelEntryCommand::New { id, name, desc, structural, body } => {
                Self::New { id, name, desc, structural, body }
            }
            | TopLevelEntryCommand::Freeze { id } => Self::Freeze { id },
            | TopLevelEntryCommand::Melt { id } => Self::Melt { id },
            | TopLevelEntryCommand::Path(args) => Self::Path(args),
            | TopLevelEntryCommand::Query { terms, exact_terms, exact, fields, format } => {
                Self::Query { terms, exact_terms, exact, fields, format }
            }
            | TopLevelEntryCommand::Rg { with_generated_footer, args } => {
                Self::Rg { with_generated_footer, args }
            }
            | TopLevelEntryCommand::Artifact { command } => Self::Artifact { command },
            | TopLevelEntryCommand::Witness { id, full } => Self::Witness { id, full },
        }
    }
}

/// Arguments for entry path lookup.
// sirno:witness:interfaces:begin
#[derive(Clone, Debug, Args)]
struct CliPathArgs {
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
    format: Option<CliPathOutputFormat>,
}
// sirno:witness:interfaces:end

/// CLI path lookup output renderer.
// sirno:witness:interfaces:begin
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliPathOutputFormat {
    /// Print a JSON array of path records.
    Json,
    /// Print an aligned table.
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
enum CliCheckMode {
    /// Editing boundary: dangling references are warnings.
    Edit,
    /// Review boundary: dangling references are errors.
    Review,
}

/// CLI query output renderer.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliQueryOutputFormat {
    /// Print a JSON array of objects.
    Json,
    /// Print an aligned table.
    Human,
}

/// CLI query output field list.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CliQueryFields {
    fields: Vec<CliQueryField>,
}

impl Default for CliQueryFields {
    fn default() -> Self {
        Self { fields: vec![CliQueryField::Id, CliQueryField::Path, CliQueryField::Name] }
    }
}

impl FromStr for CliQueryFields {
    type Err = CliQueryFieldsParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if raw.trim().is_empty() {
            return Err(CliQueryFieldsParseError::Empty);
        }

        let mut fields = Vec::new();
        for raw_field in raw.split(',') {
            let field = raw_field.trim();
            if field.is_empty() {
                return Err(CliQueryFieldsParseError::EmptyField);
            }
            fields.push(field.parse()?);
        }

        Ok(Self { fields })
    }
}

/// One field printable by `sirno query`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CliQueryField {
    /// Entry id.
    Id,
    /// Human-readable entry name.
    Name,
    /// Markdown path.
    Path,
    /// Short entry desc.
    Desc,
}

impl FromStr for CliQueryField {
    type Err = CliQueryFieldsParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            | "id" => Ok(Self::Id),
            | "name" => Ok(Self::Name),
            | "path" => Ok(Self::Path),
            | "desc" => Ok(Self::Desc),
            | field => Err(CliQueryFieldsParseError::UnknownField(field.to_owned())),
        }
    }
}

impl CliQueryField {
    fn label(self) -> &'static str {
        match self {
            | Self::Id => "id",
            | Self::Name => "name",
            | Self::Path => "path",
            | Self::Desc => "desc",
        }
    }
}

/// Error raised while parsing one `--fields` field list.
#[derive(Debug, Error)]
enum CliQueryFieldsParseError {
    /// The list contains no fields.
    #[error("query fields must include at least one field")]
    Empty,
    /// The list contains a separator without a field.
    #[error("query fields contain an empty field")]
    EmptyField,
    /// The list contains an unknown output field.
    #[error("unknown query field `{0}`; expected id, name, path, or desc")]
    UnknownField(String),
}

/// Structural metadata predicate parsed from `FIELD=ENTRY_ID`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CliStructuralPredicate {
    field: String,
    target: EntryId,
}

impl FromStr for CliStructuralPredicate {
    type Err = CliStructuralPredicateParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let Some((field, target)) = raw.split_once('=') else {
            return Err(CliStructuralPredicateParseError::MissingEquals);
        };
        if field.is_empty() {
            return Err(CliStructuralPredicateParseError::EmptyField);
        }
        let target = EntryId::new(target)?;
        Ok(Self { field: field.to_owned(), target })
    }
}

/// Error raised while parsing one structural `FIELD=ENTRY_ID` argument.
#[derive(Debug, Error)]
enum CliStructuralPredicateParseError {
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

/// Tide item selector parsed from one CLI argument.
#[derive(Clone, Debug, PartialEq, Eq)]
enum CliTideItem {
    /// Select every open workitem whose neighbor matches this entry.
    Neighbor(EntryId),
    /// Select one full workitem tuple.
    Workitem(TideWorkitem),
}

impl FromStr for CliTideItem {
    type Err = CliTideItemParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if raw.contains(',') {
            return Ok(Self::Workitem(raw.parse()?));
        }
        Ok(Self::Neighbor(EntryId::new(raw)?))
    }
}

/// Error raised while parsing one tide item selector.
#[derive(Debug, Error)]
enum CliTideItemParseError {
    /// Entry id parsing failed.
    #[error(transparent)]
    EntryId(#[from] EntryIdError),
    /// Full workitem parsing failed.
    #[error(transparent)]
    Workitem(#[from] sirno::TideWorkitemParseError),
}

/// CLI shell target for completion generation.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliCompletionShell {
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

/// Supported utility commands.
#[derive(Debug, Subcommand)]
enum UtilCommand {
    /// Generate a shell completion script.
    Completion {
        /// Shell whose completion script should be generated.
        #[arg(value_enum)]
        shell: CliCompletionShell,
    },
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
    Move {
        /// New Sirno Frost path written to Sirno.toml.
        frost: PathBuf,
    },
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
    /// Check out Frost entries into the public Markdown lake.
    #[command(visible_alias = "defrost")]
    Checkout {
        /// Version coordinate to materialize in the current Frost generation.
        #[arg(required_unless_present = "latest", conflicts_with = "latest")]
        version: Option<u64>,
        /// Check out the latest Frost version as the mutable current lake.
        #[arg(long, conflicts_with = "unsafe_mutable")]
        latest: bool,
        /// Leave an explicit version checkout writable.
        #[arg(long)]
        unsafe_mutable: bool,
    },
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
        format: Option<CliTideOutputFormat>,
    },
    /// Resolve tide workitems.
    Resolve {
        /// Resolve workitems whose neighbor also appears in the current ripple set.
        #[arg(long, conflicts_with_all = ["items", "json"])]
        infer: bool,
        /// JSON array of full workitem tuples.
        #[arg(long, conflicts_with_all = ["infer", "items"])]
        json: Option<String>,
        /// Entry ids or full workitem tuples.
        #[arg(required_unless_present_any = ["infer", "json"])]
        items: Vec<CliTideItem>,
    },
    /// Reopen resolved tide workitems.
    Reopen {
        /// Entry ids or full workitem tuples.
        #[arg(required = true)]
        items: Vec<CliTideItem>,
    },
    /// Clear all tide resolutions from the lock.
    Reset,
}
// sirno:witness:tide:end

/// CLI tide output renderer.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliTideOutputFormat {
    /// Print a JSON object.
    Json,
    /// Print a human-readable list.
    Human,
}

/// Supported rendered-footer commands.
#[derive(Debug, Subcommand)]
enum RenderCommand {
    /// Delete generated Markdown link footers.
    Delete,
}

impl From<CliCheckMode> for CheckMode {
    fn from(value: CliCheckMode) -> Self {
        match value {
            | CliCheckMode::Edit => CheckMode::Edit,
            | CliCheckMode::Review => CheckMode::Review,
        }
    }
}

impl From<CliCompletionShell> for Shell {
    fn from(value: CliCompletionShell) -> Self {
        match value {
            | CliCompletionShell::Bash => Shell::Bash,
            | CliCompletionShell::Elvish => Shell::Elvish,
            | CliCompletionShell::Fish => Shell::Fish,
            | CliCompletionShell::PowerShell => Shell::PowerShell,
            | CliCompletionShell::Zsh => Shell::Zsh,
        }
    }
}

fn main() -> ExitCode {
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
    fn run(self) -> Result<ExitCode, CliError> {
        let config_path = self.config.unwrap_or_else(default_config_path);
        let lake_path = self.lake_path;
        let frost_path = self.frost_path;
        match self.command {
            | Command::TopLevelEntry(command) => {
                if frost_path.is_some() {
                    return Err(CliError::FrostPathRequiresCheck);
                }
                EntryCommand::from(command).run(&config_path, lake_path.as_deref())
            }
            | Command::TopLevelLake(command) => {
                command.run(&config_path, lake_path.as_deref(), frost_path.as_deref())
            }
            | Command::TopLevelFrost(command) => {
                if frost_path.is_some() {
                    return Err(CliError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::Init { mono, lake, frost } => {
                if frost_path.is_some() {
                    return Err(CliError::FrostPathRequiresCheck);
                }
                run_top_level_init(mono, lake, frost, &config_path, lake_path.as_deref())
            }
            | Command::Move { .. } => {
                if frost_path.is_some() {
                    return Err(CliError::FrostPathRequiresCheck);
                }
                Err(CliError::ReservedTopLevelCommand("move"))
            }
            | Command::Entry { command } => {
                if frost_path.is_some() {
                    return Err(CliError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::Lake { command } => {
                command.run(&config_path, lake_path.as_deref(), frost_path.as_deref())
            }
            | Command::Frost { command } => {
                if frost_path.is_some() {
                    return Err(CliError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::Tide { command } => {
                if frost_path.is_some() {
                    return Err(CliError::FrostPathRequiresCheck);
                }
                command.run(&config_path, lake_path.as_deref())
            }
            | Command::Util { command } => {
                if frost_path.is_some() {
                    return Err(CliError::FrostPathRequiresCheck);
                }
                command.run()
            }
        }
    }
}

fn run_top_level_init(
    mono: Option<PathBuf>, lake: Option<PathBuf>, frost: Option<PathBuf>, config_path: &Path,
    lake_path: Option<&Path>,
) -> Result<ExitCode, CliError> {
    run_lake_init(mono, lake, config_path, lake_path)?;
    FrostCommand::Init { frost }.run(config_path, lake_path)
}

fn run_lake_init(
    mono: Option<PathBuf>, lake: Option<PathBuf>, config_path: &Path, lake_path: Option<&Path>,
) -> Result<ExitCode, CliError> {
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
    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CliError> {
        match self {
            | EntryCommand::New { id, name, desc, structural, body } => {
                let (lake, settings) = resolve_lake_directory(lake_path, config_path)?;
                let id = EntryId::new(&id)?;
                let mut metadata =
                    EntryMetadata::new(name.unwrap_or_else(|| title_name_from_id(&id)), desc)?;
                for (field, targets) in
                    structural_targets_by_field(structural, &settings.structural)?
                {
                    metadata.set_structural_targets(field, targets);
                }

                let entry = Entry::new(id, metadata, body.unwrap_or_default());
                let path = EntryDirectory::new(&lake).create_entry(&entry)?;
                println!("created {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
            | EntryCommand::Rename { old_id, new_id } => {
                let (lake, settings) = resolve_lake_directory(lake_path, config_path)?;
                let old_id = EntryId::new(&old_id)?;
                let new_id = EntryId::new(&new_id)?;
                let report =
                    EntryDirectory::new(&lake).rename_entry(&old_id, &new_id, &settings)?;
                let mut changed_paths = report.changed_paths().to_vec();
                if let Some(witness) = &settings.witness {
                    changed_paths.extend(witness.rename_entry_references(&old_id, &new_id)?);
                }
                changed_paths.sort();
                changed_paths.dedup();
                println!("renamed entry {old_id} to {new_id}");
                println!("updated {} paths", changed_paths.len());
                Ok(ExitCode::SUCCESS)
            }
            | EntryCommand::Freeze { id } => {
                let context = FrostContext::load(config_path, lake_path)?;
                context.reject_immutable_checkout()?;
                let id = EntryId::new(&id)?;
                let directory = context.lake();
                let entry = directory.read_entry(&id)?;
                let artifacts = directory.read_entry_artifacts(&id)?;
                let frost = SirnoFrost::open(&context.frost_path)?;
                frost.ensure_entry_bundle_current(&entry, &artifacts)?;
                let path = directory.freeze_entry(&id)?;
                println!("froze entry {id} at {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
            | EntryCommand::Melt { id } => {
                let (lake, _) = resolve_lake_directory(lake_path, config_path)?;
                let id = EntryId::new(&id)?;
                let path = EntryDirectory::new(&lake).melt_entry(&id)?;
                println!("melted entry {id} at {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
            | EntryCommand::Path(args) => {
                let records = entry_path_records(config_path, lake_path, &args)?;
                print_path_records(&records, args.format.unwrap_or(CliPathOutputFormat::Human))?;
                Ok(ExitCode::SUCCESS)
            }
            | EntryCommand::Query { terms, exact_terms, exact, fields, format } => {
                let (lake, mut settings) = resolve_lake_directory(lake_path, config_path)?;
                settings.render = false;
                settings.witness = None;
                let report =
                    EntryDirectory::new(&lake).check_with_settings(CheckMode::Edit, &settings)?;
                if report.has_errors() {
                    print_entry_directory_report(&report);
                    return Ok(ExitCode::FAILURE);
                }

                let vague_query = VagueEntryQuery::new().with_text_terms(terms);
                let exact_query = exact_query_from_predicates(
                    EntryQuery::new().with_text_terms(exact_terms),
                    exact,
                    &settings.structural,
                )?;
                let vague_matches = vague_query.select_entries(report.entries());
                let matches = exact_query.select_entries(vague_matches);
                let fields = fields.unwrap_or_default();
                let format = format.unwrap_or(CliQueryOutputFormat::Json);
                print_query_results(&report, &matches, &fields, format)?;
                Ok(ExitCode::SUCCESS)
            }
            | EntryCommand::Rg { with_generated_footer, args } => {
                run_rg_command(lake_path, config_path, with_generated_footer, args)
            }
            | EntryCommand::Artifact { command } => command.run(config_path, lake_path),
            | EntryCommand::Witness { id, full } => {
                run_witness_command(config_path, lake_path, &id, full)
            }
        }
    }
}

impl LakeCommand {
    fn run(
        self, config_path: &Path, lake_path: Option<&Path>, frost_path: Option<&Path>,
    ) -> Result<ExitCode, CliError> {
        match self {
            | LakeCommand::Init { .. } | LakeCommand::Move { .. } if frost_path.is_some() => {
                Err(CliError::FrostPathRequiresCheck)
            }
            | LakeCommand::Init { lake } => run_lake_init(None, lake, config_path, lake_path),
            | LakeCommand::Move { lake } => {
                let config = SirnoConfig::from_file(config_path)?;
                let old_lake = config.resolve_lake(config_path);
                let config = config.with_lake(lake);
                config.validate_for_file(config_path)?;
                let new_lake = config.resolve_lake(config_path);
                move_configured_path_and_write_config(&old_lake, &new_lake, &config, config_path)?;
                println!("moved lake {} to {}", old_lake.display(), new_lake.display());
                Ok(ExitCode::SUCCESS)
            }
            | LakeCommand::TopLevel(command) => command.run(config_path, lake_path, frost_path),
        }
    }
}

impl TopLevelLakeCommand {
    fn run(
        self, config_path: &Path, lake_path: Option<&Path>, frost_path: Option<&Path>,
    ) -> Result<ExitCode, CliError> {
        match self {
            | TopLevelLakeCommand::Check { mode } => {
                if lake_path.is_some() && frost_path.is_some() {
                    return Err(CliError::LakePathWithFrostPath);
                }
                let mode = mode.unwrap_or(CliCheckMode::Review);
                if lake_path.is_some() {
                    let (lake, settings) = resolve_lake_directory(lake_path, config_path)?;
                    let report =
                        EntryDirectory::new(lake).check_with_settings(mode.into(), &settings)?;
                    print_entry_directory_report(&report);
                    return if report.has_errors() {
                        Ok(ExitCode::FAILURE)
                    } else {
                        Ok(ExitCode::SUCCESS)
                    };
                }

                let Some(frost_path) = frost_path else {
                    let config = SirnoConfig::from_file(config_path)?;
                    let report = EntryDirectory::new(config.resolve_lake(config_path))
                        .check_with_settings(
                            mode.into(),
                            &entry_directory_check_settings(config_path, &config),
                        )?;
                    print_entry_directory_report(&report);
                    return if report.has_errors() {
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
                Err(CliError::FrostPathRequiresCheck)
            }
            | TopLevelLakeCommand::Render { command, dry } => match command {
                | None => {
                    let (lake, mut settings) = resolve_lake_directory(lake_path, config_path)?;
                    settings.render = false;
                    settings.witness = None;

                    let directory = EntryDirectory::new(&lake);
                    let check = directory.check_with_settings(CheckMode::Review, &settings)?;
                    if check.has_errors() {
                        print_entry_directory_report(&check);
                        return Ok(ExitCode::FAILURE);
                    }

                    if dry {
                        let report = directory.check_generated_links_with_ignored_paths(
                            &settings.structural,
                            settings.ignore.clone(),
                        )?;
                        print_gen_link_report(&report);
                        return Ok(ExitCode::SUCCESS);
                    }

                    let report = directory.generate_links_with_ignored_paths(
                        &settings.structural,
                        settings.ignore.clone(),
                    )?;
                    print_gen_link_report(&report);
                    Ok(ExitCode::SUCCESS)
                }
                | Some(RenderCommand::Delete) => {
                    if dry {
                        return Err(CliError::DryWithRenderSubcommand);
                    }
                    let (lake, mut settings) = resolve_lake_directory(lake_path, config_path)?;
                    settings.witness = None;

                    let report = EntryDirectory::new(&lake)
                        .delete_generated_links_with_ignored_paths(settings.ignore)?;
                    print_gen_link_report(&report);
                    Ok(ExitCode::SUCCESS)
                }
            },
            | TopLevelLakeCommand::Status => {
                let config = SirnoConfig::from_file(config_path)?;
                let mono = config.resolve_mono(config_path);
                let frost = config.resolve_frost(config_path);
                let lock_path = SirnoLock::path_for_config(config_path);
                let lock = if frost.is_some() {
                    SirnoLock::from_file_if_exists(&lock_path)?
                } else {
                    None
                };
                let (lake, settings) = resolve_lake_directory(lake_path, config_path)?;
                let report =
                    EntryDirectory::new(&lake).check_with_settings(CheckMode::Review, &settings)?;
                print_status(
                    config_path,
                    mono.as_deref(),
                    frost.as_deref(),
                    lock.as_ref(),
                    &config,
                    &report,
                );
                if report.has_errors() { Ok(ExitCode::FAILURE) } else { Ok(ExitCode::SUCCESS) }
            }
        }
    }
}

impl TopLevelFrostCommand {
    fn run(
        self, config_path: &std::path::Path, lake_path: Option<&Path>,
    ) -> Result<ExitCode, CliError> {
        match self {
            | TopLevelFrostCommand::Commit { unsafe_resolve_all } => {
                let context = FrostContext::load(config_path, lake_path)?;
                context.reject_immutable_checkout()?;
                // sirno:witness:tide:begin
                if !unsafe_resolve_all {
                    let tide_context = TideContext::load(config_path, lake_path)?;
                    let lock = tide_context.load_lock_or_current()?;
                    let tide = tide_context.tide(&lock)?;
                    if !tide.is_clear() {
                        return Err(CliError::OpenTide {
                            count: tide.open_statuses().count(),
                            tutorial: OpenTideTutorial::new(
                                context.tutorial,
                                lock.frost.version == Eterator::EMPTY.version(),
                            ),
                        });
                    }
                }
                // sirno:witness:tide:end
                let mut frost = SirnoFrost::open(&context.frost_path)?;
                let version =
                    frost.commit_entry_directory(&context.lake_path, &context.settings)?;
                context.lake().set_writable(&context.settings)?;
                let mut lock = SirnoLock::current(version);
                lock.tide.clear();
                lock.write(&context.lock_path)?;
                println!(
                    "froze version {} from {}",
                    version.version(),
                    context.lake_path.display()
                );
                Ok(ExitCode::SUCCESS)
            }
            | TopLevelFrostCommand::Checkout { version, latest, unsafe_mutable } => {
                let context = FrostContext::load(config_path, lake_path)?;
                let frost = SirnoFrost::open(&context.frost_path)?;
                let snapshot = if latest {
                    frost.current_snapshot()?
                } else {
                    frost.snapshot_for_version(frost_version(
                        version.expect("clap requires VERSION unless --latest is present"),
                    )?)?
                };
                if snapshot.version() == Eterator::EMPTY.version() {
                    return Err(CliError::InvalidFrostVersion(snapshot.version()));
                }
                let paths = frost.checkout_entry_directory(
                    snapshot,
                    &context.lake_path,
                    EntryDirectoryWritePolicy::ReplaceDirectory {
                        ignore: context.settings.ignore.clone(),
                    },
                )?;
                if latest || unsafe_mutable {
                    context.lake().set_writable(&context.settings)?;
                } else {
                    context.lake().add_readonly_checkout_warnings(&paths)?;
                    context.lake().set_readonly(&context.settings)?;
                }
                if latest {
                    SirnoLock::current(snapshot).write(&context.lock_path)?;
                } else {
                    SirnoLock::checked_out(snapshot, unsafe_mutable).write(&context.lock_path)?;
                }
                println!(
                    "checked out {}frost version {} into {} ({} entries, {})",
                    if latest { "latest " } else { "" },
                    snapshot.version(),
                    context.lake_path.display(),
                    paths.len(),
                    if latest {
                        "mutable"
                    } else if unsafe_mutable {
                        "unsafe mutable"
                    } else {
                        "immutable"
                    }
                );
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

impl FrostCommand {
    fn run(
        self, config_path: &std::path::Path, lake_path: Option<&Path>,
    ) -> Result<ExitCode, CliError> {
        match self {
            | FrostCommand::Init { frost } => {
                let config = SirnoConfig::from_file(config_path)?;
                let existing_frost = config.frost.as_ref().map(|settings| settings.path.clone());
                let frost = frost
                    .or_else(|| existing_frost.clone())
                    .unwrap_or_else(|| default_frost_path(config_path));
                if let Some(existing_frost) = existing_frost
                    && existing_frost != frost
                {
                    return Err(CliError::FrostAlreadyConfigured(existing_frost));
                }

                let needs_config_write = config.frost.is_none();
                let config = if needs_config_write { config.with_frost(frost) } else { config };
                config.validate_for_file(config_path)?;

                let frost_path =
                    config.resolve_frost(config_path).expect("frost path configured by init");
                let frost = SirnoFrost::open(&frost_path)?;
                let version = frost.current_snapshot()?;
                if needs_config_write {
                    config.write(config_path)?;
                }
                SirnoLock::current(version).write(SirnoLock::path_for_config(config_path))?;
                println!(
                    "initialized frost {} at version {}",
                    frost_path.display(),
                    version.version(),
                );
                Ok(ExitCode::SUCCESS)
            }
            | FrostCommand::Move { frost } => {
                let config = SirnoConfig::from_file(config_path)?;
                let Some(old_frost) = config.resolve_frost(config_path) else {
                    return Err(CliError::FrostNotConfigured);
                };
                let config = config.with_frost(frost);
                config.validate_for_file(config_path)?;
                let new_frost =
                    config.resolve_frost(config_path).expect("frost path configured by move");
                move_configured_path_and_write_config(
                    &old_frost,
                    &new_frost,
                    &config,
                    config_path,
                )?;
                println!("moved frost {} to {}", old_frost.display(), new_frost.display());
                Ok(ExitCode::SUCCESS)
            }
            | FrostCommand::Snapshot(command) => command.run(config_path, lake_path),
        }
    }
}

impl TideCommand {
    fn run(
        self, config_path: &std::path::Path, lake_path: Option<&Path>,
    ) -> Result<ExitCode, CliError> {
        match self {
            | TideCommand::Status { all, format } => {
                let context = TideContext::load(config_path, lake_path)?;
                let lock = context.load_lock_or_current()?;
                let tide = context.tide(&lock)?;
                let format = format.unwrap_or(CliTideOutputFormat::Human);
                print_tide_status(&tide, all, format)?;
                Ok(if tide.is_clear() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
            }
            | TideCommand::Resolve { infer, json, items } => {
                let context = TideContext::load(config_path, lake_path)?;
                let mut lock = context.load_lock_or_current()?;
                let tide = context.tide(&lock)?;
                let (resolutions, count) = if infer {
                    tide.resolve_where(|status| {
                        tide.ripple_ids().contains(&status.workitem.neighbor)
                    })
                } else if let Some(json) = json {
                    let workitems = tide_workitems_from_json(&json)?;
                    tide.resolve_where(|status| workitems.contains(&status.workitem))
                } else {
                    tide.resolve_where(|status| tide_item_matches(&items, status))
                };
                lock.tide.set_resolved(resolutions);
                lock.write(&context.lock_path)?;
                println!("resolved {count} tide workitems");
                Ok(ExitCode::SUCCESS)
            }
            | TideCommand::Reopen { items } => {
                let context = TideContext::load(config_path, lake_path)?;
                let mut lock = context.load_lock_or_current()?;
                let tide = context.tide(&lock)?;
                let (resolutions, count) =
                    tide.reopen_where(|status| tide_item_matches(&items, status));
                lock.tide.set_resolved(resolutions);
                lock.write(&context.lock_path)?;
                println!("reopened {count} tide workitems");
                Ok(ExitCode::SUCCESS)
            }
            | TideCommand::Reset => {
                let context = TideContext::load(config_path, lake_path)?;
                let mut lock = context.load_lock_or_current()?;
                let count = lock.tide.resolved.len();
                lock.tide.clear();
                lock.write(&context.lock_path)?;
                println!("cleared {count} tide resolutions");
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CliTideJsonWorkitems {
    One(TideWorkitem),
    Many(Vec<TideWorkitem>),
}

fn tide_workitems_from_json(source: &str) -> Result<Vec<TideWorkitem>, CliError> {
    Ok(match serde_json::from_str::<CliTideJsonWorkitems>(source)? {
        | CliTideJsonWorkitems::One(workitem) => vec![workitem],
        | CliTideJsonWorkitems::Many(workitems) => workitems,
    })
}

fn tide_item_matches(items: &[CliTideItem], status: &sirno::TideStatus) -> bool {
    items.iter().any(|item| match item {
        | CliTideItem::Neighbor(id) => &status.workitem.neighbor == id,
        | CliTideItem::Workitem(workitem) => &status.workitem == workitem,
    })
}

fn print_tide_status(tide: &Tide, all: bool, format: CliTideOutputFormat) -> Result<(), CliError> {
    let statuses =
        tide.statuses().iter().filter(|status| all || !status.resolved).collect::<Vec<_>>();
    match format {
        | CliTideOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&statuses)?);
        }
        | CliTideOutputFormat::Human => {
            if statuses.is_empty() {
                println!("tide: clear");
            } else {
                for status in statuses {
                    let state = if status.resolved { "resolved" } else { "open" };
                    let sources = status
                        .sources
                        .iter()
                        .map(|source| match source {
                            | sirno::TideSource::Lake => "lake",
                            | sirno::TideSource::Frost => "frost",
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
    fn run(self, config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CliError> {
        let (lake, _) = resolve_lake_directory(lake_path, config_path)?;
        let directory = EntryDirectory::new(&lake);
        match self {
            | ArtifactCommand::List { id } => {
                let id = EntryId::new(&id)?;
                directory.read_entry(&id)?;
                for artifact in directory.read_entry_artifacts(&id)? {
                    println!("{}", artifact.path);
                }
                Ok(ExitCode::SUCCESS)
            }
            | ArtifactCommand::Add { id, source, artifact_path } => {
                let id = EntryId::new(&id)?;
                let artifact_path = match artifact_path {
                    | Some(path) => artifact_path_from_cli(&path)?,
                    | None => default_artifact_path_from_source(&source)?,
                };
                let path = directory.add_entry_artifact(&id, &source, &artifact_path)?;
                println!("added artifact {artifact_path} at {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
            | ArtifactCommand::Rename { id, old_path, new_path } => {
                let id = EntryId::new(&id)?;
                let old_path = artifact_path_from_cli(&old_path)?;
                let new_path = artifact_path_from_cli(&new_path)?;
                let path = directory.rename_entry_artifact(&id, &old_path, &new_path)?;
                println!("renamed artifact {old_path} to {new_path} at {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
            | ArtifactCommand::Remove { id, artifact_path } => {
                let id = EntryId::new(&id)?;
                let artifact_path = artifact_path_from_cli(&artifact_path)?;
                let path = directory.remove_entry_artifact(&id, &artifact_path)?;
                println!("removed artifact {artifact_path} at {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

impl UtilCommand {
    fn run(self) -> Result<ExitCode, CliError> {
        match self {
            | UtilCommand::Completion { shell } => {
                let shell = Shell::from(shell);
                let mut command = Cli::command();
                let mut stdout = std::io::stdout();
                generate(shell, &mut command, "sirno", &mut stdout);
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

fn move_configured_path_and_write_config(
    source: &Path, destination: &Path, config: &SirnoConfig, config_path: &Path,
) -> Result<(), CliError> {
    let moved = move_configured_path(source, destination)?;
    if let Err(config_error) = config.write(config_path) {
        if moved && let Err(rollback) = fs::rename(destination, source) {
            return Err(CliError::MoveConfigWriteRollback {
                source_path: source.to_path_buf(),
                destination_path: destination.to_path_buf(),
                source: Box::new(config_error),
                rollback,
            });
        }
        return Err(CliError::Config(config_error));
    }
    Ok(())
}

fn move_configured_path(source: &Path, destination: &Path) -> Result<bool, CliError> {
    if source == destination {
        return Ok(false);
    }
    match fs::symlink_metadata(destination) {
        | Ok(_) => return Err(CliError::MoveDestinationExists(destination.to_path_buf())),
        | Err(source) if source.kind() == ErrorKind::NotFound => {}
        | Err(source) => {
            return Err(CliError::ReadMoveDestination { path: destination.to_path_buf(), source });
        }
    }
    fs::rename(source, destination).map_err(|error| CliError::MovePath {
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
    fn load(config_path: &Path, lake_path: Option<&Path>) -> Result<Self, CliError> {
        let config = SirnoConfig::from_file(config_path)?;
        let Some(frost_path) = config.resolve_frost(config_path) else {
            return Err(CliError::FrostNotConfigured);
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

    fn reject_immutable_checkout(&self) -> Result<(), CliError> {
        let Some(lock) = SirnoLock::from_file_if_exists(&self.lock_path)? else {
            return Ok(());
        };
        if lock.frost.is_checked_out() && !lock.frost.is_unsafe_mutable_checkout() {
            return Err(CliError::ImmutableFrostCheckout(lock.frost.version));
        }
        Ok(())
    }
}

impl TideContext {
    fn load(config_path: &Path, lake_path: Option<&Path>) -> Result<Self, CliError> {
        let config = SirnoConfig::from_file(config_path)?;
        let Some(frost_path) = config.resolve_frost(config_path) else {
            return Err(CliError::FrostNotConfigured);
        };
        Ok(Self {
            frost_path,
            lock_path: SirnoLock::path_for_config(config_path),
            settings: entry_directory_check_settings(config_path, &config),
            lake_path: resolve_lake_path(lake_path, config_path, &config),
        })
    }

    fn load_lock_or_current(&self) -> Result<SirnoLock, CliError> {
        let Some(lock) = SirnoLock::from_file_if_exists(&self.lock_path)? else {
            let frost = SirnoFrost::open(&self.frost_path)?;
            return Ok(SirnoLock::current(frost.current_snapshot()?));
        };
        Ok(lock)
    }

    fn tide(&self, lock: &SirnoLock) -> Result<Tide, CliError> {
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

fn frost_version(version: u64) -> Result<Eterator, CliError> {
    if version == Eterator::EMPTY.version() {
        return Err(CliError::InvalidFrostVersion(version));
    }
    Ok(Eterator(version))
}

fn run_witness_command(
    config_path: &Path, lake_path: Option<&Path>, raw_id: &str, full: bool,
) -> Result<ExitCode, CliError> {
    let config = SirnoConfig::from_file(config_path)?;
    let id = EntryId::new(raw_id)?;
    let lake = resolve_lake_path(lake_path, config_path, &config);
    if !EntryDirectory::new(&lake).entry_exists(&id)? {
        return Err(CliError::MissingWitnessEntry(id));
    }
    let Some(settings) = witness_check_settings(config_path, &config) else {
        return Err(CliError::RepoMembersNotConfigured);
    };
    let index = settings.scan()?;
    let records = index.records_for(&id);
    if records.is_empty() {
        println!("no witness found for {id}");
        return Ok(ExitCode::FAILURE);
    }
    print_witness_records(records, full);
    Ok(ExitCode::SUCCESS)
}

fn print_witness_records(records: &[WitnessRecord], full: bool) {
    print!("{}", format_witness_records(records, full));
}

fn run_rg_command(
    lake_path: Option<&Path>, config_path: &Path, with_generated_footer: bool, args: Vec<OsString>,
) -> Result<ExitCode, CliError> {
    if !with_generated_footer && rg_args_include_preprocessor(&args) {
        return Err(CliError::RgPreprocessorConflict);
    }

    let lake = resolve_lake_path_for_rg(lake_path, config_path)?;
    let preprocessor =
        if with_generated_footer { None } else { Some(RgPreprocessorLink::create()?) };

    let mut command = ProcessCommand::new("rg");
    if let Some(preprocessor) = &preprocessor {
        command.arg("--pre").arg(preprocessor.path()).arg("--pre-glob").arg("*.md");
    }
    let status = command.args(args).arg(lake).status().map_err(CliError::RunRg)?;
    Ok(exit_code_from_status(status))
}

fn rg_args_include_preprocessor(args: &[OsString]) -> bool {
    args.iter()
        .filter_map(|arg| arg.to_str())
        .any(|arg| arg == "--pre" || arg.starts_with("--pre="))
}

fn resolve_lake_path_for_rg(
    lake_path: Option<&Path>, config_path: &Path,
) -> Result<PathBuf, CliError> {
    if let Some(lake_path) = lake_path {
        return Ok(lake_path.to_path_buf());
    }

    let config = SirnoConfig::from_file(config_path)?;
    Ok(config.resolve_lake(config_path))
}

fn exit_code_from_status(status: ExitStatus) -> ExitCode {
    if let Some(code) = status.code().and_then(|code| u8::try_from(code).ok()) {
        return ExitCode::from(code);
    }

    ExitCode::FAILURE
}

fn is_rg_preprocessor_invocation() -> bool {
    env::args_os()
        .next()
        .and_then(|arg| PathBuf::from(arg).file_name().map(|name| name.to_os_string()))
        .is_some_and(|name| name.to_string_lossy().starts_with(RG_PREPROCESSOR_ARGV0_PREFIX))
}

fn run_rg_preprocessor_from_env() -> Result<ExitCode, CliError> {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        return Err(CliError::RgPreprocessorArgumentCount);
    };
    if args.next().is_some() {
        return Err(CliError::RgPreprocessorArgumentCount);
    }

    run_rg_preprocessor(&PathBuf::from(path))
}

fn run_rg_preprocessor(path: &Path) -> Result<ExitCode, CliError> {
    let body = fs::read_to_string(path)
        .map_err(|source| CliError::ReadRgPreprocessorInput { path: path.to_path_buf(), source })?;
    let masked = GeneratedLinkBody::new(&body).mask()?;
    io::stdout().write_all(masked.as_bytes()).map_err(CliError::WriteRgPreprocessorOutput)?;
    Ok(ExitCode::SUCCESS)
}

#[derive(Debug)]
struct RgPreprocessorLink {
    path: PathBuf,
}

impl RgPreprocessorLink {
    fn create() -> Result<Self, CliError> {
        let current_exe = env::current_exe().map_err(CliError::LocateCurrentExe)?;
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
            CliError::CreateRgPreprocessorInvoker { path: path.clone(), source }
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

fn artifact_path_from_cli(path: &Path) -> Result<EntryArtifactPath, CliError> {
    Ok(EntryArtifactPath::new(path)?)
}

fn default_artifact_path_from_source(source: &Path) -> Result<EntryArtifactPath, CliError> {
    let Some(file_name) = source.file_name() else {
        return Err(CliError::ArtifactSourceHasNoFileName(source.to_path_buf()));
    };
    Ok(EntryArtifactPath::new(Path::new(file_name))?)
}

fn explicit_lake_check_settings(
    config_path: &std::path::Path,
) -> Result<EntryDirectoryCheckSettings, CliError> {
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
) -> Result<(PathBuf, EntryDirectoryCheckSettings), CliError> {
    if let Some(lake_path) = lake_path {
        return Ok((lake_path.to_path_buf(), explicit_lake_check_settings(config_path)?));
    }

    let config = SirnoConfig::from_file(config_path)?;
    Ok((config.resolve_lake(config_path), entry_directory_check_settings(config_path, &config)))
}

fn exact_query_from_predicates(
    mut query: EntryQuery, predicates: Vec<CliStructuralPredicate>, structural: &StructuralSettings,
) -> Result<EntryQuery, CliError> {
    for (field, targets) in structural_targets_by_field(predicates, structural)? {
        query = query.with_structural_targets(field, targets);
    }
    Ok(query)
}

fn structural_targets_by_field(
    predicates: Vec<CliStructuralPredicate>, structural: &StructuralSettings,
) -> Result<IndexMap<String, Vec<EntryId>>, CliError> {
    let mut targets_by_field = IndexMap::<String, Vec<EntryId>>::new();
    for predicate in predicates {
        if !structural.contains_field(&predicate.field) {
            return Err(CliError::UnconfiguredStructuralField(predicate.field));
        }
        targets_by_field.entry(predicate.field).or_default().push(predicate.target);
    }
    Ok(targets_by_field)
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

fn print_status(
    config_path: &std::path::Path, mono: Option<&std::path::Path>, frost: Option<&std::path::Path>,
    lock: Option<&SirnoLock>, config: &SirnoConfig, report: &EntryDirectoryReport,
) {
    println!("config: {}", config_path.display());
    if let Some(mono) = mono {
        println!("mono: {}", mono.display());
    } else {
        println!("mono: (not configured)");
    }
    println!("lake: {}", report.root().display());
    if let Some(frost) = frost {
        println!("frost: {}", frost.display());
        println!("frost-state: {}", frost_state_label(lock));
    } else {
        println!("frost: (not configured)");
    }
    println!("entries: {}", report.entries().len());
    println!("checks:");
    println!("  render: {}", config.check.render);
    println!("structural:");
    for (field, settings) in config.structural.fields() {
        println!("  {field}.to: {}", settings.to);
        println!("  {field}.from: {}", settings.from);
        println!("  {field}.clique: {}", settings.clique);
    }
    if report.has_errors() {
        println!("check: failed");
        print_entry_directory_report(report);
    } else {
        println!("check: ok");
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

fn print_gen_link_report(report: &GenLinkDirectoryReport) {
    println!(
        "{}",
        format_gen_link_report(report.root(), report.entry_count(), report.changed_paths())
    );
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
    report: &EntryDirectoryReport, entries: &[&Entry], fields: &CliQueryFields,
    format: CliQueryOutputFormat,
) -> Result<(), CliError> {
    let rows = query_result_rows(report, entries, fields)?;
    match format {
        | CliQueryOutputFormat::Json => {
            println!("{}", format_query_json(fields, &rows)?);
        }
        | CliQueryOutputFormat::Human => {
            print!("{}", format_query_table(fields, &rows));
        }
    }
    Ok(())
}

fn entry_path_records(
    config_path: &Path, lake_path: Option<&Path>, args: &CliPathArgs,
) -> Result<Vec<CliPathRecord>, CliError> {
    let config = SirnoConfig::from_file(config_path)?;
    let lake = resolve_lake_path(lake_path, config_path, &config);
    let directory = EntryDirectory::new(&lake);
    let id = EntryId::new(&args.id)?;
    directory.read_entry(&id)?;
    let artifacts = directory.read_entry_artifacts(&id)?;
    let selection = CliPathSelection::from_args(args);
    let mut records = Vec::new();

    if selection.entry {
        records.push(CliPathRecord::new(
            "entry",
            output_path(directory.entry_path(&id), args.absolute)?,
        ));
    }
    if selection.artifact {
        records.push(CliPathRecord::new(
            "artifact-root",
            output_path(directory.entry_artifact_root_path(&id), args.absolute)?,
        ));
        for artifact in &artifacts {
            records.push(CliPathRecord::new(
                "artifact",
                output_path(directory.entry_artifact_path(&id, &artifact.path), args.absolute)?,
            ));
        }
    }
    if selection.frost
        && let Some(frost) = config.resolve_frost(config_path)
    {
        records.push(CliPathRecord::new(
            "frost-entry",
            output_path(SirnoFrost::entry_storage_path(&frost, &id)?, args.absolute)?,
        ));
        for artifact in &artifacts {
            records.push(CliPathRecord::new(
                "frost-artifact",
                output_path(
                    SirnoFrost::artifact_storage_path(&frost, &id, &artifact.path)?,
                    args.absolute,
                )?,
            ));
        }
    }

    Ok(records)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CliPathSelection {
    entry: bool,
    artifact: bool,
    frost: bool,
}

impl CliPathSelection {
    fn from_args(args: &CliPathArgs) -> Self {
        let all = !args.show_entry && !args.show_artifact && !args.show_frost;
        Self {
            entry: all || args.show_entry,
            artifact: all || args.show_artifact,
            frost: all || args.show_frost,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct CliPathRecord {
    kind: &'static str,
    path: String,
}

impl CliPathRecord {
    fn new(kind: &'static str, path: PathBuf) -> Self {
        Self { kind, path: path.display().to_string() }
    }
}

fn output_path(path: PathBuf, absolute: bool) -> Result<PathBuf, CliError> {
    if !absolute || path.is_absolute() {
        return Ok(path);
    }
    Ok(env::current_dir().map_err(CliError::CurrentDirectory)?.join(path))
}

fn print_path_records(
    records: &[CliPathRecord], format: CliPathOutputFormat,
) -> Result<(), CliError> {
    match format {
        | CliPathOutputFormat::Json => println!("{}", serde_json::to_string_pretty(records)?),
        | CliPathOutputFormat::Human => print!("{}", format_path_table(records)),
        | CliPathOutputFormat::Paths => {
            for record in records {
                println!("{}", record.path);
            }
        }
    }
    Ok(())
}

fn format_path_table(records: &[CliPathRecord]) -> String {
    let headers = ["kind", "path"];
    let mut widths = headers.iter().map(|header| cell_width(header)).collect::<Vec<_>>();
    for record in records {
        widths[0] = widths[0].max(cell_width(record.kind));
        widths[1] = widths[1].max(cell_width(&record.path));
    }

    let mut table = String::new();
    push_query_table_row(&mut table, headers, &widths);
    push_query_table_separator(&mut table, &widths);
    for record in records {
        push_query_table_row(&mut table, [record.kind, record.path.as_str()], &widths);
    }
    table
}

fn query_result_rows(
    report: &EntryDirectoryReport, entries: &[&Entry], fields: &CliQueryFields,
) -> Result<Vec<Vec<String>>, CliError> {
    entries
        .iter()
        .map(|entry| {
            fields
                .fields
                .iter()
                .map(|field| format_query_field(report, entry, *field))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect()
}

fn format_query_field(
    report: &EntryDirectoryReport, entry: &Entry, field: CliQueryField,
) -> Result<String, CliError> {
    match field {
        | CliQueryField::Id => Ok(entry.id.to_string()),
        | CliQueryField::Name => Ok(entry.metadata.name.clone()),
        | CliQueryField::Path => {
            let path = report
                .entry_path(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
            Ok(path.display().to_string())
        }
        | CliQueryField::Desc => Ok(entry.metadata.desc.clone()),
    }
}

fn format_query_json(fields: &CliQueryFields, rows: &[Vec<String>]) -> Result<String, CliError> {
    let records = rows.iter().map(|row| QueryJsonRecord { fields, row }).collect::<Vec<_>>();
    Ok(serde_json::to_string_pretty(&records)?)
}

struct QueryJsonRecord<'a> {
    fields: &'a CliQueryFields,
    row: &'a [String],
}

impl serde::Serialize for QueryJsonRecord<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.fields.fields.len()))?;
        for (field, value) in self.fields.fields.iter().zip(self.row) {
            map.serialize_entry(field.label(), value)?;
        }
        map.end()
    }
}

fn format_query_table(fields: &CliQueryFields, rows: &[Vec<String>]) -> String {
    let headers = fields.fields.iter().map(|field| field.label()).collect::<Vec<_>>();
    let mut widths = headers.iter().map(|header| cell_width(header)).collect::<Vec<_>>();
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell_width(cell));
        }
    }

    let mut table = String::new();
    push_query_table_row(&mut table, headers.iter().copied(), &widths);
    push_query_table_separator(&mut table, &widths);
    for row in rows {
        push_query_table_row(&mut table, row.iter().map(String::as_str), &widths);
    }
    table
}

fn push_query_table_row<'a>(
    table: &mut String, cells: impl IntoIterator<Item = &'a str>, widths: &[usize],
) {
    table.push('|');
    for (cell, width) in cells.into_iter().zip(widths) {
        table.push(' ');
        table.push_str(cell);
        table.push_str(&" ".repeat(width.saturating_sub(cell_width(cell))));
        table.push_str(" |");
    }
    table.push('\n');
}

fn push_query_table_separator(table: &mut String, widths: &[usize]) {
    table.push('|');
    for width in widths {
        table.push(' ');
        table.push_str(&"-".repeat(*width));
        table.push_str(" |");
    }
    table.push('\n');
}

fn cell_width(cell: &str) -> usize {
    cell.chars().count()
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
struct OpenTideTutorial {
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
        writeln!(formatter, "Resolve reviewed work with `sirno tide resolve ...`,",)?;
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
enum CliError {
    /// Sirno Frost has already been configured at another path.
    #[error("frost is already configured at {0}")]
    FrostAlreadyConfigured(PathBuf),
    /// Sirno Frost is required for a frost command but is not configured.
    #[error("frost is not configured; run `sirno frost init` first")]
    FrostNotConfigured,
    /// Immutable Frost checkouts cannot be committed.
    #[error("frost version {0} is checked out immutably; use checkout --unsafe-mutable first")]
    ImmutableFrostCheckout(u64),
    /// A top-level command name is intentionally reserved.
    #[error("top-level `sirno {0}` is reserved")]
    ReservedTopLevelCommand(&'static str),
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

    use crate::OpenTideTutorial;

    use sirno::{
        CONFIG_FILE_NAME, Entry, EntryId, EntryMetadata, EntryQuery, Eterator, FrostError,
        FrostLockStatus, FrostSettings, LOCK_FILE_NAME, RepoMember, RepoSettings, SirnoConfig,
        SirnoFrost, SirnoLock, StructuralEdgeSettings, StructuralFieldSettings,
        StructuralRippleSettings, StructuralSettings, TutorialSettings, WitnessRecord, WitnessSpan,
    };

    use crate::{
        ArtifactCommand, Cli, CliCheckMode, CliError, CliPathArgs, CliPathOutputFormat,
        CliQueryField, CliQueryFields, CliQueryOutputFormat, CliStructuralPredicate, CliTideItem,
        Command, EntryCommand, FrostCommand, LakeCommand, TideCommand, TopLevelEntryCommand,
        TopLevelFrostCommand, TopLevelLakeCommand, entry_path_records, exact_query_from_predicates,
        format_gen_link_report, format_path_table, format_query_json, format_query_table,
        format_witness_record, format_witness_records, rg_args_include_preprocessor,
    };

    fn assert_before(source: &str, before: &str, after: &str) {
        assert!(source.find(before).unwrap() < source.find(after).unwrap());
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
    fn top_level_help_orders_entry_lake_then_frost_commands() {
        let help = Cli::command().render_help().to_string();

        assert_before(&help, "  new", "  check");
        assert_before(&help, "  status", "  init");
        assert_before(&help, "  checkout", "  entry");
        assert_before(&help, "  entry", "  lake");
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
    fn frost_checkout_accepts_top_level_form_and_alias() {
        let checkout = Cli::parse_from(["sirno", "checkout", "--latest"]);
        let defrost = Cli::parse_from(["sirno", "defrost", "--latest"]);

        assert!(matches!(
            checkout.command,
            Command::TopLevelFrost(TopLevelFrostCommand::Checkout {
                version: None,
                latest: true,
                unsafe_mutable: false,
            })
        ));
        assert!(matches!(
            defrost.command,
            Command::TopLevelFrost(TopLevelFrostCommand::Checkout {
                version: None,
                latest: true,
                unsafe_mutable: false,
            })
        ));
    }

    #[test]
    fn frost_init_rejects_global_frost_path() {
        let error = Cli::parse_from(["sirno", "frost", "init", "--frost-path", "sirno-frost"])
            .run()
            .unwrap_err();

        assert!(matches!(error, CliError::FrostPathRequiresCheck));
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
        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "frost",
            "checkout",
            "1",
        ])
        .run()
        .unwrap();
        assert!(fs::metadata(docs.join("alpha.md")).unwrap().permissions().readonly());

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "frost",
            "checkout",
            "--latest",
        ])
        .run()
        .unwrap();

        let lock = SirnoLock::from_file(temp.path().join(LOCK_FILE_NAME)).unwrap();
        let source = fs::read_to_string(docs.join("alpha.md")).unwrap();
        assert_eq!(lock.frost.status, FrostLockStatus::Current);
        assert_eq!(lock.frost.version, 1);
        assert!(!lock.frost.mutable);
        assert!(!source.contains("read-only Sirno Frost checkout"));
        assert!(!fs::metadata(&docs).unwrap().permissions().readonly());
        assert!(!fs::metadata(docs.join("alpha.md")).unwrap().permissions().readonly());
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
            CliError::OpenTide { count, tutorial }
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

        assert!(matches!(&error, CliError::OpenTide { count, .. } if *count == 1));
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
    fn move_is_reserved_at_top_level() {
        let cli = Cli::parse_from(["sirno", "move", "sirno-docs"]);

        assert!(matches!(cli.command, Command::Move { .. }));
        assert!(matches!(cli.run(), Err(CliError::ReservedTopLevelCommand("move"))));
    }

    #[test]
    fn mv_alias_is_reserved_at_top_level() {
        let cli = Cli::parse_from(["sirno", "mv", "sirno-docs"]);

        assert!(matches!(cli.command, Command::Move { .. }));
    }

    #[test]
    fn lake_move_accepts_mv_alias() {
        let cli = Cli::parse_from(["sirno", "lake", "mv", "sirno-docs"]);

        assert!(matches!(
            cli.command,
            Command::Lake { command: LakeCommand::Move { lake } }
                if lake == Path::new("sirno-docs")
        ));
    }

    #[test]
    fn frost_move_accepts_frost_path() {
        let cli = Cli::parse_from(["sirno", "frost", "move", "sirno-frost-2"]);

        assert!(matches!(
            cli.command,
            Command::Frost { command: FrostCommand::Move { frost } }
                if frost == Path::new("sirno-frost-2")
        ));
    }

    #[test]
    fn frost_mv_alias_accepts_frost_path() {
        let cli = Cli::parse_from(["sirno", "frost", "mv", "sirno-frost-2"]);

        assert!(matches!(
            cli.command,
            Command::Frost { command: FrostCommand::Move { frost } }
                if frost == Path::new("sirno-frost-2")
        ));
    }

    #[test]
    fn frost_checkout_accepts_unsafe_mutable_flag() {
        let cli = Cli::parse_from(["sirno", "frost", "checkout", "3", "--unsafe-mutable"]);

        assert!(matches!(
            cli.command,
            Command::Frost {
                command: FrostCommand::Snapshot(TopLevelFrostCommand::Checkout {
                    version: Some(3),
                    latest: false,
                    unsafe_mutable: true
                })
            }
        ));
    }

    #[test]
    fn frost_checkout_accepts_latest_flag() {
        let cli = Cli::parse_from(["sirno", "frost", "checkout", "--latest"]);

        assert!(matches!(
            cli.command,
            Command::Frost {
                command: FrostCommand::Snapshot(TopLevelFrostCommand::Checkout {
                    version: None,
                    latest: true,
                    unsafe_mutable: false
                })
            }
        ));
    }

    #[test]
    fn frost_defrost_alias_accepts_latest_flag() {
        let cli = Cli::parse_from(["sirno", "frost", "defrost", "--latest"]);

        assert!(matches!(
            cli.command,
            Command::Frost {
                command: FrostCommand::Snapshot(TopLevelFrostCommand::Checkout {
                    version: None,
                    latest: true,
                    unsafe_mutable: false
                })
            }
        ));
    }

    #[test]
    fn frost_checkout_rejects_latest_with_version() {
        let error =
            Cli::try_parse_from(["sirno", "frost", "checkout", "3", "--latest"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn tide_resolve_accepts_neighbor_and_tuple_selectors() {
        let neighbor = Cli::parse_from(["sirno", "tide", "resolve", "beta"]);
        let tuple = Cli::parse_from(["sirno", "tide", "resolve", "alpha,belongs,to,beta"]);

        assert!(matches!(
            neighbor.command,
            Command::Tide {
                command: TideCommand::Resolve {
                    items,
                    infer: false,
                    json: None
                }
            } if items == vec![CliTideItem::Neighbor(EntryId::new("beta").unwrap())]
        ));
        assert!(matches!(
            tuple.command,
            Command::Tide {
                command: TideCommand::Resolve {
                    items,
                    infer: false,
                    json: None
                }
            } if matches!(&items[..], [CliTideItem::Workitem(workitem)]
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
            Command::Tide { command: TideCommand::Resolve { infer: true, .. } }
        ));
        assert!(matches!(
            json.command,
            Command::Tide { command: TideCommand::Resolve { json: Some(_), infer: false, .. } }
        ));
    }

    #[test]
    fn tide_resolve_requires_selector_json_or_infer() {
        let error = Cli::try_parse_from(["sirno", "tide", "resolve"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::MissingRequiredArgument);
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
                    CliStructuralPredicate {
                        field: "topic".to_owned(),
                        target: EntryId::new("concept").unwrap(),
                    },
                    CliStructuralPredicate {
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
            Command::Entry { command: EntryCommand::Rename { old_id, new_id } }
                if old_id == "old-entry" && new_id == "new-entry"
        ));
        assert!(matches!(
            short.command,
            Command::Entry { command: EntryCommand::Rename { old_id, new_id } }
                if old_id == "old-entry" && new_id == "new-entry"
        ));
        assert!(matches!(
            mnemonic.command,
            Command::Entry { command: EntryCommand::Rename { old_id, new_id } }
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
            Command::TopLevelEntry(TopLevelEntryCommand::Path(CliPathArgs {
                id,
                show_entry: false,
                show_artifact: true,
                show_frost: true,
                absolute: false,
                format: Some(CliPathOutputFormat::Paths),
            })) if id == "alpha"
        ));
        assert!(matches!(
            entry.command,
            Command::Entry { command: EntryCommand::Path(CliPathArgs {
                id,
                show_entry: true,
                show_artifact: false,
                show_frost: false,
                absolute: false,
                format: None,
            }) } if id == "alpha"
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
                command: EntryCommand::Artifact {
                    command: ArtifactCommand::Add { id, source, artifact_path: Some(path) },
                },
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
                command: EntryCommand::Artifact {
                    command: ArtifactCommand::Remove { id, artifact_path },
                },
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
                command: EntryCommand::Artifact {
                    command: ArtifactCommand::List { id },
                },
            } if id == "alpha"
        ));
        assert!(matches!(
            rename.command,
            Command::Entry {
                command: EntryCommand::Artifact {
                    command: ArtifactCommand::Rename { id, old_path, new_path },
                },
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
        let args = CliPathArgs {
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

        assert_eq!(kinds, ["entry", "artifact-root", "artifact", "frost-entry", "frost-artifact"]);
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

        assert!(matches!(error, CliError::LakePathWithFrostPath));
    }

    #[test]
    fn check_rejects_old_frost_root_flag() {
        let error =
            Cli::try_parse_from(["sirno", "check", "--frost-root", "sirno-frost"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn query_accepts_exact_structural_predicate() {
        let cli = Cli::parse_from(["sirno", "query", "--exact", "topic=concept"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Query { exact, .. })
                if exact == vec![CliStructuralPredicate {
                    field: "topic".to_owned(),
                    target: EntryId::new("concept").unwrap(),
                }]
        ));
    }

    #[test]
    fn query_accepts_short_alias_and_options() {
        let cli =
            Cli::parse_from(["sirno", "q", "-x", "topic=concept", "-f", "id,path", "-o", "human"]);
        let Command::TopLevelEntry(TopLevelEntryCommand::Query {
            exact,
            fields: Some(fields),
            format: Some(format),
            ..
        }) = cli.command
        else {
            panic!("expected query command with short options");
        };

        assert_eq!(
            exact,
            vec![CliStructuralPredicate {
                field: "topic".to_owned(),
                target: EntryId::new("concept").unwrap(),
            }]
        );
        assert_eq!(fields.fields, vec![CliQueryField::Id, CliQueryField::Path]);
        assert!(matches!(format, CliQueryOutputFormat::Human));
    }

    #[test]
    fn entry_query_accepts_short_alias_and_options() {
        let cli = Cli::parse_from([
            "sirno",
            "entry",
            "q",
            "-x",
            "topic=concept",
            "-f",
            "id,path",
            "-o",
            "human",
        ]);
        let Command::Entry {
            command: EntryCommand::Query { exact, fields: Some(fields), format: Some(format), .. },
        } = cli.command
        else {
            panic!("expected entry query command with short options");
        };

        assert_eq!(
            exact,
            vec![CliStructuralPredicate {
                field: "topic".to_owned(),
                target: EntryId::new("concept").unwrap(),
            }]
        );
        assert_eq!(fields.fields, vec![CliQueryField::Id, CliQueryField::Path]);
        assert!(matches!(format, CliQueryOutputFormat::Human));
    }

    #[test]
    fn query_accepts_comma_separated_fields() {
        let cli = Cli::parse_from(["sirno", "query", "--fields", "id,name,path,desc"]);
        let Command::TopLevelEntry(TopLevelEntryCommand::Query { fields: Some(fields), .. }) =
            cli.command
        else {
            panic!("expected query command with fields");
        };

        assert_eq!(
            fields.fields,
            vec![CliQueryField::Id, CliQueryField::Name, CliQueryField::Path, CliQueryField::Desc,]
        );
    }

    #[test]
    fn query_accepts_json_format() {
        let cli = Cli::parse_from(["sirno", "query", "--format", "json"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelEntry(TopLevelEntryCommand::Query {
                format: Some(CliQueryOutputFormat::Json),
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
                format: Some(CliQueryOutputFormat::Human),
                ..
            })
        ));
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
    fn query_rejects_unknown_field() {
        let error = Cli::try_parse_from(["sirno", "query", "--fields", "id,summary"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn query_rejects_empty_field() {
        let error = Cli::try_parse_from(["sirno", "query", "--fields", "id,,desc"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn query_json_uses_selected_field_names() {
        let fields = "id,desc".parse::<CliQueryFields>().unwrap();
        let json = format_query_json(&fields, &[vec!["query".to_owned(), "Selection".to_owned()]])
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
    fn query_table_uses_selected_field_headers_and_widths() {
        let fields = "id,desc".parse::<CliQueryFields>().unwrap();
        let table =
            format_query_table(&fields, &[vec!["query".to_owned(), "Selection".to_owned()]]);

        assert_eq!(
            table,
            "\
| id    | desc      |
| ----- | --------- |
| query | Selection |
"
        );
    }

    #[test]
    fn query_rejects_old_exact_structural_flags() {
        let error =
            Cli::try_parse_from(["sirno", "query", "--exact-topic", "concept"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn check_accepts_short_mode() {
        let cli = Cli::parse_from(["sirno", "check", "-m", "review"]);

        assert!(matches!(
            cli.command,
            Command::TopLevelLake(TopLevelLakeCommand::Check {
                mode: Some(CliCheckMode::Review),
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
    fn exact_query_rejects_unconfigured_structural_field() {
        let error = exact_query_from_predicates(
            EntryQuery::new(),
            vec!["topic=concept".parse::<CliStructuralPredicate>().unwrap()],
            &StructuralSettings::default(),
        )
        .unwrap_err();

        assert!(matches!(error, CliError::UnconfiguredStructuralField(field) if field == "topic"));
    }

    #[test]
    fn exact_query_keeps_repeated_field_targets_disjunctive() {
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target("topic", EntryId::new("meta").unwrap());
        let entry = Entry::new(EntryId::new("concept").unwrap(), metadata, "");
        let settings =
            StructuralSettings::from_fields([("topic", StructuralFieldSettings::default())]);
        let query = exact_query_from_predicates(
            EntryQuery::new(),
            vec![
                "topic=concept".parse::<CliStructuralPredicate>().unwrap(),
                "topic=meta".parse::<CliStructuralPredicate>().unwrap(),
            ],
            &settings,
        )
        .unwrap();

        assert!(query.matches(&entry));
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

        assert!(matches!(error, CliError::MoveDestinationExists(_)));
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
            matches!(error, CliError::Frost(FrostError::FrozenEntryChanged(id)) if id.as_str() == "alpha")
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
            Command::Entry { command: EntryCommand::Query { terms, .. } }
                if terms == vec!["alpha"]
        ));
        assert!(matches!(
            short_witness.command,
            Command::Entry { command: EntryCommand::Witness { id, full: false } }
                if id == "alpha"
        ));
        assert!(matches!(
            mnemonic_witness.command,
            Command::Entry { command: EntryCommand::Witness { id, full: false } }
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
            matches!(error, CliError::MissingWitnessEntry(id) if id.as_str() == "missing-entry")
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
