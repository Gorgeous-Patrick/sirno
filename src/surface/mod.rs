//! Shared command surface for Sirno.
//!
//! The `surface` module keeps the public command facade stable while the implementation
//! is organized by interface layer.

// sirno:witness:interfaces:begin
mod cli;
mod context;
mod dto;
mod error;
mod output;
mod rg;

pub use crate::surface::cli::{Cli, run_cli_from_env};
pub use crate::surface::context::SurfaceContext;
pub use crate::surface::dto::*;
pub use crate::surface::error::{CommandError, OpenTideTutorial};
pub use crate::surface::output::format_json;
// sirno:witness:interfaces:end
