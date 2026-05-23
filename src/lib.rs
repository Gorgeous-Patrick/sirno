//! Core library for Sirno.
//!
//! Sirno keeps design as named Markdown entries with exact metadata.
//! The public model follows the repository design:
//! entries are nominal objects, structural links are explicit,
//! and repository witnesses are discovered by entry address.

pub mod artifact;
pub mod check;
pub mod config;
pub mod surface;
pub mod entry;
pub mod freeze;
pub mod frost;
pub mod identifier;
pub mod lake;
pub mod lock;
pub mod mcp;
pub mod query;
pub mod render;
pub mod structural;
pub mod tide;
pub mod upstream;
pub mod witness;

pub use eter::{Eterator, GcGeneration, SnapshotRef};

pub use crate::artifact::{
    ARTIFACT_DIRECTORY_NAME, EntryArtifact, EntryArtifactPath, EntryArtifactPathError,
};
pub use crate::check::{
    CheckDiagnostic, CheckDiagnosticKind, CheckMode, CheckReport, CheckSeverity,
};
pub use crate::config::{
    CONFIG_FILE_NAME, CheckSettings, ConfigError, FrostSettings, LakeSettings, RepoMember,
    RepoSettings, SirnoConfig, TutorialSettings, UpstreamRef, UpstreamSettings,
    UpstreamSettingsMap, WitnessDelimiterSettings, WitnessSettings,
};
pub use crate::entry::{
    DESC_FIELD, Entry, EntryMeta, EntryMetadata, EntryParseError, EntryStructuralFields,
    FROZEN_FIELD, FrozenMarker, META_FIELD, NAME_FIELD,
};
pub use crate::freeze::{FreezeError, FrozenPath};
pub use crate::frost::{FrostError, FrostGcReport, SirnoFrost};
pub use crate::identifier::{EntryAddress, EntryAddressError, EntryAtom, EntryAtomError};
pub use crate::lake::{
    EntryDirectory, EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryReport,
    EntryDirectoryWritePolicy, EntryFileDiagnostic, EntryRenameReport, GenLinkDirectoryReport,
    GlacierReport,
};
pub use crate::lock::{
    FrostLock, FrostLockStatus, LOCK_FILE_NAME, LockError, SirnoLock, TideLock, UpstreamLock,
    UpstreamLockMap,
};
pub use crate::query::{EntryQuery, EntryStructuralMatcher, EntryTextTerm, VagueEntryQuery};
pub use crate::render::{
    BEGIN_LINKS_GUARD, END_LINKS_GUARD, GeneratedLinkBody, GeneratedLinkError,
};
pub use crate::structural::{
    StructuralEdgeDirection, StructuralEdgeDirectionParseError, StructuralEdgeIndex,
    StructuralEdgeSettings, StructuralFieldMap, StructuralFieldSettings, StructuralRippleSettings,
    StructuralSettings,
};
pub use crate::tide::{
    Tide, TideError, TideResolution, TideSource, TideStatus, TideWorkitem, TideWorkitemParseError,
};
pub use crate::upstream::{
    UpstreamCrystallizeReport, UpstreamError, UpstreamGitCache, UpstreamStatus,
    UpstreamStatusReport, UpstreamStatusState,
};
pub use crate::witness::{
    WitnessCheckSettings, WitnessError, WitnessIndex, WitnessRecord, WitnessSpan,
};
