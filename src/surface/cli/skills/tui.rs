//! Terminal UI for packaged skill wrapper maintenance.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap};
use ratatui::{DefaultTerminal, Frame};

use crate::surface::SurfaceContext;
use crate::surface::cli::tui::run_terminal_ui;
use crate::surface::dto::{SkillWrapperRecord, SkillWrapperResult};
use crate::surface::error::CommandError;

/// Run the interactive skill-wrapper maintenance UI.
pub(crate) fn run(config_path: &Path, claude_skills: bool) -> Result<ExitCode, CommandError> {
    run_terminal_ui(|terminal| run_app(terminal, config_path, claude_skills))
}

fn run_app(
    terminal: &mut DefaultTerminal, config_path: &Path, claude_skills: bool,
) -> Result<ExitCode, CommandError> {
    let mut app = SkillManagerTui::load(config_path.to_path_buf(), claude_skills)?;
    loop {
        terminal.draw(|frame| app.render(frame)).map_err(CommandError::TerminalUi)?;
        if let Event::Key(key) = event::read().map_err(CommandError::TerminalUi)? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            // sirno:witness:utility-commands:begin
            match key.code {
                | KeyCode::Char('q') | KeyCode::Esc => return Ok(ExitCode::SUCCESS),
                | KeyCode::Char('j') | KeyCode::Down => app.next(),
                | KeyCode::Char('k') | KeyCode::Up => app.previous(),
                | KeyCode::Char('c') => app.check()?,
                | KeyCode::Char('i') => app.install()?,
                | KeyCode::Char('l') => app.toggle_claude_links()?,
                | _ => {}
            }
            // sirno:witness:utility-commands:end
        }
    }
}

#[derive(Debug)]
struct SkillManagerTui {
    config_path: PathBuf,
    claude_skills: bool,
    rows: Vec<SkillWrapperRecord>,
    selected: usize,
    message: String,
}

impl SkillManagerTui {
    fn load(config_path: PathBuf, claude_skills: bool) -> Result<Self, CommandError> {
        let mut app = Self {
            config_path,
            claude_skills,
            rows: Vec::new(),
            selected: 0,
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
        let selected_target = self.rows.get(self.selected).map(|row| row.target_path.clone());
        self.rows = result.records;
        self.selected = selected_target
            .and_then(|target| self.rows.iter().position(|row| row.target_path == target))
            .unwrap_or(0)
            .min(self.rows.len().saturating_sub(1));
        self.message = result.message;
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1).min(self.rows.len().saturating_sub(1));
    }

    fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(4)])
            .split(frame.area());

        let header = Row::new(["Status", "Name", "Kind", "Target"])
            .style(Style::default().add_modifier(Modifier::BOLD));
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
        .block(Block::default().title(self.title()).borders(Borders::ALL))
        .row_highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
        let mut state = TableState::default().with_selected(Some(self.selected));
        frame.render_stateful_widget(table, chunks[0], &mut state);

        let footer = Paragraph::new(self.footer_text())
            .block(Block::default().title("Keys").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(footer, chunks[1]);
    }

    fn title(&self) -> &'static str {
        if self.claude_skills { "Skill Wrappers and Claude Links" } else { "Skill Wrappers" }
    }

    fn footer_text(&self) -> String {
        let selected = self
            .rows
            .get(self.selected)
            .map(|record| format!("selected: {} -> {}", record.name, record.target_path))
            .unwrap_or_else(|| "selected: none".to_owned());
        format!(
            "{}\nj/k or arrows move; c checks; i installs or repairs; l toggles Claude links; q quits\n{}",
            self.message, selected
        )
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
            selected: 0,
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

        assert_eq!(app.selected, 1);
        assert_eq!(app.rows[app.selected].target_path, target);
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
