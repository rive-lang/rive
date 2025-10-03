//! Compiler pipeline implementation.
//!
//! Pipeline: Source → Lexer → Parser → AST → Semantic → RIR → CodeGen → Rust

use anyhow::{Context, Result};
use rive_codegen::CodeGenerator;
use rive_core::type_system::TypeRegistry;
use rive_ir::AstLowering;
use rive_lexer::tokenize;
use rive_parser::parse;
use rive_utils::Config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    pub fn build(&self, release: bool) -> Result<(PathBuf, std::time::Duration)> {
        use colored::Colorize;
        use std::time::Instant;

        let start = Instant::now();

        println!(
            "   {} {} v{} ({})",
            "Compiling".green().bold(),
            self.config.package.name,
            self.config.package.version,
            self.project_root.display()
        );

        // Read main.rive source file
        let source_path = self.project_root.join("src/main.rive");
        let source = fs::read_to_string(&source_path)
            .with_context(|| format!("Failed to read {}", source_path.display()))?;

        // Lexical analysis
        let tokens = tokenize(&source).map_err(|e| {
            let report = miette::Report::new(e)
                .with_source_code(miette::NamedSource::new("main.rive", source.clone()));
            eprintln!("{report:?}");
            anyhow::anyhow!("Lexical analysis failed")
        })?;

        // Parsing
        let ast = parse(&tokens).map_err(|e| {
            let report = miette::Report::new(e)
                .with_source_code(miette::NamedSource::new("main.rive", source.clone()));
            eprintln!("{report:?}");
            anyhow::anyhow!("Parsing failed")
        })?;

        // Semantic analysis (type checking)
        rive_semantic::analyze(&ast).map_err(|e| {
            let report = miette::Report::new(e)
                .with_source_code(miette::NamedSource::new("main.rive", source.clone()));
            eprintln!("{report:?}");
            anyhow::anyhow!("Semantic analysis failed")
        })?;

        // AST → RIR lowering
        let type_registry = TypeRegistry::new();
        let mut lowering = AstLowering::new(type_registry);
        let rir_module = lowering.lower_program(&ast).map_err(|e| {
            let report = miette::Report::new(e)
                .with_source_code(miette::NamedSource::new("main.rive", source.clone()));
            eprintln!("{report:?}");
            anyhow::anyhow!("RIR lowering failed")
        })?;

        // Code generation (RIR → Rust)
        let mut codegen = CodeGenerator::new();
        let rust_code = codegen
            .generate(&rir_module)
            .with_context(|| "Code generation failed")?;

        // Save generated Rust code to target directory
        let target_dir = self.project_root.join("target");
        fs::create_dir_all(&target_dir).with_context(|| "Failed to create target directory")?;

        // Extract filename from source path (e.g., main.rive -> main.rs)
        let source_filename = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("generated");
        let rust_output_path = target_dir.join(format!("{source_filename}.rs"));

        fs::write(&rust_output_path, &rust_code)
            .with_context(|| "Failed to save generated Rust code")?;

        // Create temporary directory for Rust project
        let temp_dir = TempDir::new().with_context(|| "Failed to create temporary directory")?;

        self.create_rust_project(&temp_dir, &rust_code)?;

        // Compile Rust code
        let binary_path = self.compile_rust(&temp_dir, release)?;

        let output_name = if cfg!(windows) {
            format!("{}.exe", self.config.package.name)
        } else {
            self.config.package.name.clone()
        };

        let final_path = target_dir.join(&output_name);
        fs::copy(&binary_path, &final_path)
            .with_context(|| "Failed to copy binary to target directory")?;

        let duration = start.elapsed();

        let profile = if release { "release" } else { "dev" };
        println!(
            "    {} project built successfully with `{}` profile in {:.2}s",
            "Finished".green().bold(),
            profile,
            duration.as_secs_f64()
        );

        Ok((final_path, duration))
    }

    /// Checks the project for errors without building.
    ///
    /// # Errors
    /// Returns an error if the project contains errors.
    pub fn check(&self) -> Result<std::time::Duration> {
        use colored::Colorize;
        use std::time::Instant;

        let start = Instant::now();

        println!(
            "    {} {} v{} ({})",
            "Checking".green().bold(),
            self.config.package.name,
            self.config.package.version,
            self.project_root.display()
        );

        let source_path = self.project_root.join("src/main.rive");
        let source = fs::read_to_string(&source_path)
            .with_context(|| format!("Failed to read {}", source_path.display()))?;

        // Lexical analysis
        let tokens = tokenize(&source).map_err(|e| {
            let report = miette::Report::new(e)
                .with_source_code(miette::NamedSource::new("main.rive", source.clone()));
            eprintln!("{report:?}");
            anyhow::anyhow!("Lexical analysis failed")
        })?;

        // Parsing
        let ast = parse(&tokens).map_err(|e| {
            let report = miette::Report::new(e)
                .with_source_code(miette::NamedSource::new("main.rive", source.clone()));
            eprintln!("{report:?}");
            anyhow::anyhow!("Parsing failed")
        })?;

        // Semantic analysis (type checking)
        rive_semantic::analyze(&ast).map_err(|e| {
            let report = miette::Report::new(e)
                .with_source_code(miette::NamedSource::new("main.rive", source.clone()));
            eprintln!("{report:?}");
            anyhow::anyhow!("Semantic analysis failed")
        })?;

        // AST → RIR lowering (to verify it can lower)
        let type_registry = TypeRegistry::new();
        let mut lowering = AstLowering::new(type_registry);
        let rir_module = lowering.lower_program(&ast).map_err(|e| {
            let report = miette::Report::new(e)
                .with_source_code(miette::NamedSource::new("main.rive", source.clone()));
            eprintln!("{report:?}");
            anyhow::anyhow!("RIR lowering failed")
        })?;

        // Code generation (to verify it can generate)
        let mut codegen = CodeGenerator::new();
        let _rust_code = codegen
            .generate(&rir_module)
            .with_context(|| "Code generation failed")?;

        let duration = start.elapsed();

        println!(
            "    {} project checked successfully in {:.2}s",
            "Finished".green().bold(),
            duration.as_secs_f64()
        );

        Ok(duration)
    }

    /// Creates a temporary Rust project with the generated code.
    fn create_rust_project(&self, temp_dir: &TempDir, rust_code: &str) -> Result<()> {
        // Create Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "{}"
edition = "2024"

[dependencies]
"#,
            self.config.package.name, self.config.package.version
        );

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)
            .with_context(|| "Failed to write Cargo.toml")?;

        // Create src directory
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).with_context(|| "Failed to create src directory")?;

        // Write main.rs
        fs::write(src_dir.join("main.rs"), rust_code).with_context(|| "Failed to write main.rs")?;

        Ok(())
    }

    /// Compiles the Rust project using rustc or cargo.
    fn compile_rust(&self, temp_dir: &TempDir, release: bool) -> Result<PathBuf> {
        // Check if cargo is available
        if which("cargo").is_ok() {
            self.compile_with_cargo(temp_dir, release)
        } else if which("rustc").is_ok() {
            self.compile_with_rustc(temp_dir)
        } else {
            anyhow::bail!(
                "Neither cargo nor rustc found in PATH. Please install Rust from https://rustup.rs/"
            );
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
        let binary_name = if cfg!(windows) {
            format!("{}.exe", self.config.package.name)
        } else {
            self.config.package.name.clone()
        };

        let binary_path = temp_dir
            .path()
            .join("target")
            .join(profile)
            .join(binary_name);

        Ok(binary_path)
    }

    /// Compiles using rustc directly.
    fn compile_with_rustc(&self, temp_dir: &TempDir) -> Result<PathBuf> {
        let output_name = if cfg!(windows) {
            format!("{}.exe", self.config.package.name)
        } else {
            self.config.package.name.clone()
        };

        let output_path = temp_dir.path().join(&output_name);

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

    /// Returns the project root directory.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}
