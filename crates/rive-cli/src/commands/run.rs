//! Implementation of the `rive run` command.

use crate::compiler::Compiler;
use crate::utils::{find_project, print_status};
use anyhow::{Context, Result};
use std::process::Command;

/// Executes the `run` command to build and run the Rive project.
///
/// # Errors
/// Returns an error if the project cannot be built or run.
pub fn execute() -> Result<()> {
    let (_config, project_root) = find_project()?;
    let compiler = Compiler::new(project_root)?;
    let (binary_path, _duration) = compiler.build(false)?;

    print_status("Running", &binary_path.display().to_string());
    println!();

    let status = Command::new(&binary_path)
        .current_dir(compiler.project_root())
        .status()
        .with_context(|| format!("Failed to execute {}", binary_path.display()))?;

    if !status.success() {
        anyhow::bail!("Program exited with status: {status}");
    }

    Ok(())
}
