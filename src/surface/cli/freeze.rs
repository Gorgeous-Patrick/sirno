//! Terminal UI for entry freeze and melt operations.

use std::collections::BTreeMap;
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
use crate::surface::context::resolve_lake_directory;
use crate::surface::error::CommandError;
use crate::{CheckMode, Entry, EntryAddress, EntryArtifact, EntryDirectory, EntryDirectoryReport};

/// Default action for the shared entry freeze/melt UI.
// sirno:witness:entry-freeze:begin
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntryFreezeTuiAction {
    Freeze,
    Melt,
}
// sirno:witness:entry-freeze:end

impl EntryFreezeTuiAction {
    fn toggle(self) -> Self {
        match self {
            | Self::Freeze => Self::Melt,
            | Self::Melt => Self::Freeze,
        }
    }

    fn verb(self) -> &'static str {
        match self {
            | Self::Freeze => "freeze",
            | Self::Melt => "melt",
        }
    }

    fn key_label(self) -> &'static str {
        match self {
            | Self::Freeze => "Space freezes selected",
            | Self::Melt => "Space melts selected",
        }
    }
}

/// Run the interactive entry freeze/melt UI.
// sirno:witness:entry-freeze:begin
pub(crate) fn run(
    config_path: &Path, lake_path: Option<&Path>, default_action: EntryFreezeTuiAction,
) -> Result<ExitCode, CommandError> {
    run_terminal_ui(|terminal| {
        let mut app = EntryFreezeTui::load(
            config_path.to_path_buf(),
            lake_path.map(Path::to_path_buf),
            default_action,
        )?;
        run_tui_app(terminal, &mut app)
    })
}
// sirno:witness:entry-freeze:end

#[derive(Debug)]
struct EntryFreezeTui {
    config_path: PathBuf,
    lake_override: Option<PathBuf>,
    lake_path: PathBuf,
    rows: Vec<EntryFreezeRow>,
    default_action: EntryFreezeTuiAction,
    selection: TuiSelection,
    message: String,
}

impl EntryFreezeTui {
    fn load(
        config_path: PathBuf, lake_override: Option<PathBuf>, default_action: EntryFreezeTuiAction,
    ) -> Result<Self, CommandError> {
        let mut app = Self {
            config_path,
            lake_override,
            lake_path: PathBuf::new(),
            rows: Vec::new(),
            default_action,
            selection: TuiSelection::default(),
            message: String::new(),
        };
        app.refresh(format!("loaded entries; default action: {}", default_action.verb()))?;
        Ok(app)
    }

    fn context(&self) -> SurfaceContext {
        SurfaceContext::from_cli_paths(&self.config_path, self.lake_override.as_deref())
    }

    fn refresh(&mut self, action: String) -> Result<(), CommandError> {
        let selected_id = self.selected_row().map(|row| row.id.clone());
        let (lake_path, mut settings) =
            resolve_lake_directory(self.lake_override.as_deref(), &self.config_path)?;
        settings.render = false;
        settings.witness = None;
        let report =
            EntryDirectory::new(&lake_path).check_with_settings(CheckMode::Edit, &settings)?;
        self.rows = entry_freeze_rows(&report);
        self.selection.set(
            selected_id.and_then(|id| self.rows.iter().position(|row| row.id == id)).unwrap_or(0),
        );
        self.lake_path = lake_path;
        self.message = format!("{action}; {}", entry_check_summary(&report));
        Ok(())
    }

    fn selected_row(&self) -> Option<&EntryFreezeRow> {
        self.rows.get(self.selection.selected())
    }

    fn toggle_default_action(&mut self) {
        self.default_action = self.default_action.toggle();
        self.message = format!("default action: {}", self.default_action.verb());
    }

    fn apply_default_action(&mut self) -> Result<(), CommandError> {
        self.apply_selected(self.default_action)
    }

    fn apply_selected(&mut self, action: EntryFreezeTuiAction) -> Result<(), CommandError> {
        let Some(row) = self.selected_row() else {
            self.message = "no entries in lake".to_owned();
            return Ok(());
        };
        let id = row.id.clone();
        let result = match action {
            | EntryFreezeTuiAction::Freeze => self.context().entry_freeze(id)?,
            | EntryFreezeTuiAction::Melt => self.context().entry_melt(id)?,
        };
        self.refresh(result.message)
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let areas = table_detail_footer_areas(frame, 4, 4);

        let header =
            Row::new(["Entry", "State", "Artifacts", "Name", "Description"]).style(header_style());
        let rows = self.rows.iter().map(|row| {
            Row::new([
                Cell::from(row.id.to_string()),
                Cell::from(row.state.label()),
                Cell::from(row.artifacts.to_string()),
                Cell::from(row.name.as_str()),
                Cell::from(row.desc.as_str()),
            ])
            .style(row.style())
        });
        let table = Table::new(
            rows,
            [
                Constraint::Length(24),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(24),
                Constraint::Min(24),
            ],
        )
        .header(header)
        .block(panel_block("Entry Freeze / Melt"));
        render_selectable_table(frame, areas.table, table, self.selection);

        let detail = Paragraph::new(self.detail_text())
            .block(panel_block("Selection"))
            .wrap(Wrap { trim: true });
        frame.render_widget(detail, areas.detail);

        render_key_footer(frame, areas.footer, &self.footer_text(), true);
    }

