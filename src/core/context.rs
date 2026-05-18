//! Typed command execution shared by CLI and tool adapters.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use indexmap::IndexMap;

use crate::core::dto::{
    ArtifactAddRequest, ArtifactChangeResult, ArtifactListResult, ArtifactRemoveRequest,
    ArtifactRenameRequest, ConfigCommentResult, CwdResult, EntryNewRequest, EntryPathRequest,
    EntryPathResult, EntryRenameResult, FrostCheckoutRequest, FrostCheckoutResult,
    FrostCommitResult, FrostInitResult, LakeCheckResult, LakeInitRequest, LakeInitResult,
    MovePathResult, PathRecord, QueryRequest, QueryResponse, QueryResults, QueryRun, RenderResult,
    RgRequest, RgResult, SkillWrapperRecord, SkillWrapperResult, StatusResult,
    StructuralFieldStatus, StructuralFilter, StructuralStateFilter, StructuralTarget,
    TideChangeResult, TideResolveRequest, TideSelectionRequest, TideStatusMode, TideStatusResult,
    WitnessRecordResult, WitnessResult,
};
use crate::core::error::{CommandError, OpenTideTutorial};
use crate::core::output::{
    diagnostics_from_entry_report, display_path, display_paths, frost_state_label, output_path,
    query_result_rows,
};
use crate::core::rg::{RgPreprocessorLink, resolve_lake_path_for_rg};
use crate::{
    CONFIG_FILE_NAME, CheckMode, Entry, EntryArtifactPath, EntryDirectory,
    EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryWritePolicy, EntryId,
    EntryMetadata, EntryQuery, EntryStructuralMatcher, Eterator, SirnoConfig, SirnoFrost,
    SirnoLock, StructuralSettings, Tide, TideStatus, TutorialSettings, VagueEntryQuery,
    WitnessCheckSettings, WitnessRecord,
};

