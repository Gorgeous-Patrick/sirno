//! Core library for Sirno.
//!
//! Sirno keeps design as named Markdown entries with exact metadata.
//! The public model follows the repository design:
//! entries are nominal objects, structural fields are explicit,
//! and repository witnesses are discovered by entry id.

pub mod check;
pub mod config;
pub mod entry;
pub mod id;
pub mod lake;
pub mod links;
pub mod lock;
pub mod query;
pub mod frost;
pub mod witness;

pub use eter::{Eterator, GcGeneration, SnapshotRef};

pub use crate::check::{
    CheckDiagnostic, CheckDiagnosticKind, CheckMode, CheckReport, CheckSeverity,
};
pub use crate::config::{
    CONFIG_FILE_NAME, CheckSettings, ConfigError, FrostSettings, LakeSettings, MonoSettings,
    RepoMember, RepoSettings, SirnoConfig, WitnessDelimiterSettings, WitnessSettings,
};
pub use crate::entry::{
    DESC_FIELD, Entry, EntryMetadata, EntryParseError, EntryStructuralFields, FROZEN_FIELD,
    FrozenMarker, NAME_FIELD,
};
pub use crate::frost::{FrostError, SirnoFrost};
pub use crate::id::{EntryId, EntryIdError};
pub use crate::lake::{
    EntryDirectory, EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryReport,
    EntryDirectoryWritePolicy, EntryFileDiagnostic, EntryRenameReport, GenLinkDirectoryReport,
};
pub use crate::links::{
    BEGIN_LINKS_GUARD, END_LINKS_GUARD, GeneratedLinkBody, GeneratedLinkError, GeneratedLinkIndex,
    StructuralFieldMap, StructuralFieldSettings, StructuralLinkSettings, StructuralSettings,
};
pub use crate::lock::{FrostLock, FrostLockStatus, LOCK_FILE_NAME, LockError, SirnoLock};
pub use crate::query::{EntryQuery, EntryTextTerm, VagueEntryQuery};
pub use crate::witness::{
    WitnessCheckSettings, WitnessError, WitnessIndex, WitnessRecord, WitnessSpan,
};