    fn detail_text(&self) -> String {
        let default = format!("default action: {}", self.default_action.verb());
        let Some(row) = self.selected_row() else {
            return format!("{default}\nno entries in {}", self.lake_path.display());
        };
        format!(
            "{default}\nentry {}; {}; {} {}; path {}",
            row.id,
            row.state.label(),
            row.artifacts,
            if row.artifacts == 1 { "artifact" } else { "artifacts" },
            row.path,
        )
    }

    fn footer_text(&self) -> String {
        let keys = key_help(&[
            self.default_action.key_label(),
            "f freezes",
            "m melts",
            "c refreshes",
            "Tab changes default",
        ]);
        format!("{}\n{}", self.message, keys)
    }
}

impl TuiApp for EntryFreezeTui {
    fn render(&self, frame: &mut Frame<'_>) {
        EntryFreezeTui::render(self, frame);
    }

    fn handle_key(&mut self, key: TuiKey) -> Result<TuiFlow, CommandError> {
        if let Some(flow) = handle_table_key(&mut self.selection, self.rows.len(), key) {
            return Ok(flow);
        }
        // sirno:witness:entry-freeze:begin
        match key {
            | TuiKey::Tab => {
                self.toggle_default_action();
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char(' ') => {
                self.apply_default_action()?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char('f') => {
                self.apply_selected(EntryFreezeTuiAction::Freeze)?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char('m') => {
                self.apply_selected(EntryFreezeTuiAction::Melt)?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char('c') => {
                self.refresh("refreshed entries".to_owned())?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Quit | TuiKey::Next | TuiKey::Prev | TuiKey::Char(_) | TuiKey::Other => {
                Ok(TuiFlow::Continue)
            }
        }
        // sirno:witness:entry-freeze:end
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EntryFreezeRow {
    id: EntryAddress,
    state: EntryFreezeState,
    artifacts: usize,
    name: String,
    desc: String,
    path: String,
}

impl EntryFreezeRow {
    fn style(&self) -> Style {
        match self.state {
            | EntryFreezeState::Frozen => Style::default().fg(Color::Cyan),
            | EntryFreezeState::Mutable => Style::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryFreezeState {
    Frozen,
    Mutable,
}

impl EntryFreezeState {
    fn from_entry(entry: &Entry) -> Self {
        if entry.metadata.meta.frozen.is_some() { Self::Frozen } else { Self::Mutable }
    }

    fn label(self) -> &'static str {
        match self {
            | Self::Frozen => "frozen",
            | Self::Mutable => "mutable",
        }
    }
}

fn entry_freeze_rows(report: &EntryDirectoryReport) -> Vec<EntryFreezeRow> {
    let artifacts_by_owner = artifact_counts(report.artifacts());
    report
        .entries()
        .iter()
        .map(|entry| {
            let artifacts = artifacts_by_owner.get(&entry.id).copied().unwrap_or(0);
            let path = report
                .entry_file_path(&entry.id)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "-".to_owned());
            EntryFreezeRow {
                id: entry.id.clone(),
                state: EntryFreezeState::from_entry(entry),
                artifacts,
                name: entry.metadata.name.clone(),
                desc: entry.metadata.desc.clone(),
                path,
            }
        })
        .collect()
}

fn artifact_counts(artifacts: &[EntryArtifact]) -> BTreeMap<EntryAddress, usize> {
    let mut counts = BTreeMap::new();
    for artifact in artifacts {
        *counts.entry(artifact.owner.clone()).or_default() += 1;
    }
    counts
}

fn entry_check_summary(report: &EntryDirectoryReport) -> String {
    let diagnostics =
        report.file_diagnostics().len() + report.structural_report().diagnostics().len();
    if report.has_errors() {
        format!("entry check: errors ({diagnostics} diagnostics)")
    } else if report.is_clean() {
        "entry check: ok".to_owned()
    } else {
        format!("entry check: warnings ({diagnostics} diagnostics)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntryArtifactPath, EntryMetadata, FrozenMarker};

    #[test]
    fn action_toggle_switches_between_freeze_and_melt() {
        assert_eq!(EntryFreezeTuiAction::Freeze.toggle(), EntryFreezeTuiAction::Melt);
        assert_eq!(EntryFreezeTuiAction::Melt.toggle(), EntryFreezeTuiAction::Freeze);
    }

    #[test]
    fn artifact_counts_group_by_owner() {
        let alpha = EntryAddress::new("alpha").unwrap();
        let beta = EntryAddress::new("beta").unwrap();
        let artifacts = vec![
            EntryArtifact::new(
                alpha.clone(),
                EntryArtifactPath::new("first.bin").unwrap(),
                b"first",
            ),
            EntryArtifact::new(
                alpha.clone(),
                EntryArtifactPath::new("second.bin").unwrap(),
                b"second",
            ),
            EntryArtifact::new(beta.clone(), EntryArtifactPath::new("note.bin").unwrap(), b"note"),
        ];

        let counts = artifact_counts(&artifacts);

        assert_eq!(counts.get(&alpha), Some(&2));
        assert_eq!(counts.get(&beta), Some(&1));
    }

    #[test]
    fn state_reports_frozen_marker() {
        let mut metadata = EntryMetadata::new("Alpha", "Alpha entry.").unwrap();
        let mutable = Entry::new(EntryAddress::new("alpha").unwrap(), metadata.clone(), "");
        metadata.meta.frozen = Some(FrozenMarker::reviewed());
        let frozen = Entry::new(EntryAddress::new("alpha").unwrap(), metadata, "");

        assert_eq!(EntryFreezeState::from_entry(&mutable), EntryFreezeState::Mutable);
        assert_eq!(EntryFreezeState::from_entry(&frozen), EntryFreezeState::Frozen);
    }
}
