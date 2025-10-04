//! Implementation of the `rive clean` command.

use crate::utils::{find_project, print_project_status, print_status};
use anyhow::{Context, Result};
use std::fs;

/// Executes the `clean` command to remove build artifacts.
///
/// # Errors
/// Returns an error if build artifacts cannot be removed.
pub fn execute() -> Result<()> {
    let (config, project_dir) = find_project()?;

    print_project_status("Cleaning", &config, &project_dir);

    let target_dir = project_dir.join("target");

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .with_context(|| format!("Failed to remove {}", target_dir.display()))?;
        print_status("Finished", "target directory removed");
    } else {
        print_status(
            "Finished",
            "target directory does not exist, nothing to clean",
        );
    }

    Ok(())
}
