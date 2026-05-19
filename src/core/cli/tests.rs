use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{CommandFactory, Parser};

use crate::core::dto::{
    ConfigCommentResult, DiagnosticRecord, LakeCheckResult, RenderResult, SkillWrapperRecord,
};

use super::OpenTideTutorial;

use crate::{
    CONFIG_FILE_NAME, Entry, EntryId, EntryMetadata, EntryQuery, Eterator, FrostError,
    FrostLockStatus, FrostSettings, LOCK_FILE_NAME, RepoMember, RepoSettings, SirnoConfig,
    SirnoFrost, SirnoLock, StructuralEdgeDirection, StructuralEdgeSettings,
    StructuralFieldSettings, StructuralRippleSettings, StructuralSettings, TideSource, TideStatus,
    TideWorkitem, TutorialSettings, WitnessRecord, WitnessSpan,
};

use super::{
    ArtifactCommand, CheckModeArg, CheckoutArgs, Cli, Command, CommandError, ConfigCommentArgs,
    CoreContext, EntryCommand, EntryNewRequest, EntryPathArgs, EntryRenameArgs, FrostCommand,
    FrostMoveArgs, LakeCommand, LakeInitRequest, LakeMoveArgs, MoveCommand, PathOutputFormat,
    QueryColumn, QueryColumns, QueryOutputFormat, ResolveArgs, SkillCommand, StructuralFieldState,
    StructuralFilter, StructuralPredicate, StructuralStateFilter, TideCommand, TideItemSelector,
    TideOutputFormat, TideReviewCommand, TideStatusGrouping, TideStatusMode, TopLevelEntryCommand,
    TopLevelFrostCommand, TopLevelLakeCommand, UnresolveArgs, UtilCommand, entry_path_records,
    entry_query_from_filters, format_config_comment_result, format_gen_link_report,
    format_human_table_with_width, format_json, format_lake_check_result, format_path_table,
    format_query_json, format_query_table, format_render_result, format_skill_wrapper_table,
    format_tide_review_entries, format_tide_review_waves, format_tide_statuses,
    format_tide_statuses_by_entry, format_witness_record, format_witness_records,
    rg_args_include_preprocessor,
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

    Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "init"]).run().unwrap();

    let config = SirnoConfig::from_file(&config_path).unwrap();
    let lock = SirnoLock::from_file(repo.join(LOCK_FILE_NAME)).unwrap();
    assert_eq!(config.lake.path, PathBuf::from("alpha-project-lake"));
    assert_eq!(config.frost, Some(FrostSettings { path: PathBuf::from("alpha-project-frost") }));
    assert!(repo.join("alpha-project-lake").join("concept.md").exists());
    assert!(repo.join("alpha-project-frost").join("Eter.lock.toml").exists());
    assert!(
        repo.join(".agents").join("skills").join("sirno-config-writer").join("SKILL.md").exists()
    );
    assert!(repo.join(".agents").join("skills").join("sirno-editor").join("SKILL.md").exists());
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
        "--lake",
        "custom-lake",
        "--frost",
        "custom-frost",
    ])
    .run()
    .unwrap();

    let config = SirnoConfig::from_file(&config_path).unwrap();
    assert_eq!(config.lake.path, PathBuf::from("custom-lake"));
    assert_eq!(config.frost.unwrap().path, PathBuf::from("custom-frost"));
    assert!(temp.path().join("custom-lake").join("concept.md").exists());
    assert!(temp.path().join("custom-frost").join("Eter.lock.toml").exists());
    assert!(
        temp.path().join(".agents").join("skills").join("sirno-editor").join("SKILL.md").exists()
    );
}

