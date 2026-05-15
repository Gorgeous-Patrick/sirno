//! Command-line interface for Sirno.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use sirno::{
    CONFIG_FILE_NAME, CheckMode, ConfigError, Entry, EntryDirectory, EntryDirectoryCheckSettings,
    EntryDirectoryError, EntryDirectoryReport, EntryDirectoryWritePolicy, EntryId, EntryIdError,
    EntryMetadata, EntryParseError, EntryQuery, Eterator, FrostError, FrostLockStatus,
    GenLinkDirectoryReport, LockError, SirnoConfig, SirnoFrost, SirnoLock, StructuralSettings,
    VagueEntryQuery, WitnessCheckSettings, WitnessError, WitnessRecord,
};
use thiserror::Error;

/// Sirno command-line entry point.
#[derive(Debug, Parser)]
#[command(name = "sirno")]
#[command(about = "Manage Sirno design entries")]
struct Cli {
    /// Sirno project config file.
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    /// Public Markdown lake path override.
    #[arg(long = "lake-path", global = true)]
    lake_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

/// Supported Sirno commands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Create a Sirno config and ordinary seed entries.
    // sirno:witness:storage-and-interfaces:begin
    Init {
        /// Monograph path written to Sirno.toml.
        #[arg(long)]
        mono: Option<PathBuf>,
        /// Public Markdown entry lake path written to Sirno.toml.
        #[arg(long)]
        lake: Option<PathBuf>,
    },
    /// Move the configured public Markdown entry lake.
    Mv {
        /// New public Markdown entry lake path written to Sirno.toml.
        lake: PathBuf,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Create one Markdown entry.
    // sirno:witness:storage-and-interfaces:begin
    New {
        /// Entry id and filename stem.
        id: String,
        /// Human-readable entry name.
        #[arg(long)]
        name: Option<String>,
        /// Short entry description.
        #[arg(long)]
        description: String,
        /// Category target.
        #[arg(long)]
        category: Vec<String>,
        /// Review-neighborhood target for `belongs`.
        #[arg(long)]
        belongs: Vec<String>,
        /// Broader entry target for `refines`.
        #[arg(long)]
        refines: Vec<String>,
        /// Initial Markdown body.
        #[arg(long)]
        body: Option<String>,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Freeze one public Markdown entry and make its file read-only.
    // sirno:witness:storage-and-interfaces:begin
    Freeze {
        /// Entry id to freeze.
        id: String,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Melt one public Markdown entry and make its file writable.
    // sirno:witness:storage-and-interfaces:begin
    #[command(visible_alias = "unfreeze")]
    Melt {
        /// Entry id to melt.
        id: String,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Query public Markdown entries.
    // sirno:witness:storage-and-interfaces:begin
    Query {
        /// Vague text terms matched against entries and structural target summaries.
        terms: Vec<String>,
        /// Exact text term matched against id, name, description, and body.
        #[arg(long = "exact-term")]
        exact_terms: Vec<String>,
        /// Exact structural predicate as FIELD=ENTRY_ID.
        #[arg(long, value_name = "FIELD=ENTRY_ID")]
        exact: Vec<CliExactPredicate>,
        /// Comma-separated output fields: id, name, path, desc.
        #[arg(long, value_name = "FIELDS")]
        format: Option<CliQueryFormat>,
        /// Print query results as a human-readable table.
        #[arg(long)]
        human: bool,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Check current entry structure.
    // sirno:witness:storage-and-interfaces:begin
    Check {
        /// Sirno Frost root.
        #[arg(long = "frost-root", conflicts_with = "lake_path")]
        frost_root: Option<PathBuf>,
        /// Check boundary.
        #[arg(long, value_enum)]
        mode: Option<CliCheckMode>,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Generate Markdown links in entry footers.
    // sirno:witness:storage-and-interfaces:begin
    #[command(name = "gen-link")]
    GenLink {
        /// Report generated-link changes without writing files.
        #[arg(long)]
        dry: bool,
        /// Generated-link command.
        #[command(subcommand)]
        command: Option<GenLinkCommand>,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Show the current Sirno project status.
    // sirno:witness:storage-and-interfaces:begin
    Status,
    // sirno:witness:storage-and-interfaces:end
    /// Show repository witness blocks for one entry id.
    // sirno:witness:storage-and-interfaces:begin
    Witness {
        /// Entry id used as the witness query key.
        id: String,
        /// Print full witness regions instead of only their locations.
        #[arg(long)]
        full: bool,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Manage optional Sirno Frost snapshots.
    // sirno:witness:storage-and-interfaces:begin
    Frost {
        /// Frost command.
        #[command(subcommand)]
        command: FrostCommand,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Utility commands.
    // sirno:witness:storage-and-interfaces:begin
    Util {
        /// Utility command.
        #[command(subcommand)]
        command: UtilCommand,
    },
    // sirno:witness:storage-and-interfaces:end
}

/// CLI representation of check boundaries.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliCheckMode {
    /// Editing boundary: dangling references are warnings.
    Edit,
    /// Review boundary: dangling references are errors.
    Review,
}

/// CLI query output field list.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CliQueryFormat {
    fields: Vec<CliQueryField>,
}

impl Default for CliQueryFormat {
    fn default() -> Self {
        Self { fields: vec![CliQueryField::Id, CliQueryField::Path, CliQueryField::Name] }
    }
}

impl FromStr for CliQueryFormat {
    type Err = CliQueryFormatParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if raw.trim().is_empty() {
            return Err(CliQueryFormatParseError::Empty);
        }

        let mut fields = Vec::new();
        for raw_field in raw.split(',') {
            let field = raw_field.trim();
            if field.is_empty() {
                return Err(CliQueryFormatParseError::EmptyField);
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
    /// Short entry description.
    Desc,
}

impl FromStr for CliQueryField {
    type Err = CliQueryFormatParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            | "id" => Ok(Self::Id),
            | "name" => Ok(Self::Name),
            | "path" => Ok(Self::Path),
            | "desc" => Ok(Self::Desc),
            | field => Err(CliQueryFormatParseError::UnknownField(field.to_owned())),
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

/// Error raised while parsing one `--format` field list.
#[derive(Debug, Error)]
enum CliQueryFormatParseError {
    /// The list contains no fields.
    #[error("query format must include at least one field")]
    Empty,
    /// The list contains a separator without a field.
    #[error("query format contains an empty field")]
    EmptyField,
    /// The list contains an unknown output field.
    #[error("unknown query format field `{0}`; expected id, name, path, or desc")]
    UnknownField(String),
}

/// Exact structural query predicate parsed from `FIELD=ENTRY_ID`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CliExactPredicate {
    field: String,
    target: EntryId,
}

impl FromStr for CliExactPredicate {
    type Err = CliExactPredicateParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let Some((field, target)) = raw.split_once('=') else {
            return Err(CliExactPredicateParseError::MissingEquals);
        };
        if field.is_empty() {
            return Err(CliExactPredicateParseError::EmptyField);
        }
        let target = EntryId::new(target)?;
        Ok(Self { field: field.to_owned(), target })
    }
}

/// Error raised while parsing one `--exact FIELD=ENTRY_ID` argument.
#[derive(Debug, Error)]
enum CliExactPredicateParseError {
    /// The argument does not contain the field-target separator.
    #[error("expected FIELD=ENTRY_ID")]
    MissingEquals,
    /// The structural field name is empty.
    #[error("exact structural field name must not be empty")]
    EmptyField,
    /// The target entry id is invalid.
    #[error(transparent)]
    EntryId(#[from] EntryIdError),
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
    /// Configure Sirno Frost and freeze the current public Markdown lake.
    Init {
        /// Sirno Frost root path written to Sirno.toml.
        #[arg(long)]
        frost: Option<PathBuf>,
    },
    /// Move the configured Sirno Frost root.
    Mv {
        /// New Sirno Frost root path written to Sirno.toml.
        frost: PathBuf,
    },
    /// Freeze the current public Markdown lake.
    Commit,
    /// Check out one Frost version into the public Markdown lake.
    Checkout {
        /// Version coordinate to materialize in the current Frost generation.
        version: u64,
        /// Leave the checked-out version writable.
        #[arg(long)]
        unsafe_mutable: bool,
    },
}

/// Supported generated-link commands.
#[derive(Debug, Subcommand)]
enum GenLinkCommand {
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
        match self.command {
            | Command::Init { mono, lake } => {
                let mut config = SirnoConfig::new(
                    lake.or_else(|| lake_path.clone()).unwrap_or_else(default_lake_path),
                );
                if let Some(mono) = mono {
                    config = config.with_mono(mono);
                }
                let lake_path = config.resolve_lake(&config_path);
                config.write_new(&config_path)?;
                let paths = EntryDirectory::new(&lake_path).init()?;
                println!(
                    "initialized {} with {} entries in {}",
                    config_path.display(),
                    paths.len(),
                    lake_path.display()
                );
                Ok(ExitCode::SUCCESS)
            }
            | Command::Mv { lake } => {
                let config = SirnoConfig::from_file(&config_path)?;
                let old_lake = config.resolve_lake(&config_path);
                let config = config.with_lake(lake);
                config.validate_for_file(&config_path)?;
                let new_lake = config.resolve_lake(&config_path);
                move_configured_path_and_write_config(&old_lake, &new_lake, &config, &config_path)?;
                println!("moved lake {} to {}", old_lake.display(), new_lake.display());
                Ok(ExitCode::SUCCESS)
            }
            | Command::New { id, name, description, category, belongs, refines, body } => {
                let (lake, _) = resolve_lake_directory(lake_path.as_deref(), &config_path)?;
                let id = EntryId::new(&id)?;
                let mut metadata = EntryMetadata::new(
                    name.unwrap_or_else(|| title_name_from_id(&id)),
                    description,
                )?;
                metadata.set_structural_targets("category", parse_entry_ids(category)?);
                metadata.set_structural_targets("belongs", parse_entry_ids(belongs)?);
                metadata.set_structural_targets("refines", parse_entry_ids(refines)?);

                let entry = Entry::new(id, metadata, body.unwrap_or_default());
                let path = EntryDirectory::new(&lake).create_entry(&entry)?;
                println!("created {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
            | Command::Freeze { id } => {
                let (lake, _) = resolve_lake_directory(lake_path.as_deref(), &config_path)?;
                let id = EntryId::new(&id)?;
                let path = EntryDirectory::new(&lake).freeze_entry(&id)?;
                println!("froze entry {id} at {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
            | Command::Melt { id } => {
                let (lake, _) = resolve_lake_directory(lake_path.as_deref(), &config_path)?;
                let id = EntryId::new(&id)?;
                let path = EntryDirectory::new(&lake).melt_entry(&id)?;
                println!("melted entry {id} at {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
            | Command::Query { terms, exact_terms, exact, format, human } => {
                let (lake, mut settings) =
                    resolve_lake_directory(lake_path.as_deref(), &config_path)?;
                settings.link = false;
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
                let format = format.unwrap_or_default();
                print_query_results(&report, &matches, &format, human)?;
                Ok(ExitCode::SUCCESS)
            }
            | Command::Check { frost_root, mode } => {
                if lake_path.is_some() && frost_root.is_some() {
                    return Err(CliError::LakePathWithFrostRoot);
                }
                let mode = mode.unwrap_or(CliCheckMode::Review);
                if lake_path.is_some() {
                    let (lake, settings) =
                        resolve_lake_directory(lake_path.as_deref(), &config_path)?;
                    let report =
                        EntryDirectory::new(lake).check_with_settings(mode.into(), &settings)?;
                    print_entry_directory_report(&report);
                    return if report.has_errors() {
                        Ok(ExitCode::FAILURE)
                    } else {
                        Ok(ExitCode::SUCCESS)
                    };
                }

                let Some(frost_root) = frost_root else {
                    let config = SirnoConfig::from_file(&config_path)?;
                    let report = EntryDirectory::new(config.resolve_lake(&config_path))
                        .check_with_settings(
                            mode.into(),
                            &entry_directory_check_settings(&config_path, &config),
                        )?;
                    print_entry_directory_report(&report);
                    return if report.has_errors() {
                        Ok(ExitCode::FAILURE)
                    } else {
                        Ok(ExitCode::SUCCESS)
                    };
                };

                let frost = SirnoFrost::open(frost_root)?;
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
            | Command::GenLink { command, dry } => match command {
                | None => {
                    let (lake, mut settings) =
                        resolve_lake_directory(lake_path.as_deref(), &config_path)?;
                    settings.link = false;
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
                | Some(GenLinkCommand::Delete) => {
                    if dry {
                        return Err(CliError::DryWithGenLinkSubcommand);
                    }
                    let (lake, mut settings) =
                        resolve_lake_directory(lake_path.as_deref(), &config_path)?;
                    settings.witness = None;

                    let report = EntryDirectory::new(&lake)
                        .delete_generated_links_with_ignored_paths(settings.ignore)?;
                    print_gen_link_report(&report);
                    Ok(ExitCode::SUCCESS)
                }
            },
            | Command::Status => {
                let config = SirnoConfig::from_file(&config_path)?;
                let mono = config.resolve_mono(&config_path);
                let frost = config.resolve_frost(&config_path);
                let lock_path = SirnoLock::path_for_config(&config_path);
                let lock = if frost.is_some() {
                    SirnoLock::from_file_if_exists(&lock_path)?
                } else {
                    None
                };
                let (lake, settings) = resolve_lake_directory(lake_path.as_deref(), &config_path)?;
                let report =
                    EntryDirectory::new(&lake).check_with_settings(CheckMode::Review, &settings)?;
                print_status(
                    &config_path,
                    mono.as_deref(),
                    frost.as_deref(),
                    lock.as_ref(),
                    &config,
                    &report,
                );
                if report.has_errors() { Ok(ExitCode::FAILURE) } else { Ok(ExitCode::SUCCESS) }
            }
            | Command::Witness { id, full } => {
                run_witness_command(&config_path, lake_path.as_deref(), &id, full)
            }
            | Command::Frost { command } => command.run(&config_path, lake_path.as_deref()),
            | Command::Util { command } => command.run(),
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
                let frost_path =
                    frost.or_else(|| existing_frost.clone()).unwrap_or_else(default_frost_path);
                if let Some(existing_frost) = existing_frost
                    && existing_frost != frost_path
                {
                    return Err(CliError::FrostAlreadyConfigured(existing_frost));
                }

                let needs_config_write = config.frost.is_none();
                let config =
                    if needs_config_write { config.with_frost(frost_path) } else { config };
                config.validate_for_file(config_path)?;

                let frost_root =
                    config.resolve_frost(config_path).expect("frost path configured by init");
                let lake_path = resolve_lake_path(lake_path, config_path, &config);
                let mut frost = SirnoFrost::open(&frost_root)?;
                let version = frost.commit_entry_directory(
                    &lake_path,
                    &entry_directory_check_settings(config_path, &config),
                )?;
                if needs_config_write {
                    config.write(config_path)?;
                }
                SirnoLock::current(version).write(SirnoLock::path_for_config(config_path))?;
                println!(
                    "initialized frost {} at version {} from {}",
                    frost_root.display(),
                    version.version(),
                    lake_path.display()
                );
                Ok(ExitCode::SUCCESS)
            }
            | FrostCommand::Mv { frost } => {
                let config = SirnoConfig::from_file(config_path)?;
                let Some(old_frost) = config.resolve_frost(config_path) else {
                    return Err(CliError::FrostNotConfigured);
                };
                let config = config.with_frost(frost);
                config.validate_for_file(config_path)?;
                let new_frost =
                    config.resolve_frost(config_path).expect("frost path configured by mv");
                move_configured_path_and_write_config(
                    &old_frost,
                    &new_frost,
                    &config,
                    config_path,
                )?;
                println!("moved frost {} to {}", old_frost.display(), new_frost.display());
                Ok(ExitCode::SUCCESS)
            }
            | FrostCommand::Commit => {
                let context = FrostContext::load(config_path, lake_path)?;
                context.reject_immutable_checkout()?;
                let mut frost = SirnoFrost::open(&context.frost_root)?;
                let version =
                    frost.commit_entry_directory(&context.lake_path, &context.settings)?;
                context.lake().set_writable(&context.settings)?;
                SirnoLock::current(version).write(&context.lock_path)?;
                println!(
                    "froze version {} from {}",
                    version.version(),
                    context.lake_path.display()
                );
                Ok(ExitCode::SUCCESS)
            }
            | FrostCommand::Checkout { version, unsafe_mutable } => {
                let context = FrostContext::load(config_path, lake_path)?;
                let version = frost_version(version)?;
                let frost = SirnoFrost::open(&context.frost_root)?;
                let snapshot = frost.snapshot_for_version(version)?;
                let paths = frost.checkout_entry_directory(
                    snapshot,
                    &context.lake_path,
                    EntryDirectoryWritePolicy::ReplaceDirectory {
                        ignore: context.settings.ignore.clone(),
                    },
                )?;
                if unsafe_mutable {
                    context.lake().set_writable(&context.settings)?;
                } else {
                    context.lake().add_readonly_checkout_warnings(&paths)?;
                    context.lake().set_readonly(&context.settings)?;
                }
                SirnoLock::checked_out(snapshot, unsafe_mutable).write(&context.lock_path)?;
                println!(
                    "checked out frost version {} into {} ({} entries, {})",
                    snapshot.version(),
                    context.lake_path.display(),
                    paths.len(),
                    if unsafe_mutable { "unsafe mutable" } else { "immutable" }
                );
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
    frost_root: PathBuf,
    lock_path: PathBuf,
    settings: EntryDirectoryCheckSettings,
    lake_path: PathBuf,
}

impl FrostContext {
    fn load(config_path: &Path, lake_path: Option<&Path>) -> Result<Self, CliError> {
        let config = SirnoConfig::from_file(config_path)?;
        let Some(frost_root) = config.resolve_frost(config_path) else {
            return Err(CliError::FrostNotConfigured);
        };
        Ok(Self {
            frost_root,
            lock_path: SirnoLock::path_for_config(config_path),
            settings: entry_directory_check_settings(config_path, &config),
            lake_path: resolve_lake_path(lake_path, config_path, &config),
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

fn default_config_path() -> PathBuf {
    PathBuf::from(CONFIG_FILE_NAME)
}

fn default_lake_path() -> PathBuf {
    PathBuf::from("docs")
}

fn default_frost_path() -> PathBuf {
    PathBuf::from("sirno-frost")
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
        link: config.check.link,
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
    mut query: EntryQuery, predicates: Vec<CliExactPredicate>, structural: &StructuralSettings,
) -> Result<EntryQuery, CliError> {
    let mut targets_by_field = BTreeMap::<String, Vec<EntryId>>::new();
    for predicate in predicates {
        if !structural.contains_field(&predicate.field) {
            return Err(CliError::UnconfiguredExactField(predicate.field));
        }
        targets_by_field.entry(predicate.field).or_default().push(predicate.target);
    }
    for (field, targets) in targets_by_field {
        query = query.with_structural_targets(field, targets);
    }
    Ok(query)
}

fn parse_entry_ids(raw: Vec<String>) -> Result<Vec<EntryId>, CliError> {
    raw.into_iter().map(|value| EntryId::new(&value).map_err(CliError::EntryId)).collect()
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
    println!("  link: {}", config.check.link);
    println!("structural:");
    for (field, settings) in config.structural.fields() {
        println!("  {field}.link: {}", settings.link);
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
            format!("current version {} (generation {})", lock.frost.version, lock.frost.generation)
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
    report: &EntryDirectoryReport, entries: &[&Entry], format: &CliQueryFormat, human: bool,
) -> Result<(), CliError> {
    let rows = query_result_rows(report, entries, format)?;
    if human {
        print!("{}", format_query_table(format, &rows));
        return Ok(());
    }

    for row in rows {
        println!("{}", row.join("\t"));
    }
    Ok(())
}

fn query_result_rows(
    report: &EntryDirectoryReport, entries: &[&Entry], format: &CliQueryFormat,
) -> Result<Vec<Vec<String>>, CliError> {
    entries
        .iter()
        .map(|entry| {
            format
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
        | CliQueryField::Desc => Ok(entry.metadata.description.clone()),
    }
}

fn format_query_table(format: &CliQueryFormat, rows: &[Vec<String>]) -> String {
    let headers = format.fields.iter().map(|field| field.label()).collect::<Vec<_>>();
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
    /// Empty Frost cannot be checked out as a version.
    #[error("frost version {0} is not a check-outable snapshot")]
    InvalidFrostVersion(u64),
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
    /// Lake path override does not apply to checking a Frost root directly.
    #[error("`--lake-path` cannot be used with `check --frost-root`")]
    LakePathWithFrostRoot,
    /// Dry-run mode applies only to generated-link writing.
    #[error("`--dry` only applies to `sirno gen-link` without a subcommand")]
    DryWithGenLinkSubcommand,
    /// Exact query named a structural field not configured for this project.
    #[error("structural field `{0}` is not configured; add it under [structural] in Sirno.toml")]
    UnconfiguredExactField(String),
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
    /// Entry metadata construction failed.
    #[error(transparent)]
    EntryParse(#[from] EntryParseError),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use clap::Parser;

    use sirno::{
        CATEGORY_FIELD, CONFIG_FILE_NAME, Entry, EntryId, EntryMetadata, EntryQuery, FrostSettings,
        RepoMember, RepoSettings, SirnoConfig, StructuralSettings, WitnessRecord, WitnessSpan,
    };

    use crate::{
        Cli, CliError, CliExactPredicate, CliQueryField, CliQueryFormat, Command, FrostCommand,
        exact_query_from_predicates, format_gen_link_report, format_query_table,
        format_witness_record, format_witness_records,
    };

    #[test]
    fn init_does_not_accept_frost_path() {
        let error = Cli::try_parse_from(["sirno", "init", "--frost", "sirno-frost"]).unwrap_err();

        assert!(error.to_string().contains("unexpected argument"));
    }

    #[test]
    fn init_uses_global_lake_path() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let docs = temp.path().join("sirno-docs");

        Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "--lake-path",
            "sirno-docs",
            "init",
        ])
        .run()
        .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.lake.path, PathBuf::from("sirno-docs"));
        assert!(docs.join("concept.md").exists());
    }

    #[test]
    fn frost_init_accepts_frost_path() {
        let cli = Cli::parse_from(["sirno", "frost", "init", "--frost", "sirno-frost"]);

        assert!(matches!(
            cli.command,
            Command::Frost { command: FrostCommand::Init { frost: Some(_) } }
        ));
    }

    #[test]
    fn mv_accepts_lake_path() {
        let cli = Cli::parse_from(["sirno", "mv", "sirno-docs"]);

        assert!(matches!(cli.command, Command::Mv { lake } if lake == Path::new("sirno-docs")));
    }

    #[test]
    fn frost_mv_accepts_frost_path() {
        let cli = Cli::parse_from(["sirno", "frost", "mv", "sirno-frost-2"]);

        assert!(matches!(
            cli.command,
            Command::Frost { command: FrostCommand::Mv { frost } }
                if frost == Path::new("sirno-frost-2")
        ));
    }

    #[test]
    fn frost_checkout_accepts_unsafe_mutable_flag() {
        let cli = Cli::parse_from(["sirno", "frost", "checkout", "3", "--unsafe-mutable"]);

        assert!(matches!(
            cli.command,
            Command::Frost { command: FrostCommand::Checkout { version: 3, unsafe_mutable: true } }
        ));
    }

    #[test]
    fn freeze_accepts_entry_id() {
        let cli = Cli::parse_from(["sirno", "freeze", "alpha"]);

        assert!(matches!(cli.command, Command::Freeze { id, .. } if id == "alpha"));
    }

    #[test]
    fn lake_path_is_global() {
        let cli = Cli::parse_from(["sirno", "freeze", "alpha", "--lake-path", "scratch-docs"]);

        assert_eq!(cli.lake_path.as_deref(), Some(Path::new("scratch-docs")));
        assert!(matches!(cli.command, Command::Freeze { id } if id == "alpha"));
    }

    #[test]
    fn lake_path_conflicts_with_frost_root_check() {
        let error = Cli::parse_from([
            "sirno",
            "--lake-path",
            "scratch-docs",
            "check",
            "--frost-root",
            "sirno-frost",
        ])
        .run()
        .unwrap_err();

        assert!(matches!(error, CliError::LakePathWithFrostRoot));
    }

    #[test]
    fn query_accepts_exact_structural_predicate() {
        let cli = Cli::parse_from(["sirno", "query", "--exact", "category=concept"]);

        assert!(matches!(
            cli.command,
            Command::Query { exact, .. }
                if exact == vec![CliExactPredicate {
                    field: "category".to_owned(),
                    target: EntryId::new("concept").unwrap(),
                }]
        ));
    }

    #[test]
    fn query_accepts_comma_separated_format_fields() {
        let cli = Cli::parse_from(["sirno", "query", "--format", "id,name,path,desc"]);
        let Command::Query { format: Some(format), .. } = cli.command else {
            panic!("expected query command with format");
        };

        assert_eq!(
            format.fields,
            vec![CliQueryField::Id, CliQueryField::Name, CliQueryField::Path, CliQueryField::Desc,]
        );
    }

    #[test]
    fn query_accepts_human_table_flag() {
        let cli = Cli::parse_from(["sirno", "query", "--human"]);

        assert!(matches!(cli.command, Command::Query { human: true, .. }));
    }

    #[test]
    fn query_rejects_unknown_format_field() {
        let error = Cli::try_parse_from(["sirno", "query", "--format", "id,summary"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn query_rejects_empty_format_field() {
        let error = Cli::try_parse_from(["sirno", "query", "--format", "id,,desc"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn query_table_uses_selected_field_headers_and_widths() {
        let format = "id,desc".parse::<CliQueryFormat>().unwrap();
        let table =
            format_query_table(&format, &[vec!["query".to_owned(), "Selection".to_owned()]]);

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
            Cli::try_parse_from(["sirno", "query", "--exact-category", "concept"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn exact_query_rejects_unconfigured_structural_field() {
        let error = exact_query_from_predicates(
            EntryQuery::new(),
            vec!["topic=concept".parse::<CliExactPredicate>().unwrap()],
            &StructuralSettings::default(),
        )
        .unwrap_err();

        assert!(matches!(error, CliError::UnconfiguredExactField(field) if field == "topic"));
    }

    #[test]
    fn exact_query_keeps_repeated_field_targets_disjunctive() {
        let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
        metadata.push_structural_target(CATEGORY_FIELD, EntryId::new("meta").unwrap());
        let entry = Entry::new(EntryId::new("concept").unwrap(), metadata, "");
        let query = exact_query_from_predicates(
            EntryQuery::new(),
            vec![
                "category=concept".parse::<CliExactPredicate>().unwrap(),
                "category=meta".parse::<CliExactPredicate>().unwrap(),
            ],
            &StructuralSettings::default(),
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

        assert!(matches!(melt.command, Command::Melt { id, .. } if id == "alpha"));
        assert!(matches!(unfreeze.command, Command::Melt { id, .. } if id == "alpha"));
    }

    #[test]
    fn mv_moves_lake_and_rewrites_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let old_lake = temp.path().join("docs");
        let new_lake = temp.path().join("sirno-docs");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&old_lake).unwrap();
        fs::write(old_lake.join("entry.md"), "entry").unwrap();

        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "mv", "sirno-docs"])
            .run()
            .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.lake.path, PathBuf::from("sirno-docs"));
        assert!(!old_lake.exists());
        assert!(new_lake.join("entry.md").exists());
    }

    #[test]
    fn mv_refuses_existing_destination() {
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
            "mv",
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
    fn frost_mv_moves_frost_and_rewrites_config() {
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
            "frost",
            "mv",
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
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
description: Alpha entry.
---

Body.
",
        )
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
    fn lake_path_override_targets_public_lake_commands() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let configured_docs = temp.path().join("docs");
        let override_docs = temp.path().join("scratch-docs");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&configured_docs).unwrap();
        fs::create_dir(&override_docs).unwrap();
        let entry = "\
---
name: Alpha
description: Alpha entry.
---

Body.
";
        fs::write(configured_docs.join("alpha.md"), entry).unwrap();
        fs::write(override_docs.join("alpha.md"), entry).unwrap();

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
        let error =
            Cli::try_parse_from(["sirno", "new", "alpha", "--description", "Alpha.", "--witness"])
                .unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn witness_accepts_entry_id() {
        let cli = Cli::parse_from(["sirno", "witness", "witness"]);

        assert!(matches!(cli.command, Command::Witness { id, full: false } if id == "witness"));
    }

    #[test]
    fn witness_accepts_full_flag() {
        let cli = Cli::parse_from(["sirno", "witness", "witness", "--full"]);

        assert!(matches!(cli.command, Command::Witness { id, full: true } if id == "witness"));
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
    fn gen_link_rejects_no_check_flag() {
        let error = Cli::try_parse_from(["sirno", "gen-link", "--no-check"]).unwrap_err();

        assert!(error.to_string().contains("unexpected argument"));
    }

    #[test]
    fn gen_link_accepts_dry_flag() {
        let cli = Cli::parse_from(["sirno", "gen-link", "--dry"]);

        assert!(matches!(cli.command, Command::GenLink { dry: true, command: None, .. }));
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
