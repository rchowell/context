use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Context CLI - Documentation cache and validation tool
#[derive(Parser)]
#[command(name = "context")]
#[command(about = "A documentation cache tool for managing cached content with validation and search", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Output format (human-readable or JSON)
    #[arg(global = true, long, value_name = "FORMAT", default_value = "human")]
    pub output: OutputFormat,

    /// The context command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Output format options
#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" | "human" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("Unknown output format: {s}")),
        }
    }
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new context cache directory
    #[command(about = "Initialize a new documentation cache")]
    Init {
        /// Directory to initialize
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Create parent directories if they don't exist
        #[arg(short, long)]
        create: bool,
    },

    /// Show cache status
    #[command(about = "Display status of documents in the cache")]
    Status {
        /// Show invalid documents only
        #[arg(short, long)]
        invalid_only: bool,

        /// Show details for each document
        #[arg(short, long)]
        detailed: bool,
    },

    /// Synchronize cache metadata
    #[command(about = "Synchronize cache metadata with actual files")]
    Sync {
        /// Path to a specific document to sync (syncs all if omitted)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Remove stale entries from cache
        #[arg(short, long)]
        cleanup: bool,

        /// Force full re-hash of all documents
        #[arg(short, long)]
        force: bool,
    },

    /// Find documents that reference given source files
    #[command(about = "Find documents that reference the given source file(s)")]
    Find {
        /// Source file paths to search for
        #[arg(value_name = "PATH", required = true, num_args = 1..)]
        paths: Vec<PathBuf>,
    },
}
