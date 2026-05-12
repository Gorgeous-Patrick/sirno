//! Command-line interface for Sirno.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use sirno::{
    CONFIG_FILE_NAME, CheckMode, CheckSeverity, ConfigError, Entry, EntryDirectoryCheckSettings,
    EntryDirectoryError, EntryDirectoryReport, EntryId, EntryIdError, EntryMetadata,
    EntryParseError, EntryQuery, GeneratedLinkSettings, SirnoConfig, SirnoStore, StoreError,
    VagueEntryQuery, WitnessMarker, check_entry_directory_with_settings, create_entry_file,
    delete_gen_link_entry_directory_with_ignored_paths,
    gen_link_entry_directory_with_ignored_paths, init_entry_directory, query_entries,
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
        /// Category relation target.
        #[arg(long)]
        category: Vec<String>,
        /// Clique closure relation target.
        #[arg(long)]
        clustee: Vec<String>,
        /// Refined entry relation target.
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
        /// Vague text terms matched against entries and relation target summaries.
        terms: Vec<String>,
        /// Exact text term matched against id, name, description, and body.
        #[arg(long = "exact-term")]
        exact_terms: Vec<String>,
        /// Exact category relation target.
        #[arg(long)]
        exact_category: Vec<String>,
        /// Exact clique closure relation target.
        #[arg(long)]
        exact_clustee: Vec<String>,
        /// Exact refined entry relation target.
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
        /// Generated-link command.
        #[command(subcommand)]
        command: Option<GenLinkCommand>,
        /// Public Markdown entry directory.
        #[arg(long)]
        entries: Option<PathBuf>,
    },
    /// Show the current Sirno project status.
    Status,
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
                    &entry_directory_check_settings(&config),
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
        | Command::GenLink { command, entries } => match command {
            | None => {
                let (entries, mut settings) = resolve_entry_directory(entries, &config_path)?;
                settings.link = false;

                let check =
                    check_entry_directory_with_settings(&entries, CheckMode::Review, &settings)?;
                if check.has_errors() {
                    print_entry_directory_report(&check);
                    return Ok(ExitCode::FAILURE);
                }

                let report = gen_link_entry_directory_with_ignored_paths(
                    &entries,
                    &settings.links,
                    settings.ignore.clone(),
                )?;
                println!(
                    "generated links for {} entries in {} ({} changed)",
                    report.entry_count(),
                    report.root().display(),
                    report.changed_paths().len()
                );
                Ok(ExitCode::SUCCESS)
            }
            | Some(GenLinkCommand::Delete { entries: delete_entries }) => {
                let (entries, settings) =
                    resolve_entry_directory(delete_entries.or(entries), &config_path)?;

                let report =
                    delete_gen_link_entry_directory_with_ignored_paths(&entries, settings.ignore)?;
                println!(
                    "deleted generated links from {} entries in {} ({} changed)",
                    report.entry_count(),
                    report.root().display(),
                    report.changed_paths().len()
                );
                Ok(ExitCode::SUCCESS)
            }
        },
        | Command::Status => {
            let config = SirnoConfig::from_file(&config_path)?;
            let mono = config.resolve_mono(&config_path);
            let store = config.resolve_store(&config_path);
            let report = check_entry_directory_with_settings(
                &store,
                CheckMode::Review,
                &entry_directory_check_settings(&config),
            )?;
            print_status(&config_path, &mono, &config, &report);
            if report.has_errors() { Ok(ExitCode::FAILURE) } else { Ok(ExitCode::SUCCESS) }
        }
        | Command::Util { command } => run_util_command(command),
    }
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

fn explicit_entries_check_settings(
    config_path: &std::path::Path,
) -> Result<EntryDirectoryCheckSettings, CliError> {
    if config_path.exists() {
        let config = SirnoConfig::from_file(config_path)?;
        Ok(entry_directory_check_settings(&config))
    } else {
        Ok(EntryDirectoryCheckSettings::default())
    }
}

fn entry_directory_check_settings(config: &SirnoConfig) -> EntryDirectoryCheckSettings {
    EntryDirectoryCheckSettings {
        link: config.check.link,
        links: config.links,
        ignore: config.store.ignore.clone(),
    }
}

fn resolve_entry_directory(
    entries: Option<PathBuf>, config_path: &std::path::Path,
) -> Result<(PathBuf, EntryDirectoryCheckSettings), CliError> {
    if let Some(entries) = entries {
        return Ok((entries, explicit_entries_check_settings(config_path)?));
    }

    let config = SirnoConfig::from_file(config_path)?;
    Ok((config.resolve_store(config_path), entry_directory_check_settings(&config)))
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
    config_path: &std::path::Path, mono: &std::path::Path, config: &SirnoConfig,
    report: &EntryDirectoryReport,
) {
    println!("config: {}", config_path.display());
    println!("mono: {}", mono.display());
    println!("store: {}", report.root().display());
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

    for diagnostic in report.relation_report().diagnostics() {
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
    /// Config-backed command failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Store-backed command failed.
    #[error(transparent)]
    Store(#[from] StoreError),
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
