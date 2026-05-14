//! Core library for Sirno.
//!
//! Sirno keeps design as named Markdown entries with exact metadata.
//! The public model follows the repository design:
//! entries are nominal objects, structural fields are explicit,
//! and repository witnesses are represented by a marker on the entry.

pub mod check;
pub mod config;
pub mod entry;
pub mod files;
pub mod id;
pub mod links;
pub mod lock;
pub mod query;
pub mod store;
pub mod witness;

pub use eter::{Eterator, GcGeneration, SnapshotRef};

pub use crate::check::{CheckDiagnostic, CheckMode, CheckReport, CheckSeverity};
pub use crate::config::{
    CONFIG_FILE_NAME, CheckSettings, CodeMember, CodeSettings, ConfigError, HistorySettings,
    LakeSettings, MonoSettings, SirnoConfig,
};
pub use crate::entry::{
    Entry, EntryMetadata, EntryParseError, WitnessMarker, default_seed_entries,
};
pub use crate::files::{
    EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryReport,
    EntryDirectoryWritePolicy, EntryFileDiagnostic, GenLinkDirectoryReport,
    add_readonly_checkout_warnings, check_entry_directory, check_entry_directory_with_settings,
    check_gen_link_entry_directory, check_gen_link_entry_directory_with_ignored_paths,
    create_entry_file, delete_gen_link_entry_directory,
    delete_gen_link_entry_directory_with_ignored_paths, gen_link_entry_directory,
    gen_link_entry_directory_with_ignored_paths, init_entry_directory,
    set_entry_directory_readonly, set_entry_directory_writable, write_entry_directory,
};
pub use crate::id::{EntryId, EntryIdError};
pub use crate::links::{
    BEGIN_LINKS_GUARD, END_LINKS_GUARD, GeneratedLinkError, GeneratedLinkFieldSettings,
    GeneratedLinkIndex, GeneratedLinkSettings, apply_generated_links, delete_generated_links,
    generated_links_are_stale, render_generated_links, validate_generated_links,
};
pub use crate::lock::{
    HistoryLock, HistoryLockStatus, LOCK_FILE_NAME, LockError, SirnoLock, resolve_lock_path,
};
pub use crate::query::{
    EntryQuery, EntryTextTerm, VagueEntryQuery, query_entries, vague_query_entries,
};
pub use crate::store::{SirnoStore, StoreError};
pub use crate::witness::{
    WitnessCheckSettings, WitnessError, WitnessIndex, WitnessRecord, WitnessSpan, scan_witnesses,
};
