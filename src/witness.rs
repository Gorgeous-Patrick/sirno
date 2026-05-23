//! Repository witness lookup through `mosaika`.
//!
//! Sirno delegates repository witness scans to `mosaika`.
//! The Sirno layer owns member selection because `[repo].members`
//! accepts recursive directory members in addition to glob patterns.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use glob::glob;
use mosaika::engine::{
    CaptureRecord, DelimiterRecord, Engine, MatchRecord, ReplacementScope, RunAnalysis, SourceSpan,
    TextEditSet,
};
use mosaika::semantics::Scheme;
use mosaika::syntax::{
    self as syn, Arrow, Delimiter, Effect, LogDestination, LogPipe, Matching, PipeName,
    RegexDelimiter, Transaction, Transform,
};
use regex::Regex;
use thiserror::Error;
use tracing::trace;

use crate::config::RepoMember;
use crate::config::WitnessSettings;
use crate::identifier::{EntryAddress, EntryAddressError};

const WITNESS_TRANSFORM: &str = "sirno-witness";

/// Settings for a witness scan.
///
/// Invariant: `root` is the directory relative to which members are resolved.
/// `members` are already validated config-relative member patterns.
/// `witness` contains the delimiter regex pairs used by `mosaika`.
/// Empty delimiter settings disable scans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessCheckSettings {
    /// Directory relative to which repo members are resolved.
    pub root: PathBuf,
    /// Configured repository members scanned for witness blocks.
    pub members: Vec<RepoMember>,
    /// Configured witness delimiter syntax.
    pub witness: WitnessSettings,
}

impl WitnessCheckSettings {
    /// Construct witness settings from a config root, repo members, and delimiter syntax.
    pub fn new(
        root: impl Into<PathBuf>, members: impl IntoIterator<Item = RepoMember>,
        witness: WitnessSettings,
    ) -> Self {
        Self { root: root.into(), members: members.into_iter().collect(), witness }
    }

