//! Terminal UI for packaged skill wrapper maintenance.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};

use crate::surface::SurfaceContext;
use crate::surface::cli::tui::{
    TuiApp, TuiFlow, TuiKey, TuiSelection, handle_table_key, header_style, key_help, panel_block,
    render_key_footer, render_selectable_table, run_terminal_ui, run_tui_app, table_footer_areas,
};
use crate::surface::dto::{SkillWrapperRecord, SkillWrapperResult};
use crate::surface::error::CommandError;

/// Run the interactive skill-wrapper maintenance UI.
pub(crate) fn run(config_path: &Path, claude_skills: bool) -> Result<ExitCode, CommandError> {
    run_terminal_ui(|terminal| {
        let mut app = SkillManagerTui::load(config_path.to_path_buf(), claude_skills)?;
        run_tui_app(terminal, &mut app)
    })
}

#[derive(Debug)]
struct SkillManagerTui {
    config_path: PathBuf,
    claude_skills: bool,
    rows: Vec<SkillWrapperRecord>,
    selection: TuiSelection,
    message: String,
}

impl SkillManagerTui {
    fn load(config_path: PathBuf, claude_skills: bool) -> Result<Self, CommandError> {
        let mut app = Self {
            config_path,
            claude_skills,
            rows: Vec::new(),
            selection: TuiSelection::default(),
            message: String::new(),
        };
        app.check()?;
        Ok(app)
    }

    fn context(&self) -> SurfaceContext {
        SurfaceContext::new(&self.config_path)
    }

    fn check(&mut self) -> Result<(), CommandError> {
        let result = self.context().skill_wrappers_check_with_claude(self.claude_skills)?;
        self.apply_result(result);
        Ok(())
    }

    fn install(&mut self) -> Result<(), CommandError> {
        let result = self.context().skill_wrappers_init_with_claude(self.claude_skills)?;
        self.apply_result(result);
        Ok(())
    }

    fn toggle_claude_links(&mut self) -> Result<(), CommandError> {
        self.claude_skills = !self.claude_skills;
        self.check()?;
        let state = if self.claude_skills { "shown" } else { "hidden" };
        self.message = format!("Claude skill links {state}; {}", self.message);
        Ok(())
    }

    fn apply_result(&mut self, result: SkillWrapperResult) {
        let selected_target =
            self.rows.get(self.selection.selected()).map(|row| row.target_path.clone());
        self.rows = result.records;
        let selected = selected_target
            .and_then(|target| self.rows.iter().position(|row| row.target_path == target))
            .unwrap_or(0)
            .min(self.rows.len().saturating_sub(1));
        self.selection.set(selected);
        self.message = result.message;
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let areas = table_footer_areas(frame, 4);

        let header = Row::new(["Status", "Name", "Kind", "Target"]).style(header_style());
        let rows = self.rows.iter().map(|record| {
            Row::new([
                Cell::from(record.status.as_str()),
                Cell::from(record.name.as_str()),
                Cell::from(record_kind(record)),
                Cell::from(record.target_path.as_str()),
            ])
            .style(status_style(record))
        });
        let table = Table::new(
            rows,
            [
                Constraint::Length(12),
                Constraint::Length(28),
                Constraint::Length(12),
                Constraint::Min(28),
            ],
        )
        .header(header)
        .block(panel_block(self.title()));
        render_selectable_table(frame, areas.table, table, self.selection);

        let footer_text = self.footer_text();
        render_key_footer(frame, areas.footer, &footer_text, true);
    }

    fn title(&self) -> &'static str {
        if self.claude_skills { "Skill Wrappers and Claude Links" } else { "Skill Wrappers" }
    }

    fn footer_text(&self) -> String {
        let selected = self
            .rows
            .get(self.selection.selected())
            .map(|record| format!("selected: {} -> {}", record.name, record.target_path))
            .unwrap_or_else(|| "selected: none".to_owned());
        let keys = key_help(&["c checks", "i installs or repairs", "l toggles Claude links"]);
        format!("{}\n{}\n{}", self.message, keys, selected)
    }
}

impl TuiApp for SkillManagerTui {
    fn render(&self, frame: &mut Frame<'_>) {
        SkillManagerTui::render(self, frame);
    }

    fn handle_key(&mut self, key: TuiKey) -> Result<TuiFlow, CommandError> {
        if let Some(flow) = handle_table_key(&mut self.selection, self.rows.len(), key) {
            return Ok(flow);
        }
        match key {
            // sirno:witness:utility-commands:begin
            | TuiKey::Char('c') => {
                self.check()?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char('i') => {
                self.install()?;
                Ok(TuiFlow::Continue)
            }
            | TuiKey::Char('l') => {
                self.toggle_claude_links()?;
                Ok(TuiFlow::Continue)
            }
            // sirno:witness:utility-commands:end
            | TuiKey::Quit
            | TuiKey::Next
            | TuiKey::Prev
            | TuiKey::Tab
            | TuiKey::Char(_)
            | TuiKey::Other => Ok(TuiFlow::Continue),
        }
    }
}

fn record_kind(record: &SkillWrapperRecord) -> &'static str {
    if record.target_path.starts_with(".claude/skills/") { "claude" } else { "wrapper" }
}

fn status_style(record: &SkillWrapperRecord) -> Style {
    match record.status.as_str() {
        | "ok" | "source" | "link" | "unchanged" => Style::default().fg(Color::Green),
        | "wrote" | "linked" => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        | "missing" => Style::default().fg(Color::Red),
        | "drifted" => Style::default().fg(Color::Yellow),
        | _ => Style::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_reload_preserves_selected_target() {
        let mut app = SkillManagerTui {
            config_path: PathBuf::from("Sirno.toml"),
            claude_skills: false,
            rows: vec![record("sirno-editor", ".agents/skills/sirno-editor/SKILL.md", "missing")],
            selection: TuiSelection::default(),
            message: String::new(),
        };
        let target = ".agents/skills/sirno-editor/SKILL.md";

        app.apply_result(SkillWrapperResult {
            ok: true,
            records: vec![
                record(
                    "sirno-narrative-session",
                    ".agents/skills/sirno-narrative-session/SKILL.md",
                    "ok",
                ),
                record("sirno-editor", target, "ok"),
            ],
            message: "all wrappers match".to_owned(),
        });

        assert_eq!(app.selection.selected(), 1);
        assert_eq!(app.rows[app.selection.selected()].target_path, target);
        assert_eq!(app.message, "all wrappers match");
    }

    #[test]
    fn record_kind_distinguishes_claude_links() {
        assert_eq!(
            record_kind(&record("sirno-editor", ".agents/skills/sirno-editor/SKILL.md", "ok")),
            "wrapper"
        );
        assert_eq!(
            record_kind(&record("sirno-editor", ".claude/skills/sirno-editor", "ok")),
            "claude"
        );
    }

    fn record(name: &str, target_path: &str, status: &str) -> SkillWrapperRecord {
        SkillWrapperRecord {
            entry_id: "lake-first-maintenance-discipline".to_owned(),
            name: name.to_owned(),
            wrapper_path: "sirno-docs/.artifacts/lake-first-maintenance-discipline/SKILL.md"
                .to_owned(),
            full_path: "sirno-docs/.artifacts/lake-first-maintenance-discipline/SKILL.full.md"
                .to_owned(),
            target_path: target_path.to_owned(),
            status: status.to_owned(),
            changed: false,
        }
    }
}
