//! Configuration file handling for Rive projects.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents the rive.toml configuration file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub package: Package,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    #[serde(default = "default_edition")]
    pub edition: String,
}

fn default_edition() -> String {
    "2025".to_string()
}

impl Config {
    /// Creates a new default configuration with the given project name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            package: Package {
                name: name.into(),
                version: "0.1.0".to_string(),
                edition: default_edition(),
            },
        }
    }

    /// Loads configuration from a rive.toml file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;

        toml::from_str(&contents).with_context(|| "Failed to parse rive.toml")
    }

    /// Saves configuration to a rive.toml file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let contents =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize configuration")?;

        fs::write(path.as_ref(), contents)
            .with_context(|| format!("Failed to write {}", path.as_ref().display()))
    }

    /// Finds the rive.toml file starting from the current directory.
    ///
    /// # Errors
    /// Returns an error if no rive.toml is found in the current or parent directories.
    pub fn find() -> Result<(Self, std::path::PathBuf)> {
        let mut current_dir =
            std::env::current_dir().with_context(|| "Failed to get current directory")?;

        loop {
            let config_path = current_dir.join("rive.toml");
            if config_path.exists() {
                let config = Self::load(&config_path)?;
                return Ok((config, current_dir));
            }

            if !current_dir.pop() {
                anyhow::bail!(
                    "Could not find rive.toml in current directory or any parent directory"
                );
            }
        }
    }
}
