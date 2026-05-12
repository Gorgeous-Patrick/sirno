//! Core library for Sirno.
//!
//! Sirno stores design as named Markdown entries with exact metadata.
//! The public model follows the repository design:
//! entries are nominal objects, relation fields are structural,
//! and repository witnesses are represented by a marker on the entry.

pub mod check;
pub mod config;
pub mod entry;
pub mod files;
pub mod id;
pub mod links;
pub mod query;
pub mod store;

pub use crate::check::{CheckDiagnostic, CheckMode, CheckReport, CheckSeverity};
pub use crate::config::{
    CONFIG_FILE_NAME, CheckSettings, ConfigError, MonoSettings, SirnoConfig, StoreSettings,
};
pub use crate::entry::{
    Entry, EntryMetadata, EntryParseError, WitnessMarker, default_seed_entries,
};
pub use crate::files::{
    EntryDirectoryCheckSettings, EntryDirectoryError, EntryDirectoryReport, EntryFileDiagnostic,
    GenLinkDirectoryReport, check_entry_directory, check_entry_directory_with_settings,
    create_entry_file, delete_gen_link_entry_directory,
    delete_gen_link_entry_directory_with_ignored_paths, gen_link_entry_directory,
    gen_link_entry_directory_with_ignored_paths, init_entry_directory,
};
pub use crate::id::{EntryId, EntryIdError};
pub use crate::links::{
    BEGIN_LINKS_GUARD, END_LINKS_GUARD, GeneratedLinkError, GeneratedLinkIndex,
    GeneratedLinkSettings, apply_generated_links, delete_generated_links,
    generated_links_are_stale, render_generated_links, validate_generated_links,
};
pub use crate::query::{
    EntryQuery, EntryTextTerm, VagueEntryQuery, query_entries, vague_query_entries,
};
pub use crate::store::{SirnoStore, StoreError};
