//! Sirno binary front door.

use std::process::ExitCode;

fn main() -> ExitCode {
    sirno::core::run_cli_from_env()
}
