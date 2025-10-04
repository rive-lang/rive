//! Implementation of the `rive new` command.

use crate::utils::{MAIN_RIVE_TEMPLATE, print_status};
use anyhow::{Context, Result};
use rive_utils::Config;
use std::fs;
use std::path::Path;

/// Executes the `new` command to create a new Rive project.
///
/// # Errors
/// Returns an error if the project directory already exists or if files cannot be created.
pub fn execute(name: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{name}' already exists");
    }

    create_project_structure(project_dir, name)?;

    print_status("Created", &format!("Rive project '{name}'"));
    println!();
    println!("To get started:");
    println!("  cd {name}");
    println!("  rive run");

    Ok(())
}

/// Creates the project directory structure.
fn create_project_structure(project_dir: &Path, name: &str) -> Result<()> {
    fs::create_dir_all(project_dir.join("src"))
        .with_context(|| format!("Failed to create directory '{name}'"))?;

    let config = Config::new(name);
    config.save(project_dir.join("rive.toml"))?;

    fs::write(project_dir.join("src/main.rive"), MAIN_RIVE_TEMPLATE)
        .with_context(|| "Failed to create src/main.rive")?;

    Ok(())
}
