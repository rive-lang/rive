//! Implementation of the `rive init` command.

use crate::utils::{MAIN_RIVE_TEMPLATE, print_status};
use anyhow::{Context, Result};
use rive_utils::Config;
use std::fs;
use std::path::Path;

/// Executes the `init` command to initialize a Rive project in the current directory.
///
/// # Errors
/// Returns an error if rive.toml already exists or if files cannot be created.
pub fn execute() -> Result<()> {
    if Path::new("rive.toml").exists() {
        anyhow::bail!("rive.toml already exists in current directory");
    }

    let project_name = get_project_name()?;
    create_project_files(&project_name)?;

    print_status("Created", &format!("Rive project '{project_name}'"));
    println!();
    println!("To get started:");
    println!("  rive run");

    Ok(())
}

/// Gets the project name from the current directory.
fn get_project_name() -> Result<String> {
    let current_dir = std::env::current_dir().with_context(|| "Failed to get current directory")?;

    let name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("rive-project");

    Ok(name.to_string())
}

/// Creates project configuration and source files.
fn create_project_files(project_name: &str) -> Result<()> {
    fs::create_dir_all("src").with_context(|| "Failed to create src directory")?;

    let config = Config::new(project_name);
    config.save("rive.toml")?;

    let main_path = Path::new("src/main.rive");
    if !main_path.exists() {
        fs::write(main_path, MAIN_RIVE_TEMPLATE)
            .with_context(|| "Failed to create src/main.rive")?;
    }

    Ok(())
}
