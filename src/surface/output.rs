//! Human and JSON rendering helpers for command results.

use std::env;
use std::path::{Path, PathBuf};

use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL};
use indexmap::IndexMap;
use serde::Serialize;
use unicode_width::UnicodeWidthStr;

use crate::surface::dto::{
    ConfigCommentResult, DiagnosticRecord, LakeCheckResult, PathRecord, QueryColumn, QueryColumns,
    QueryOutputFormat, QueryResults, RenderResult, SkillWrapperRecord, StatusResult,
};
use crate::surface::error::CommandError;
use crate::{
    Entry, EntryDirectoryError, EntryDirectoryReport, FrostLockStatus, SirnoLock, WitnessRecord,
};

/// Render any serializable value as pretty JSON.
pub fn format_json<T: Serialize + ?Sized>(value: &T) -> Result<String, CommandError> {
    Ok(serde_json::to_string_pretty(value)?)
}

pub(crate) fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<(), CommandError> {
    println!("{}", format_json(value)?);
    Ok(())
}

pub(crate) fn print_witness_records(records: &[WitnessRecord], full: bool) {
    print!("{}", format_witness_records(records, full));
}

pub(crate) fn format_witness_records(records: &[WitnessRecord], full: bool) -> String {
    let mut out = String::new();
    for (index, record) in records.iter().enumerate() {
        if full && index > 0 {
            out.push_str("---\n\n");
        }
        out.push_str(&format_witness_record(record, full));
    }
    out
}

pub(crate) fn format_witness_record(record: &WitnessRecord, full: bool) -> String {
    let range = format_witness_summary(record);
    if !full {
        let marker =
            record.body.lines().next().map(str::to_owned).unwrap_or_else(|| record.marker.clone());
        return format!("{range}\t{marker}\n");
    }

    let mut out = format!("{range}\n\n");
    out.push_str(&record.body);
    if !record.body.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out
}

fn format_witness_summary(record: &WitnessRecord) -> String {
    format!(
        "{}:{}:{}-{} :: {}:{}-{}",
        record.path.display(),
        record.opening.start_line,
        record.opening.start_column,
        record.opening.end_column,
        record.closing.start_line,
        record.closing.start_column,
        record.closing.end_column
    )
}

pub(crate) fn display_path(path: &Path) -> String {
    path.display().to_string()
}

pub(crate) fn display_paths(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().map(|path| display_path(path)).collect()
}

pub(crate) fn diagnostics_from_entry_report(
    report: &EntryDirectoryReport,
) -> Vec<DiagnosticRecord> {
    let mut diagnostics = Vec::new();
    for diagnostic in report.file_diagnostics() {
        diagnostics.push(DiagnosticRecord {
            severity: diagnostic.severity.label().to_owned(),
            path: Some(display_path(&diagnostic.path)),
            message: diagnostic.message.clone(),
        });
    }
    for diagnostic in report.structural_report().diagnostics() {
        diagnostics.push(DiagnosticRecord {
            severity: diagnostic.severity.label().to_owned(),
            path: diagnostic
                .entry
                .as_ref()
                .and_then(|entry| report.entry_path(entry))
                .map(display_path),
            message: diagnostic.message(),
        });
    }
    diagnostics
}

pub(crate) fn print_status_result(result: &StatusResult) {
    println!("config: {}", result.config_path);
    println!("lake: {}", result.lake_path);
    if let Some(frost) = &result.frost_path {
        println!("frost: {frost}");
        println!("frost-state: {}", result.frost_state);
    } else {
        println!("frost: (not configured)");
    }
    println!("entries: {}", result.entry_count);
    println!("checks:");
    println!("  render: {}", result.check_render);
    println!("structural:");
    for field in &result.structural_fields {
        println!("  {}.to: {}", field.field, field.to);
        println!("  {}.from: {}", field.field, field.from);
        println!("  {}.clique: {}", field.field, field.clique);
    }
    if result.ok {
        println!("check: ok");
    } else {
        print_diagnostics(&result.check.diagnostics);
        println!("check: failed");
    }
}

pub(crate) fn print_lake_check_result(result: &LakeCheckResult) {
    print!("{}", format_lake_check_result(result));
}

pub(crate) fn format_lake_check_result(result: &LakeCheckResult) -> String {
    if result.diagnostics.is_empty() {
        return format!("ok: {}\n", result.root);
    }

    let mut output = format_diagnostics(&result.diagnostics);
    output.push_str(&format!("{}\n", lake_check_summary(result)));
    output
}

fn lake_check_summary(result: &LakeCheckResult) -> String {
    if result.has_errors {
        format!("check: failed in {}", result.root)
    } else {
        format!("check: warnings in {}", result.root)
    }
}

