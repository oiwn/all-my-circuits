//! A command-line tool for concatenating and annotating files with Git metadata
//!
//! This tool walks through a directory, finds files with specified extensions,
//! and outputs their contents along with Git commit information. Each file's
//! content is preceded by a header containing:
//! - The relative file path
//! - The last Git commit hash that modified the file
//! - The timestamp of the last update
//!
//! # Configuration
//!
//! The tool uses a TOML configuration file (default: `.amc.toml`) that specifies:
//! - `delimiter`: String used to separate file headers from content
//! - `extensions`: List of file extensions to process
//!
//! # Example Usage
//!
//! ```bash
//! $ amc output.txt --dir ./src --config .amc.toml
//! ```
//!
//! Or with default output file (code.txt):
//!
//! ```bash
//! $ amc --dir ./src
//! ```
//!
//! # Command Line Arguments
//!
//! - `[OUTPUT]`: Output file path (default: "code.txt")
//! - `-d, --dir`: Directory to scan (default: ".")
//! - `-c, --config`: Path to config file (default: ".amc.toml")
use clap::Parser;
use git2::Repository;
use log::{LevelFilter, info};
use simple_logger::SimpleLogger;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod config;
mod walk;

use config::Config;
use walk::FileWalker;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output file path (defaults to "code.txt" if not specified)
    #[arg(default_value = "code.txt")]
    output: String,

    /// Directory to scan
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Config file path
    #[arg(short, long, default_value = ".amc.toml")]
    config: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Check if the directory is a Git repository
    if !is_git_repository(&cli.dir) {
        return Err(anyhow::anyhow!(
            "The specified directory '{}' is not a Git repository or within one. \
            This tool only works with Git-managed directories.",
            cli.dir
        ));
    }

    setup_logging(cli.verbose);

    // Load config from the specified file
    let config = Config::load(&cli.config)?;
    info!("Loaded configuration from: {}", cli.config);

    let walker = FileWalker::new(config.extensions, config.excluded_folders);
    let files = walker.walk(&cli.dir)?;

    // Create or open the output file
    let mut output_file = fs::File::create(&cli.output)?;
    info!("Writing output to file: {}", cli.output);

    writeln!(output_file, "{}", config.llm_prompt)?;

    for file in files {
        info!("Processing file: {}", file.absolute_path.display());
        let content = fs::read_to_string(&file.absolute_path)?;

        // Get git information
        let (commit_hash, commit_time) = get_git_info(&file.absolute_path)
            .unwrap_or(("unknown".to_string(), "unknown".to_string()));

        info!("Git info - commit: {commit_hash}, time: {commit_time}");

        // Print file annotation
        writeln!(output_file, "{}", config.delimiter)?;
        writeln!(output_file, "File: {}", file.relative_path.display())?;
        writeln!(output_file, "Last commit: {commit_hash}")?;
        writeln!(output_file, "Last update: {commit_time}")?;
        writeln!(output_file, "{}", config.delimiter)?;

        // Print file content
        writeln!(output_file, "{content}\n")?;
    }

    Ok(())
}

fn setup_logging(verbose: bool) {
    if verbose {
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .without_timestamps()
            .init()
            .unwrap();
    }
}

fn get_git_info(path: &PathBuf) -> anyhow::Result<(String, String)> {
    let repo = Repository::discover(path)?;
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;

    Ok((commit.id().to_string(), commit.time().seconds().to_string()))
}

fn is_git_repository(path: &str) -> bool {
    Repository::discover(path).is_ok()
}
