//! Shared command surface for Sirno.
//!
//! The `core` module keeps the public command facade stable while the implementation
//! is organized by interface layer.

mod cli;
mod context;
mod dto;
mod error;
mod output;
mod rg;

pub use crate::core::cli::{Cli, run_cli_from_env};
pub use crate::core::context::CoreContext;
pub use crate::core::dto::*;
pub use crate::core::error::{CommandError, OpenTideTutorial};
pub use crate::core::output::format_json;
