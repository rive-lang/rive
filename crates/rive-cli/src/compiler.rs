//! Compiler pipeline implementation.
//!
//! Pipeline: Source → Lexer → Parser → AST → Semantic → RIR → CodeGen → Rust

use crate::pipeline;
use crate::utils::{binary_name, print_project_status};
use anyhow::{Context, Result};
use rive_utils::Config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use which::which;

/// Compiler for Rive programs.
pub struct Compiler {
    project_root: PathBuf,
    config: Config,
}

impl Compiler {
    /// Creates a new compiler for the given project.
    ///
    /// # Errors
    /// Returns an error if the project configuration cannot be loaded.
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let config = Config::load(project_root.join("rive.toml"))?;
        Ok(Self {
            project_root,
            config,
        })
    }

    /// Compiles the Rive project to a binary executable.
    ///
    /// # Errors
    /// Returns an error if compilation fails at any stage.
    pub fn build(&self, release: bool) -> Result<(PathBuf, Duration)> {
        let start = Instant::now();

        print_project_status("Compiling", &self.config, &self.project_root);

        let source = self.read_main_source()?;
        let rust_code = pipeline::build_pipeline(&source)?;

        self.save_generated_code(&rust_code)?;
        let binary_path = self.compile_to_binary(&rust_code, release)?;

        let duration = start.elapsed();
        self.print_build_success(release, duration);

        Ok((binary_path, duration))
    }

    /// Checks the project for errors without building.
    ///
    /// # Errors
    /// Returns an error if the project contains errors.
    pub fn check(&self) -> Result<Duration> {
        let start = Instant::now();

        print_project_status("Checking", &self.config, &self.project_root);

        let source = self.read_main_source()?;
        pipeline::check_pipeline(&source)?;

        let duration = start.elapsed();
        self.print_check_success(duration);

        Ok(duration)
    }

    /// Returns the project root directory.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Reads the main source file.
    fn read_main_source(&self) -> Result<String> {
        let source_path = self.project_root.join("src/main.rive");
        fs::read_to_string(&source_path)
            .with_context(|| format!("Failed to read {}", source_path.display()))
    }

    /// Saves generated Rust code to target directory.
    fn save_generated_code(&self, rust_code: &str) -> Result<()> {
        let target_dir = self.project_root.join("target");
        fs::create_dir_all(&target_dir).with_context(|| "Failed to create target directory")?;

        let rust_output_path = target_dir.join("main.rs");
        fs::write(&rust_output_path, rust_code)
            .with_context(|| "Failed to save generated Rust code")?;

        Ok(())
    }

    /// Compiles generated Rust code to binary.
    fn compile_to_binary(&self, rust_code: &str, release: bool) -> Result<PathBuf> {
        let temp_dir = TempDir::new().with_context(|| "Failed to create temporary directory")?;
        self.create_rust_project(&temp_dir, rust_code)?;

        let binary_path = self.compile_rust(&temp_dir, release)?;
        let target_dir = self.project_root.join("target");
        let final_path = target_dir.join(binary_name(&self.config.package.name));

        fs::copy(&binary_path, &final_path)
            .with_context(|| "Failed to copy binary to target directory")?;

        Ok(final_path)
    }

    /// Creates a temporary Rust project with the generated code.
    fn create_rust_project(&self, temp_dir: &TempDir, rust_code: &str) -> Result<()> {
        let cargo_toml = format!(
            "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2024\"\n\n[dependencies]\n",
            self.config.package.name, self.config.package.version
        );

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)
            .with_context(|| "Failed to write Cargo.toml")?;

        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).with_context(|| "Failed to create src directory")?;

        fs::write(src_dir.join("main.rs"), rust_code).with_context(|| "Failed to write main.rs")?;

        Ok(())
    }

    /// Compiles the Rust project using rustc or cargo.
    fn compile_rust(&self, temp_dir: &TempDir, release: bool) -> Result<PathBuf> {
        if which("cargo").is_ok() {
            self.compile_with_cargo(temp_dir, release)
        } else if which("rustc").is_ok() {
            self.compile_with_rustc(temp_dir)
        } else {
            anyhow::bail!(
                "Neither cargo nor rustc found in PATH. Please install Rust from https://rustup.rs/"
            )
        }
    }

    /// Compiles using cargo.
    fn compile_with_cargo(&self, temp_dir: &TempDir, release: bool) -> Result<PathBuf> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build").current_dir(temp_dir.path());

        if release {
            cmd.arg("--release");
        }

        let output = cmd
            .output()
            .with_context(|| "Failed to execute cargo build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Rust compilation failed:\n{stderr}");
        }

        let profile = if release { "release" } else { "debug" };
        let binary_path = temp_dir
            .path()
            .join("target")
            .join(profile)
            .join(binary_name(&self.config.package.name));

        Ok(binary_path)
    }

    /// Compiles using rustc directly.
    fn compile_with_rustc(&self, temp_dir: &TempDir) -> Result<PathBuf> {
        let output_path = temp_dir.path().join(binary_name(&self.config.package.name));

        let output = Command::new("rustc")
            .arg(temp_dir.path().join("src/main.rs"))
            .arg("-o")
            .arg(&output_path)
            .output()
            .with_context(|| "Failed to execute rustc")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Rust compilation failed:\n{stderr}");
        }

        Ok(output_path)
    }

    /// Prints build success message.
    fn print_build_success(&self, release: bool, duration: Duration) {
        use colored::Colorize;
        let profile = if release { "release" } else { "dev" };
        println!(
            "    {} project built successfully with `{profile}` profile in {:.2}s",
            "Finished".green().bold(),
            duration.as_secs_f64()
        );
    }

    /// Prints check success message.
    fn print_check_success(&self, duration: Duration) {
        use colored::Colorize;
        println!(
            "    {} project checked successfully in {:.2}s",
            "Finished".green().bold(),
            duration.as_secs_f64()
        );
    }
}
