//! Command-line interface for Sirno.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use sirno::{
    CONFIG_FILE_NAME, CheckMode, CheckSeverity, ConfigError, Entry, EntryDirectoryCheckSettings,
    EntryDirectoryError, EntryDirectoryReport, EntryDirectoryWritePolicy, EntryId, EntryIdError,
    EntryMetadata, EntryParseError, EntryQuery, Eterator, GenLinkDirectoryReport,
    GeneratedLinkSettings, HistoryLockStatus, LockError, SirnoConfig, SirnoLock, SirnoStore,
    StoreError, VagueEntryQuery, WitnessCheckSettings, WitnessError, WitnessMarker, WitnessRecord,
    add_readonly_checkout_warnings, check_entry_directory_with_settings,
    check_gen_link_entry_directory_with_ignored_paths, create_entry_file,
    delete_gen_link_entry_directory_with_ignored_paths,
    gen_link_entry_directory_with_ignored_paths, init_entry_directory, query_entries,
    resolve_lock_path, scan_witnesses, set_entry_directory_readonly, set_entry_directory_writable,
    vague_query_entries,
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
        /// Clique closure target.
        #[arg(long)]
        clustee: Vec<String>,
        /// Refined entry target.
        #[arg(long)]
        refiner: Vec<String>,
        /// Add a canonical witness marker.
        #[arg(long)]
        witness: bool,
        /// Initial Markdown body.
        #[arg(long)]
        body: Option<String>,
        /// Public Markdown entry directory.
        #[arg(long)]
        entries: Option<PathBuf>,
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
        /// Exact category target.
        #[arg(long)]
        exact_category: Vec<String>,
        /// Exact clique closure target.
        #[arg(long)]
        exact_clustee: Vec<String>,
        /// Exact refined entry target.
        #[arg(long)]
        exact_refiner: Vec<String>,
        /// Select only entries with a canonical witness marker.
        #[arg(long)]
        exact_witness: bool,
        /// Output format.
        #[arg(long, value_enum)]
        format: Option<CliQueryFormat>,
        /// Public Markdown entry directory.
        #[arg(long)]
        entries: Option<PathBuf>,
    },
    // sirno:witness:storage-and-interfaces:end
    /// Check current entry structure.
    // sirno:witness:storage-and-interfaces:begin
    Check {
        /// Eter-backed history store root.
        #[arg(long = "history-store", conflicts_with = "entries")]
        history_store: Option<PathBuf>,
        /// Public Markdown entry directory.
        #[arg(long, conflicts_with = "history_store")]
        entries: Option<PathBuf>,
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
        /// Public Markdown entry directory.
        #[arg(long)]
        entries: Option<PathBuf>,
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
    /// Manage optional eter-backed history.
    // sirno:witness:storage-and-interfaces:begin
    History {
        /// History command.
        #[command(subcommand)]
        command: HistoryCommand,
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

/// CLI query output shape.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliQueryFormat {
    /// Print id, path, and name.
    Summary,
    /// Print only entry ids.
    Id,
    /// Print only Markdown paths.
    Path,
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

/// Supported history commands.
#[derive(Debug, Subcommand)]
enum HistoryCommand {
    /// Configure history and commit the current public Markdown lake.
    Init {
        /// Private eter history store path written to Sirno.toml.
        #[arg(long)]
        history: Option<PathBuf>,
    },
    /// Move the configured private history store.
    Mv {
        /// New private eter history store path written to Sirno.toml.
        history: PathBuf,
    },
    /// Commit the current public Markdown lake into history.
    Commit,
    /// Check out one history version into the public Markdown lake.
    Checkout {
        /// Version coordinate to materialize in the current history generation.
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
    Delete {
        /// Public Markdown entry directory.
        #[arg(long)]
        entries: Option<PathBuf>,
    },
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
    match run(Cli::parse()) {
        | Ok(code) => code,
        | Err(error) => {
            eprintln!("sirno: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, CliError> {
    let config_path = cli.config.unwrap_or_else(default_config_path);
    match cli.command {
        | Command::Init { mono, lake } => {
            let mut config = SirnoConfig::new(lake.unwrap_or_else(default_lake_path));
            if let Some(mono) = mono {
                config = config.with_mono(mono);
            }
            let lake_path = config.resolve_lake(&config_path);
            config.write_new(&config_path)?;
            let paths = init_entry_directory(&lake_path)?;
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
        | Command::New {
            id,
            name,
            description,
            category,
            clustee,
            refiner,
            witness,
            body,
            entries,
        } => {
            let entries = match entries {
                | Some(entries) => entries,
                | None => {
                    let config = SirnoConfig::from_file(&config_path)?;
                    config.resolve_lake(&config_path)
                }
            };
            let id = EntryId::new(&id)?;
            let mut metadata =
                EntryMetadata::new(name.unwrap_or_else(|| title_name_from_id(&id)), description)?;
            metadata.category = parse_entry_ids(category)?;
            metadata.clustee = parse_entry_ids(clustee)?;
            metadata.refiner = parse_entry_ids(refiner)?;
            if witness {
                metadata.witness = Some(WitnessMarker::Present);
            }

            let entry = Entry::new(id, metadata, body.unwrap_or_default());
            let path = create_entry_file(&entries, &entry)?;
            println!("created {}", path.display());
            Ok(ExitCode::SUCCESS)
        }
        | Command::Query {
            terms,
            exact_terms,
            exact_category,
            exact_clustee,
            exact_refiner,
            exact_witness,
            format,
            entries,
        } => {
            let (entries, mut settings) = resolve_entry_directory(entries, &config_path)?;
            settings.link = false;
            settings.links = GeneratedLinkSettings::default();
            settings.witness = None;
            let report = check_entry_directory_with_settings(&entries, CheckMode::Edit, &settings)?;
            if report.has_errors() {
                print_entry_directory_report(&report);
                return Ok(ExitCode::FAILURE);
            }

            let vague_query = VagueEntryQuery::new().with_text_terms(terms);
            let exact_query = EntryQuery::new()
                .with_text_terms(exact_terms)
                .with_category(parse_entry_ids(exact_category)?)
                .with_clustee(parse_entry_ids(exact_clustee)?)
                .with_refiner(parse_entry_ids(exact_refiner)?)
                .with_witness(exact_witness);
            let vague_matches = vague_query_entries(report.entries(), &vague_query);
            let matches = query_entries(vague_matches, &exact_query);
            print_query_results(&report, &matches, format.unwrap_or(CliQueryFormat::Summary))?;
            Ok(ExitCode::SUCCESS)
        }
        | Command::Check { history_store, entries, mode } => {
            let mode = mode.unwrap_or(CliCheckMode::Review);
            if let Some(entries) = entries {
                let settings = explicit_entries_check_settings(&config_path)?;
                let report = check_entry_directory_with_settings(entries, mode.into(), &settings)?;
                print_entry_directory_report(&report);
                return if report.has_errors() {
                    Ok(ExitCode::FAILURE)
                } else {
                    Ok(ExitCode::SUCCESS)
                };
            }

            let Some(history_store) = history_store else {
                let config = SirnoConfig::from_file(&config_path)?;
                let report = check_entry_directory_with_settings(
                    config.resolve_lake(&config_path),
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

            let store = SirnoStore::open(history_store)?;
            let report = store.check_current(mode.into())?;
            if report.is_clean() {
                println!("ok: {}", store.root().display());
                return Ok(ExitCode::SUCCESS);
            }

            for diagnostic in report.diagnostics() {
                println!("{}: {}", severity_label(diagnostic.severity), diagnostic.message());
            }

            if report.has_errors() { Ok(ExitCode::FAILURE) } else { Ok(ExitCode::SUCCESS) }
        }
        | Command::GenLink { command, entries, dry } => match command {
            | None => {
                let (entries, mut settings) = resolve_entry_directory(entries, &config_path)?;
                settings.link = false;
                settings.witness = None;

                let check =
                    check_entry_directory_with_settings(&entries, CheckMode::Review, &settings)?;
                if check.has_errors() {
                    print_entry_directory_report(&check);
                    return Ok(ExitCode::FAILURE);
                }

                if dry {
                    let report = check_gen_link_entry_directory_with_ignored_paths(
                        &entries,
                        &settings.links,
                        settings.ignore.clone(),
                    )?;
                    print_gen_link_report(&report);
                    return Ok(ExitCode::SUCCESS);
                }

                let report = gen_link_entry_directory_with_ignored_paths(
                    &entries,
                    &settings.links,
                    settings.ignore.clone(),
                )?;
                print_gen_link_report(&report);
                Ok(ExitCode::SUCCESS)
            }
            | Some(GenLinkCommand::Delete { entries: delete_entries }) => {
                if dry {
                    return Err(CliError::DryWithGenLinkSubcommand);
                }
                let (entries, mut settings) =
                    resolve_entry_directory(delete_entries.or(entries), &config_path)?;
                settings.witness = None;

                let report =
                    delete_gen_link_entry_directory_with_ignored_paths(&entries, settings.ignore)?;
                print_gen_link_report(&report);
                Ok(ExitCode::SUCCESS)
            }
        },
        | Command::Status => {
            let config = SirnoConfig::from_file(&config_path)?;
            let mono = config.resolve_mono(&config_path);
            let history = config.resolve_history(&config_path);
            let lock_path = resolve_lock_path(&config_path);
            let lock = if history.is_some() { read_lock_if_exists(&lock_path)? } else { None };
            let lake = config.resolve_lake(&config_path);
            let report = check_entry_directory_with_settings(
                &lake,
                CheckMode::Review,
                &entry_directory_check_settings(&config_path, &config),
            )?;
            print_status(
                &config_path,
                mono.as_deref(),
                history.as_deref(),
                lock.as_ref(),
                &config,
                &report,
            );
            if report.has_errors() { Ok(ExitCode::FAILURE) } else { Ok(ExitCode::SUCCESS) }
        }
        | Command::Witness { id, full } => run_witness_command(&config_path, &id, full),
        | Command::History { command } => run_history_command(command, &config_path),
        | Command::Util { command } => run_util_command(command),
    }
}

fn run_history_command(
    command: HistoryCommand, config_path: &std::path::Path,
) -> Result<ExitCode, CliError> {
    match command {
        | HistoryCommand::Init { history } => {
            let config = SirnoConfig::from_file(config_path)?;
            let existing_history = config.history.as_ref().map(|settings| settings.path.clone());
            let history_path =
                history.or_else(|| existing_history.clone()).unwrap_or_else(default_history_path);
            if let Some(existing_history) = existing_history
                && existing_history != history_path
            {
                return Err(CliError::HistoryAlreadyConfigured(existing_history));
            }

            let needs_config_write = config.history.is_none();
            let config =
                if needs_config_write { config.with_history(history_path) } else { config };
            config.validate_for_file(config_path)?;

            let history_root =
                config.resolve_history(config_path).expect("history path configured by init");
            let lake_path = config.resolve_lake(config_path);
            let mut store = SirnoStore::open(&history_root)?;
            let version = store.commit_entry_directory(
                &lake_path,
                &entry_directory_check_settings(config_path, &config),
            )?;
            if needs_config_write {
                config.write(config_path)?;
            }
            SirnoLock::current(version).write(resolve_lock_path(config_path))?;
            println!(
                "initialized history {} at version {} from {}",
                history_root.display(),
                version.version(),
                lake_path.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        | HistoryCommand::Mv { history } => {
            let config = SirnoConfig::from_file(config_path)?;
            let Some(old_history) = config.resolve_history(config_path) else {
                return Err(CliError::HistoryNotConfigured);
            };
            let config = config.with_history(history);
            config.validate_for_file(config_path)?;
            let new_history =
                config.resolve_history(config_path).expect("history path configured by mv");
            move_configured_path_and_write_config(
                &old_history,
                &new_history,
                &config,
                config_path,
            )?;
            println!("moved history {} to {}", old_history.display(), new_history.display());
            Ok(ExitCode::SUCCESS)
        }
        | HistoryCommand::Commit => {
            let context = HistoryContext::load(config_path)?;
            reject_immutable_checkout(&context.lock_path)?;
            let mut store = SirnoStore::open(&context.history_root)?;
            let version = store.commit_entry_directory(&context.lake_path, &context.settings)?;
            set_entry_directory_writable(&context.lake_path, &context.settings)?;
            SirnoLock::current(version).write(&context.lock_path)?;
            println!(
                "committed history version {} from {}",
                version.version(),
                context.lake_path.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        | HistoryCommand::Checkout { version, unsafe_mutable } => {
            let context = HistoryContext::load(config_path)?;
            let version = history_version(version)?;
            let store = SirnoStore::open(&context.history_root)?;
            let snapshot = store.snapshot_for_version(version)?;
            let paths = store.checkout_entry_directory(
                snapshot,
                &context.lake_path,
                EntryDirectoryWritePolicy::ReplaceDirectory {
                    ignore: context.settings.ignore.clone(),
                },
            )?;
            if unsafe_mutable {
                set_entry_directory_writable(&context.lake_path, &context.settings)?;
            } else {
                add_readonly_checkout_warnings(&paths)?;
                set_entry_directory_readonly(&context.lake_path, &context.settings)?;
            }
            SirnoLock::checked_out(snapshot, unsafe_mutable).write(&context.lock_path)?;
            println!(
                "checked out history version {} into {} ({} entries, {})",
                snapshot.version(),
                context.lake_path.display(),
                paths.len(),
                if unsafe_mutable { "unsafe mutable" } else { "immutable" }
            );
            Ok(ExitCode::SUCCESS)
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

struct HistoryContext {
    history_root: PathBuf,
    lock_path: PathBuf,
    settings: EntryDirectoryCheckSettings,
    lake_path: PathBuf,
}

impl HistoryContext {
    fn load(config_path: &Path) -> Result<Self, CliError> {
        let config = SirnoConfig::from_file(config_path)?;
        let Some(history_root) = config.resolve_history(config_path) else {
            return Err(CliError::HistoryNotConfigured);
        };
        Ok(Self {
            history_root,
            lock_path: resolve_lock_path(config_path),
            settings: entry_directory_check_settings(config_path, &config),
            lake_path: config.resolve_lake(config_path),
        })
    }
}

fn read_lock_if_exists(lock_path: &Path) -> Result<Option<SirnoLock>, CliError> {
    match SirnoLock::from_file(lock_path) {
        | Ok(lock) => Ok(Some(lock)),
        | Err(LockError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => Ok(None),
        | Err(source) => Err(CliError::Lock(source)),
    }
}

fn reject_immutable_checkout(lock_path: &Path) -> Result<(), CliError> {
    let Some(lock) = read_lock_if_exists(lock_path)? else {
        return Ok(());
    };
    if lock.history.is_checked_out() && !lock.history.is_unsafe_mutable_checkout() {
        return Err(CliError::ImmutableHistoryCheckout(lock.history.version));
    }
    Ok(())
}

fn history_version(version: u64) -> Result<Eterator, CliError> {
    if version == Eterator::EMPTY.version() {
        return Err(CliError::InvalidHistoryVersion(version));
    }
    Ok(Eterator(version))
}

fn run_witness_command(config_path: &Path, raw_id: &str, full: bool) -> Result<ExitCode, CliError> {
    let config = SirnoConfig::from_file(config_path)?;
    let id = EntryId::new(raw_id)?;
    let Some(settings) = witness_check_settings(config_path, &config) else {
        return Err(CliError::CodeMembersNotConfigured);
    };
    let index = scan_witnesses(&settings)?;
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
    let body = dedent_witness_body(&record.body);
    let range = format_witness_summary(record);
    if !full {
        let marker = body
            .lines()
            .next()
            .map(str::to_owned)
            .unwrap_or_else(|| dedent_witness_body(&record.marker));
        return format!("{range}\t{marker}\n");
    }

    let mut out = format!("{range}\n\n");
    out.push_str(&body);
    if !body.ends_with('\n') {
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

fn dedent_witness_body(body: &str) -> String {
    let indent = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(leading_whitespace_len)
        .min()
        .unwrap_or(0);
    let mut out =
        body.lines().map(|line| strip_indent(line, indent)).collect::<Vec<_>>().join("\n");
    if body.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn leading_whitespace_len(line: &str) -> usize {
    line.bytes().take_while(|byte| matches!(byte, b' ' | b'\t')).count()
}

fn strip_indent(line: &str, indent: usize) -> &str {
    let removable = leading_whitespace_len(line).min(indent);
    &line[removable..]
}

fn run_util_command(command: UtilCommand) -> Result<ExitCode, CliError> {
    match command {
        | UtilCommand::Completion { shell } => {
            let shell = Shell::from(shell);
            let mut command = Cli::command();
            let mut stdout = std::io::stdout();
            generate(shell, &mut command, "sirno", &mut stdout);
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn default_config_path() -> PathBuf {
    PathBuf::from(CONFIG_FILE_NAME)
}

fn default_lake_path() -> PathBuf {
    PathBuf::from("docs")
}

fn default_history_path() -> PathBuf {
    PathBuf::from("sirno-history")
}

fn explicit_entries_check_settings(
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
        links: config.links,
        ignore: config.lake.ignore.clone(),
        witness: witness_check_settings(config_path, config),
    }
}

fn witness_check_settings(
    config_path: &Path, config: &SirnoConfig,
) -> Option<WitnessCheckSettings> {
    let code = config.code.as_ref()?;
    if code.members.is_empty() {
        return None;
    }
    Some(WitnessCheckSettings::new(
        config_path.parent().unwrap_or_else(|| Path::new(".")),
        code.members.clone(),
    ))
}

fn resolve_entry_directory(
    entries: Option<PathBuf>, config_path: &std::path::Path,
) -> Result<(PathBuf, EntryDirectoryCheckSettings), CliError> {
    if let Some(entries) = entries {
        return Ok((entries, explicit_entries_check_settings(config_path)?));
    }

    let config = SirnoConfig::from_file(config_path)?;
    Ok((config.resolve_lake(config_path), entry_directory_check_settings(config_path, &config)))
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
    config_path: &std::path::Path, mono: Option<&std::path::Path>,
    history: Option<&std::path::Path>, lock: Option<&SirnoLock>, config: &SirnoConfig,
    report: &EntryDirectoryReport,
) {
    println!("config: {}", config_path.display());
    if let Some(mono) = mono {
        println!("mono: {}", mono.display());
    } else {
        println!("mono: (not configured)");
    }
    println!("lake: {}", report.root().display());
    if let Some(history) = history {
        println!("history: {}", history.display());
        println!("history-state: {}", history_state_label(lock));
    } else {
        println!("history: (not configured)");
    }
    println!("entries: {}", report.entries().len());
    println!("checks:");
    println!("  link: {}", config.check.link);
    println!("links:");
    println!("  category: {}", config.links.category);
    println!("  clustee: {}", config.links.clustee);
    println!("  clique: {}", config.links.clique);
    println!("  refiner: {}", config.links.refiner);
    if report.has_errors() {
        println!("check: failed");
        print_entry_directory_report(report);
    } else {
        println!("check: ok");
    }
}

fn history_state_label(lock: Option<&SirnoLock>) -> String {
    let Some(lock) = lock else {
        return "(unlocked)".to_owned();
    };
    match lock.history.status {
        | HistoryLockStatus::Current => {
            format!(
                "current version {} (generation {})",
                lock.history.version, lock.history.generation
            )
        }
        | HistoryLockStatus::CheckedOut if lock.history.mutable => {
            format!(
                "checked-out version {} (generation {}, unsafe mutable)",
                lock.history.version, lock.history.generation
            )
        }
        | HistoryLockStatus::CheckedOut => {
            format!(
                "checked-out version {} (generation {}, immutable)",
                lock.history.version, lock.history.generation
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
    report: &EntryDirectoryReport, entries: &[&Entry], format: CliQueryFormat,
) -> Result<(), CliError> {
    for entry in entries {
        let path = report
            .entry_path(&entry.id)
            .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
        match format {
            | CliQueryFormat::Summary => {
                println!("{}\t{}\t{}", entry.id, path.display(), entry.metadata.name);
            }
            | CliQueryFormat::Id => {
                println!("{}", entry.id);
            }
            | CliQueryFormat::Path => {
                println!("{}", path.display());
            }
        }
    }
    Ok(())
}

fn print_entry_directory_report(report: &EntryDirectoryReport) {
    if report.is_clean() {
        println!("ok: {}", report.root().display());
        return;
    }

    for diagnostic in report.file_diagnostics() {
        println!(
            "{}: {}: {}",
            severity_label(diagnostic.severity),
            diagnostic.path.display(),
            diagnostic.message
        );
    }

    for diagnostic in report.structural_report().diagnostics() {
        if let Some(path) = report.entry_path(&diagnostic.entry) {
            println!(
                "{}: {}: {}",
                severity_label(diagnostic.severity),
                path.display(),
                diagnostic.message()
            );
        } else {
            println!("{}: {}", severity_label(diagnostic.severity), diagnostic.message());
        }
    }
}

fn severity_label(severity: CheckSeverity) -> &'static str {
    match severity {
        | CheckSeverity::Warning => "warning",
        | CheckSeverity::Error => "error",
    }
}

/// Error raised while running the CLI.
#[derive(Debug, Error)]
enum CliError {
    /// History has already been configured at another path.
    #[error("history is already configured at {0}")]
    HistoryAlreadyConfigured(PathBuf),
    /// History is required for a history command but is not configured.
    #[error("history is not configured; run `sirno history init` first")]
    HistoryNotConfigured,
    /// Immutable history checkouts cannot be committed.
    #[error("history version {0} is checked out immutably; use checkout --unsafe-mutable first")]
    ImmutableHistoryCheckout(u64),
    /// Empty history cannot be checked out as a version.
    #[error("history version {0} is not a check-outable snapshot")]
    InvalidHistoryVersion(u64),
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
    /// Witness lookup requires configured code members.
    #[error("code members are not configured; add [code].members to Sirno.toml")]
    CodeMembersNotConfigured,
    /// Dry-run mode applies only to generated-link writing.
    #[error("`--dry` only applies to `sirno gen-link` without a subcommand")]
    DryWithGenLinkSubcommand,
    /// Config-backed command failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Lock-backed command failed.
    #[error(transparent)]
    Lock(#[from] LockError),
    /// History-store-backed command failed.
    #[error(transparent)]
    Store(#[from] StoreError),
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
        CONFIG_FILE_NAME, EntryId, HistorySettings, SirnoConfig, WitnessRecord, WitnessSpan,
    };

    use crate::{
        Cli, CliError, Command, HistoryCommand, format_gen_link_report, format_witness_record,
        format_witness_records, run,
    };

    #[test]
    fn init_does_not_accept_history_path() {
        let error =
            Cli::try_parse_from(["sirno", "init", "--history", "sirno-history"]).unwrap_err();

        assert!(error.to_string().contains("unexpected argument"));
    }

    #[test]
    fn history_init_accepts_history_path() {
        let cli = Cli::parse_from(["sirno", "history", "init", "--history", "sirno-history"]);

        assert!(matches!(
            cli.command,
            Command::History { command: HistoryCommand::Init { history: Some(_) } }
        ));
    }

    #[test]
    fn mv_accepts_lake_path() {
        let cli = Cli::parse_from(["sirno", "mv", "sirno-docs"]);

        assert!(matches!(cli.command, Command::Mv { lake } if lake == Path::new("sirno-docs")));
    }

    #[test]
    fn history_mv_accepts_history_path() {
        let cli = Cli::parse_from(["sirno", "history", "mv", "sirno-history-2"]);

        assert!(matches!(
            cli.command,
            Command::History { command: HistoryCommand::Mv { history } }
                if history == Path::new("sirno-history-2")
        ));
    }

    #[test]
    fn history_checkout_accepts_unsafe_mutable_flag() {
        let cli = Cli::parse_from(["sirno", "history", "checkout", "3", "--unsafe-mutable"]);

        assert!(matches!(
            cli.command,
            Command::History {
                command: HistoryCommand::Checkout { version: 3, unsafe_mutable: true }
            }
        ));
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

        run(Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "mv",
            "sirno-docs",
        ]))
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

        let error = run(Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "mv",
            "sirno-docs",
        ]))
        .unwrap_err();

        assert!(matches!(error, CliError::MoveDestinationExists(_)));
        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.lake.path, PathBuf::from("docs"));
        assert!(old_lake.exists());
    }

    #[test]
    fn history_mv_moves_history_and_rewrites_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let old_history = temp.path().join("sirno-history");
        let new_history = temp.path().join("history");
        SirnoConfig::new("docs").with_history("sirno-history").write_new(&config_path).unwrap();
        fs::create_dir(&old_history).unwrap();
        fs::write(old_history.join("row"), "history").unwrap();

        run(Cli::parse_from([
            "sirno",
            "--config",
            config_path.to_str().unwrap(),
            "history",
            "mv",
            "history",
        ]))
        .unwrap();

        let config = SirnoConfig::from_file(&config_path).unwrap();
        assert_eq!(config.history, Some(HistorySettings { path: PathBuf::from("history") }));
        assert!(!old_history.exists());
        assert!(new_history.join("row").exists());
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

    // sirno:witness:witness-fixture-isolation:begin
    #[test]
    fn format_witness_record_prints_range_and_dedents_body() {
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
            "src/lib.rs:10:5-33 :: 14:5-25\t// sample:start entry\n"
        );
        assert_eq!(
            format_witness_record(&record, true),
            concat!(
                "src/lib.rs:10:5-33 :: 14:5-25\n",
                "\n",
                "// sample:start entry\n",
                "    fn main() {}\n",
                "// sample:end\n",
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
            "// sample:end\n",
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
