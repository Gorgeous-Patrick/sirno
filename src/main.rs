//! Command-line interface for Sirno.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use eter::Eterator;
use sirno::{
    CONFIG_FILE_NAME, CheckMode, CheckSeverity, ConfigError, Entry, EntryDirectoryCheckSettings,
    EntryDirectoryError, EntryDirectoryReport, EntryDirectoryWritePolicy, EntryId, EntryIdError,
    EntryMetadata, EntryParseError, EntryQuery, GenLinkDirectoryReport, GeneratedLinkSettings,
    HistoryLockStatus, LockError, SirnoConfig, SirnoLock, SirnoStore, StoreError, VagueEntryQuery,
    WitnessCheckSettings, WitnessError, WitnessMarker, add_readonly_checkout_warnings,
    check_entry_directory_with_settings, check_gen_link_entry_directory_with_ignored_paths,
    create_entry_file, delete_gen_link_entry_directory_with_ignored_paths,
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
    Init {
        /// Monograph path written to Sirno.toml.
        #[arg(long)]
        mono: Option<PathBuf>,
        /// Public Markdown entry store path written to Sirno.toml.
        #[arg(long)]
        store: Option<PathBuf>,
    },
    /// Create one Markdown entry.
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
    /// Query public Markdown entries.
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
    /// Check current store structure.
    Check {
        /// Eter-backed entry store root.
        #[arg(long, conflicts_with = "entries")]
        store: Option<PathBuf>,
        /// Public Markdown entry directory.
        #[arg(long, conflicts_with = "store")]
        entries: Option<PathBuf>,
        /// Check boundary.
        #[arg(long, value_enum)]
        mode: Option<CliCheckMode>,
    },
    /// Generate Markdown links in entry footers.
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
    /// Show the current Sirno project status.
    Status,
    /// Show repository witness markers for one entry id.
    Witness {
        /// Entry id used as the witness query key.
        id: String,
    },
    /// Manage optional eter-backed history.
    History {
        /// History command.
        #[command(subcommand)]
        command: HistoryCommand,
    },
    /// Utility commands.
    Util {
        /// Utility command.
        #[command(subcommand)]
        command: UtilCommand,
    },
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
    /// Configure history and commit the current public Markdown store.
    Init {
        /// Private eter history store path written to Sirno.toml.
        #[arg(long)]
        history: Option<PathBuf>,
    },
    /// Commit the current public Markdown store into history.
    Commit,
    /// Check out one history version into the public Markdown store.
    Checkout {
        /// Raw Eterator version to materialize.
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
        | Command::Init { mono, store } => {
            let config = SirnoConfig::new(
                mono.unwrap_or_else(default_mono_path),
                store.unwrap_or_else(default_store_path),
            );
            let store_path = config.resolve_store(&config_path);
            config.write_new(&config_path)?;
            let paths = init_entry_directory(&store_path)?;
            println!(
                "initialized {} with {} entries in {}",
                config_path.display(),
                paths.len(),
                store_path.display()
            );
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
                    config.resolve_store(&config_path)
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
        | Command::Check { store, entries, mode } => {
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

            let Some(store) = store else {
                let config = SirnoConfig::from_file(&config_path)?;
                let report = check_entry_directory_with_settings(
                    config.resolve_store(&config_path),
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

            let store = SirnoStore::open(store)?;
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
            let store = config.resolve_store(&config_path);
            let report = check_entry_directory_with_settings(
                &store,
                CheckMode::Review,
                &entry_directory_check_settings(&config_path, &config),
            )?;
            print_status(&config_path, &mono, history.as_deref(), lock.as_ref(), &config, &report);
            if report.has_errors() { Ok(ExitCode::FAILURE) } else { Ok(ExitCode::SUCCESS) }
        }
        | Command::Witness { id } => run_witness_command(&config_path, &id),
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
            let store_path = config.resolve_store(config_path);
            let mut store = SirnoStore::open(&history_root)?;
            let version = store.commit_entry_directory(
                &store_path,
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
                store_path.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        | HistoryCommand::Commit => {
            let context = HistoryContext::load(config_path)?;
            reject_immutable_checkout(&context.lock_path)?;
            let mut store = SirnoStore::open(&context.history_root)?;
            let version = store.commit_entry_directory(&context.store_path, &context.settings)?;
            set_entry_directory_writable(&context.store_path, &context.settings)?;
            SirnoLock::current(version).write(&context.lock_path)?;
            println!(
                "committed history version {} from {}",
                version.version(),
                context.store_path.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        | HistoryCommand::Checkout { version, unsafe_mutable } => {
            let context = HistoryContext::load(config_path)?;
            let version = history_version(version)?;
            let store = SirnoStore::open(&context.history_root)?;
            let paths = store.checkout_entry_directory(
                version,
                &context.store_path,
                EntryDirectoryWritePolicy::ReplaceDirectory {
                    ignore: context.settings.ignore.clone(),
                },
            )?;
            if unsafe_mutable {
                set_entry_directory_writable(&context.store_path, &context.settings)?;
            } else {
                add_readonly_checkout_warnings(&paths)?;
                set_entry_directory_readonly(&context.store_path, &context.settings)?;
            }
            SirnoLock::checked_out(version, unsafe_mutable).write(&context.lock_path)?;
            println!(
                "checked out history version {} into {} ({} entries, {})",
                version.version(),
                context.store_path.display(),
                paths.len(),
                if unsafe_mutable { "unsafe mutable" } else { "immutable" }
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

struct HistoryContext {
    history_root: PathBuf,
    lock_path: PathBuf,
    settings: EntryDirectoryCheckSettings,
    store_path: PathBuf,
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
            store_path: config.resolve_store(config_path),
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

fn run_witness_command(config_path: &Path, raw_id: &str) -> Result<ExitCode, CliError> {
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
    for record in records {
        println!("{}:{}:{}\t{}", record.path.display(), record.line, record.column, record.marker);
    }
    Ok(ExitCode::SUCCESS)
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

fn default_mono_path() -> PathBuf {
    PathBuf::from("DESIGN.md")
}

fn default_store_path() -> PathBuf {
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
        ignore: config.store.ignore.clone(),
        witness: witness_check_settings(config_path, config),
    }
}

fn witness_check_settings(
    config_path: &Path, config: &SirnoConfig,
) -> Option<WitnessCheckSettings> {
    if config.code.members.is_empty() {
        return None;
    }
    Some(WitnessCheckSettings::new(
        config_path.parent().unwrap_or_else(|| Path::new(".")),
        config.code.members.clone(),
    ))
}

fn resolve_entry_directory(
    entries: Option<PathBuf>, config_path: &std::path::Path,
) -> Result<(PathBuf, EntryDirectoryCheckSettings), CliError> {
    if let Some(entries) = entries {
        return Ok((entries, explicit_entries_check_settings(config_path)?));
    }

    let config = SirnoConfig::from_file(config_path)?;
    Ok((config.resolve_store(config_path), entry_directory_check_settings(config_path, &config)))
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
    config_path: &std::path::Path, mono: &std::path::Path, history: Option<&std::path::Path>,
    lock: Option<&SirnoLock>, config: &SirnoConfig, report: &EntryDirectoryReport,
) {
    println!("config: {}", config_path.display());
    println!("mono: {}", mono.display());
    println!("store: {}", report.root().display());
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
        | HistoryLockStatus::Current => format!("current version {}", lock.history.version),
        | HistoryLockStatus::CheckedOut if lock.history.mutable => {
            format!("checked-out version {} (unsafe mutable)", lock.history.version)
        }
        | HistoryLockStatus::CheckedOut => {
            format!("checked-out version {} (immutable)", lock.history.version)
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
    /// Store-backed command failed.
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
    use std::path::{Path, PathBuf};

    use clap::Parser;

    use crate::{Cli, Command, HistoryCommand, format_gen_link_report};

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
    fn witness_accepts_entry_id() {
        let cli = Cli::parse_from(["sirno", "witness", "witness"]);

        assert!(matches!(cli.command, Command::Witness { id } if id == "witness"));
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

        assert_eq!(report, "No changes in sirno-docs.");
    }
}
