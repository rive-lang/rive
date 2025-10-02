//! Implementation of the `rive new` command.

use anyhow::{Context, Result};
use rive_utils::Config;
use std::fs;
use std::path::Path;

const MAIN_RIVE_TEMPLATE: &str = r#"fun main() {
    print("Hello, Rive!")
}
"#;

/// Executes the `new` command to create a new Rive project.
///
/// # Errors
/// Returns an error if the project directory already exists or if files cannot be created.
pub fn execute(name: &str) -> Result<()> {
    let project_dir = Path::new(name);

    // Check if directory already exists
    if project_dir.exists() {
        anyhow::bail!("Directory '{name}' already exists");
    }

    // Create project structure
    fs::create_dir_all(project_dir.join("src"))
        .with_context(|| format!("Failed to create directory '{name}'"))?;

    // Create rive.toml
    let config = Config::new(name);
    config.save(project_dir.join("rive.toml"))?;

    // Create src/main.rive
    fs::write(project_dir.join("src/main.rive"), MAIN_RIVE_TEMPLATE)
        .with_context(|| "Failed to create src/main.rive")?;

    use colored::Colorize;

    println!("     {} Rive project '{}'", "Created".green().bold(), name);
    println!();
    println!("To get started:");
    println!("  cd {name}");
    println!("  rive run");

    Ok(())
}