#[test]
fn top_level_init_can_skip_skills() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    let repo_name = temp.path().file_name().unwrap().to_string_lossy();
    let lake = PathBuf::from(format!("{repo_name}-lake"));
    let frost = PathBuf::from(format!("{repo_name}-frost"));

    Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "init", "--no-skills"])
        .run()
        .unwrap();

    let config = SirnoConfig::from_file(&config_path).unwrap();
    let configured_frost = config.frost.as_ref().unwrap().path.clone();
    assert_eq!(config.lake.path, lake);
    assert_eq!(configured_frost, frost);
    assert!(temp.path().join(&config.lake.path).join("concept.md").exists());
    assert!(temp.path().join(configured_frost).join("Eter.lock.toml").exists());
    assert!(!temp.path().join(".agents").join("skills").exists());
}

#[test]
fn top_level_init_can_skip_frost_and_skills() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    let repo_name = temp.path().file_name().unwrap().to_string_lossy();
    let lake = PathBuf::from(format!("{repo_name}-lake"));

    Cli::parse_from([
        "sirno",
        "--config",
        config_path.to_str().unwrap(),
        "init",
        "--no-frost",
        "--no-skills",
    ])
    .run()
    .unwrap();

    let config = SirnoConfig::from_file(&config_path).unwrap();
    assert_eq!(config.lake.path, lake);
    assert!(config.frost.is_none());
    assert!(temp.path().join(&config.lake.path).join("concept.md").exists());
    assert!(!temp.path().join(LOCK_FILE_NAME).exists());
    assert!(!temp.path().join(".agents").join("skills").exists());
}

#[test]
fn top_level_init_can_skip_lake_and_skills() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    let repo_name = temp.path().file_name().unwrap().to_string_lossy();
    let lake = PathBuf::from(format!("{repo_name}-lake"));
    let frost = PathBuf::from(format!("{repo_name}-frost"));

    Cli::parse_from([
        "sirno",
        "--config",
        config_path.to_str().unwrap(),
        "init",
        "--no-lake",
        "--no-skills",
    ])
    .run()
    .unwrap();

    let config = SirnoConfig::from_file(&config_path).unwrap();
    let configured_frost = config.frost.as_ref().unwrap().path.clone();
    assert_eq!(config.lake.path, lake);
    assert_eq!(configured_frost, frost);
    assert!(!temp.path().join(&config.lake.path).exists());
    assert!(temp.path().join(configured_frost).join("Eter.lock.toml").exists());
    assert!(temp.path().join(LOCK_FILE_NAME).exists());
    assert!(!temp.path().join(".agents").join("skills").exists());
}

#[test]
fn top_level_init_can_skip_lake_and_frost() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);

    Cli::parse_from([
        "sirno",
        "--config",
        config_path.to_str().unwrap(),
        "init",
        "--no-lake",
        "--no-frost",
    ])
    .run()
    .unwrap();

    assert!(!config_path.exists());
    assert!(!temp.path().join("sirno-frost").exists());
    assert!(
        temp.path().join(".agents").join("skills").join("sirno-editor").join("SKILL.md").exists()
    );
}

#[test]
fn top_level_init_rejects_path_flags_for_disabled_parts() {
    let no_lake_with_lake = Cli::try_parse_from(["sirno", "init", "--no-lake", "--lake", "docs"]);
    let no_frost_with_frost =
        Cli::try_parse_from(["sirno", "init", "--no-frost", "--frost", "sirno-frost"]);

    assert!(no_lake_with_lake.is_err());
    assert!(no_frost_with_frost.is_err());
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

    let init = context.lake_init(LakeInitRequest { lake: Some(PathBuf::from("docs")) }).unwrap();
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
fn top_level_version_flag_reports_package_version() {
    let error = Cli::try_parse_from(["sirno", "--version"]).unwrap_err();

    assert_eq!(error.kind(), clap::error::ErrorKind::DisplayVersion);
    assert_eq!(error.to_string(), format!("sirno {}\n", env!("CARGO_PKG_VERSION")));
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
fn util_config_accepts_check_and_fix_form() {
    let check = Cli::parse_from(["sirno", "util", "config"]);
    let fix = Cli::parse_from(["sirno", "util", "config", "--fix"]);

    assert!(matches!(
        check.command,
        Command::Util { command: UtilCommand::Config(ConfigCommentArgs { fix: false }) }
    ));
    assert!(matches!(
        fix.command,
        Command::Util { command: UtilCommand::Config(ConfigCommentArgs { fix: true }) }
    ));
}

#[test]
fn util_config_check_reports_missing_comments_without_writing() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    SirnoConfig::new("docs").write_new(&config_path).unwrap();
    let uncommented = fs::read_to_string(&config_path)
        .unwrap()
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&config_path, format!("{uncommented}\n")).unwrap();

    let result = CoreContext::new(&config_path).config_comments_check().unwrap();

    assert!(!result.ok);
    assert!(!result.changed);
    assert!(
        result.missing_comments.contains(
            &"Markdown entry lake path, resolved relative to this config file.".to_owned()
        )
    );
    assert!(!fs::read_to_string(&config_path).unwrap().contains("# Markdown entry lake path"));
}