    /// Returns true when no repository witness scan can produce records.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty() || self.witness.delimiters.is_empty()
    }

    /// Scan configured repository members for Sirno witness blocks.
    ///
    /// The scan uses configured delimiter regexes.
    // sirno:witness:witness-lookup:begin
    pub fn scan(&self) -> Result<WitnessIndex, WitnessError> {
        trace!(
            root = %self.root.display(),
            member_count = self.members.len(),
            "scan_witnesses begin"
        );
        if self.is_empty() {
            trace!("scan_witnesses end: empty settings");
            return Ok(WitnessIndex::new());
        }
        let files = self.resolve_member_files()?;
        let analysis = self.run_mosaika_analysis(&files)?;
        let mut index = WitnessIndex::from_mosaika_matches(analysis.match_records())?;
        index.set_orphan_delimiters(self.orphan_delimiters(&files, &index)?);
        trace!(file_count = files.len(), "scan_witnesses end");
        Ok(index)
    }
    // sirno:witness:witness-lookup:end

    /// Rename configured witness sentinel paths that reference one entry.
    pub fn rename_entry_references(
        &self, old_id: &EntryAddress, new_id: &EntryAddress,
    ) -> Result<Vec<PathBuf>, WitnessError> {
        if old_id == new_id || self.is_empty() {
            return Ok(Vec::new());
        }

        let files = self.resolve_member_files()?;
        let analysis = self.run_mosaika_analysis(&files)?;
        let mut edits = TextEditSet::new();
        for record in analysis.match_records() {
            if &entry_address_from_match(record)? != old_id {
                continue;
            }
            add_witness_capture_edit(&mut edits, record, 0, new_id)?;
            add_witness_capture_edit(&mut edits, record, 1, new_id)?;
        }

        let report = edits.apply_in_place().map_err(WitnessError::Patch)?;
        Ok(report.changed_paths().iter().cloned().collect())
    }

    // sirno:witness:witness-lookup:begin
    fn resolve_member_files(&self) -> Result<Vec<PathBuf>, WitnessError> {
        let mut files = BTreeSet::new();
        for member in &self.members {
            let before = files.len();
            if member.has_glob_meta() {
                self.collect_glob_member(member, &mut files)?;
            } else {
                let path = self.root.join(member.as_str());
                self.collect_path_member(member, &path, &mut files)?;
            }
            if files.len() == before {
                return Err(WitnessError::MissingMember { member: member.as_str().to_owned() });
            }
        }
        Ok(files.into_iter().collect())
    }
    // sirno:witness:witness-lookup:end

    // sirno:witness:witness-lookup:begin
    fn collect_glob_member(
        &self, member: &RepoMember, files: &mut BTreeSet<PathBuf>,
    ) -> Result<(), WitnessError> {
        let pattern = self.root.join(member.as_str()).to_string_lossy().to_string();
        let matches = glob(&pattern).map_err(|source| WitnessError::InvalidGlob {
            member: member.as_str().to_owned(),
            source,
        })?;
        for path in matches {
            self.collect_path_member(
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
    // sirno:witness:witness-lookup:end

    // sirno:witness:witness-lookup:begin
    fn collect_path_member(
        &self, member: &RepoMember, path: &Path, files: &mut BTreeSet<PathBuf>,
    ) -> Result<(), WitnessError> {
        if !path.exists() {
            return Ok(());
        }
        if path.is_file() {
            files.insert(path.to_path_buf());
            return Ok(());
        }
        if path.is_dir() {
            self.collect_directory_files(member, path, files)?;
            return Ok(());
        }
        Err(WitnessError::UnsupportedMember {
            member: member.as_str().to_owned(),
            path: path.to_path_buf(),
        })
    }
    // sirno:witness:witness-lookup:end

    // sirno:witness:witness-lookup:begin
    fn collect_directory_files(
        &self, member: &RepoMember, root: &Path, files: &mut BTreeSet<PathBuf>,
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
                self.collect_directory_files(member, &path, files)?;
            } else if path.is_file() {
                files.insert(path);
            }
        }
        Ok(())
    }
    // sirno:witness:witness-lookup:end

    // sirno:witness:witness-lookup:begin
    fn run_mosaika_analysis(&self, files: &[PathBuf]) -> Result<RunAnalysis, WitnessError> {
        let projection = self.witness.projection(files);
        let scheme = Scheme::from_syntax(projection, &self.root).map_err(WitnessError::Scheme)?;
        Engine::new("sirno witness scan", scheme)
            .plan()
            .map_err(WitnessError::Engine)?
            .analyze()
            .map_err(WitnessError::Engine)
    }

    fn orphan_delimiters(
        &self, files: &[PathBuf], index: &WitnessIndex,
    ) -> Result<Vec<WitnessDelimiterToken>, WitnessError> {
        let resolved = index.resolved_delimiter_keys();
        let mut orphans = self
            .witness
            .delimiter_tokens(files)?
            .into_iter()
            .filter(|token| !resolved.contains(&token.key()))
            .collect::<Vec<_>>();
        orphans.sort_by_key(WitnessDelimiterToken::sort_key);
        Ok(orphans)
    }
    // sirno:witness:witness-lookup:end
}

/// Repository locations grouped by witnessed entry address.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WitnessIndex {
    records_by_entry: BTreeMap<EntryAddress, Vec<WitnessRecord>>,
    orphan_delimiters: Vec<WitnessDelimiterToken>,
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

    /// Return every record for one entry address.
    pub fn records_for(&self, id: &EntryAddress) -> &[WitnessRecord] {
        self.records_by_entry.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Returns true when the index contains at least one record for the entry.
    pub fn contains_entry(&self, id: &EntryAddress) -> bool {
        self.records_by_entry.contains_key(id)
    }

    /// Iterate over entry addresses with at least one witness block.
    pub fn entry_addresses(&self) -> impl Iterator<Item = &EntryAddress> {
        self.records_by_entry.keys()
    }

    // sirno:witness:witness-integrity:begin
    pub(crate) fn orphan_delimiters(&self) -> &[WitnessDelimiterToken] {
        &self.orphan_delimiters
    }

    fn set_orphan_delimiters(&mut self, orphan_delimiters: Vec<WitnessDelimiterToken>) {
        self.orphan_delimiters = orphan_delimiters;
    }
    // sirno:witness:witness-integrity:end

    fn resolved_delimiter_keys(&self) -> BTreeSet<WitnessDelimiterKey> {
        self.records_by_entry
            .values()
            .flat_map(|records| records.iter())
            .flat_map(|record| {
                [
                    WitnessDelimiterKey::new(
                        &record.path,
                        WitnessDelimiterKind::Begin,
                        record.opening,
                    ),
                    WitnessDelimiterKey::new(
                        &record.path,
                        WitnessDelimiterKind::End,
                        record.closing,
                    ),
                ]
            })
            .collect()
    }

    // sirno:witness:witness-lookup:begin
    fn from_mosaika_matches<'a>(
        records: impl IntoIterator<Item = &'a MatchRecord>,
    ) -> Result<Self, WitnessError> {
        let mut index = Self::new();
        for record in records {
            let (marker, closing) = witness_delimiters(record)?;
            let entry = entry_address_from_match(record)?;
            index.push(WitnessRecord {
                entry,
                path: record.source_path.clone(),
                region: WitnessSpan::from(&record.span),
                opening: witness_span_for_delimiter(marker),
                closing: witness_span_for_delimiter(closing),
                marker: marker.matched_text.clone(),
                body: record.matched_text.clone(),
            });
        }
        Ok(index)
    }
    // sirno:witness:witness-lookup:end
}

