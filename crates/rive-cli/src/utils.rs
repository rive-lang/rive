//! Utility functions and constants shared across the CLI.

use anyhow::{Context, Result};
use colored::Colorize;
use rive_utils::Config;
use std::path::PathBuf;

/// Template for main.rive file in new projects.
pub const MAIN_RIVE_TEMPLATE: &str = r#"fun main() {
    print("Hello, Rive!")
}
"#;

/// Finds the Rive project root and config.
///
/// # Errors
/// Returns an error if not in a Rive project directory.
pub fn find_project() -> Result<(Config, PathBuf)> {
    Config::find().with_context(|| "Not in a Rive project directory")
}

/// Returns the binary file name with platform-specific extension.
pub fn binary_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// Prints a status message with colored output.
pub fn print_status(status: &str, message: &str) {
    println!("{} {message}", status.green().bold());
}

/// Prints a status message with project info.
pub fn print_project_status(status: &str, config: &Config, path: &std::path::Path) {
    println!(
        "{} {} v{} ({})",
        status.green().bold(),
        config.package.name,
        config.package.version,
        path.display()
    );
}