#[test]
fn util_config_fix_writes_missing_comments() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    SirnoConfig::new("docs").write_new(&config_path).unwrap();
    let uncommented = fs::read_to_string(&config_path)
        .unwrap()
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&config_path, format!("{uncommented}\n")).unwrap();

    let fix = CoreContext::new(&config_path).config_comments_fix().unwrap();
    let check = CoreContext::new(&config_path).config_comments_check().unwrap();

    assert!(fix.ok);
    assert!(fix.changed);
    assert!(check.ok);
    assert!(check.missing_comments.is_empty());
    assert!(fs::read_to_string(&config_path).unwrap().contains("# Markdown entry lake path"));
}

#[test]
fn util_config_rejects_lake_and_frost_path_overrides() {
    let lake_error =
        Cli::parse_from(["sirno", "--lake-path", "docs", "util", "config"]).run().unwrap_err();
    let frost_error =
        Cli::parse_from(["sirno", "--frost-path", "frost", "util", "config"]).run().unwrap_err();

    assert!(matches!(lake_error, CommandError::ConfigRejectsLakePath));
    assert!(matches!(frost_error, CommandError::ConfigRejectsFrostPath));
}

#[test]
fn util_skills_init_accepts_nested_command() {
    let cli = Cli::parse_from(["sirno", "util", "skills", "init"]);

    assert!(matches!(
        cli.command,
        Command::Util { command: UtilCommand::Skills { command: SkillCommand::Init } }
    ));
}

#[test]
fn util_skills_init_installs_bundled_wrappers() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    let context = CoreContext::new(&config_path);

    let init = context.skill_wrappers_init().unwrap();
    let target = temp.path().join(".agents").join("skills").join("sirno-editor").join("SKILL.md");
    let check = context.skill_wrappers_check().unwrap();

    assert!(init.ok);
    assert_eq!(init.records.len(), 6);
    assert_eq!(init.records[0].status, "wrote");
    assert!(fs::read_to_string(target).unwrap().contains("sirno://skills/sirno-editor"));
    assert!(check.ok);
    assert_eq!(check.records[0].status, "ok");
}

#[test]
fn util_skills_check_reports_drift_without_writing() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    let context = CoreContext::new(&config_path);
    context.skill_wrappers_init().unwrap();
    let target = temp.path().join(".agents").join("skills").join("sirno-editor").join("SKILL.md");
    fs::write(&target, "local edit\n").unwrap();

    let check = context.skill_wrappers_check().unwrap();
    let drifted = check.records.iter().find(|record| record.status == "drifted").unwrap();

    assert!(!check.ok);
    assert_eq!(drifted.target_path, ".agents/skills/sirno-editor/SKILL.md");
    assert_eq!(fs::read_to_string(target).unwrap(), "local edit\n");
}

