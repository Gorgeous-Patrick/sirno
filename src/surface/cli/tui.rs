//! Shared terminal UI setup for CLI utilities.

use std::process::ExitCode;

use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Table, TableState, Wrap};

use crate::surface::error::CommandError;

/// A keyboard input normalized across utility TUIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TuiKey {
    Quit,
    Next,
    Prev,
    Tab,
    Char(char),
    Other,
}

impl TuiKey {
    // sirno:witness:utility-commands:begin
    fn from_code(code: KeyCode) -> Self {
        match code {
            | KeyCode::Char('q') | KeyCode::Esc => Self::Quit,
            | KeyCode::Char('j') | KeyCode::Down => Self::Next,
            | KeyCode::Char('k') | KeyCode::Up => Self::Prev,
            | KeyCode::Tab => Self::Tab,
            | KeyCode::Char(ch) => Self::Char(ch),
            | _ => Self::Other,
        }
    }
    // sirno:witness:utility-commands:end
}

/// The result of handling one terminal UI key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TuiFlow {
    Continue,
    Exit(ExitCode),
}

/// A utility TUI that can render itself and handle normalized keys.
pub(crate) trait TuiApp {
    fn render(&self, frame: &mut Frame<'_>);

    fn handle_key(&mut self, key: TuiKey) -> Result<TuiFlow, CommandError>;
}

/// Selected table row state shared by utility TUIs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TuiSelection {
    selected: usize,
}

impl TuiSelection {
    pub(crate) fn selected(self) -> usize {
        self.selected
    }

    pub(crate) fn set(&mut self, selected: usize) {
        self.selected = selected;
    }

    pub(crate) fn next(&mut self, row_count: usize) {
        self.selected = (self.selected + 1).min(row_count.saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn table_state(self) -> TableState {
        TableState::default().with_selected(Some(self.selected))
    }
}

/// Handle shared utility table keys.
// sirno:witness:utility-commands:begin
pub(crate) fn handle_table_key(
    selection: &mut TuiSelection, row_count: usize, key: TuiKey,
) -> Option<TuiFlow> {
    match key {
        | TuiKey::Quit => Some(TuiFlow::Exit(ExitCode::SUCCESS)),
        | TuiKey::Next => {
            selection.next(row_count);
            Some(TuiFlow::Continue)
        }
        | TuiKey::Prev => {
            selection.previous();
            Some(TuiFlow::Continue)
        }
        | TuiKey::Tab | TuiKey::Char(_) | TuiKey::Other => None,
    }
}
// sirno:witness:utility-commands:end

/// Main table and footer areas for utility TUIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TuiAreas {
    pub(crate) table: Rect,
    pub(crate) footer: Rect,
}

/// Main table, selected-row detail, and footer areas for utility TUIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TuiDetailAreas {
    pub(crate) table: Rect,
    pub(crate) detail: Rect,
    pub(crate) footer: Rect,
}

/// Run a terminal UI and restore the terminal before returning.
pub(crate) fn run_terminal_ui(
    run: impl FnOnce(&mut DefaultTerminal) -> Result<ExitCode, CommandError>,
) -> Result<ExitCode, CommandError> {
    let mut terminal = ratatui::try_init().map_err(CommandError::TerminalUi)?;
    let result = run(&mut terminal);
    let restore = ratatui::try_restore().map_err(CommandError::TerminalUi);
    match (result, restore) {
        | (Ok(code), Ok(())) => Ok(code),
        | (Err(error), _) => Err(error),
        | (Ok(_), Err(error)) => Err(error),
    }
}

/// Run a utility TUI event loop with common drawing and keyboard handling.
pub(crate) fn run_tui_app(
    terminal: &mut DefaultTerminal, app: &mut impl TuiApp,
) -> Result<ExitCode, CommandError> {
    loop {
        terminal.draw(|frame| app.render(frame)).map_err(CommandError::TerminalUi)?;
        let Event::Key(key) = event::read().map_err(CommandError::TerminalUi)? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match app.handle_key(TuiKey::from_code(key.code))? {
            | TuiFlow::Continue => {}
            | TuiFlow::Exit(code) => return Ok(code),
        }
    }
}

