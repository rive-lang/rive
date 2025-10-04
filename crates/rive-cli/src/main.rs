//! Command-line interface for the Rive compiler.

mod commands;
mod compiler;
mod pipeline;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rive")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Rive project
    #[command(visible_alias = "n")]
    New {
        /// Name of the project to create
        name: String,
    },

    /// Initialize a Rive project in an existing directory
    Init,

    /// Compile the current project
    #[command(visible_alias = "b")]
    Build,

    /// Build and execute the current project
    #[command(visible_alias = "r")]
    Run,

    /// Check the current project for errors
    Check,

    /// Remove the target directory
    Clean,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => commands::new::execute(&name),
        Commands::Init => commands::init::execute(),
        Commands::Build => commands::build::execute(),
        Commands::Run => commands::run::execute(),
        Commands::Check => commands::check::execute(),
        Commands::Clean => commands::clean::execute(),
    }
}
