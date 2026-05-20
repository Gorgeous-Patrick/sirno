//! Human and JSON rendering helpers for command results.

use std::env;
use std::path::{Path, PathBuf};

use anstyle::{AnsiColor, Style};
use comfy_table::{Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL};
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

// sirno:witness:interfaces:begin
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OutputStyle {
    Plain,
    Styled,
    #[cfg(test)]
    Forced,
}

impl OutputStyle {
    fn colors(self) -> bool {
        self != Self::Plain
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticStyle {
    Changed,
    Error,
    Muted,
    Success,
    Warning,
}

impl SemanticStyle {
    fn text_style(self) -> Style {
        let color = match self {
            | Self::Changed => AnsiColor::Cyan,
            | Self::Error => AnsiColor::Red,
            | Self::Muted => AnsiColor::BrightBlack,
            | Self::Success => AnsiColor::Green,
            | Self::Warning => AnsiColor::Yellow,
        };
        Style::new().fg_color(Some(color.into()))
    }

    fn table_color(self) -> TableColor {
        match self {
            | Self::Changed => TableColor::Cyan,
            | Self::Error => TableColor::Red,
            | Self::Muted => TableColor::DarkGrey,
            | Self::Success => TableColor::Green,
            | Self::Warning => TableColor::Yellow,
        }
    }
}

pub(crate) fn print_cli_error(error: &CommandError) {
    anstream::eprintln!(
        "{} {error}",
        style_text("sirno:", SemanticStyle::Error, OutputStyle::Styled)
    );
}
// sirno:witness:interfaces:end

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
    let style = OutputStyle::Styled;
    anstream::println!("config: {}", result.config_path);
    anstream::println!("lake: {}", result.lake_path);
    if let Some(frost) = &result.frost_path {
        anstream::println!("frost: {frost}");
        anstream::println!("frost-state: {}", styled_frost_state(&result.frost_state, style));
    } else {
        anstream::println!(
            "frost: {}",
            style_text("(not configured)", SemanticStyle::Muted, style)
        );
    }
    anstream::println!("entries: {}", result.entry_count);
    anstream::println!("checks:");
    anstream::println!("  render: {}", result.check_render);
    anstream::println!("structural:");
    for field in &result.structural_fields {
        anstream::println!("  {}.to: {}", field.field, field.to);
        anstream::println!("  {}.from: {}", field.field, field.from);
        anstream::println!("  {}.clique: {}", field.field, field.clique);
    }
    if result.ok {
        anstream::println!("check: {}", style_text("ok", SemanticStyle::Success, style));
    } else {
        print_diagnostics_with_style(&result.check.diagnostics, style);
        anstream::println!("check: {}", style_text("failed", SemanticStyle::Error, style));
    }
}

pub(crate) fn print_lake_check_result(result: &LakeCheckResult) {
    anstream::print!("{}", format_lake_check_result_with_style(result, OutputStyle::Styled));
}

#[cfg(test)]
pub(crate) fn format_lake_check_result(result: &LakeCheckResult) -> String {
    format_lake_check_result_with_style(result, OutputStyle::Plain)
}

fn format_lake_check_result_with_style(result: &LakeCheckResult, style: OutputStyle) -> String {
    if result.diagnostics.is_empty() {
        return format!("{}\n", format_ok_line(&result.root, style));
    }

    let mut output = format_diagnostics_with_style(&result.diagnostics, style);
    output.push_str(&format!("{}\n", lake_check_summary(result, style)));
    output
}

fn lake_check_summary(result: &LakeCheckResult, style: OutputStyle) -> String {
    if result.has_errors {
        format!("check: {} in {}", style_text("failed", SemanticStyle::Error, style), result.root)
    } else {
        format!(
            "check: {} in {}",
            style_text("warnings", SemanticStyle::Warning, style),
            result.root
        )
    }
}

pub(crate) fn print_render_result(result: &RenderResult) {
    anstream::print!("{}", format_render_result_with_style(result, OutputStyle::Styled));
}

#[cfg(test)]
pub(crate) fn format_render_result(result: &RenderResult) -> String {
    format_render_result_with_style(result, OutputStyle::Plain)
}

fn format_render_result_with_style(result: &RenderResult, style: OutputStyle) -> String {
    if result.diagnostics.is_empty() {
        return format!("{}\n", result.message);
    }

    let mut output = format_diagnostics_with_style(&result.diagnostics, style);
    output.push_str(&result.message);
    output.push('\n');
    output
}

pub(crate) fn print_config_comment_result(result: &ConfigCommentResult) {
    anstream::print!("{}", format_config_comment_result_with_style(result, OutputStyle::Styled));
}

#[cfg(test)]
pub(crate) fn format_config_comment_result(result: &ConfigCommentResult) -> String {
    format_config_comment_result_with_style(result, OutputStyle::Plain)
}

fn format_config_comment_result_with_style(
    result: &ConfigCommentResult, style: OutputStyle,
) -> String {
    let mut output = String::new();
    if !result.changed {
        for comment in &result.missing_comments {
            output.push_str(&style_text("missing:", SemanticStyle::Warning, style));
            output.push(' ');
            output.push_str(comment);
            output.push('\n');
        }
    }
    output.push_str(&result.message);
    output.push('\n');
    output
}

fn print_diagnostics_with_style(diagnostics: &[DiagnosticRecord], style: OutputStyle) {
    anstream::print!("{}", format_diagnostics_with_style(diagnostics, style));
}

fn format_diagnostics_with_style(diagnostics: &[DiagnosticRecord], style: OutputStyle) -> String {
    let mut output = String::new();
    for diagnostic in diagnostics {
        let severity = styled_diagnostic_severity(&diagnostic.severity, style);
        if let Some(path) = &diagnostic.path {
            output.push_str(&format!("{severity}: {path}: {}\n", diagnostic.message));
        } else {
            output.push_str(&format!("{severity}: {}\n", diagnostic.message));
        }
    }
    output
}

// sirno:witness:interfaces:begin
fn styled_diagnostic_severity(severity: &str, style: OutputStyle) -> String {
    match severity {
        | "error" => style_text(severity, SemanticStyle::Error, style),
        | "warning" => style_text(severity, SemanticStyle::Warning, style),
        | _ => severity.to_owned(),
    }
}

fn styled_frost_state(value: &str, style: OutputStyle) -> String {
    if value.starts_with("current ") {
        style_text(value, SemanticStyle::Success, style)
    } else if value.starts_with("checked-out ") {
        style_text(value, SemanticStyle::Warning, style)
    } else if value == "(unlocked)" {
        style_text(value, SemanticStyle::Muted, style)
    } else {
        value.to_owned()
    }
}

fn style_text(value: &str, semantic: SemanticStyle, style: OutputStyle) -> String {
    if !style.colors() {
        return value.to_owned();
    }
    let text_style = semantic.text_style();
    format!("{text_style}{value}{text_style:#}")
}
// sirno:witness:interfaces:end

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

#[cfg(test)]
pub(crate) fn format_skill_wrapper_table(records: &[SkillWrapperRecord]) -> String {
    format_skill_wrapper_table_with_style(records, OutputStyle::Plain)
}

pub(crate) fn format_skill_wrapper_table_for_terminal(records: &[SkillWrapperRecord]) -> String {
    format_skill_wrapper_table_with_style(records, OutputStyle::Styled)
}

fn format_skill_wrapper_table_with_style(
    records: &[SkillWrapperRecord], style: OutputStyle,
) -> String {
    let headers = ["status", "name", "target"];
    let rows = records
        .iter()
        .map(|record| [record.status.as_str(), record.name.as_str(), record.target_path.as_str()]);
    format_human_table_with_style(headers, rows, style)
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
    format_human_table_with_style(headers, rows, OutputStyle::Plain)
}

fn format_human_table_with_style<'a>(
    headers: impl IntoIterator<Item = &'a str>,
    rows: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>, style: OutputStyle,
) -> String {
    let headers = headers.into_iter().map(str::to_owned).collect::<Vec<_>>();
    let rows = rows
        .into_iter()
        .map(|row| row.into_iter().map(str::to_owned).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    format_human_table_with_width_and_style(headers, rows, None, style)
}

#[cfg(test)]
pub(crate) fn format_human_table_with_width(
    headers: Vec<String>, rows: Vec<Vec<String>>, width: Option<u16>,
) -> String {
    format_human_table_with_width_and_style(headers, rows, width, OutputStyle::Plain)
}

pub(crate) fn format_human_table_semantic_with_width(
    headers: Vec<String>, rows: Vec<Vec<String>>, width: Option<u16>, style: OutputStyle,
) -> String {
    format_human_table_with_width_and_style(headers, rows, width, style)
}

fn format_human_table_with_width_and_style(
    headers: Vec<String>, rows: Vec<Vec<String>>, width: Option<u16>, style: OutputStyle,
) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    if let Some(width) = width {
        table.set_width(width);
    }
    #[cfg(test)]
    if style == OutputStyle::Forced {
        table.enforce_styling();
    }
    let (headers, rows) = elide_human_table_columns(headers, rows, table.width());
    let styled_rows = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .enumerate()
                .map(|(index, cell)| {
                    let header = headers.get(index).map(String::as_str).unwrap_or_default();
                    semantic_cell(header, cell, style)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    table.set_header(headers);
    table.add_rows(styled_rows);
    let mut output = table.to_string();
    output.push('\n');
    output
}

fn semantic_cell(header: &str, value: String, style: OutputStyle) -> Cell {
    let cell = Cell::new(value.clone());
    if let Some(semantic) = semantic_table_cell_style(header, &value).filter(|_| style.colors()) {
        cell.fg(semantic.table_color())
    } else {
        cell
    }
}

// sirno:witness:interfaces:begin
fn semantic_table_cell_style(header: &str, value: &str) -> Option<SemanticStyle> {
    match header {
        | "state" | "status" => semantic_status_style(value),
        | _ => None,
    }
}

fn semantic_status_style(value: &str) -> Option<SemanticStyle> {
    match value {
        | "ok" | "resolved" | "unchanged" => Some(SemanticStyle::Success),
        | "drifted" | "missing" | "open" => Some(SemanticStyle::Warning),
        | "linked" | "wrote" => Some(SemanticStyle::Changed),
        | _ => None,
    }
}
// sirno:witness:interfaces:end

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
        print_ok_path(report.root());
        return;
    }

    anstream::print!("{}", format_entry_directory_report_with_style(report, OutputStyle::Styled));
}