/// One repository witness block resolved by `mosaika`.
///
/// Invariant: `entry` is the parsed path captured from the opening delimiter.
/// `region` identifies the matched block.
/// `opening` and `closing` identify the delimiter spans.
#[derive(Clone, Debug, PartialEq, Eq)]
// sirno:witness:sirno-witness:begin
pub struct WitnessRecord {
    /// Entry address captured from `sirno:witness:<entry-address>:begin`.
    pub entry: EntryAddress,
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
// sirno:witness:sirno-witness:end

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum WitnessDelimiterKind {
    Begin,
    End,
}

impl WitnessDelimiterKind {
    fn label(self) -> &'static str {
        match self {
            | Self::Begin => "opening",
            | Self::End => "closing",
        }
    }

    fn missing_label(self) -> &'static str {
        match self {
            | Self::Begin => "closing",
            | Self::End => "opening",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WitnessDelimiterToken {
    path: PathBuf,
    kind: WitnessDelimiterKind,
    span: WitnessSpan,
    entry: EntryAddress,
}

impl WitnessDelimiterToken {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn diagnostic_message(&self) -> String {
        format!(
            "repository witness {} delimiter for `{}` at {}:{} has no {} delimiter",
            self.kind.label(),
            self.entry,
            self.span.start_line,
            self.span.start_column,
            self.kind.missing_label()
        )
    }

    fn key(&self) -> WitnessDelimiterKey {
        WitnessDelimiterKey::new(&self.path, self.kind, self.span)
    }

    fn sort_key(&self) -> WitnessDelimiterKey {
        self.key()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct WitnessDelimiterKey {
    path: PathBuf,
    kind: WitnessDelimiterKind,
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
}

impl WitnessDelimiterKey {
    fn new(path: &Path, kind: WitnessDelimiterKind, span: WitnessSpan) -> Self {
        Self {
            path: path.to_path_buf(),
            kind,
            start_line: span.start_line,
            start_column: span.start_column,
            end_line: span.end_line,
            end_column: span.end_column,
        }
    }
}

/// One source span reported by `mosaika`.
///
/// Invariant: line and column values are one-based.
/// End columns point after the matched span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// sirno:witness:sirno-witness:begin
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
// sirno:witness:sirno-witness:end

impl From<&SourceSpan> for WitnessSpan {
    fn from(span: &SourceSpan) -> Self {
        Self {
            start_line: span.start_line(),
            start_column: span.start_column(),
            end_line: span.end_line(),
            end_column: span.end_column(),
        }
    }
}

fn entry_address_from_match(record: &MatchRecord) -> Result<EntryAddress, WitnessError> {
    let (marker, closing) = witness_delimiters(record)?;
    let raw_entry = witness_capture(record, marker)?.text.as_str();
    let raw_closing_entry = witness_capture(record, closing)?.text.as_str();
    if raw_entry != raw_closing_entry {
        return Err(WitnessError::MismatchedEntryAddress {
            path: record.source_path.clone(),
            opening: raw_entry.to_owned(),
            closing: raw_closing_entry.to_owned(),
        });
    }
    EntryAddress::new(raw_entry).map_err(|source| WitnessError::InvalidEntryAddress {
        path: record.source_path.clone(),
        marker: raw_entry.to_owned(),
        source,
    })
}

fn witness_delimiters(
    record: &MatchRecord,
) -> Result<(&DelimiterRecord, &DelimiterRecord), WitnessError> {
    let marker = record.delimiters.first().ok_or_else(|| WitnessError::MissingDelimiter {
        path: record.source_path.clone(),
        transform: record.transform.clone(),
    })?;
    let closing = record.delimiters.last().ok_or_else(|| WitnessError::MissingDelimiter {
        path: record.source_path.clone(),
        transform: record.transform.clone(),
    })?;
    Ok((marker, closing))
}

fn witness_capture<'a>(
    record: &MatchRecord, delimiter: &'a DelimiterRecord,
) -> Result<&'a CaptureRecord, WitnessError> {
    delimiter.captures.first().ok_or_else(|| WitnessError::MissingCapture {
        path: record.source_path.clone(),
        transform: record.transform.clone(),
        delimiter_index: delimiter.delimiter_index,
    })
}

fn add_witness_capture_edit(
    edits: &mut TextEditSet, record: &MatchRecord, delimiter_index: usize, new_id: &EntryAddress,
) -> Result<(), WitnessError> {
    let edit = record
        .edit_for_scope(
            ReplacementScope::Capture { delimiter_index, capture_index: 0 },
            new_id.as_str(),
        )
        .ok_or_else(|| WitnessError::MissingCaptureSpan {
            path: record.source_path.clone(),
            transform: record.transform.clone(),
            delimiter_index,
            capture_index: 0,
        })?;
    edits.add(edit).map_err(WitnessError::Patch)
}

fn witness_span_for_delimiter(delimiter: &DelimiterRecord) -> WitnessSpan {
    let mut span = WitnessSpan::from(&delimiter.span);
    span.start_column += leading_whitespace_len(&delimiter.matched_text);
    span
}

fn leading_whitespace_len(line: &str) -> usize {
    line.bytes().take_while(|byte| matches!(byte, b' ' | b'\t')).count()
}

impl RepoMember {
    fn has_glob_meta(&self) -> bool {
        self.as_str().contains('*') || self.as_str().contains('?') || self.as_str().contains('[')
    }
}

impl WitnessSettings {
    // sirno:witness:witness-lookup:begin
    fn projection(&self, files: &[PathBuf]) -> syn::Projection {
        let transforms = self
            .delimiters
            .iter()
            .enumerate()
            .map(|(index, delimiter)| Transform {
                name: Self::transform_name(index),
                matching: Matching::Balanced,
                delimiters: vec![
                    Delimiter::Regex(RegexDelimiter { regex: delimiter.begin.clone() }),
                    Delimiter::Regex(RegexDelimiter { regex: delimiter.end.clone() }),
                ],
                effects: vec![Effect::Log { log: true }],
            })
            .collect::<Vec<_>>();
        let transform_names =
            (0..self.delimiters.len()).map(Self::transform_name).collect::<Vec<_>>();
        syn::Projection {
            transforms,
            transactions: files
                .iter()
                .map(|path| Transaction {
                    arrow: Arrow {
                        src: path.clone(),
                        dst: None,
                        log: Some(LogDestination::Pipe(LogPipe { pipe: PipeName::Stdout })),
                        pattern: None,
                    },
                    transform: transform_names.clone(),
                })
                .collect(),
            posts: Vec::new(),
        }
    }
    // sirno:witness:witness-lookup:end

    fn transform_name(index: usize) -> String {
        format!("{WITNESS_TRANSFORM}-{index}")
    }

    // sirno:witness:witness-lookup:begin
    fn delimiter_tokens(
        &self, files: &[PathBuf],
    ) -> Result<Vec<WitnessDelimiterToken>, WitnessError> {
        let regexes = self.delimiter_regexes()?;
        let mut tokens = Vec::new();
        for path in files {
            let content = fs::read_to_string(path)
                .map_err(|source| WitnessError::ReadFile { path: path.to_path_buf(), source })?;
            let locator = WitnessLocator::new(&content);
            for regex in &regexes {
                regex.scan(&content, path, &locator, &mut tokens)?;
            }
        }
        Ok(tokens)
    }

    fn delimiter_regexes(&self) -> Result<Vec<WitnessDelimiterRegex>, WitnessError> {
        self.delimiters
            .iter()
            .enumerate()
            .map(|(index, delimiter)| {
                WitnessDelimiterRegex::new(index, &delimiter.begin, &delimiter.end)
            })
            .collect()
    }
    // sirno:witness:witness-lookup:end
}

// sirno:witness:witness-lookup:begin
struct WitnessDelimiterRegex {
    index: usize,
    begin: Regex,
    end: Regex,
}

impl WitnessDelimiterRegex {
    fn new(index: usize, begin: &str, end: &str) -> Result<Self, WitnessError> {
        Ok(Self {
            index,
            begin: compile_witness_delimiter_regex("witness.delimiters.begin", index, begin)?,
            end: compile_witness_delimiter_regex("witness.delimiters.end", index, end)?,
        })
    }

    fn scan(
        &self, content: &str, path: &Path, locator: &WitnessLocator<'_>,
        tokens: &mut Vec<WitnessDelimiterToken>,
    ) -> Result<(), WitnessError> {
        scan_witness_delimiter_regex(
            &self.begin,
            self.index,
            WitnessDelimiterKind::Begin,
            content,
            path,
            locator,
            tokens,
        )?;
        scan_witness_delimiter_regex(
            &self.end,
            self.index,
            WitnessDelimiterKind::End,
            content,
            path,
            locator,
            tokens,
        )
    }
}

fn compile_witness_delimiter_regex(
    field: &'static str, index: usize, source: &str,
) -> Result<Regex, WitnessError> {
    let regex = Regex::new(source).map_err(|source| WitnessError::DelimiterRegex {
        field,
        index,
        source,
    })?;
    if regex.captures_len() < 2 {
        return Err(WitnessError::DelimiterCapture { field, index });
    }
    if regex.is_match("") {
        return Err(WitnessError::DelimiterEmptyMatch { field, index });
    }
    Ok(regex)
}

fn scan_witness_delimiter_regex(
    regex: &Regex, index: usize, kind: WitnessDelimiterKind, content: &str, path: &Path,
    locator: &WitnessLocator<'_>, tokens: &mut Vec<WitnessDelimiterToken>,
) -> Result<(), WitnessError> {
    for captures in regex.captures_iter(content) {
        let marker = captures.get(0).expect("regex captures include the full match");
        let raw_entry = captures.get(1).ok_or(WitnessError::MissingTokenCapture {
            path: path.to_path_buf(),
            delimiter_index: index,
            kind: kind.label(),
        })?;
        let entry = EntryAddress::new(raw_entry.as_str()).map_err(|source| {
            WitnessError::InvalidEntryAddress {
                path: path.to_path_buf(),
                marker: raw_entry.as_str().to_owned(),
                source,
            }
        })?;
        let mut span = locator.span(marker.start(), marker.end());
        span.start_column += leading_whitespace_len(marker.as_str());
        tokens.push(WitnessDelimiterToken { path: path.to_path_buf(), kind, span, entry });
    }
    Ok(())
}

struct WitnessLocator<'a> {
    content: &'a str,
    line_breaks: Vec<usize>,
}

impl<'a> WitnessLocator<'a> {
    fn new(content: &'a str) -> Self {
        Self { content, line_breaks: content.match_indices('\n').map(|(index, _)| index).collect() }
    }

