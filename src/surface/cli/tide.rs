//! Terminal UI for tide resolution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Cell, Paragraph, Row, Table, Wrap};

use crate::surface::SurfaceContext;
use crate::surface::cli::tui::{
    TuiApp, TuiFlow, TuiKey, TuiSelection, handle_table_key, header_style, key_help, panel_block,
    render_key_footer, render_selectable_table, run_terminal_ui, run_tui_app,
    table_detail_footer_areas,
};
use crate::surface::dto::{TideResolveRequest, TideSelectionRequest, TideStatusMode};
use crate::surface::error::CommandError;
use crate::{EntryAddress, TideSource, TideStatus, TideWorkitem};

/// Run the interactive tide resolution UI.
pub(crate) fn run(config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
    run_terminal_ui(|terminal| {
        let mut app =
            TideResolutionTui::load(config_path.to_path_buf(), lake_path.map(Path::to_path_buf))?;
        run_tui_app(terminal, &mut app)
    })
}

#[derive(Debug)]
struct TideResolutionTui {
    config_path: PathBuf,
    lake_override: Option<PathBuf>,
    statuses: Vec<TideStatus>,
    rows: Vec<TideTuiRow>,
    grouping: TideTuiGrouping,
    detail: TideTuiDetail,
    selection: TuiSelection,
    message: String,
}

impl TideResolutionTui {
    fn load(config_path: PathBuf, lake_override: Option<PathBuf>) -> Result<Self, CommandError> {
        let mut app = Self {
            config_path,
            lake_override,
            statuses: Vec::new(),
            rows: Vec::new(),
            grouping: TideTuiGrouping::default(),
            detail: TideTuiDetail::default(),
            selection: TuiSelection::default(),
            message: String::new(),
        };
        app.refresh("loaded tide status".to_owned())?;
        Ok(app)
    }

    fn context(&self) -> SurfaceContext {
        SurfaceContext::from_cli_paths(&self.config_path, self.lake_override.as_deref())
    }

    fn refresh(&mut self, action: String) -> Result<(), CommandError> {
        let statuses = self.context().tide_statuses(TideStatusMode::All)?;
        self.apply_statuses(statuses, action);
        Ok(())
    }

    fn apply_statuses(&mut self, statuses: Vec<TideStatus>, action: String) {
        let previous = self.selected_identity();
        let message = tide_count_message(&statuses);
        self.statuses = statuses;
        self.rebuild_rows(previous);
        self.message = format!("{action}; {message}");
    }

    fn rebuild_rows(&mut self, previous: Option<TideRowIdentity>) {
        self.rows = tide_rows(&self.statuses, self.grouping, self.detail);
        let selected = previous
            .and_then(|identity| self.rows.iter().position(|row| row.identity == identity))
            .unwrap_or(0)
            .min(self.rows.len().saturating_sub(1));
        self.selection.set(selected);
    }

    fn selected_identity(&self) -> Option<TideRowIdentity> {
        self.selected_row().map(|row| row.identity.clone())
    }

    fn selected_row(&self) -> Option<&TideTuiRow> {
        self.rows.get(self.selection.selected())
    }

    fn toggle_grouping(&mut self) {
        let previous = self.selected_identity();
        self.grouping = self.grouping.toggle();
        self.rebuild_rows(previous);
        self.message = format!("grouped by {}", self.grouping.label());
    }

    fn toggle_detail(&mut self) {
        let previous = self.selected_identity();
        self.detail = self.detail.toggle();
        self.rebuild_rows(previous);
        self.message = format!("showing {}", self.detail.label());
    }

    fn resolve_selected(&mut self) -> Result<(), CommandError> {
        if self.rows.is_empty() {
            self.message = "tide is clear; nothing to resolve".to_owned();
            return Ok(());
        }
        let Some(request) = self.selected_row().and_then(TideTuiRow::resolve_request) else {
            self.message = "selected row has no open tide workitems".to_owned();
            return Ok(());
        };
        let result = self.context().tide_resolve(request)?;
        self.refresh(result.message)
    }

