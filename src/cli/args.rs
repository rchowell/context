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

    /// Validate documents in the cache
    #[command(about = "Validate cached documents against their hashes")]
    Validate {
        /// Recursively validate subdirectories
        #[arg(short, long)]
        recursive: bool,

        /// Pattern for files to validate
        #[arg(short, long, value_name = "PATTERN")]
        filter: Option<String>,
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

    /// Search documents
    #[command(about = "Search cached documents by content")]
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,

        /// Case-sensitive search
        #[arg(short, long)]
        case_sensitive: bool,

        /// Limit number of results
        #[arg(short, long, value_name = "COUNT")]
        limit: Option<usize>,
    },

    /// Find a document by hash
    #[command(about = "Find a cached document by its content hash")]
    Find {
        /// Hash to search for
        #[arg(value_name = "HASH")]
        hash: String,
    },

    /// Synchronize cache metadata
    #[command(about = "Synchronize cache metadata with actual files")]
    Sync {
        /// Remove stale entries from cache
        #[arg(short, long)]
        cleanup: bool,

        /// Force full re-hash of all documents
        #[arg(short, long)]
        force: bool,
    },
}
