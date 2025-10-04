//! Implementation of the `rive build` command.

use crate::compiler::Compiler;
use crate::utils::find_project;
use anyhow::Result;

/// Executes the `build` command to compile the Rive project.
///
/// # Errors
/// Returns an error if the project cannot be built.
pub fn execute() -> Result<()> {
    let (_config, project_root) = find_project()?;
    let compiler = Compiler::new(project_root)?;
    compiler.build(false)?;
    Ok(())
}
