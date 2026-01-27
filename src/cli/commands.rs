use crate::core::{find_context_root_from_cwd, Cache};
use crate::error::{ContextError, Result};

use super::args::{Cli, Commands, FindArgs, InitArgs, OutputFormat, ServeArgs, StatusArgs, SyncArgs};
use super::console;

/// Execute a CLI command and return exit code
pub async fn execute(cli: Cli) -> Result<i32> {
    match cli.command {
        Commands::Init(args) => init(args).await,
        Commands::Status(args) => status(args, cli.output).await,
        Commands::Sync(args) => sync(args, cli.output).await,
        Commands::Find(args) => find(args, cli.output).await,
        Commands::Serve(args) => serve(args).await,
    }
}

/// Initialize a new context cache directory
#[allow(clippy::unused_async)]
async fn init(args: InitArgs) -> Result<i32> {
    let context_dir = args.path.join(".context");
    Cache::init(context_dir)?;
    println!("Initialized context cache at {}", args.path.display());
    Ok(0)
}

/// Show cache status
#[allow(clippy::unused_async)]
async fn status(args: StatusArgs, output: OutputFormat) -> Result<i32> {
    let context_dir = find_context_root_from_cwd()?;
    let mut cache = Cache::create(context_dir)?;
    cache.load()?;
    let mut statuses = cache.status()?;

    if args.invalid_only {
        statuses.retain(|s| s.status != crate::core::models::Status::Valid);
    }

    console::print_status(output, &statuses)?;

    let has_orphaned = statuses
        .iter()
        .any(|s| s.status == crate::core::models::Status::Orphaned);
    let has_stale = statuses
        .iter()
        .any(|s| s.status == crate::core::models::Status::Stale);

    if has_orphaned {
        Ok(2)
    } else {
        Ok(i32::from(has_stale))
    }
}

/// Synchronize cache metadata
#[allow(clippy::unused_async)]
async fn sync(args: SyncArgs, output: OutputFormat) -> Result<i32> {
    let context_dir = find_context_root_from_cwd()?;
    let mut cache = Cache::create(context_dir)?;
    cache.load()?;

    let resolved = args
        .path
        .as_ref()
        .map(|p| cache.resolve_doc_path(p))
        .transpose()?;

    match cache.sync(resolved.as_deref()) {
        Ok(result) => {
            console::print_sync(output, &result)?;
            Ok(i32::from(!result.failed.is_empty()))
        }
        Err(ContextError::InvalidReferences { documents, .. }) => {
            console::print_invalid_references(output, &documents)?;
            Ok(1)
        }
        Err(e) => Err(e),
    }
}

/// Find documents that reference given source files
#[allow(clippy::unused_async)]
async fn find(args: FindArgs, output: OutputFormat) -> Result<i32> {
    let context_dir = find_context_root_from_cwd()?;
    let mut cache = Cache::create(context_dir)?;
    cache.load()?;

    let mut results = Vec::new();
    let mut has_matches = false;

    for path in &args.paths {
        let path_str = path.display().to_string();
        let result = cache.find_by_reference(&path_str)?;
        if !result.matches.is_empty() {
            has_matches = true;
        }
        results.push(result);
    }

    console::print_find(output, &results)?;

    Ok(i32::from(!has_matches))
}

/// Start the MCP server
#[allow(clippy::unused_async)]
async fn serve(_args: ServeArgs) -> Result<i32> {
    crate::mcp::server::run_server()
        .await
        .map_err(|e| ContextError::Other(e.to_string()))?;
    Ok(0)
}

/// Map exit codes for different scenarios
#[must_use]
pub fn map_exit_code(success: bool, error: Option<&ContextError>) -> i32 {
    if success {
        return 0;
    }

    match error {
        Some(ContextError::NotARepository) => 128,
        Some(ContextError::NotInitialized(_)) => 3,
        _ => 1,
    }
}