// sirno:witness:agent-skills:begin
const SKILL_WRAPPERS: &[SkillWrapperSpec] = &[
    SkillWrapperSpec {
        name: "sirno-config-writer",
        entry_id: "config-writing-discipline",
        wrapper_path: "sirno-docs/.artifacts/config-writing-discipline/SKILL.md",
        full_path: "sirno-docs/.artifacts/config-writing-discipline/SKILL.full.md",
        target_path: ".agents/skills/sirno-config-writer/SKILL.md",
        content: include_str!("../../sirno-docs/.artifacts/config-writing-discipline/SKILL.md"),
    },
    SkillWrapperSpec {
        name: "sirno-editor",
        entry_id: "lake-editing-discipline",
        wrapper_path: "sirno-docs/.artifacts/lake-editing-discipline/SKILL.md",
        full_path: "sirno-docs/.artifacts/lake-editing-discipline/SKILL.full.md",
        target_path: ".agents/skills/sirno-editor/SKILL.md",
        content: include_str!("../../sirno-docs/.artifacts/lake-editing-discipline/SKILL.md"),
    },
    SkillWrapperSpec {
        name: "sirno-explorer",
        entry_id: "lake-exploration-discipline",
        wrapper_path: "sirno-docs/.artifacts/lake-exploration-discipline/SKILL.md",
        full_path: "sirno-docs/.artifacts/lake-exploration-discipline/SKILL.full.md",
        target_path: ".agents/skills/sirno-explorer/SKILL.md",
        content: include_str!("../../sirno-docs/.artifacts/lake-exploration-discipline/SKILL.md"),
    },
    SkillWrapperSpec {
        name: "sirno-narrative-session",
        entry_id: "narrative-session-discipline",
        wrapper_path: "sirno-docs/.artifacts/narrative-session-discipline/SKILL.md",
        full_path: "sirno-docs/.artifacts/narrative-session-discipline/SKILL.full.md",
        target_path: ".agents/skills/sirno-narrative-session/SKILL.md",
        content: include_str!("../../sirno-docs/.artifacts/narrative-session-discipline/SKILL.md"),
    },
    SkillWrapperSpec {
        name: "sirno-skill-synthesizer",
        entry_id: "skill-synthesis-discipline",
        wrapper_path: "sirno-docs/.artifacts/skill-synthesis-discipline/SKILL.md",
        full_path: "sirno-docs/.artifacts/skill-synthesis-discipline/SKILL.full.md",
        target_path: ".agents/skills/sirno-skill-synthesizer/SKILL.md",
        content: include_str!("../../sirno-docs/.artifacts/skill-synthesis-discipline/SKILL.md"),
    },
    SkillWrapperSpec {
        name: "sirno-witness",
        entry_id: "witness-linking-discipline",
        wrapper_path: "sirno-docs/.artifacts/witness-linking-discipline/SKILL.md",
        full_path: "sirno-docs/.artifacts/witness-linking-discipline/SKILL.full.md",
        target_path: ".agents/skills/sirno-witness/SKILL.md",
        content: include_str!("../../sirno-docs/.artifacts/witness-linking-discipline/SKILL.md"),
    },
];
// sirno:witness:agent-skills:end

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

    pub(crate) fn from_cli_paths(config_path: &Path, lake_path: Option<&Path>) -> Self {
        let mut context = Self::new(config_path.to_path_buf());
        if let Some(lake_path) = lake_path {
            context = context.with_lake_path(lake_path.to_path_buf());
        }
        context
    }

    /// Read or change the process current working directory.
    pub fn cwd(&self, path: Option<PathBuf>) -> Result<CwdResult, CommandError> {
        let changed = path.is_some();
        if let Some(path) = path {
            env::set_current_dir(&path).map_err(|source| CommandError::ChangeCurrentDirectory {
                path: path.to_path_buf(),
                source,
            })?;
        }

        let cwd = env::current_dir().map_err(CommandError::CurrentDirectory)?;
        let path = display_path(&cwd);
        let message = if changed {
            format!("changed current working directory to {path}")
        } else {
            format!("current working directory is {path}")
        };

        Ok(CwdResult { ok: true, changed, path, message })
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
    pub fn tide_statuses(&self, mode: TideStatusMode) -> Result<Vec<TideStatus>, CommandError> {
        let context = TideContext::load(&self.config_path, self.lake_path.as_deref())?;
        let lock = context.load_lock_or_current()?;
        let tide = context.tide(&lock)?;
        Ok(tide_statuses_for_output(&tide, mode.includes_resolved()))
    }

    /// Return entry ids that still need tide review.
    pub fn tide_review_entries(&self) -> Result<Vec<EntryId>, CommandError> {
        let context = TideContext::load(&self.config_path, self.lake_path.as_deref())?;
        let lock = context.load_lock_or_current()?;
        let tide = context.tide(&lock)?;
        Ok(tide.review_entries())
    }

    /// Return tide review entries and optional full statuses as a JSON-first command result.
    pub fn tide_status(&self, mode: TideStatusMode) -> Result<TideStatusResult, CommandError> {
        let context = TideContext::load(&self.config_path, self.lake_path.as_deref())?;
        let lock = context.load_lock_or_current()?;
        let tide = context.tide(&lock)?;
        let review_entries = tide.review_entries();
        let statuses = if mode.includes_workitems() {
            tide_statuses_for_output(&tide, mode.includes_resolved())
        } else {
            Vec::new()
        };
        Ok(TideStatusResult { ok: review_entries.is_empty(), review_entries, statuses })
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

    // sirno:witness:project-config-comments:begin
    /// Check whether `Sirno.toml` contains every canonical config comment.
    pub fn config_comments_check(&self) -> Result<ConfigCommentResult, CommandError> {
        self.config_comments(false)
    }

    /// Rewrite `Sirno.toml` through the canonical commented renderer when comments are missing.
    pub fn config_comments_fix(&self) -> Result<ConfigCommentResult, CommandError> {
        self.config_comments(true)
    }

    fn config_comments(&self, fix: bool) -> Result<ConfigCommentResult, CommandError> {
        let source = fs::read_to_string(&self.config_path).map_err(|source| {
            crate::ConfigError::Read { path: self.config_path.clone(), source }
        })?;
        let config = SirnoConfig::from_file(&self.config_path)?;
        let missing_comments = config.missing_comments_in(&source)?;
        let missing_count = missing_comments.len();
        let changed = fix && missing_count > 0;
        if changed {
            config.write(&self.config_path)?;
        }

        let message = match (missing_count, fix) {
            | (0, _) => format!("config comments ok in {}", self.config_path.display()),
            | (_, true) => format!(
                "updated config comments in {} ({missing_count} missing)",
                self.config_path.display()
            ),
            | (_, false) => format!(
                "config comments missing in {} ({missing_count} missing); run `sirno util config --fix`",
                self.config_path.display()
            ),
        };

        Ok(ConfigCommentResult {
            ok: missing_count == 0 || fix,
            changed,
            config_path: display_path(&self.config_path),
            missing_comments,
            message,
        })
    }
    // sirno:witness:project-config-comments:end

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

    // sirno:witness:agent-skills:begin
    /// List bundled Sirno skill wrapper constants and package targets.
    pub fn skill_wrappers_list(&self) -> Result<SkillWrapperResult, CommandError> {
        let records =
            SKILL_WRAPPERS.iter().map(|source| source.record("source", false)).collect::<Vec<_>>();
        Ok(SkillWrapperResult {
            ok: true,
            message: format!("found {} Sirno skill wrappers", records.len()),
            records,
        })
    }

    /// Check installed Sirno skill wrapper packages against bundled constants.
    pub fn skill_wrappers_check(&self) -> Result<SkillWrapperResult, CommandError> {
        let root = config_parent(&self.config_path);
        let mut records = Vec::new();
        for source in SKILL_WRAPPERS {
            let target = root.join(source.target_path);
            let status = match fs::read(&target) {
                | Ok(current) if current == source.content.as_bytes() => "ok",
                | Ok(_) => "drifted",
                | Err(error) if error.kind() == ErrorKind::NotFound => "missing",
                | Err(source) => {
                    return Err(CommandError::ReadSkillWrapperTarget { path: target, source });
                }
            };
            records.push(source.record(status, status != "ok"));
        }

        let changed = records.iter().filter(|record| record.changed).count();
        Ok(SkillWrapperResult {
            ok: changed == 0,
            message: if changed == 0 {
                format!("all {} Sirno skill wrappers match artifacts", records.len())
            } else {
                format!("{changed} Sirno skill wrappers differ from artifacts")
            },
            records,
        })
    }

    /// Install bundled Sirno skill wrapper constants into their package targets.
    pub fn skill_wrappers_init(&self) -> Result<SkillWrapperResult, CommandError> {
        let root = config_parent(&self.config_path);
        let mut records = Vec::new();
        for source in SKILL_WRAPPERS {
            let target = root.join(source.target_path);
            let status = match fs::read(&target) {
                | Ok(current) if current == source.content.as_bytes() => "unchanged",
                | Ok(_) => {
                    write_skill_wrapper_target(&target, source.content.as_bytes())?;
                    "wrote"
                }
                | Err(error) if error.kind() == ErrorKind::NotFound => {
                    write_skill_wrapper_target(&target, source.content.as_bytes())?;
                    "wrote"
                }
                | Err(source) => {
                    return Err(CommandError::ReadSkillWrapperTarget { path: target, source });
                }
            };
            records.push(source.record(status, status == "wrote"));
        }

        let changed = records.iter().filter(|record| record.changed).count();
        Ok(SkillWrapperResult {
            ok: true,
            message: format!(
                "installed {} Sirno skill wrappers ({changed} changed)",
                records.len()
            ),
            records,
        })
    }
    // sirno:witness:agent-skills:end

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

fn move_configured_path_and_write_config(
    source: &Path, destination: &Path, config: &SirnoConfig, config_path: &Path,
) -> Result<bool, CommandError> {
    let move_result = move_configured_path(source, destination)?;
    if let Err(config_error) = config.write(config_path) {
        if move_result.moved()
            && let Err(rollback) = rollback_configured_path(source, destination, move_result)
        {
            return Err(CommandError::MoveConfigWriteRollback {
                source_path: source.to_path_buf(),
                destination_path: destination.to_path_buf(),
                source: Box::new(config_error),
                rollback,
            });
        }
        return Err(CommandError::Config(config_error));
    }
    Ok(move_result.moved())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConfiguredPathMove {
    Unchanged,
    Direct,
    Nested,
}

impl ConfiguredPathMove {
    fn moved(self) -> bool {
        !matches!(self, Self::Unchanged)
    }
}

fn move_configured_path(
    source: &Path, destination: &Path,
) -> Result<ConfiguredPathMove, CommandError> {
    if source == destination {
        return Ok(ConfiguredPathMove::Unchanged);
    }

    ensure_move_destination_missing(destination)?;
    if destination.starts_with(source) {
        move_configured_path_nested(source, destination)?;
        return Ok(ConfiguredPathMove::Nested);
    }

    create_move_destination_parent(destination)?;
    fs::rename(source, destination).map_err(|error| CommandError::MovePath {
        source_path: source.to_path_buf(),
        destination_path: destination.to_path_buf(),
        source: error,
    })?;
    Ok(ConfiguredPathMove::Direct)
}

fn ensure_move_destination_missing(destination: &Path) -> Result<(), CommandError> {
    match fs::symlink_metadata(destination) {
        | Ok(_) => Err(CommandError::MoveDestinationExists(destination.to_path_buf())),
        | Err(source) if source.kind() == ErrorKind::NotFound => Ok(()),
        | Err(source) => {
            Err(CommandError::ReadMoveDestination { path: destination.to_path_buf(), source })
        }
    }
}

fn create_move_destination_parent(destination: &Path) -> Result<(), CommandError> {
    let Some(parent) = move_destination_parent(destination) else {
        return Ok(());
    };
    create_move_destination_parent_io(destination).map_err(|source| {
        CommandError::CreateMoveDestinationParent { path: parent.to_path_buf(), source }
    })
}

fn move_configured_path_nested(source: &Path, destination: &Path) -> Result<(), CommandError> {
    let staging_parent = move_staging_parent(source);
    let staging = move_staging_path(source).map_err(|source| {
        CommandError::PrepareMoveStagingPath { path: staging_parent.to_path_buf(), source }
    })?;

    fs::rename(source, &staging).map_err(|error| CommandError::MovePath {
        source_path: source.to_path_buf(),
        destination_path: destination.to_path_buf(),
        source: error,
    })?;

    if let Err(error) = create_move_destination_parent_io(destination) {
        if let Err(rollback) = rollback_nested_staging_path(&staging, source, destination) {
            return Err(CommandError::MovePathRollback {
                source_path: source.to_path_buf(),
                destination_path: destination.to_path_buf(),
                staging_path: staging,
                source: error,
                rollback,
            });
        }
        let parent = move_destination_parent(destination)
            .expect("destination parent exists after create failed");
        return Err(CommandError::CreateMoveDestinationParent {
            path: parent.to_path_buf(),
            source: error,
        });
    }

    if let Err(error) = fs::rename(&staging, destination) {
        if let Err(rollback) = rollback_nested_staging_path(&staging, source, destination) {
            return Err(CommandError::MovePathRollback {
                source_path: source.to_path_buf(),
                destination_path: destination.to_path_buf(),
                staging_path: staging,
                source: error,
                rollback,
            });
        }
        return Err(CommandError::MovePath {
            source_path: source.to_path_buf(),
            destination_path: destination.to_path_buf(),
            source: error,
        });
    }

    Ok(())
}

fn rollback_configured_path(
    source: &Path, destination: &Path, move_result: ConfiguredPathMove,
) -> std::io::Result<()> {
    match move_result {
        | ConfiguredPathMove::Unchanged => Ok(()),
        | ConfiguredPathMove::Direct => fs::rename(destination, source),
        | ConfiguredPathMove::Nested => rollback_nested_destination(source, destination),
    }
}

fn rollback_nested_destination(source: &Path, destination: &Path) -> std::io::Result<()> {
    let staging = move_staging_path(source)?;
    fs::rename(destination, &staging)?;
    remove_empty_nested_destination_parent(source, destination)?;
    fs::rename(staging, source)
}

fn rollback_nested_staging_path(
    staging: &Path, source: &Path, destination: &Path,
) -> std::io::Result<()> {
    remove_empty_nested_destination_parent(source, destination)
        .and_then(|()| fs::rename(staging, source))
}

fn remove_empty_nested_destination_parent(
    source: &Path, destination: &Path,
) -> std::io::Result<()> {
    let mut path = destination.parent();
    while let Some(parent) = path {
        if !parent.starts_with(source) {
            break;
        }

        match fs::remove_dir(parent) {
            | Ok(()) => {}
            | Err(error) if error.kind() == ErrorKind::NotFound => {}
            | Err(error) => return Err(error),
        }

        if parent == source {
            break;
        }
        path = parent.parent();
    }
    Ok(())
}

fn move_destination_parent(destination: &Path) -> Option<&Path> {
    destination.parent().filter(|parent| !parent.as_os_str().is_empty())
}

fn create_move_destination_parent_io(destination: &Path) -> std::io::Result<()> {
    let Some(parent) = move_destination_parent(destination) else {
        return Ok(());
    };
    fs::create_dir_all(parent)
}

fn move_staging_parent(source: &Path) -> &Path {
    source
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn move_staging_path(source: &Path) -> std::io::Result<PathBuf> {
    let parent = move_staging_parent(source);
    let source_name = source.file_name().map(OsString::from).unwrap_or_else(|| "path".into());

    for index in 0..1000 {
        let mut name = OsString::from(".sirno-move-");
        name.push(&source_name);
        name.push(format!("-{}-{index}", std::process::id()));
        let candidate = parent.join(name);
        match fs::symlink_metadata(&candidate) {
            | Ok(_) => {}
            | Err(error) if error.kind() == ErrorKind::NotFound => return Ok(candidate),
            | Err(error) => return Err(error),
        }
    }

    Err(std::io::Error::new(
        ErrorKind::AlreadyExists,
        format!("move staging paths are unavailable near {}", parent.display()),
    ))
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

#[derive(Clone, Copy, Debug)]
struct SkillWrapperSpec {
    name: &'static str,
    entry_id: &'static str,
    wrapper_path: &'static str,
    full_path: &'static str,
    target_path: &'static str,
    content: &'static str,
}

impl SkillWrapperSpec {
    fn record(&self, status: impl Into<String>, changed: bool) -> SkillWrapperRecord {
        SkillWrapperRecord {
            name: self.name.to_owned(),
            entry_id: self.entry_id.to_owned(),
            wrapper_path: self.wrapper_path.to_owned(),
            full_path: self.full_path.to_owned(),
            target_path: self.target_path.to_owned(),
            status: status.into(),
            changed,
        }
    }
}

fn config_parent(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn write_skill_wrapper_target(target: &Path, content: &[u8]) -> Result<(), CommandError> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CommandError::CreateSkillWrapperTargetDirectory { path: parent.to_path_buf(), source }
        })?;
    }
    fs::write(target, content).map_err(|source| CommandError::WriteSkillWrapperTarget {
        path: target.to_path_buf(),
        source,
    })
}

fn frost_version(version: u64) -> Result<Eterator, CommandError> {
    if version == Eterator::EMPTY.version() {
        return Err(CommandError::InvalidFrostVersion(version));
    }
    Ok(Eterator(version))
}

pub(crate) fn default_config_path() -> PathBuf {
    PathBuf::from(CONFIG_FILE_NAME)
}

pub(crate) fn default_lake_path(config_path: &Path) -> PathBuf {
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

pub(crate) fn entry_query_from_filters(
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

fn tide_statuses_for_output(tide: &Tide, all: bool) -> Vec<TideStatus> {
    tide.statuses().iter().filter(|status| all || !status.resolved).cloned().collect()
}