fn format_entry_directory_report_with_style(
    report: &EntryDirectoryReport, style: OutputStyle,
) -> String {
    let mut output = String::new();
    for diagnostic in report.file_diagnostics() {
        output.push_str(&format!(
            "{}: {}: {}\n",
            styled_diagnostic_severity(diagnostic.severity.label(), style),
            diagnostic.path.display(),
            diagnostic.message
        ));
    }

    for diagnostic in report.structural_report().diagnostics() {
        if let Some(path) = diagnostic.entry.as_ref().and_then(|entry| report.entry_path(entry)) {
            output.push_str(&format!(
                "{}: {}: {}\n",
                styled_diagnostic_severity(diagnostic.severity.label(), style),
                path.display(),
                diagnostic.message()
            ));
        } else {
            output.push_str(&format!(
                "{}: {}\n",
                styled_diagnostic_severity(diagnostic.severity.label(), style),
                diagnostic.message()
            ));
        }
    }
    output.push_str(&format!("{}\n", entry_directory_report_summary(report, style)));
    output
}

fn entry_directory_report_summary(report: &EntryDirectoryReport, style: OutputStyle) -> String {
    if report.has_errors() {
        format!(
            "check: {} in {}",
            style_text("failed", SemanticStyle::Error, style),
            report.root().display()
        )
    } else {
        format!(
            "check: {} in {}",
            style_text("warnings", SemanticStyle::Warning, style),
            report.root().display()
        )
    }
}

pub(crate) fn print_ok_path(path: &Path) {
    anstream::println!("{}", format_ok_line(&path.display().to_string(), OutputStyle::Styled));
}

pub(crate) fn print_check_diagnostic(severity: &str, message: &str) {
    let style = OutputStyle::Styled;
    anstream::println!("{}: {message}", styled_diagnostic_severity(severity, style));
}

pub(crate) fn print_check_summary(has_errors: bool, root: &Path) {
    let style = OutputStyle::Styled;
    let label = if has_errors {
        style_text("failed", SemanticStyle::Error, style)
    } else {
        style_text("warnings", SemanticStyle::Warning, style)
    };
    anstream::println!("check: {label} in {}", root.display());
}

fn format_ok_line(location: &str, style: OutputStyle) -> String {
    format!("{}: {location}", style_text("ok", SemanticStyle::Success, style))
}