pub(crate) fn print_render_result(result: &RenderResult) {
    print!("{}", format_render_result(result));
}

pub(crate) fn format_render_result(result: &RenderResult) -> String {
    if result.diagnostics.is_empty() {
        return format!("{}\n", result.message);
    }

    let mut output = format_diagnostics(&result.diagnostics);
    output.push_str(&result.message);
    output.push('\n');
    output
}

pub(crate) fn print_config_comment_result(result: &ConfigCommentResult) {
    print!("{}", format_config_comment_result(result));
}

pub(crate) fn format_config_comment_result(result: &ConfigCommentResult) -> String {
    let mut output = String::new();
    if !result.changed {
        for comment in &result.missing_comments {
            output.push_str("missing: ");
            output.push_str(comment);
            output.push('\n');
        }
    }
    output.push_str(&result.message);
    output.push('\n');
    output
}

fn print_diagnostics(diagnostics: &[DiagnosticRecord]) {
    print!("{}", format_diagnostics(diagnostics));
}

fn format_diagnostics(diagnostics: &[DiagnosticRecord]) -> String {
    let mut output = String::new();
    for diagnostic in diagnostics {
        if let Some(path) = &diagnostic.path {
            output
                .push_str(&format!("{}: {}: {}\n", diagnostic.severity, path, diagnostic.message));
        } else {
            output.push_str(&format!("{}: {}\n", diagnostic.severity, diagnostic.message));
        }
    }
    output
}

pub(crate) fn frost_state_label(lock: Option<&SirnoLock>) -> String {
    let Some(lock) = lock else {
        return "(unlocked)".to_owned();
    };
    match lock.frost.status {
        | FrostLockStatus::Current => {
            format!(
                "current version {} (generation {}, mutable)",
                lock.frost.version, lock.frost.generation
            )
        }
        | FrostLockStatus::CheckedOut if lock.frost.mutable => {
            format!(
                "checked-out version {} (generation {}, unsafe mutable)",
                lock.frost.version, lock.frost.generation
            )
        }
        | FrostLockStatus::CheckedOut => {
            format!(
                "checked-out version {} (generation {}, immutable)",
                lock.frost.version, lock.frost.generation
            )
        }
    }
}

pub(crate) fn format_gen_link_report(
    root: &Path, entry_count: usize, changed_paths: &[PathBuf],
) -> String {
    if changed_paths.is_empty() {
        return format!("No changes in {}", root.display());
    }

    let mut report = String::new();
    for path in changed_paths {
        report.push_str("- ");
        report.push_str(&path.display().to_string());
        report.push('\n');
    }
    report.push_str("Total changes: ");
    report.push_str(&changed_paths.len().to_string());
    report.push('/');
    report.push_str(&entry_count.to_string());
    report.push_str(" in ");
    report.push_str(&root.display().to_string());
    report
}

pub(crate) fn print_query_results(
    results: &QueryResults, format: QueryOutputFormat,
) -> Result<(), CommandError> {
    match format {
        | QueryOutputFormat::Json => {
            println!("{}", results.to_json()?);
        }
        | QueryOutputFormat::Human => {
            print!("{}", format_query_table(&results.columns, &results.rows));
        }
    }
    Ok(())
}

pub(crate) fn output_path(path: PathBuf, absolute: bool) -> Result<PathBuf, CommandError> {
    if !absolute || path.is_absolute() {
        return Ok(path);
    }
    Ok(env::current_dir().map_err(CommandError::CurrentDirectory)?.join(path))
}

pub(crate) fn format_path_table(records: &[PathRecord]) -> String {
    let headers = ["kind", "path"];
    let rows = records.iter().map(|record| [record.kind, record.path.as_str()]);
    format_human_table(headers, rows)
}

pub(crate) fn format_skill_wrapper_table(records: &[SkillWrapperRecord]) -> String {
    let headers = ["status", "name", "target"];
    let rows = records
        .iter()
        .map(|record| [record.status.as_str(), record.name.as_str(), record.target_path.as_str()]);
    format_human_table(headers, rows)
}

