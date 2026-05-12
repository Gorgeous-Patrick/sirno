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
pub mod store;

pub use crate::check::{CheckDiagnostic, CheckMode, CheckReport, CheckSeverity};
pub use crate::config::{CONFIG_FILE_NAME, ConfigError, SirnoConfig};
pub use crate::entry::{
    Entry, EntryMetadata, EntryParseError, WitnessMarker, default_seed_entries,
};
pub use crate::files::{
    EntryDirectoryError, EntryDirectoryReport, EntryFileDiagnostic, GenLinkDirectoryReport,
    check_entry_directory, create_entry_file, gen_link_entry_directory, init_entry_directory,
};
pub use crate::id::{EntryId, EntryIdError};
pub use crate::links::{
    BEGIN_LINKS_GUARD, END_LINKS_GUARD, GeneratedLinkError, GeneratedLinkSettings,
    apply_generated_links, render_generated_links, validate_generated_links,
};
pub use crate::store::{SirnoStore, StoreError};
