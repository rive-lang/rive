//! Implementation of the `rive clean` command.

use anyhow::{Context, Result};
use colored::Colorize;
use rive_utils::Config;
use std::fs;

/// Executes the `clean` command to remove build artifacts.
///
/// # Errors
/// Returns an error if build artifacts cannot be removed.
pub fn execute() -> Result<()> {
    let (config, project_dir) =
        Config::find().with_context(|| "Failed to find rive.toml. Are you in a Rive project?")?;

    println!(
        "     {} {} v{} ({})",
        "Cleaning".green().bold(),
        config.package.name,
        config.package.version,
        project_dir.display()
    );

    let target_dir = project_dir.join("target");

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .with_context(|| format!("Failed to remove {}", target_dir.display()))?;

        println!("    {} target directory removed", "Finished".green().bold());
    } else {
        println!(
            "    {} target directory does not exist, nothing to clean",
            "Finished".green().bold()
        );
    }

    Ok(())
}
