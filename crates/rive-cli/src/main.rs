//! Command-line interface for the Rive compiler.

mod commands;
mod compiler;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "rive")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(disable_help_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
        Some(Commands::New { name }) => commands::new::execute(&name),
        Some(Commands::Init) => commands::init::execute(),
        Some(Commands::Build) => commands::build::execute(),
        Some(Commands::Run) => commands::run::execute(),
        Some(Commands::Check) => commands::check::execute(),
        Some(Commands::Clean) => commands::clean::execute(),
        None => {
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    println!(
        "{} {} {}",
        "Usage:".green().bold(),
        "rive".cyan().bold(),
        "[command]".cyan()
    );
    println!();
    println!("Commands:");
    println!(
        "    {}, {}    Compile the current project",
        "build".cyan().bold(),
        "b".cyan().bold()
    );
    println!(
        "    {}       Check the current project for errors",
        "check".cyan().bold()
    );
    println!(
        "    {}       Remove the target directory",
        "clean".cyan().bold()
    );
    println!(
        "    {}        Initialize a Rive project in an existing directory",
        "init".cyan().bold()
    );
    println!(
        "    {}, {}      Create a new Rive project",
        "new".cyan().bold(),
        "n".cyan().bold()
    );
    println!(
        "    {}, {}      Build and execute the current project",
        "run".cyan().bold(),
        "r".cyan().bold()
    );
    println!();
    println!(
        "See '{} {}{}' for more information on a specific command.",
        "rive help".cyan().bold(),
        "<".cyan(),
        "command>".cyan()
    );
}
