//! Repository witness lookup through `mosaika`.
//!
//! Sirno keeps witness intent in entry metadata and delegates repository scans to
//! `mosaika`. The Sirno layer owns member selection because `[code].members`
//! accepts recursive directory members in addition to glob patterns.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use glob::glob;
use mosaika::engine::{Engine, OverwriteMode};
use mosaika::semantics::Scheme;
use mosaika::syntax::{
    self as syn, Arrow, Delimiter, Effect, LogDestination, LogPipe, PipeName, RegexDelimiter,
    Transaction, Transform,
};
use serde::Deserialize;
use thiserror::Error;
use tracing::trace;

use crate::config::CodeMember;
use crate::id::{EntryId, EntryIdError};

const WITNESS_TRANSFORM: &str = "sirno-witness";
const WITNESS_START_REGEX: &str = r"(?m)^[ \t]*//[ \t]*sirno:witness:start ([A-Za-z0-9_-]+)";
const WITNESS_END_REGEX: &str = r"(?m)^[ \t]*//[ \t]*sirno:witness:end";

/// Settings for a witness scan.
///
/// Invariant: `root` is the directory relative to which members are resolved.
/// `members` are already validated config-relative member patterns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessCheckSettings {
    /// Directory relative to which code members are resolved.
    pub root: PathBuf,
    /// Configured repository members scanned for witness markers.
    pub members: Vec<CodeMember>,
}

impl WitnessCheckSettings {
    /// Construct witness settings from a config root and code members.
    pub fn new(root: impl Into<PathBuf>, members: impl IntoIterator<Item = CodeMember>) -> Self {
        Self { root: root.into(), members: members.into_iter().collect() }
    }

    /// Returns true when there is no repository surface to scan.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

/// Repository locations grouped by witnessed entry id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WitnessIndex {
    records_by_entry: BTreeMap<EntryId, Vec<WitnessRecord>>,
}

impl WitnessIndex {
    /// Construct an empty witness index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one witness record.
    pub fn push(&mut self, record: WitnessRecord) {
        self.records_by_entry.entry(record.entry.clone()).or_default().push(record);
    }

    /// Return every record for one entry id.
    pub fn records_for(&self, id: &EntryId) -> &[WitnessRecord] {
        self.records_by_entry.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Returns true when the index contains at least one record for the entry.
    pub fn contains_entry(&self, id: &EntryId) -> bool {
        self.records_by_entry.contains_key(id)
    }

    /// Iterate over entry ids with at least one witness marker.
    pub fn entry_ids(&self) -> impl Iterator<Item = &EntryId> {
        self.records_by_entry.keys()
    }
}

/// One repository witness block resolved by `mosaika`.
///
/// Invariant: `entry` is the parsed id captured from the opening delimiter.
/// `region` identifies the matched block.
/// `opening` and `closing` identify the delimiter spans.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:start witness
pub struct WitnessRecord {
    /// Entry id captured from `sirno:witness:start <entry-id>`.
    pub entry: EntryId,
    /// Repository file that contains the witness block.
    pub path: PathBuf,
    /// Full matched block region.
    pub region: WitnessSpan,
    /// Matched opening delimiter span.
    pub opening: WitnessSpan,
    /// Matched closing delimiter span.
    pub closing: WitnessSpan,
    /// Matched opening delimiter text.
    pub marker: String,
    /// Full witness block body emitted by `mosaika`.
    pub body: String,
}
// sirno:witness:end

/// One source span reported by `mosaika`.
///
/// Invariant: line and column values are one-based.
/// End columns point after the matched span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// sirno:witness:start witness
pub struct WitnessSpan {
    /// One-based starting line.
    pub start_line: usize,
    /// One-based starting column.
    pub start_column: usize,
    /// One-based ending line.
    pub end_line: usize,
    /// One-based column after the span.
    pub end_column: usize,
}
// sirno:witness:end

// sirno:witness:start witness
/// Scan configured repository members for Sirno witness blocks.
pub fn scan_witnesses(settings: &WitnessCheckSettings) -> Result<WitnessIndex, WitnessError> {
    trace!(
        root = %settings.root.display(),
        member_count = settings.members.len(),
        "scan_witnesses begin"
    );
    let files = resolve_member_files(settings)?;
    let output = run_mosaika_witness_scan(&settings.root, &files)?;
    let index = parse_witness_output(&output)?;
    trace!(file_count = files.len(), "scan_witnesses end");
    Ok(index)
}
// sirno:witness:end

// sirno:witness:start witness
fn resolve_member_files(settings: &WitnessCheckSettings) -> Result<Vec<PathBuf>, WitnessError> {
    let mut files = BTreeSet::new();
    for member in &settings.members {
        let before = files.len();
        if has_glob_meta(member.as_str()) {
            collect_glob_member(&settings.root, member, &mut files)?;
        } else {
            let path = settings.root.join(member.as_str());
            collect_path_member(member, &path, &mut files)?;
        }
        if files.len() == before {
            return Err(WitnessError::MissingMember { member: member.as_str().to_owned() });
        }
    }
    Ok(files.into_iter().collect())
}
// sirno:witness:end

// sirno:witness:start witness
fn collect_glob_member(
    root: &Path, member: &CodeMember, files: &mut BTreeSet<PathBuf>,
) -> Result<(), WitnessError> {
    let pattern = root.join(member.as_str()).to_string_lossy().to_string();
    let matches = glob(&pattern).map_err(|source| WitnessError::InvalidGlob {
        member: member.as_str().to_owned(),
        source,
    })?;
    for path in matches {
        collect_path_member(
            member,
            &path.map_err(|source| WitnessError::Glob {
                member: member.as_str().to_owned(),
                source,
            })?,
            files,
        )?;
    }
    Ok(())
}
// sirno:witness:end

// sirno:witness:start witness
fn collect_path_member(
    member: &CodeMember, path: &Path, files: &mut BTreeSet<PathBuf>,
) -> Result<(), WitnessError> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        files.insert(path.to_path_buf());
        return Ok(());
    }
    if path.is_dir() {
        collect_directory_files(member, path, files)?;
        return Ok(());
    }
    Err(WitnessError::UnsupportedMember {
        member: member.as_str().to_owned(),
        path: path.to_path_buf(),
    })
}
// sirno:witness:end

