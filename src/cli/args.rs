use clap::{Args, Parser, Subcommand};
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

/// Arguments for the init command
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Directory to initialize
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Create parent directories if they don't exist
    #[arg(short, long)]
    pub create: bool,
}

/// Arguments for the status command
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Show invalid documents only
    #[arg(short, long)]
    pub invalid_only: bool,

    /// Show details for each document
    #[arg(short, long)]
    pub detailed: bool,
}

/// Arguments for the sync command
#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Path to a specific document to sync (syncs all if omitted)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Remove stale entries from cache
    #[arg(short, long)]
    pub cleanup: bool,

    /// Force full re-hash of all documents
    #[arg(short, long)]
    pub force: bool,
}

/// Arguments for the find command
#[derive(Args, Debug)]
pub struct FindArgs {
    /// Source file paths to search for
    #[arg(value_name = "PATH", required = true, num_args = 1..)]
    pub paths: Vec<PathBuf>,
}

/// Arguments for the serve command
#[derive(Args, Debug)]
pub struct ServeArgs {}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new context cache directory
    #[command(about = "Initialize a new documentation cache")]
    Init(InitArgs),

    /// Show cache status
    #[command(about = "Display status of documents in the cache")]
    Status(StatusArgs),

    /// Synchronize cache metadata
    #[command(about = "Synchronize cache metadata with actual files")]
    Sync(SyncArgs),

    /// Find documents that reference given source files
    #[command(about = "Find documents that reference the given source file(s)")]
    Find(FindArgs),

    /// Start the MCP server
    #[command(about = "Start the Context MCP server")]
    Serve(ServeArgs),
}