#[test]
fn util_skills_rejects_global_lake_path() {
    let error = Cli::parse_from(["sirno", "--lake-path", "docs", "util", "skills", "check"])
        .run()
        .unwrap_err();

    assert!(matches!(error, CommandError::SkillsRejectsLakePath));
}

#[test]
fn skill_wrapper_output_uses_table() {
    let table = format_skill_wrapper_table(&[SkillWrapperRecord {
        entry_id: "lake-editing-discipline".to_owned(),
        name: "sirno-editor".to_owned(),
        wrapper_path: "sirno-docs/.artifacts/lake-editing-discipline/SKILL.md".to_owned(),
        full_path: "sirno-docs/.artifacts/lake-editing-discipline/SKILL.full.md".to_owned(),
        target_path: ".agents/skills/sirno-editor/SKILL.md".to_owned(),
        status: "ok".to_owned(),
        changed: false,
    }]);

    assert!(table.contains("status"));
    assert!(table.contains("name"));
    assert!(table.contains("target"));
    assert!(table.contains("sirno-editor"));
    assert!(table.contains(".agents/skills/sirno-editor/SKILL.md"));
    assert!(!table.contains("wrapper"));
    assert!(!table.contains('\t'));
}

#[test]
fn top_level_mcp_is_not_a_command() {
    let error = Cli::try_parse_from(["sirno", "mcp"]).unwrap_err();

    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
}

#[test]
fn util_mcp_rejects_global_lake_path() {
    let error = Cli::parse_from(["sirno", "--lake-path", "docs", "util", "mcp"]).run().unwrap_err();

    assert!(matches!(error, CommandError::McpRejectsLakePath));
}

#[test]
fn util_mcp_rejects_global_frost_path() {
    let error =
        Cli::parse_from(["sirno", "--frost-path", "sirno-frost", "util", "mcp"]).run().unwrap_err();

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

    assert_eq!(config.frost, Some(FrostSettings { path: PathBuf::from("frost-project-frost") }));
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

    let error =
        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "commit"])
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
    let error =
        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "commit"])
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
    let error = Cli::try_parse_from(["sirno", "frost", "checkout", "3", "--latest"]).unwrap_err();

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
fn tide_status_accepts_show_modes() {
    let review = Cli::parse_from(["sirno", "tide", "status"]);
    let full = Cli::parse_from(["sirno", "tide", "status", "--show", "full"]);
    let all = Cli::parse_from(["sirno", "tide", "status", "--show=all"]);

    assert!(matches!(
        review.command,
        Command::Tide { command: TideCommand::Status { show: TideStatusMode::Review, .. } }
    ));
    assert!(matches!(
        full.command,
        Command::Tide { command: TideCommand::Status { show: TideStatusMode::Full, .. } }
    ));
    assert!(matches!(
        all.command,
        Command::Tide { command: TideCommand::Status { show: TideStatusMode::All, .. } }
    ));
}

#[test]
fn tide_status_accepts_grouping_modes() {
    let default = Cli::parse_from(["sirno", "tide", "status"]);
    let wave = Cli::parse_from(["sirno", "tide", "status", "--by", "wave"]);
    let entry = Cli::parse_from(["sirno", "tide", "status", "--by", "entry"]);
    let full_entry =
        Cli::parse_from(["sirno", "tide", "status", "--show", "full", "--by", "entry"]);

    assert!(matches!(
        default.command,
        Command::Tide { command: TideCommand::Status { by: TideStatusGrouping::Entry, .. } }
    ));
    assert!(matches!(
        wave.command,
        Command::Tide { command: TideCommand::Status { by: TideStatusGrouping::Wave, .. } }
    ));
    assert!(matches!(
        entry.command,
        Command::Tide { command: TideCommand::Status { by: TideStatusGrouping::Entry, .. } }
    ));
    assert!(matches!(
        full_entry.command,
        Command::Tide {
            command: TideCommand::Status {
                show: TideStatusMode::Full,
                by: TideStatusGrouping::Entry,
                ..
            }
        }
    ));
}