// sirno:witness:start witness
fn collect_directory_files(
    member: &CodeMember, root: &Path, files: &mut BTreeSet<PathBuf>,
) -> Result<(), WitnessError> {
    for entry in std::fs::read_dir(root).map_err(|source| WitnessError::ReadDirectory {
        member: member.as_str().to_owned(),
        path: root.to_path_buf(),
        source,
    })? {
        let path = entry
            .map_err(|source| WitnessError::ReadDirectory {
                member: member.as_str().to_owned(),
                path: root.to_path_buf(),
                source,
            })?
            .path();
        if path.is_dir() {
            collect_directory_files(member, &path, files)?;
        } else if path.is_file() {
            files.insert(path);
        }
    }
    Ok(())
}
// sirno:witness:end

// sirno:witness:start witness
fn run_mosaika_witness_scan(root: &Path, files: &[PathBuf]) -> Result<String, WitnessError> {
    if files.is_empty() {
        return Ok(String::new());
    }

    let projection = witness_projection(files);
    let scheme = Scheme::from_syntax(projection, root).map_err(WitnessError::Scheme)?;
    let mut output = Vec::new();
    Engine::new("sirno witness scan", scheme)
        .run_with_stdout(OverwriteMode::RejectExisting, &mut output)
        .map_err(WitnessError::Engine)?;
    String::from_utf8(output).map_err(WitnessError::Utf8)
}
// sirno:witness:end

// sirno:witness:start witness
fn witness_projection(files: &[PathBuf]) -> syn::Projection {
    syn::Projection {
        transforms: vec![Transform {
            name: WITNESS_TRANSFORM.to_owned(),
            delimiters: vec![
                Delimiter::Regex(RegexDelimiter { regex: WITNESS_START_REGEX.to_owned() }),
                Delimiter::Regex(RegexDelimiter { regex: WITNESS_END_REGEX.to_owned() }),
            ],
            effects: vec![Effect::Log { log: true }],
        }],
        transactions: files
            .iter()
            .map(|path| Transaction {
                arrow: Arrow {
                    src: path.clone(),
                    dst: None,
                    log: Some(LogDestination::Pipe(LogPipe { pipe: PipeName::Stdout })),
                    pattern: None,
                },
                transform: vec![WITNESS_TRANSFORM.to_owned()],
            })
            .collect(),
        posts: Vec::new(),
    }
}
// sirno:witness:end

// sirno:witness:start witness
fn parse_witness_output(output: &str) -> Result<WitnessIndex, WitnessError> {
    let mut index = WitnessIndex::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let record: MosaikaLogRecord = serde_json::from_str(line).map_err(WitnessError::Json)?;
        let marker = record
            .delimiters
            .first()
            .ok_or_else(|| WitnessError::MissingDelimiter { line: line.to_owned() })?;
        let closing = record
            .delimiters
            .last()
            .ok_or_else(|| WitnessError::MissingDelimiter { line: line.to_owned() })?;
        let raw_entry = marker
            .captures
            .first()
            .ok_or_else(|| WitnessError::MissingCapture { line: line.to_owned() })?;
        let entry = EntryId::new(raw_entry).map_err(|source| WitnessError::InvalidEntryId {
            path: PathBuf::from(&record.file),
            marker: raw_entry.clone(),
            source,
        })?;
        index.push(WitnessRecord {
            entry,
            path: PathBuf::from(record.file),
            region: record.region.into(),
            opening: marker.span.into(),
            closing: closing.span.into(),
            marker: marker.matched.clone(),
            body: record.body,
        });
    }
    Ok(index)
}
// sirno:witness:end

fn has_glob_meta(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}

