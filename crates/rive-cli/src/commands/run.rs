//! Implementation of the `rive run` command.

use crate::compiler::Compiler;
use anyhow::{Context, Result};
use rive_utils::Config;
use std::process::Command;

/// Executes the `run` command to build and run the Rive project.
///
/// # Errors
/// Returns an error if the project cannot be built or run.
pub fn execute() -> Result<()> {
    use colored::Colorize;

    let (_config, project_root) =
        Config::find().with_context(|| "Not in a Rive project directory")?;

    let compiler = Compiler::new(project_root)?;
    let (binary_path, _duration) = compiler.build(false)?;

    println!(
        "     {} {}",
        "Running".green().bold(),
        binary_path.display()
    );
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
