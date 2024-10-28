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
//! $ amc --dir ./src --config .amc.toml
//! ```
//!
//! # Command Line Arguments
//!
//! - `-d, --dir`: Directory to scan (default: ".")
//! - `-c, --config`: Path to config file (default: ".amc.toml")
//!
use clap::Parser;
use git2::Repository;
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use std::fs;
use std::path::PathBuf;

mod config;
mod walk;

use config::Config;
use walk::FileWalker;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
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

    let walker = FileWalker::new(config.extensions);
    let files = walker.walk(&cli.dir)?;

    for file in files {
        info!("Processing file: {}", file.absolute_path.display());
        let content = fs::read_to_string(&file.absolute_path)?;

        // Get git information
        let (commit_hash, commit_time) = get_git_info(&file.absolute_path)
            .unwrap_or(("unknown".to_string(), "unknown".to_string()));

        info!("Git info - commit: {}, time: {}", commit_hash, commit_time);

        // Print file annotation
        println!("{}", config.llm_prompt);
        println!("{}", config.delimiter);
        println!("File: {}", file.relative_path.display());
        println!("Last commit: {}", commit_hash);
        println!("Last update: {}", commit_time);
        println!("{}", config.delimiter);

        // Print file content
        println!("{}\n", content);
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