#[derive(Debug, Deserialize)]
struct MosaikaLogRecord {
    file: String,
    region: MosaikaSourceSpan,
    delimiters: Vec<MosaikaDelimiterRecord>,
    body: String,
}

#[derive(Debug, Deserialize)]
struct MosaikaDelimiterRecord {
    span: MosaikaSourceSpan,
    matched: String,
    captures: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct MosaikaSourceSpan {
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
}

impl From<MosaikaSourceSpan> for WitnessSpan {
    fn from(span: MosaikaSourceSpan) -> Self {
        Self {
            start_line: span.start_line,
            start_column: span.start_column,
            end_line: span.end_line,
            end_column: span.end_column,
        }
    }
}

/// Error raised while scanning repository witnesses.
#[derive(Debug, Error)]
pub enum WitnessError {
    /// A configured code member did not select any files.
    #[error("code member did not select any files: {member}")]
    MissingMember {
        /// Configured member pattern.
        member: String,
    },
    /// A configured glob is malformed.
    #[error("code member contains an invalid glob: {member}")]
    InvalidGlob {
        /// Configured member pattern.
        member: String,
        /// Underlying glob parse error.
        #[source]
        source: glob::PatternError,
    },
    /// Glob expansion failed.
    #[error("failed to expand code member glob: {member}")]
    Glob {
        /// Configured member pattern.
        member: String,
        /// Underlying glob expansion error.
        #[source]
        source: glob::GlobError,
    },
    /// A configured member resolved to an unsupported filesystem item.
    #[error("code member resolved to an unsupported filesystem item: {member} -> {path}")]
    UnsupportedMember {
        /// Configured member pattern.
        member: String,
        /// Resolved path.
        path: PathBuf,
    },
    /// A directory member could not be read.
    #[error("failed to read code member directory {path} from {member}")]
    ReadDirectory {
        /// Configured member pattern.
        member: String,
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The generated `mosaika` scheme is invalid.
    #[error("failed to build mosaika witness scheme")]
    Scheme(#[source] mosaika::semantics::SchemeError),
    /// The `mosaika` scan failed.
    #[error("failed to run mosaika witness scan")]
    Engine(#[source] mosaika::engine::EngineError),
    /// Mosaika output was not UTF-8.
    #[error("mosaika witness output was not UTF-8")]
    Utf8(#[source] std::string::FromUtf8Error),
    /// One Mosaika JSON record could not be decoded.
    #[error("failed to decode mosaika witness output")]
    Json(#[source] serde_json::Error),
    /// Mosaika emitted a record without delimiter data.
    #[error("mosaika witness output did not include delimiter data: {line}")]
    MissingDelimiter {
        /// Raw JSONL line.
        line: String,
    },
    /// Mosaika emitted a record without the witness id capture.
    #[error("mosaika witness output did not include a witness id capture: {line}")]
    MissingCapture {
        /// Raw JSONL line.
        line: String,
    },
    /// A witness block captured an invalid Sirno entry id.
    #[error("witness block `{marker}` in {path} is not a valid Sirno entry id")]
    InvalidEntryId {
        /// Repository path containing the marker.
        path: PathBuf,
        /// Captured marker payload.
        marker: String,
        /// Underlying id parse error.
        #[source]
        source: EntryIdError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn witness_block(id: &str) -> String {
        format!("// sirno:witness:start {id}\nbody\n// sirno:witness:end\n")
    }

    #[test]
    fn scans_recursive_directory_members_with_mosaika() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src/nested");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("lib.rs"), witness_block("witness-lookup")).unwrap();
        let settings = WitnessCheckSettings::new(temp.path(), [CodeMember::new("src").unwrap()]);

        let index = scan_witnesses(&settings).unwrap();
        let records = index.records_for(&EntryId::new("witness-lookup").unwrap());

        assert!(index.contains_entry(&EntryId::new("witness-lookup").unwrap()));
        assert_eq!(
            records[0].body,
            "// sirno:witness:start witness-lookup\nbody\n// sirno:witness:end"
        );
        assert_eq!(
            records[0].region,
            WitnessSpan { start_line: 1, start_column: 1, end_line: 3, end_column: 21 }
        );
        assert_eq!(
            records[0].opening,
            WitnessSpan { start_line: 1, start_column: 1, end_line: 1, end_column: 38 }
        );
        assert_eq!(
            records[0].closing,
            WitnessSpan { start_line: 3, start_column: 1, end_line: 3, end_column: 21 }
        );
    }

    #[test]
    fn scans_glob_members_with_mosaika() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("crates/core/src")).unwrap();
        std::fs::write(temp.path().join("crates/core/src/lib.rs"), witness_block("code-member"))
            .unwrap();
        let settings =
            WitnessCheckSettings::new(temp.path(), [CodeMember::new("crates/*/src").unwrap()]);

        let index = scan_witnesses(&settings).unwrap();

        assert!(index.contains_entry(&EntryId::new("code-member").unwrap()));
    }
}