/// Split a utility TUI into a main table and bottom key/message footer.
// sirno:witness:utility-commands:begin
pub(crate) fn table_footer_areas(frame: &Frame<'_>, footer_height: u16) -> TuiAreas {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(footer_height)])
        .split(frame.area());
    TuiAreas { table: chunks[0], footer: chunks[1] }
}
// sirno:witness:utility-commands:end

/// Split a utility TUI into a main table, detail panel, and bottom key/message footer.
pub(crate) fn table_detail_footer_areas(
    frame: &Frame<'_>, detail_height: u16, footer_height: u16,
) -> TuiDetailAreas {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(detail_height),
            Constraint::Length(footer_height),
        ])
        .split(frame.area());
    TuiDetailAreas { table: chunks[0], detail: chunks[1], footer: chunks[2] }
}

/// Standard header style for utility TUI tables.
pub(crate) fn header_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// Build a concise help line from shared movement keys and app-specific action keys.
pub(crate) fn key_help(actions: &[&str]) -> String {
    let mut parts = Vec::with_capacity(actions.len() + 2);
    parts.push("j/k or arrows move");
    parts.extend(actions.iter().copied());
    parts.push("q quits");
    parts.join("; ")
}

/// Standard bordered block for a utility TUI panel.
pub(crate) fn panel_block(title: &'static str) -> Block<'static> {
    Block::default().title(title).borders(Borders::ALL)
}

/// Render a selectable table with the shared utility highlight.
// sirno:witness:utility-commands:begin
pub(crate) fn render_selectable_table<'a>(
    frame: &mut Frame<'_>, area: Rect, table: Table<'a>, selection: TuiSelection,
) {
    let table = table
        .row_highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    let mut state = selection.table_state();
    frame.render_stateful_widget(table, area, &mut state);
}
// sirno:witness:utility-commands:end

/// Render the bottom key/message footer.
// sirno:witness:utility-commands:begin
pub(crate) fn render_key_footer(frame: &mut Frame<'_>, area: Rect, text: &str, wrap: bool) {
    let footer = Paragraph::new(text).block(panel_block("Keys"));
    let footer = if wrap { footer.wrap(Wrap { trim: true }) } else { footer };
    frame.render_widget(footer, area);
}
// sirno:witness:utility-commands:end

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_mapping_normalizes_shared_bindings() {
        assert_eq!(TuiKey::from_code(KeyCode::Char('q')), TuiKey::Quit);
        assert_eq!(TuiKey::from_code(KeyCode::Esc), TuiKey::Quit);
        assert_eq!(TuiKey::from_code(KeyCode::Char('j')), TuiKey::Next);
        assert_eq!(TuiKey::from_code(KeyCode::Down), TuiKey::Next);
        assert_eq!(TuiKey::from_code(KeyCode::Char('k')), TuiKey::Prev);
        assert_eq!(TuiKey::from_code(KeyCode::Up), TuiKey::Prev);
        assert_eq!(TuiKey::from_code(KeyCode::Tab), TuiKey::Tab);
        assert_eq!(TuiKey::from_code(KeyCode::Char('i')), TuiKey::Char('i'));
    }

    #[test]
    fn selection_stays_inside_rows() {
        let mut selection = TuiSelection::default();

        selection.next(3);
        selection.next(3);
        selection.next(3);
        selection.next(3);
        assert_eq!(selection.selected(), 2);

        selection.previous();
        selection.previous();
        selection.previous();
        assert_eq!(selection.selected(), 0);
    }

    #[test]
    fn table_key_handler_updates_selection_and_exits() {
        let mut selection = TuiSelection::default();

        assert_eq!(handle_table_key(&mut selection, 3, TuiKey::Next), Some(TuiFlow::Continue));
        assert_eq!(selection.selected(), 1);
        assert_eq!(handle_table_key(&mut selection, 3, TuiKey::Prev), Some(TuiFlow::Continue));
        assert_eq!(selection.selected(), 0);
        assert_eq!(
            handle_table_key(&mut selection, 3, TuiKey::Quit),
            Some(TuiFlow::Exit(ExitCode::SUCCESS))
        );
        assert_eq!(handle_table_key(&mut selection, 3, TuiKey::Char('i')), None);
    }

    #[test]
    fn key_help_places_common_bindings_around_actions() {
        assert_eq!(
            key_help(&["i inserts selected", "a inserts all missing"]),
            "j/k or arrows move; i inserts selected; a inserts all missing; q quits"
        );
    }
}