#[test]
fn tide_status_rejects_reason_grouping() {
    let error = Cli::try_parse_from(["sirno", "tide", "status", "--by", "reason"]).unwrap_err();

    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
}

#[test]
fn tide_status_rejects_old_full_and_all_flags() {
    let full = Cli::try_parse_from(["sirno", "tide", "status", "--full"]).unwrap_err();
    let all = Cli::try_parse_from(["sirno", "tide", "status", "--all"]).unwrap_err();

    assert_eq!(full.kind(), clap::error::ErrorKind::UnknownArgument);
    assert_eq!(all.kind(), clap::error::ErrorKind::UnknownArgument);
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
    let error = Cli::try_parse_from(["sirno", "frost", "checkout", "--latest", "--unsafe-mutable"])
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
    let error =
        Cli::try_parse_from(["sirno", "new", "alpha", "-d", "Alpha desc.", "-x", "topic=concept"])
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
    let error = Cli::try_parse_from(["sirno", "check", "--frost-root", "sirno-frost"]).unwrap_err();

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
    let json =
        format_query_json(&columns, &[vec!["query".to_owned(), "Selection".to_owned()]]).unwrap();
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
    let table = format_query_table(&columns, &[vec!["query".to_owned(), "Selection".to_owned()]]);

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
    let table = format_query_table(&columns, &[vec!["界界".to_owned()], vec!["aaa".to_owned()]]);

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

fn tide_status_fixture(
    ripple: &str, field: &str, direction: StructuralEdgeDirection, neighbor: &str,
    sources: &[TideSource], resolved: bool,
) -> TideStatus {
    TideStatus {
        workitem: TideWorkitem::new(
            EntryId::new(ripple).unwrap(),
            field,
            direction,
            EntryId::new(neighbor).unwrap(),
        )
        .unwrap(),
        sources: sources.iter().copied().collect(),
        fingerprint: format!("{ripple}-{neighbor}"),
        resolved,
    }
}

fn heavy_wave_separator_count(output: &str) -> usize {
    let mut header_separator_seen = false;
    output
        .lines()
        .filter(|line| {
            if !line.starts_with('╞') {
                return false;
            }
            if header_separator_seen {
                true
            } else {
                header_separator_seen = true;
                false
            }
        })
        .count()
}

#[test]
fn tide_review_waves_merge_into_one_table() {
    let statuses = vec![
        tide_status_fixture(
            "interfaces",
            "belongs",
            StructuralEdgeDirection::Clique,
            "agent-skills",
            &[TideSource::Lake],
            false,
        ),
        tide_status_fixture(
            "interfaces",
            "belongs",
            StructuralEdgeDirection::Clique,
            "form",
            &[TideSource::Lake],
            false,
        ),
        tide_status_fixture(
            "tide",
            "refines",
            StructuralEdgeDirection::From,
            "wave",
            &[TideSource::Lake, TideSource::Frost],
            false,
        ),
    ];

    let output = format_tide_review_waves(&statuses);

    assert!(
        output.contains("The tide has 3 open workitems in 2 waves, with 3 unique review entries.")
    );
    assert!(!output.contains("review entries:"));
    assert_eq!(output.matches('┌').count(), 1);
    assert_eq!(heavy_wave_separator_count(&output), 1);
    assert!(output.contains("│ wave       ┆ entry"));
    assert!(output.contains("│ interfaces ┆ agent-skills │"));
    assert!(output.contains("│            ┆ form"));
    assert!(output.contains("│ tide       ┆ wave"));
    assert_before(&output, "│ tide       ┆ wave", "The tide has 3 open workitems");
}

#[test]
fn tide_review_entries_group_by_review_entry() {
    let statuses = vec![
        tide_status_fixture(
            "interfaces",
            "belongs",
            StructuralEdgeDirection::Clique,
            "agent-skills",
            &[TideSource::Lake],
            false,
        ),
        tide_status_fixture(
            "tide",
            "refines",
            StructuralEdgeDirection::From,
            "agent-skills",
            &[TideSource::Frost],
            false,
        ),
        tide_status_fixture(
            "tide",
            "belongs",
            StructuralEdgeDirection::To,
            "form",
            &[TideSource::Lake],
            false,
        ),
    ];

    let output = format_tide_review_entries(&statuses);

    assert!(
        output.contains("The tide has 3 open workitems in 2 waves, with 2 unique review entries.")
    );
    assert_eq!(output.matches('┌').count(), 1);
    assert_eq!(heavy_wave_separator_count(&output), 1);
    assert!(output.contains("│ entry        ┆ reason"));
    assert!(output.contains("│ agent-skills ┆ interfaces"));
    assert!(output.contains("│              ┆ tide"));
    assert!(output.contains("│ form         ┆ tide"));
    assert_before(&output, "│ form         ┆ tide", "The tide has 3 open workitems");
}

#[test]
fn tide_full_statuses_group_by_wave() {
    let statuses = vec![
        tide_status_fixture(
            "interfaces",
            "belongs",
            StructuralEdgeDirection::Clique,
            "agent-skills",
            &[TideSource::Lake],
            false,
        ),
        tide_status_fixture(
            "tide",
            "refines",
            StructuralEdgeDirection::From,
            "wave",
            &[TideSource::Lake, TideSource::Frost],
            false,
        ),
        tide_status_fixture(
            "tide",
            "belongs",
            StructuralEdgeDirection::To,
            "frost-versioning",
            &[TideSource::Lake],
            true,
        ),
    ];

    let output = format_tide_statuses(&statuses);

    assert!(output.contains(
        "The tide has 2 open workitems and 1 resolved workitem in 2 waves, \
         with 2 unique review entries."
    ));
    assert!(!output.contains("review entries:"));
    assert_eq!(output.matches('┌').count(), 1);
    assert_eq!(heavy_wave_separator_count(&output), 1);
    assert!(output.contains("│ wave       ┆ entry"));
    assert!(output.contains("┆ state"));
    assert!(output.contains("│ interfaces ┆ agent-skills"));
    assert!(output.contains("│ tide       ┆ wave"));
    assert!(output.contains("│            ┆ frost-versioning"));
    assert!(output.contains("lake,frost"));
    assert!(output.contains("resolved"));
    assert_before(&output, "│            ┆ frost-versioning", "The tide has 2 open workitems");
}

#[test]
fn tide_full_statuses_group_by_review_entry() {
    let statuses = vec![
        tide_status_fixture(
            "interfaces",
            "belongs",
            StructuralEdgeDirection::Clique,
            "agent-skills",
            &[TideSource::Lake],
            false,
        ),
        tide_status_fixture(
            "tide",
            "refines",
            StructuralEdgeDirection::From,
            "agent-skills",
            &[TideSource::Lake, TideSource::Frost],
            false,
        ),
        tide_status_fixture(
            "tide",
            "belongs",
            StructuralEdgeDirection::To,
            "frost-versioning",
            &[TideSource::Lake],
            true,
        ),
    ];

    let output = format_tide_statuses_by_entry(&statuses);

    assert!(output.contains(
        "The tide has 2 open workitems and 1 resolved workitem in 2 waves, \
         with 1 unique review entry."
    ));
    assert_eq!(output.matches('┌').count(), 1);
    assert_eq!(heavy_wave_separator_count(&output), 1);
    assert!(output.contains("│ entry"));
    assert!(output.contains("┆ reason"));
    assert!(output.contains("│ agent-skills"));
    assert!(output.contains("┆ interfaces"));
    assert!(output.contains("┆ tide"));
    assert!(output.contains("│ frost-versioning"));
    assert!(output.contains("lake,frost"));
    assert!(output.contains("resolved"));
    assert_before(&output, "│ frost-versioning", "The tide has 2 open workitems");
}

#[test]
fn query_rejects_old_exact_structural_flags() {
    let error = Cli::try_parse_from(["sirno", "query", "--exact", "topic=concept"]).unwrap_err();

    assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);

    let error = Cli::try_parse_from(["sirno", "query", "-x", "topic=concept"]).unwrap_err();

    assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);

    let error = Cli::try_parse_from(["sirno", "query", "--exact-topic", "concept"]).unwrap_err();

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
        Command::TopLevelLake(TopLevelLakeCommand::Check { mode: Some(CheckModeArg::Review), .. })
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
    assert!(!rg_args_include_preprocessor(&[OsString::from("--pre-glob"), OsString::from("*.md")]));
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

    assert!(matches!(error, CommandError::UnconfiguredStructuralField(field) if field == "topic"));
}