pub(crate) fn query_result_rows(
    report: &EntryDirectoryReport, entries: &[&Entry], columns: &QueryColumns,
) -> Result<Vec<Vec<String>>, CommandError> {
    entries
        .iter()
        .map(|entry| {
            columns
                .columns
                .iter()
                .map(|column| format_query_column(report, entry, *column))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect()
}

fn format_query_column(
    report: &EntryDirectoryReport, entry: &Entry, column: QueryColumn,
) -> Result<String, CommandError> {
    match column {
        | QueryColumn::Id => Ok(entry.id.to_string()),
        | QueryColumn::Name => Ok(entry.metadata.name.clone()),
        | QueryColumn::Path => {
            let path = report
                .entry_path(&entry.id)
                .ok_or_else(|| EntryDirectoryError::MissingEntryPath(entry.id.clone()))?;
            Ok(path.display().to_string())
        }
        | QueryColumn::Desc => Ok(entry.metadata.desc.clone()),
    }
}

pub(crate) fn format_query_json(
    columns: &QueryColumns, rows: &[Vec<String>],
) -> Result<String, CommandError> {
    format_json(&query_result_records(columns, rows))
}

pub(crate) fn query_result_records(
    columns: &QueryColumns, rows: &[Vec<String>],
) -> Vec<IndexMap<String, String>> {
    rows.iter()
        .map(|row| {
            columns
                .columns
                .iter()
                .zip(row)
                .map(|(column, value)| (column.label().to_owned(), value.clone()))
                .collect()
        })
        .collect()
}

pub(crate) fn format_query_table(columns: &QueryColumns, rows: &[Vec<String>]) -> String {
    let headers = columns.columns.iter().map(|column| column.label()).collect::<Vec<_>>();
    format_human_table(headers, rows.iter().map(|row| row.iter().map(String::as_str)))
}

fn format_human_table<'a>(
    headers: impl IntoIterator<Item = &'a str>,
    rows: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
) -> String {
    let headers = headers.into_iter().map(str::to_owned).collect::<Vec<_>>();
    let rows = rows
        .into_iter()
        .map(|row| row.into_iter().map(str::to_owned).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    format_human_table_with_width(headers, rows, None)
}

pub(crate) fn format_human_table_with_width(
    headers: Vec<String>, rows: Vec<Vec<String>>, width: Option<u16>,
) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    if let Some(width) = width {
        table.set_width(width);
    }
    let (headers, rows) = elide_human_table_columns(headers, rows, table.width());
    table.set_header(headers);
    table.add_rows(rows);
    let mut output = table.to_string();
    output.push('\n');
    output
}

fn elide_human_table_columns(
    headers: Vec<String>, rows: Vec<Vec<String>>, width: Option<u16>,
) -> (Vec<String>, Vec<Vec<String>>) {
    let Some(width) = width.map(usize::from) else {
        return (headers, rows);
    };
    if headers.len() <= 2 || min_table_width(&headers) <= width {
        return (headers, rows);
    }

    for visible in (1..headers.len()).rev() {
        let mut candidate_headers = headers.iter().take(visible).cloned().collect::<Vec<_>>();
        candidate_headers.push("...".to_owned());
        if min_table_width(&candidate_headers) <= width {
            let candidate_rows = rows
                .into_iter()
                .map(|row| {
                    let mut cells = row.into_iter().take(visible).collect::<Vec<_>>();
                    cells.push("...".to_owned());
                    cells
                })
                .collect();
            return (candidate_headers, candidate_rows);
        }
    }

    (
        headers.into_iter().take(1).collect(),
        rows.into_iter().map(|row| row.into_iter().take(1).collect()).collect(),
    )
}

fn min_table_width(headers: &[String]) -> usize {
    headers.iter().map(|header| UnicodeWidthStr::width(header.as_str()).max(1)).sum::<usize>()
        + headers.len() * 3
        + 1
}

pub(crate) fn print_entry_directory_report(report: &EntryDirectoryReport) {
    if report.is_clean() {
        println!("ok: {}", report.root().display());
        return;
    }

    print!("{}", format_entry_directory_report(report));
}

fn format_entry_directory_report(report: &EntryDirectoryReport) -> String {
    let mut output = String::new();
    for diagnostic in report.file_diagnostics() {
        output.push_str(&format!(
            "{}: {}: {}\n",
            diagnostic.severity.label(),
            diagnostic.path.display(),
            diagnostic.message
        ));
    }

    for diagnostic in report.structural_report().diagnostics() {
        if let Some(path) = diagnostic.entry.as_ref().and_then(|entry| report.entry_path(entry)) {
            output.push_str(&format!(
                "{}: {}: {}\n",
                diagnostic.severity.label(),
                path.display(),
                diagnostic.message()
            ));
        } else {
            output.push_str(&format!(
                "{}: {}\n",
                diagnostic.severity.label(),
                diagnostic.message()
            ));
        }
    }
    output.push_str(&format!("{}\n", entry_directory_report_summary(report)));
    output
}

fn entry_directory_report_summary(report: &EntryDirectoryReport) -> String {
    if report.has_errors() {
        format!("check: failed in {}", report.root().display())
    } else {
        format!("check: warnings in {}", report.root().display())
    }
}
