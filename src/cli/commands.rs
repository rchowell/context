use crate::core::{find_context_root_from_cwd, Cache};
use crate::error::{ContextError, Result};

use super::args::{Commands, Cli};
use super::output;

/// Execute a CLI command and return exit code
pub fn execute(cli: Cli) -> Result<i32> {
    match cli.command {
        Commands::Init { path, .. } => {
            let context_dir = path.join(".context");
            Cache::init(context_dir)?;
            println!("Initialized context cache at {}", path.display());
            Ok(0)
        }
        Commands::Status {
            invalid_only,
            detailed: _,
        } => {
            let context_dir = find_context_root_from_cwd()?;
            let mut cache = Cache::create(context_dir)?;
            cache.load()?;
            let mut statuses = cache.status()?;

            if invalid_only {
                statuses.retain(|s| s.status != crate::core::models::Status::Valid);
            }

            output::print_status(cli.output, &statuses)?;

            // Return appropriate exit code based on severity
            let has_orphaned = statuses.iter().any(|s| s.status == crate::core::models::Status::Orphaned);
            let has_stale = statuses.iter().any(|s| s.status == crate::core::models::Status::Stale);

            if has_orphaned {
                Ok(2)
            } else {
                Ok(i32::from(has_stale))
            }
        }
        Commands::Sync { path, cleanup: _, force: _ } => {
            let context_dir = find_context_root_from_cwd()?;
            let mut cache = Cache::create(context_dir)?;
            cache.load()?;

            // Resolve the document path if provided
            let resolved = path
                .as_ref()
                .map(|p| cache.resolve_doc_path(p))
                .transpose()?;

            match cache.sync(resolved.as_deref()) {
                Ok(result) => {
                    output::print_sync(cli.output, &result)?;
                    Ok(i32::from(!result.failed.is_empty()))
                }
                Err(ContextError::InvalidReferences { documents, .. }) => {
                    output::print_invalid_references(cli.output, &documents)?;
                    Ok(1)
                }
                Err(e) => Err(e),
            }
        }
        Commands::Find { paths } => {
            let context_dir = find_context_root_from_cwd()?;
            let mut cache = Cache::create(context_dir)?;
            cache.load()?;

            let mut results = Vec::new();
            let mut has_matches = false;

            for path in &paths {
                let path_str = path.display().to_string();
                let result = cache.find_by_reference(&path_str)?;
                if !result.matches.is_empty() {
                    has_matches = true;
                }
                results.push(result);
            }

            output::print_find(cli.output, &results)?;

            // Exit code 0 if any matches found, 1 if no matches
            Ok(i32::from(!has_matches))
        }
    }
}

/// Map exit codes for different scenarios
#[must_use]
pub fn map_exit_code(success: bool, error: Option<&crate::error::ContextError>) -> i32 {
    if success {
        return 0;
    }

    match error {
        Some(crate::error::ContextError::NotARepository) => 128,
        Some(crate::error::ContextError::NotInitialized(_)) => 3,
        _ => 1,
    }
}
