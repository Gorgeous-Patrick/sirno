//! Core library for Sirno.
//!
//! Sirno keeps design as named Markdown entries with exact metadata.
//! The public model follows the repository design:
//! entries are nominal objects, structural fields are explicit,
//! and repository witnesses are discovered by entry id.

pub mod artifact;
pub mod check;
pub mod config;
pub mod entry;
pub mod freeze;
pub mod frost;
pub mod id;
pub mod lake;
pub mod lock;
pub mod query;
pub mod render;
pub mod structural;
pub mod tide;
pub mod witness;

pub use eter::{Eterator, GcGeneration, SnapshotRef};

pub use crate::artifact::{
    ARTIFACT_DIRECTORY_NAME, EntryArtifact, EntryArtifactPath, EntryArtifactPathError,
};
pub use crate::check::{
    CheckDiagnostic, CheckDiagnosticKind, CheckMode, CheckReport, CheckSeverity,
};
pub use crate::config::{
    CONFIG_FILE_NAME, CheckSettings, ConfigError, FrostSettings, LakeSettings, MonoSettings,
    RepoMember, RepoSettings, SirnoConfig, TutorialSettings, WitnessDelimiterSettings,
    WitnessSettings,
};
pub use crate::entry::{
    DESC_FIELD, Entry, EntryMetadata, EntryParseError, EntryStructuralFields, FROZEN_FIELD,
    FrozenMarker, NAME_FIELD,
};
pub use crate::freeze::{FreezeError, FrozenPath};
pub use crate::frost::{FrostError, SirnoFrost};
pub use crate::id::{EntryId, EntryIdError};
pub use crate::lake::{
    EntryDirectory, EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryReport,
    EntryDirectoryWritePolicy, EntryFileDiagnostic, EntryRenameReport, GenLinkDirectoryReport,
};
pub use crate::lock::{FrostLock, FrostLockStatus, LOCK_FILE_NAME, LockError, SirnoLock, TideLock};
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
pub use crate::witness::{
    WitnessCheckSettings, WitnessError, WitnessIndex, WitnessRecord, WitnessSpan,
};
