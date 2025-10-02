//! Implementation of the `rive build` command.

use crate::compiler::Compiler;
use anyhow::{Context, Result};
use rive_utils::Config;

/// Executes the `build` command to compile the Rive project.
///
/// # Errors
/// Returns an error if the project cannot be built.
pub fn execute() -> Result<()> {
    let (_config, project_root) =
        Config::find().with_context(|| "Not in a Rive project directory")?;

    let compiler = Compiler::new(project_root)?;
    let _ = compiler.build(false)?;

    Ok(())
}