    fn unresolve_selected(&mut self) -> Result<(), CommandError> {
        if self.rows.is_empty() {
            self.message = "tide is clear; nothing to reopen".to_owned();
            return Ok(());
        }
        let Some(request) = self.selected_row().and_then(TideTuiRow::unresolve_request) else {
            self.message = "selected row has no resolved tide workitems".to_owned();
            return Ok(());
        };
        let result = self.context().tide_unresolve(request)?;
        self.refresh(result.message)
    }

    fn infer_resolution(&mut self) -> Result<(), CommandError> {
        if self.statuses.is_empty() {
            self.message = "tide is clear; nothing to infer".to_owned();
            return Ok(());
        }
        let result = self.context().tide_resolve(infer_request())?;
        self.refresh(result.message)
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let areas = table_detail_footer_areas(frame, 4, 4);

        let header = Row::new(table_headers(self.grouping, self.detail)).style(header_style());
        let rows = self.rows.iter().map(|row| {
            Row::new(row.cells.iter().cloned().map(Cell::from).collect::<Vec<_>>())
                .style(row.style())
        });
        let table = Table::new(rows, table_constraints(self.grouping, self.detail))
            .header(header)
            .block(panel_block("Tide"));
        render_selectable_table(frame, areas.table, table, self.selection);

        let detail_text = self.detail_text();
        let detail =
            Paragraph::new(detail_text).block(panel_block("Selection")).wrap(Wrap { trim: true });
        frame.render_widget(detail, areas.detail);

        let footer_text = self.footer_text();
        render_key_footer(frame, areas.footer, &footer_text, true);
    }

    fn detail_text(&self) -> String {
        let mode = format!("view: {} {}", self.grouping.label(), self.detail.label());
        let selected = self.selected_row().map(|row| row.detail.as_str()).unwrap_or("tide: clear");
        format!("{mode}\n{selected}")
    }

    fn footer_text(&self) -> String {
        let keys = key_help(&[
            "Space resolves",
            "u reopens",
            "i infers",
            "c refreshes",
            "Tab groups",
            "f toggles detail",
        ]);
        format!("{}\n{}", self.message, keys)
    }
}

impl TuiApp for TideResolutionTui {
    fn render(&self, frame: &mut Frame<'_>) {
        TideResolutionTui::render(self, frame);
    }

