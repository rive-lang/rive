//! Implementation of the `rive check` command.

use crate::compiler::Compiler;
use anyhow::{Context, Result};
use rive_utils::Config;

/// Executes the `check` command to validate the Rive project without building.
///
/// # Errors
/// Returns an error if the project cannot be validated.
pub fn execute() -> Result<()> {
    let (_config, project_root) =
        Config::find().with_context(|| "Not in a Rive project directory")?;

    let compiler = Compiler::new(project_root)?;
    let _ = compiler.check()?;

    Ok(())
}