#[test]
fn query_filter_keeps_comma_separated_targets_disjunctive() {
    let mut metadata = EntryMetadata::new("Concept", "A named idea.").unwrap();
    metadata.push_structural_target("topic", EntryId::new("meta").unwrap());
    let entry = Entry::new(EntryId::new("concept").unwrap(), metadata, "");
    let settings = StructuralSettings::from_fields([("topic", StructuralFieldSettings::default())]);
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
    let settings = StructuralSettings::from_fields([("topic", StructuralFieldSettings::default())]);
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
    let settings = StructuralSettings::from_fields([("topic", StructuralFieldSettings::default())]);
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
    let settings = StructuralSettings::from_fields([("topic", StructuralFieldSettings::default())]);
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
    let error =
        Cli::try_parse_from(["sirno", "freeze", "alpha", "--entries", "scratch-docs"]).unwrap_err();

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
fn lake_mv_creates_destination_parent() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    let old_lake = temp.path().join("docs");
    let new_lake = temp.path().join("sirno-lakes").join("sirno");
    SirnoConfig::new("docs").write_new(&config_path).unwrap();
    fs::create_dir(&old_lake).unwrap();
    fs::write(old_lake.join("entry.md"), "entry").unwrap();

    Cli::parse_from([
        "sirno",
        "--config",
        config_path.to_str().unwrap(),
        "lake",
        "mv",
        "sirno-lakes/sirno",
    ])
    .run()
    .unwrap();

    let config = SirnoConfig::from_file(&config_path).unwrap();
    assert_eq!(config.lake.path, PathBuf::from("sirno-lakes/sirno"));
    assert!(!old_lake.exists());
    assert!(new_lake.join("entry.md").exists());
}