    fn position(&self, byte_index: usize) -> (usize, usize) {
        let line_index = self.line_breaks.partition_point(|index| *index < byte_index);
        let line_start = if line_index == 0 { 0 } else { self.line_breaks[line_index - 1] + 1 };
        let column = self.content[line_start..byte_index].chars().count() + 1;
        (line_index + 1, column)
    }

    fn span(&self, start: usize, end: usize) -> WitnessSpan {
        let (start_line, start_column) = self.position(start);
        let (end_line, end_column) = self.position(end);
        WitnessSpan { start_line, start_column, end_line, end_column }
    }
}
// sirno:witness:witness-lookup:end

/// Error raised while scanning repository witnesses.
#[derive(Debug, Error)]
pub enum WitnessError {
    /// A configured repo member file could not be read.
    #[error("failed to read repo member file {path}")]
    ReadFile {
        /// File selected by configured repo members.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A configured repo member did not select any files.
    #[error("repo member did not select any files: {member}")]
    MissingMember {
        /// Configured member pattern.
        member: String,
    },
    /// A configured glob is malformed.
    #[error("repo member contains an invalid glob: {member}")]
    InvalidGlob {
        /// Configured member pattern.
        member: String,
        /// Underlying glob parse error.
        #[source]
        source: glob::PatternError,
    },
    /// Glob expansion failed.
    #[error("failed to expand repo member glob: {member}")]
    Glob {
        /// Configured member pattern.
        member: String,
        /// Underlying glob expansion error.
        #[source]
        source: glob::GlobError,
    },
    /// A configured member resolved to an unsupported filesystem item.
    #[error("repo member resolved to an unsupported filesystem item: {member} -> {path}")]
    UnsupportedMember {
        /// Configured member pattern.
        member: String,
        /// Resolved path.
        path: PathBuf,
    },
    /// A directory member could not be read.
    #[error("failed to read repo member directory {path} from {member}")]
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
    /// A witness delimiter regex is invalid.
    #[error("{field} at index {index} contains an invalid regex")]
    DelimiterRegex {
        /// Config field that contained an invalid regex.
        field: &'static str,
        /// Zero-based delimiter pair index.
        index: usize,
        /// Regex parser error.
        #[source]
        source: regex::Error,
    },
    /// A witness delimiter regex does not capture an entry address.
    #[error("{field} at index {index} must capture the entry address")]
    DelimiterCapture {
        /// Config field that did not declare a capture group.
        field: &'static str,
        /// Zero-based delimiter pair index.
        index: usize,
    },
    /// A witness delimiter regex can match empty text.
    #[error("{field} at index {index} must not match empty text")]
    DelimiterEmptyMatch {
        /// Config field that can match empty text.
        field: &'static str,
        /// Zero-based delimiter pair index.
        index: usize,
    },
    /// Mosaika witness analysis failed.
    #[error("failed to analyze mosaika witness matches")]
    Engine(#[source] mosaika::engine::EngineError),
    /// Mosaika in-place patching failed.
    #[error("failed to patch witness captures")]
    Patch(#[source] mosaika::engine::PatchError),
    /// Mosaika emitted a match without delimiter data.
    #[error("mosaika witness match in {path} from `{transform}` did not include delimiter data")]
    MissingDelimiter {
        /// Repository path containing the match.
        path: PathBuf,
        /// Transform that produced the match.
        transform: String,
    },
    /// Mosaika emitted a delimiter without the witness id capture.
    #[error(
        "mosaika witness delimiter {delimiter_index} in {path} from `{transform}` did not include a witness id capture"
    )]
    MissingCapture {
        /// Repository path containing the match.
        path: PathBuf,
        /// Transform that produced the match.
        transform: String,
        /// Delimiter missing the capture.
        delimiter_index: usize,
    },
    /// Mosaika emitted a capture that cannot be edited in place.
    #[error(
        "mosaika witness capture {capture_index} in delimiter {delimiter_index} in {path} from `{transform}` did not include an editable span"
    )]
    MissingCaptureSpan {
        /// Repository path containing the match.
        path: PathBuf,
        /// Transform that produced the match.
        transform: String,
        /// Delimiter missing the editable capture span.
        delimiter_index: usize,
        /// Capture missing the editable span.
        capture_index: usize,
    },
    /// A configured witness delimiter matched without the required capture.
    #[error(
        "witness {kind} delimiter {delimiter_index} in {path} did not capture an entry address"
    )]
    MissingTokenCapture {
        /// Repository path containing the delimiter.
        path: PathBuf,
        /// Zero-based delimiter pair index.
        delimiter_index: usize,
        /// Delimiter kind that matched.
        kind: &'static str,
    },
    /// A witness block opened and closed with different entry addresses.
    #[error("witness block in {path} opens for `{opening}` but closes for `{closing}`")]
    MismatchedEntryAddress {
        /// Repository path containing the block.
        path: PathBuf,
        /// Entry address captured from the opening delimiter.
        opening: String,
        /// Entry address captured from the closing delimiter.
        closing: String,
    },
    /// A witness block captured an invalid Sirno entry address.
    #[error("witness block `{marker}` in {path} is not a valid Sirno entry address")]
    InvalidEntryAddress {
        /// Repository path containing the marker.
        path: PathBuf,
        /// Captured marker payload.
        marker: String,
        /// Underlying path parse error.
        #[source]
        source: EntryAddressError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WitnessDelimiterSettings;

    // sirno:witness:witness-lookup:begin
    const WITNESS_COMMENT_PREFIX: &str = "// sirno";
    const WITNESS_SENTINEL_PREFIX: &str = ":witness:";
    const WITNESS_BEGIN_SUFFIX: &str = ":begin";
    const WITNESS_END_SUFFIX: &str = ":end";

    fn witness_begin(id: &str) -> String {
        format!("{WITNESS_COMMENT_PREFIX}{WITNESS_SENTINEL_PREFIX}{id}{WITNESS_BEGIN_SUFFIX}")
    }

    fn witness_end(id: &str) -> String {
        format!("{WITNESS_COMMENT_PREFIX}{WITNESS_SENTINEL_PREFIX}{id}{WITNESS_END_SUFFIX}")
    }

    fn witness_block_with_end(id: &str, end_id: &str) -> String {
        format!("{}\nbody\n{}\n", witness_begin(id), witness_end(end_id))
    }

    fn witness_block(id: &str) -> String {
        witness_block_with_end(id, id)
    }
    // sirno:witness:witness-lookup:end

    fn markdown_witness_block(id: &str) -> String {
        format!("<!-- sirno:witness:{id}:begin -->\nbody\n<!-- sirno:witness:{id}:end -->\n")
    }

    fn custom_witness_block(id: &str) -> String {
        format!("BEGIN {id}\nbody\nEND {id}\n")
    }

    fn indented_witness_block(id: &str) -> String {
        format!("    {}\n        body\n    {}\n", witness_begin(id), witness_end(id))
    }

    fn nested_witness_block(outer: &str, inner: &str) -> String {
        format!(
            "{}\nouter start\n{}\ninner body\n{}\nouter end\n{}\n",
            witness_begin(outer),
            witness_begin(inner),
            witness_end(inner),
            witness_end(outer)
        )
    }

    #[test]
    fn scans_recursive_directory_members_with_mosaika() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src/nested");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("lib.rs"), witness_block("witness-lookup")).unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("src").unwrap()],
            WitnessSettings::standard(),
        );

        let index = settings.scan().unwrap();
        let records = index.records_for(&EntryAddress::new("witness-lookup").unwrap());

        assert!(index.contains_entry(&EntryAddress::new("witness-lookup").unwrap()));
        assert_eq!(records[0].body, witness_block("witness-lookup").trim_end());
        assert_eq!(
            records[0].region,
            WitnessSpan { start_line: 1, start_column: 1, end_line: 3, end_column: 36 }
        );
        assert_eq!(
            records[0].opening,
            WitnessSpan { start_line: 1, start_column: 1, end_line: 1, end_column: 38 }
        );
        assert_eq!(
            records[0].closing,
            WitnessSpan { start_line: 3, start_column: 1, end_line: 3, end_column: 36 }
        );
    }

    #[test]
    fn scans_glob_members_with_mosaika() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("crates/core/src")).unwrap();
        std::fs::write(temp.path().join("crates/core/src/lib.rs"), witness_block("repo-member"))
            .unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("crates/*/src").unwrap()],
            WitnessSettings::standard(),
        );

        let index = settings.scan().unwrap();

        assert!(index.contains_entry(&EntryAddress::new("repo-member").unwrap()));
    }

    #[test]
    fn scans_markdown_comment_witness_blocks() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("README.md"), markdown_witness_block("readme")).unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("README.md").unwrap()],
            WitnessSettings::standard(),
        );

        let index = settings.scan().unwrap();
        let records = index.records_for(&EntryAddress::new("readme").unwrap());

        assert!(index.contains_entry(&EntryAddress::new("readme").unwrap()));
        assert_eq!(records[0].body, markdown_witness_block("readme").trim_end());
        assert_eq!(
            records[0].opening,
            WitnessSpan { start_line: 1, start_column: 1, end_line: 1, end_column: 36 }
        );
        assert_eq!(
            records[0].closing,
            WitnessSpan { start_line: 3, start_column: 1, end_line: 3, end_column: 34 }
        );
    }

    #[test]
    fn renames_standard_witness_entry_references() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            format!(
                "    {}\n        let preserved = \"old-entry stays in the body\";\n    {}\n",
                witness_begin("old-entry"),
                witness_end("old-entry")
            ),
        )
        .unwrap();
        std::fs::write(temp.path().join("README.md"), markdown_witness_block("old-entry")).unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("src").unwrap(), RepoMember::new("README.md").unwrap()],
            WitnessSettings::standard(),
        );

        let paths = settings
            .rename_entry_references(
                &EntryAddress::new("old-entry").unwrap(),
                &EntryAddress::new("new-entry").unwrap(),
            )
            .unwrap();
        let rust_source = std::fs::read_to_string(src.join("lib.rs")).unwrap();
        let readme_source = std::fs::read_to_string(temp.path().join("README.md")).unwrap();

        assert_eq!(paths.len(), 2);
        assert!(rust_source.contains("sirno:witness:new-entry:begin"));
        assert!(rust_source.contains("sirno:witness:new-entry:end"));
        assert!(rust_source.contains("old-entry stays in the body"));
        assert!(!rust_source.contains("sirno:witness:old-entry"));
        assert!(readme_source.contains("sirno:witness:new-entry:begin"));
        assert!(!readme_source.contains("old-entry"));
    }

    #[test]
    fn scans_standard_witness_blocks_for_filename_like_entry_addresses() {
        let temp = tempfile::tempdir().unwrap();
        let id = EntryAddress::new("Design Note_v2+1").unwrap();
        std::fs::write(temp.path().join("README.md"), markdown_witness_block(id.as_str())).unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("README.md").unwrap()],
            WitnessSettings::standard(),
        );

        let index = settings.scan().unwrap();

        assert!(index.contains_entry(&id));
    }

    #[test]
    fn scans_standard_witness_blocks_for_dotted_entry_addresses() {
        let temp = tempfile::tempdir().unwrap();
        let id = EntryAddress::new("core.design").unwrap();
        std::fs::write(temp.path().join("README.md"), markdown_witness_block(id.as_str())).unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("README.md").unwrap()],
            WitnessSettings::standard(),
        );

        let index = settings.scan().unwrap();
        let records = index.records_for(&id);

        assert!(index.contains_entry(&id));
        assert_eq!(records[0].entry, id);
    }

    #[test]
    fn scans_nested_witness_blocks_with_balanced_mosaika_matching() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("README.md"), nested_witness_block("outer", "inner"))
            .unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("README.md").unwrap()],
            WitnessSettings::standard(),
        );

        let index = settings.scan().unwrap();
        let outer = index.records_for(&EntryAddress::new("outer").unwrap());
        let inner = index.records_for(&EntryAddress::new("inner").unwrap());

        assert!(index.orphan_delimiters().is_empty());
        assert_eq!(outer.len(), 1);
        assert_eq!(inner.len(), 1);
        assert_eq!(
            outer[0].region,
            WitnessSpan { start_line: 1, start_column: 1, end_line: 7, end_column: 27 }
        );
        assert_eq!(
            inner[0].region,
            WitnessSpan { start_line: 3, start_column: 1, end_line: 5, end_column: 27 }
        );
    }

    #[test]
    fn scans_configured_witness_syntax() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("notes.txt"), custom_witness_block("custom")).unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("notes.txt").unwrap()],
            WitnessSettings {
                delimiters: vec![WitnessDelimiterSettings::new(
                    r"(?m)^BEGIN ([A-Za-z0-9_-]+)$",
                    r"(?m)^END ([A-Za-z0-9_-]+)$",
                )],
            },
        );

        let index = settings.scan().unwrap();
        let records = index.records_for(&EntryAddress::new("custom").unwrap());

        assert!(index.contains_entry(&EntryAddress::new("custom").unwrap()));
        assert_eq!(records[0].body, custom_witness_block("custom").trim_end());
    }

    #[test]
    fn empty_witness_delimiters_disable_scanning() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("lib.rs"), witness_block("witness-lookup")).unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("src").unwrap()],
            WitnessSettings { delimiters: Vec::new() },
        );

        let index = settings.scan().unwrap();

        assert!(settings.is_empty());
        assert!(!index.contains_entry(&EntryAddress::new("witness-lookup").unwrap()));
        assert!(index.entry_addresses().next().is_none());
        assert!(index.orphan_delimiters().is_empty());
    }

    #[test]
    fn delimiter_spans_exclude_prefixing_spaces() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("lib.rs"), indented_witness_block("witness-lookup")).unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("src").unwrap()],
            WitnessSettings::standard(),
        );

        let index = settings.scan().unwrap();
        let records = index.records_for(&EntryAddress::new("witness-lookup").unwrap());

        assert_eq!(
            records[0].opening,
            WitnessSpan { start_line: 1, start_column: 5, end_line: 1, end_column: 42 }
        );
        assert_eq!(
            records[0].closing,
            WitnessSpan { start_line: 3, start_column: 5, end_line: 3, end_column: 40 }
        );
    }

    #[test]
    fn rejects_mismatched_witness_sentinel_paths() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("lib.rs"), witness_block_with_end("witness-lookup", "query"))
            .unwrap();
        let settings = WitnessCheckSettings::new(
            temp.path(),
            [RepoMember::new("src").unwrap()],
            WitnessSettings::standard(),
        );

        let error = settings.scan().unwrap_err();

        assert!(matches!(
            error,
            WitnessError::MismatchedEntryAddress { opening, closing, .. }
                if opening == "witness-lookup" && closing == "query"
        ));
    }
}
