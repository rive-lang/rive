//! Implementation of the `rive init` command.

use anyhow::{Context, Result};
use rive_utils::Config;
use std::fs;
use std::path::Path;

const MAIN_RIVE_TEMPLATE: &str = r#"fun main() {
    print("Hello, Rive!")
}
"#;

/// Executes the `init` command to initialize a Rive project in the current directory.
///
/// # Errors
/// Returns an error if rive.toml already exists or if files cannot be created.
pub fn execute() -> Result<()> {
    let current_dir = std::env::current_dir().with_context(|| "Failed to get current directory")?;

    // Check if rive.toml already exists
    if Path::new("rive.toml").exists() {
        anyhow::bail!("rive.toml already exists in current directory");
    }

    // Get project name from directory name
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("rive-project");

    // Create src directory if it doesn't exist
    fs::create_dir_all("src").with_context(|| "Failed to create src directory")?;

    // Create rive.toml
    let config = Config::new(project_name);
    config.save("rive.toml")?;

    // Create src/main.rive if it doesn't exist
    let main_path = Path::new("src/main.rive");
    if !main_path.exists() {
        fs::write(main_path, MAIN_RIVE_TEMPLATE)
            .with_context(|| "Failed to create src/main.rive")?;
    }

    use colored::Colorize;

    println!(
        "     {} Rive project '{}'",
        "Created".green().bold(),
        project_name
    );
    println!();
    println!("To get started:");
    println!("  rive run");

    Ok(())
}