#[test]
fn lake_mv_allows_destination_inside_current_lake() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    let old_lake = temp.path().join("docs");
    let new_lake = old_lake.join("sirno");
    SirnoConfig::new("docs").write_new(&config_path).unwrap();
    fs::create_dir(&old_lake).unwrap();
    fs::write(old_lake.join("entry.md"), "entry").unwrap();

    Cli::parse_from([
        "sirno",
        "--config",
        config_path.to_str().unwrap(),
        "lake",
        "mv",
        "docs/sirno",
    ])
    .run()
    .unwrap();

    let config = SirnoConfig::from_file(&config_path).unwrap();
    assert_eq!(config.lake.path, PathBuf::from("docs/sirno"));
    assert!(old_lake.exists());
    assert!(!old_lake.join("entry.md").exists());
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

    Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "frost", "move", "frost"])
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
fn frost_mv_creates_destination_parent() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    let old_frost = temp.path().join("sirno-frost");
    let new_frost = temp.path().join("sirno-lakes").join("sirno-frost");
    SirnoConfig::new("docs").with_frost("sirno-frost").write_new(&config_path).unwrap();
    fs::create_dir(&old_frost).unwrap();
    fs::write(old_frost.join("row"), "frost").unwrap();

    Cli::parse_from([
        "sirno",
        "--config",
        config_path.to_str().unwrap(),
        "frost",
        "mv",
        "sirno-lakes/sirno-frost",
    ])
    .run()
    .unwrap();

    let config = SirnoConfig::from_file(&config_path).unwrap();
    assert_eq!(
        config.frost,
        Some(FrostSettings { path: PathBuf::from("sirno-lakes/sirno-frost") })
    );
    assert!(!old_frost.exists());
    assert!(new_frost.join("row").exists());
}