    fn handle_key(&mut self, key: TuiKey) -> Result<TuiFlow, CommandError> {
        if let Some(flow) = handle_table_key(&mut self.selection, self.rows.len(), key) {
            return Ok(flow);
        }
        // sirno:witness:tide-commands:begin
        match key {
            | TuiKey::Tab => {
                self.toggle_grouping();
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char('f') => {
                self.toggle_detail();
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char(' ') => {
                self.resolve_selected()?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char('u') => {
                self.unresolve_selected()?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char('i') => {
                self.infer_resolution()?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char('c') => {
                self.refresh("refreshed tide status".to_owned())?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Quit | TuiKey::Next | TuiKey::Prev | TuiKey::Char(_) | TuiKey::Other => {
                Ok(TuiFlow::Continue)
            }
        }
        // sirno:witness:tide-commands:end
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TideTuiGrouping {
    #[default]
    Entry,
    Wave,
}

impl TideTuiGrouping {
    fn toggle(self) -> Self {
        match self {
            | Self::Entry => Self::Wave,
            | Self::Wave => Self::Entry,
        }
    }

    fn label(self) -> &'static str {
        match self {
            | Self::Entry => "entry",
            | Self::Wave => "wave",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TideTuiDetail {
    #[default]
    Summary,
    Full,
}

impl TideTuiDetail {
    fn toggle(self) -> Self {
        match self {
            | Self::Summary => Self::Full,
            | Self::Full => Self::Summary,
        }
    }

    fn label(self) -> &'static str {
        match self {
            | Self::Summary => "summary",
            | Self::Full => "workitems",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TideTuiRow {
    identity: TideRowIdentity,
    cells: Vec<String>,
    detail: String,
    open: usize,
    resolved: usize,
    target: TideRowTarget,
}

impl TideTuiRow {
    fn resolve_request(&self) -> Option<TideResolveRequest> {
        match &self.target {
            | TideRowTarget::Neighbor(neighbor) if self.open > 0 => Some(TideResolveRequest {
                neighbors: vec![neighbor.clone()],
                ..TideResolveRequest::default()
            }),
            | TideRowTarget::Workitems { resolve, .. } if !resolve.is_empty() => {
                Some(TideResolveRequest {
                    workitems: resolve.clone(),
                    ..TideResolveRequest::default()
                })
            }
            | TideRowTarget::Exact(workitem) if self.open > 0 => Some(TideResolveRequest {
                workitems: vec![workitem.clone()],
                ..TideResolveRequest::default()
            }),
            | TideRowTarget::Neighbor(_)
            | TideRowTarget::Workitems { .. }
            | TideRowTarget::Exact(_) => None,
        }
    }

    fn unresolve_request(&self) -> Option<TideSelectionRequest> {
        match &self.target {
            | TideRowTarget::Neighbor(neighbor) if self.resolved > 0 => {
                Some(TideSelectionRequest {
                    neighbors: vec![neighbor.clone()],
                    ..TideSelectionRequest::default()
                })
            }
            | TideRowTarget::Workitems { unresolve, .. } if !unresolve.is_empty() => {
                Some(TideSelectionRequest {
                    workitems: unresolve.clone(),
                    ..TideSelectionRequest::default()
                })
            }
            | TideRowTarget::Exact(workitem) if self.resolved > 0 => Some(TideSelectionRequest {
                workitems: vec![workitem.clone()],
                ..TideSelectionRequest::default()
            }),
            | TideRowTarget::Neighbor(_)
            | TideRowTarget::Workitems { .. }
            | TideRowTarget::Exact(_) => None,
        }
    }

    fn style(&self) -> Style {
        if self.open > 0 {
            Style::default().fg(Color::Yellow)
        } else if self.resolved > 0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TideRowIdentity {
    Entry(EntryAddress),
    Wave(EntryAddress),
    Workitem(TideWorkitem),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TideRowTarget {
    Neighbor(EntryAddress),
    Workitems { resolve: Vec<TideWorkitem>, unresolve: Vec<TideWorkitem> },
    Exact(TideWorkitem),
}

#[derive(Default)]
struct TideSummary {
    open: usize,
    resolved: usize,
    reasons: BTreeSet<EntryAddress>,
    entries: BTreeSet<EntryAddress>,
    open_workitems: Vec<TideWorkitem>,
    resolved_workitems: Vec<TideWorkitem>,
}

fn tide_rows(
    statuses: &[TideStatus], grouping: TideTuiGrouping, detail: TideTuiDetail,
) -> Vec<TideTuiRow> {
    match (grouping, detail) {
        | (TideTuiGrouping::Entry, TideTuiDetail::Summary) => entry_summary_rows(statuses),
        | (TideTuiGrouping::Wave, TideTuiDetail::Summary) => wave_summary_rows(statuses),
        | (_, TideTuiDetail::Full) => full_workitem_rows(statuses, grouping),
    }
}

fn entry_summary_rows(statuses: &[TideStatus]) -> Vec<TideTuiRow> {
    let mut summaries = BTreeMap::<EntryAddress, TideSummary>::new();
    for status in statuses {
        let summary = summaries.entry(status.workitem.neighbor.clone()).or_default();
        summary.reasons.insert(status.workitem.ripple.clone());
        push_status(summary, status);
    }
    summaries
        .into_iter()
        .map(|(entry, summary)| {
            let reasons = ids_label(&summary.reasons);
            TideTuiRow {
                identity: TideRowIdentity::Entry(entry.clone()),
                cells: vec![
                    entry.to_string(),
                    summary.open.to_string(),
                    summary.resolved.to_string(),
                    reasons.clone(),
                ],
                detail: format!(
                    "entry {entry}; reasons {reasons}; {} open, {} resolved",
                    summary.open, summary.resolved
                ),
                open: summary.open,
                resolved: summary.resolved,
                target: TideRowTarget::Neighbor(entry),
            }
        })
        .collect()
}

fn wave_summary_rows(statuses: &[TideStatus]) -> Vec<TideTuiRow> {
    let mut summaries = BTreeMap::<EntryAddress, TideSummary>::new();
    for status in statuses {
        let summary = summaries.entry(status.workitem.ripple.clone()).or_default();
        summary.entries.insert(status.workitem.neighbor.clone());
        push_status(summary, status);
    }
    summaries
        .into_iter()
        .map(|(wave, summary)| {
            let entries = ids_label(&summary.entries);
            TideTuiRow {
                identity: TideRowIdentity::Wave(wave.clone()),
                cells: vec![
                    wave.to_string(),
                    summary.open.to_string(),
                    summary.resolved.to_string(),
                    entries.clone(),
                ],
                detail: format!(
                    "wave {wave}; entries {entries}; {} open, {} resolved",
                    summary.open, summary.resolved
                ),
                open: summary.open,
                resolved: summary.resolved,
                target: TideRowTarget::Workitems {
                    resolve: summary.open_workitems,
                    unresolve: summary.resolved_workitems,
                },
            }
        })
        .collect()
}

fn full_workitem_rows(statuses: &[TideStatus], grouping: TideTuiGrouping) -> Vec<TideTuiRow> {
    let mut statuses = statuses.iter().collect::<Vec<_>>();
    statuses.sort_by(|left, right| {
        let order = match grouping {
            | TideTuiGrouping::Entry => left.workitem.neighbor.cmp(&right.workitem.neighbor),
            | TideTuiGrouping::Wave => left.workitem.ripple.cmp(&right.workitem.ripple),
        };
        order.then_with(|| left.workitem.cmp(&right.workitem))
    });
    statuses
        .into_iter()
        .map(|status| {
            let state = tide_state_label(status);
            let sources = tide_sources_label(status);
            let workitem = status.workitem.clone();
            TideTuiRow {
                identity: TideRowIdentity::Workitem(workitem.clone()),
                cells: vec![
                    state.to_owned(),
                    workitem.ripple.to_string(),
                    workitem.neighbor.to_string(),
                    workitem.field.clone(),
                    workitem.direction.label().to_owned(),
                    sources.clone(),
                ],
                detail: format!("workitem {workitem}; sources {sources}; state {state}"),
                open: usize::from(!status.resolved),
                resolved: usize::from(status.resolved),
                target: TideRowTarget::Exact(workitem),
            }
        })
        .collect()
}

fn push_status(summary: &mut TideSummary, status: &TideStatus) {
    if status.resolved {
        summary.resolved += 1;
        summary.resolved_workitems.push(status.workitem.clone());
    } else {
        summary.open += 1;
        summary.open_workitems.push(status.workitem.clone());
    }
}

fn table_headers(grouping: TideTuiGrouping, detail: TideTuiDetail) -> Vec<&'static str> {
    match (grouping, detail) {
        | (TideTuiGrouping::Entry, TideTuiDetail::Summary) => {
            vec!["entry", "open", "resolved", "reasons"]
        }
        | (TideTuiGrouping::Wave, TideTuiDetail::Summary) => {
            vec!["wave", "open", "resolved", "entries"]
        }
        | (_, TideTuiDetail::Full) => {
            vec!["state", "wave", "entry", "field", "direction", "sources"]
        }
    }
}

fn table_constraints(grouping: TideTuiGrouping, detail: TideTuiDetail) -> Vec<Constraint> {
    match (grouping, detail) {
        | (TideTuiGrouping::Entry, TideTuiDetail::Summary) => {
            vec![
                Constraint::Length(24),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Min(24),
            ]
        }
        | (TideTuiGrouping::Wave, TideTuiDetail::Summary) => {
            vec![
                Constraint::Length(24),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Min(24),
            ]
        }
        | (_, TideTuiDetail::Full) => {
            vec![
                Constraint::Length(10),
                Constraint::Length(22),
                Constraint::Length(22),
                Constraint::Length(18),
                Constraint::Length(12),
                Constraint::Min(12),
            ]
        }
    }
}

fn tide_count_message(statuses: &[TideStatus]) -> String {
    if statuses.is_empty() {
        return "tide: clear".to_owned();
    }
    let open = statuses.iter().filter(|status| !status.resolved).count();
    let resolved = statuses.len() - open;
    let waves =
        statuses.iter().map(|status| status.workitem.ripple.clone()).collect::<BTreeSet<_>>().len();
    let entries = statuses
        .iter()
        .filter(|status| !status.resolved)
        .map(|status| status.workitem.neighbor.clone())
        .collect::<BTreeSet<_>>()
        .len();
    format!("{open} open, {resolved} resolved, {waves} waves, {entries} review entries")
}

fn tide_state_label(status: &TideStatus) -> &'static str {
    if status.resolved { "resolved" } else { "open" }
}

fn tide_sources_label(status: &TideStatus) -> String {
    status
        .sources
        .iter()
        .map(|source| match source {
            | TideSource::Lake => "lake",
            | TideSource::Anchor => "anchor",
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn ids_label(ids: &BTreeSet<EntryAddress>) -> String {
    if ids.is_empty() {
        "-".to_owned()
    } else {
        ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
    }
}

fn infer_request() -> TideResolveRequest {
    TideResolveRequest { infer: true, ..TideResolveRequest::default() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StructuralEdgeDirection;

    #[test]
    fn entry_summary_rows_count_reasons() {
        let statuses = tide_statuses();

        let rows = tide_rows(&statuses, TideTuiGrouping::Entry, TideTuiDetail::Summary);

        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].cells,
            ["agent-skills", "1", "1", "interfaces,tide"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
        assert_eq!(rows[0].open, 1);
        assert_eq!(rows[0].resolved, 1);
        assert_eq!(
            rows[1].cells,
            ["wave", "1", "0", "tide"].into_iter().map(str::to_owned).collect::<Vec<_>>()
        );
    }

    #[test]
    fn wave_summary_rows_count_entries() {
        let statuses = tide_statuses();

        let rows = tide_rows(&statuses, TideTuiGrouping::Wave, TideTuiDetail::Summary);

        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].cells,
            ["interfaces", "1", "0", "agent-skills"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            rows[1].cells,
            ["tide", "1", "1", "agent-skills,wave"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn full_rows_sort_by_selected_grouping() {
        let statuses = tide_statuses();

        let by_entry = tide_rows(&statuses, TideTuiGrouping::Entry, TideTuiDetail::Full);
        let by_wave = tide_rows(&statuses, TideTuiGrouping::Wave, TideTuiDetail::Full);

        assert_eq!(by_entry[0].cells[2], "agent-skills");
        assert_eq!(by_entry[1].cells[2], "agent-skills");
        assert_eq!(by_entry[2].cells[2], "wave");
        assert_eq!(by_wave[0].cells[1], "interfaces");
        assert_eq!(by_wave[1].cells[1], "tide");
        assert_eq!(by_wave[2].cells[1], "tide");
        assert_eq!(by_wave[2].cells[0], "resolved");
        assert_eq!(by_wave[2].cells[5], "lake,anchor");
    }

    #[test]
    fn entry_summary_actions_select_neighbor() {
        let rows = tide_rows(&tide_statuses(), TideTuiGrouping::Entry, TideTuiDetail::Summary);

        let resolve = rows[0].resolve_request().unwrap();
        let unresolve = rows[0].unresolve_request().unwrap();

        assert_eq!(resolve.neighbors, [entry_id("agent-skills")]);
        assert_eq!(resolve.workitems, []);
        assert_eq!(unresolve.neighbors, [entry_id("agent-skills")]);
        assert_eq!(unresolve.workitems, []);
    }

    #[test]
    fn wave_summary_actions_select_exact_workitems() {
        let rows = tide_rows(&tide_statuses(), TideTuiGrouping::Wave, TideTuiDetail::Summary);

        let resolve = rows[1].resolve_request().unwrap();
        let unresolve = rows[1].unresolve_request().unwrap();

        assert_eq!(resolve.neighbors, []);
        assert_eq!(resolve.workitems, [workitem("tide", "belongs", "wave")]);
        assert_eq!(unresolve.neighbors, []);
        assert_eq!(unresolve.workitems, [workitem("tide", "refines", "agent-skills")]);
    }

    #[test]
    fn full_row_actions_select_exact_workitem() {
        let rows = tide_rows(&tide_statuses(), TideTuiGrouping::Entry, TideTuiDetail::Full);

        let resolve = rows[0].resolve_request().unwrap();
        let unresolve = rows[1].unresolve_request().unwrap();

        assert_eq!(resolve.neighbors, []);
        assert_eq!(resolve.workitems, [workitem("interfaces", "belongs", "agent-skills")]);
        assert!(rows[0].unresolve_request().is_none());
        assert_eq!(unresolve.neighbors, []);
        assert_eq!(unresolve.workitems, [workitem("tide", "refines", "agent-skills")]);
        assert!(rows[1].resolve_request().is_none());
    }

    #[test]
    fn infer_action_uses_infer_request() {
        let request = infer_request();

        assert!(request.infer);
        assert!(request.neighbors.is_empty());
        assert!(request.workitems.is_empty());
    }

    #[test]
    fn toggles_change_modes_and_reload_preserves_selected_identity() {
        let mut app = TideResolutionTui::from_statuses(tide_statuses());
        app.selection.set(1);
        let selected = app.selected_identity();

        app.apply_statuses(tide_statuses_reordered(), "refreshed".to_owned());

        assert_eq!(app.selected_identity(), selected);
        app.toggle_grouping();
        assert_eq!(app.grouping, TideTuiGrouping::Wave);
        assert_eq!(app.selection.selected(), 0);
        app.toggle_detail();
        assert_eq!(app.detail, TideTuiDetail::Full);
        assert_eq!(app.selection.selected(), 0);
    }

    #[test]
    fn empty_tide_actions_are_noops_with_messages() {
        let mut app = TideResolutionTui::from_statuses(Vec::new());

        app.resolve_selected().unwrap();
        assert_eq!(app.message, "tide is clear; nothing to resolve");

        app.unresolve_selected().unwrap();
        assert_eq!(app.message, "tide is clear; nothing to reopen");
    }

    impl TideResolutionTui {
        fn from_statuses(statuses: Vec<TideStatus>) -> Self {
            let mut app = Self {
                config_path: PathBuf::from("Sirno.toml"),
                lake_override: None,
                statuses: Vec::new(),
                rows: Vec::new(),
                grouping: TideTuiGrouping::default(),
                detail: TideTuiDetail::default(),
                selection: TuiSelection::default(),
                message: String::new(),
            };
            app.apply_statuses(statuses, "loaded".to_owned());
            app
        }
    }

    fn tide_statuses() -> Vec<TideStatus> {
        vec![
            status("interfaces", "belongs", "agent-skills", &[TideSource::Lake], false),
            status(
                "tide",
                "refines",
                "agent-skills",
                &[TideSource::Lake, TideSource::Anchor],
                true,
            ),
            status("tide", "belongs", "wave", &[TideSource::Lake], false),
        ]
    }

    fn tide_statuses_reordered() -> Vec<TideStatus> {
        vec![
            status("tide", "belongs", "wave", &[TideSource::Lake], false),
            status("interfaces", "belongs", "agent-skills", &[TideSource::Lake], false),
            status(
                "tide",
                "refines",
                "agent-skills",
                &[TideSource::Lake, TideSource::Anchor],
                true,
            ),
        ]
    }

    fn status(
        ripple: &str, field: &str, neighbor: &str, sources: &[TideSource], resolved: bool,
    ) -> TideStatus {
        TideStatus {
            workitem: workitem(ripple, field, neighbor),
            sources: sources.iter().copied().collect(),
            fingerprint: format!("{ripple}-{neighbor}"),
            resolved,
        }
    }

    fn workitem(ripple: &str, field: &str, neighbor: &str) -> TideWorkitem {
        TideWorkitem::new(
            entry_id(ripple),
            field,
            if field == "refines" {
                StructuralEdgeDirection::From
            } else {
                StructuralEdgeDirection::Clique
            },
            entry_id(neighbor),
        )
        .unwrap()
    }

    fn entry_id(raw: &str) -> EntryAddress {
        EntryAddress::new(raw).unwrap()
    }
}
