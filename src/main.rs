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
use anyhow::{Context, Result};
use clap::Parser;
use git2::Repository;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

mod walk;

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
}

#[derive(Deserialize)]
struct Config {
    delimiter: String,
    extensions: Vec<String>,
}

fn get_git_info(path: &PathBuf) -> Result<(String, String)> {
    let repo = Repository::discover(path)?;
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;

    Ok((commit.id().to_string(), commit.time().seconds().to_string()))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config from the specified file
    let config_content = fs::read_to_string(&cli.config)
        .context(format!("Failed to read config file: {}", cli.config))?;
    let config: Config =
        toml::from_str(&config_content).context("Failed to parse config")?;

    let walker = FileWalker::new(config.extensions);
    let files = walker.walk(&cli.dir)?;

    for file in files {
        let content = fs::read_to_string(&file.absolute_path)?;

        // Get git information
        let (commit_hash, commit_time) = get_git_info(&file.absolute_path)
            .unwrap_or(("unknown".to_string(), "unknown".to_string()));

        // Print file annotation
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
