//! Error types for command execution and presentation.

use std::ffi::OsString;
use std::path::PathBuf;

use thiserror::Error;

use crate::{
    AnchorError, CharmError, ConfigError, EntryAddress, EntryAddressError, EntryArtifactPathError,
    EntryAtomError, EntryDirectoryError, EntryParseError, GeneratedLinkError, MistError, TideError,
    TideFileError, UpstreamError, UpstreamFileError, WitnessError,
};

/// Error raised while running the CLI.
#[derive(Debug, Error)]
pub enum CommandError {
    /// An artifact source path did not have a file name for the default artifact path.
    #[error("artifact source has no file name: {0}")]
    ArtifactSourceHasNoFileName(PathBuf),
    /// A configured path move cannot replace an existing destination.
    #[error("move destination already exists: {0}")]
    MoveDestinationExists(PathBuf),
    /// A configured path move could not inspect its destination.
    #[error("failed to inspect move destination {path}")]
    ReadMoveDestination {
        /// Destination path that could not be inspected.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A configured path move could not prepare a temporary staging path.
    #[error("failed to prepare move staging path near {path}")]
    PrepareMoveStagingPath {
        /// Directory where the staging path would be created.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A configured path move could not create the destination parent.
    #[error("failed to create move destination parent {path}")]
    CreateMoveDestinationParent {
        /// Destination parent path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A configured path could not be moved.
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
    /// A configured path move failed after staging and could not restore the source path.
    #[error(
        "failed to move {source_path} to {destination_path}; rollback from {staging_path} failed: {rollback}"
    )]
    MovePathRollback {
        /// Source path configured before the move.
        source_path: PathBuf,
        /// Destination path configured by the move.
        destination_path: PathBuf,
        /// Temporary staging path that still holds the moved directory.
        staging_path: PathBuf,
        /// Underlying move error.
        #[source]
        source: std::io::Error,
        /// Rollback rename error.
        rollback: std::io::Error,
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
    /// Witness lookup requires an existing entry address.
    #[error("entry `{0}` does not exist")]
    MissingWitnessEntry(EntryAddress),
    /// The MCP server selects its project only through the config path.
    #[error("`--lake-path` cannot be used with `sirno util mcp`; configure the lake in Sirno.toml")]
    McpRejectsLakePath,
    /// The config utility only inspects the config file.
    #[error("`--lake-path` cannot be used with `sirno util config`; use `--config` only")]
    ConfigRejectsLakePath,
    /// The terminal UI failed.
    #[error("terminal UI failed")]
    TerminalUi(#[source] std::io::Error),
    /// The interactive init prompt failed while reading or writing the terminal.
    #[error("interactive init prompt failed")]
    InteractiveInit(#[source] std::io::Error),
    /// The interactive init prompt reached the end of its input.
    #[error("interactive init prompt reached end of input")]
    InteractiveInitEof,
    /// The async MCP runtime could not be created.
    #[error("failed to create MCP runtime")]
    CreateMcpRuntime(#[source] std::io::Error),
    /// The MCP server failed.
    #[error("MCP server failed: {0}")]
    McpServer(String),
    /// The skill wrapper utility uses bundled wrapper constants.
    #[error("`--lake-path` cannot be used with `sirno util skills`; wrappers are bundled")]
    SkillsRejectsLakePath,
    /// A skill wrapper package target could not be read.
    #[error("failed to read skill wrapper target {path}")]
    ReadSkillWrapperTarget {
        /// Target path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A skill wrapper package directory could not be created.
    #[error("failed to create skill wrapper target directory {path}")]
    CreateSkillWrapperTargetDirectory {
        /// Target directory that could not be created.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A skill wrapper package target could not be written.
    #[error("failed to write skill wrapper target {path}")]
    WriteSkillWrapperTarget {
        /// Target path that could not be written.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A skill wrapper link target already exists as a non-link filesystem object.
    #[error("skill wrapper target exists and is not a symlink: {0}")]
    SkillWrapperTargetExists(PathBuf),
    /// A stale skill wrapper symlink could not be removed.
    #[error("failed to remove skill wrapper target {path}")]
    RemoveSkillWrapperTarget {
        /// Target path that could not be removed.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A skill wrapper package target could not be linked.
    #[error("failed to link skill wrapper target {target_path} to {source_path}")]
    LinkSkillWrapperTarget {
        /// Link source path.
        source_path: PathBuf,
        /// Link target path.
        target_path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Dry-run mode applies only to render writing.
    #[error("`--dry` only applies to `sirno mist render` without a subcommand")]
    DryWithRenderSubcommand,
    /// Render JSON overrides apply only to generated-link writing.
    #[error("`--override-json` only applies to `sirno mist render` without a subcommand")]
    OverrideJsonWithRenderSubcommand,
    /// A command named a link relation not defined in the lake.
    #[error(
        "link relation `{0}` is not defined in the lake; add `{0}` with `meta.type: \"structural\"`"
    )]
    UndefinedStructuralField(String),
    /// A command named a query column that is not built-in or discovered in the lake.
    #[error(
        "query column `{0}` is not defined in the lake; select `id`, `path`, a discovered \
         intrinsic field, or a structural relation"
    )]
    UndefinedQueryColumn(String),
    /// Generated-footer masking cannot compose with another ripgrep preprocessor.
    #[error(
        "generated-footer filtering cannot be combined with `rg --pre`; use `--with-generated-footer`"
    )]
    RgPreprocessorConflict,
    /// Anchor update requires all current tide workitems to be reviewed.
    #[error("anchor update blocked by {open_workitems} open tide workitems")]
    AnchorUpdateOpenTide {
        /// Number of open workitems.
        open_workitems: usize,
    },
    /// Anchor update requires editable mist projections to be clean.
    #[error("anchor update blocked by mist state: {0}")]
    AnchorUpdateMist(String),
    /// Mist intake requires a fresh editable projection.
    #[error("mist intake blocked: {0}")]
    MistIntakeBlocked(String),
    /// Ripgrep generated-footer preprocessor received an unexpected argument shape.
    #[error("rg generated-footer preprocessor expects one path argument")]
    RgPreprocessorArgumentCount,
    /// The current executable path could not be resolved.
    #[error("failed to locate current executable for rg preprocessor")]
    LocateCurrentExe(#[source] std::io::Error),
    /// The process current working directory could not be changed.
    #[error("failed to change current working directory to {path}")]
    ChangeCurrentDirectory {
        /// Target working directory.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
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
    /// Git could not be started.
    #[error("failed to run git")]
    RunGit(#[source] std::io::Error),
    /// Git exited unsuccessfully.
    #[error("git command failed: {stderr}")]
    GitFailed {
        /// Captured standard error.
        stderr: String,
    },
    /// Git output was not valid UTF-8.
    #[error("git output is not valid UTF-8")]
    GitOutput(#[source] std::string::FromUtf8Error),
    /// A charm command requires an enabled charm.
    #[error("charm `{0}` is not enabled; run `sirno charm enable {0}` first")]
    CharmNotEnabled(EntryAddress),
    /// A charm process could not be started.
    #[error("failed to run {phase} command for charm `{id}`")]
    RunCharmProcess {
        /// Entry address whose charm was selected.
        id: EntryAddress,
        /// Operation phase.
        phase: &'static str,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A spell cache directory could not be created.
    #[error("failed to create spell cache directory {path}")]
    CreateSpellCache {
        /// Cache path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A spell cache directory could not be removed.
    #[error("failed to remove spell cache directory {path}")]
    RemoveSpellCache {
        /// Cache path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Config-backed command failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Mist-backed command failed.
    #[error(transparent)]
    Mist(#[from] MistError),
    /// Charm-backed command failed.
    #[error(transparent)]
    Charm(#[from] CharmError),
    /// Anchor-backed command failed.
    #[error(transparent)]
    Anchor(#[from] AnchorError),
    /// Upstream-file-backed command failed.
    #[error(transparent)]
    UpstreamFile(#[from] UpstreamFileError),
    /// Witness lookup failed.
    #[error(transparent)]
    Witness(#[from] WitnessError),
    /// Sirno Lake entry directory command failed.
    #[error(transparent)]
    EntryDirectory(#[from] EntryDirectoryError),
    /// Entry address parsing failed.
    #[error(transparent)]
    EntryAddress(#[from] EntryAddressError),
    /// Entry atom parsing failed.
    #[error(transparent)]
    EntryAtom(#[from] EntryAtomError),
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
    /// Tide file operation failed.
    #[error(transparent)]
    TideFile(#[from] TideFileError),
    /// Upstream operation failed.
    #[error(transparent)]
    Upstream(#[from] UpstreamError),
    /// Ripgrep could not be started.
    #[error("failed to run rg")]
    RunRg(#[source] std::io::Error),
    /// JSON-oriented ripgrep execution needs UTF-8 arguments.
    #[error("rg argument is not valid UTF-8: {0:?}")]
    RgArgumentNotUtf8(OsString),
    /// JSON parsing or rendering failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
