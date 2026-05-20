//! Shared terminal UI setup for CLI utilities.

use std::process::ExitCode;

use ratatui::DefaultTerminal;

use crate::surface::error::CommandError;

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