#[test]
fn frost_mv_allows_destination_inside_current_frost() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join(CONFIG_FILE_NAME);
    let old_frost = temp.path().join("sirno-frost");
    let new_frost = old_frost.join("sirno");
    SirnoConfig::new("docs").with_frost("sirno-frost").write_new(&config_path).unwrap();
    fs::create_dir(&old_frost).unwrap();
    fs::write(old_frost.join("row"), "frost").unwrap();

    Cli::parse_from([
        "sirno",
        "--config",
        config_path.to_str().unwrap(),
        "frost",
        "mv",
        "sirno-frost/sirno",
    ])
    .run()
    .unwrap();

    let config = SirnoConfig::from_file(&config_path).unwrap();
    assert_eq!(config.frost, Some(FrostSettings { path: PathBuf::from("sirno-frost/sirno") }));
    assert!(old_frost.exists());
    assert!(!old_frost.join("row").exists());
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

    Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "move", "frost", "frost"])
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

    let error =
        Cli::parse_from(["sirno", "--config", config_path.to_str().unwrap(), "freeze", "alpha"])
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
        structural: StructuralSettings::from_fields([("area", StructuralFieldSettings::default())]),
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
        body: concat!("    // sample:start entry\n", "        fn main() {}\n", "    // sample:end")
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
        body: concat!("    // sample:start entry\n", "        fn main() {}\n", "    // sample:end")
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
        "- sirno-docs/concept.md\n- sirno-docs/entry.md\nTotal changes: 2/31 in sirno-docs"
    );
}

#[test]
fn format_gen_link_report_summarizes_no_changes() {
    let report = format_gen_link_report(Path::new("sirno-docs"), 31, &[]);

    assert_eq!(report, "No changes in sirno-docs");
}

#[test]
fn diagnostic_renderers_print_summary_last() {
    let diagnostic = DiagnosticRecord {
        severity: "error".to_owned(),
        path: Some("sirno-docs/entry.md".to_owned()),
        message: "dangling reference".to_owned(),
    };
    let check = format_lake_check_result(&LakeCheckResult {
        ok: false,
        root: "sirno-docs".to_owned(),
        has_errors: true,
        diagnostics: vec![diagnostic.clone()],
    });
    let render = format_render_result(&RenderResult {
        ok: false,
        dry: false,
        root: "sirno-docs".to_owned(),
        entry_count: 31,
        changed_paths: Vec::new(),
        diagnostics: vec![diagnostic],
        message: "render blocked by check errors in sirno-docs".to_owned(),
    });

    assert_before(&check, "error: sirno-docs/entry.md", "check: failed in sirno-docs");
    assert!(check.ends_with("check: failed in sirno-docs\n"));
    assert_before(
        &render,
        "error: sirno-docs/entry.md",
        "render blocked by check errors in sirno-docs",
    );
    assert!(render.ends_with("render blocked by check errors in sirno-docs\n"));
}

#[test]
fn config_comment_report_prints_summary_last() {
    let output = format_config_comment_result(&ConfigCommentResult {
        ok: false,
        changed: false,
        config_path: "Sirno.toml".to_owned(),
        missing_comments: vec!["Markdown entry lake path.".to_owned()],
        message: "1 config comments missing in Sirno.toml".to_owned(),
    });

    assert_before(&output, "missing: Markdown entry lake path.", "1 config comments missing");
    assert!(output.ends_with("1 config comments missing in Sirno.toml\n"));
}
