use crate::core::models::{Validation, Status, SearchResult, FindResult, SyncResult};
use crate::error::{ContextError, InvalidReference, Result};
use serde_json::json;
use std::path::PathBuf;
use super::args::OutputFormat;

/// Print document status
pub fn print_status(format: OutputFormat, statuses: &[Validation]) -> Result<()> {
    match format {
        OutputFormat::Text => {
            for status in statuses {
                println!("{:<12} {}", status.status, status.path.display());
                if !status.changed.is_empty() {
                    println!("               changed: {}", status.changed.join(", "));
                }
                if !status.missing.is_empty() {
                    println!("               missing: {}", status.missing.join(", "));
                }
            }
        }
        OutputFormat::Json => {
            let json_statuses: Vec<_> = statuses
                .iter()
                .map(|s| {
                    json!({
                        "path": s.path.display().to_string(),
                        "status": s.status.to_string(),
                        "changed": s.changed,
                        "missing": s.missing,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_statuses)?);
        }
    }
    Ok(())
}

/// Print validation results
pub fn print_validation(format: OutputFormat, results: &[Validation]) -> Result<()> {
    match format {
        OutputFormat::Text => {
            for result in results {
                let status_str = match result.status {
                    Status::Valid => "✓ VALID",
                    Status::Stale => "⚠ STALE",
                    Status::Orphaned => "✗ ORPHANED",
                };
                println!("{} {}", status_str, result.path.display());

                if !result.changed.is_empty() {
                    println!("  Changed files:");
                    for file in &result.changed {
                        println!("    - {file}");
                    }
                }
                if !result.missing.is_empty() {
                    println!("  Missing files:");
                    for file in &result.missing {
                        println!("    - {file}");
                    }
                }
            }
        }
        OutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|r| {
                    json!({
                        "path": r.path.display().to_string(),
                        "status": r.status.to_string(),
                        "changed": r.changed,
                        "missing": r.missing,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
    }
    Ok(())
}

/// Print search results
pub fn print_search(format: OutputFormat, results: &[SearchResult]) -> Result<()> {
    match format {
        OutputFormat::Text => {
            for result in results {
                println!("{}", result.path.display());
                println!("  {}", result.description);
                if let Some(snippet) = &result.snippet {
                    println!("  {snippet}");
                }
            }
        }
        OutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|r| {
                    json!({
                        "path": r.path.display().to_string(),
                        "description": r.description,
                        "snippet": r.snippet,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
    }
    Ok(())
}

/// Print find results
pub fn print_find(format: OutputFormat, results: &[FindResult]) -> Result<()> {
    match format {
        OutputFormat::Text => {
            for result in results {
                println!("{}", result.path.display());
                println!("  {}", result.description);
                println!("  references: {}", result.references.join(", "));
            }
        }
        OutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|r| {
                    json!({
                        "path": r.path.display().to_string(),
                        "description": r.description,
                        "references": r.references,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
    }
    Ok(())
}

/// Print sync results
pub fn print_sync(format: OutputFormat, result: &SyncResult) -> Result<()> {
    match format {
        OutputFormat::Text => {
            println!("Synced {} documents", result.count);
            if !result.updated.is_empty() {
                println!("Updated:");
                for path in &result.updated {
                    println!("  {}", path.display());
                }
            }
            if !result.failed.is_empty() {
                println!("Failed:");
                for error in &result.failed {
                    println!("  {error}");
                }
            }
        }
        OutputFormat::Json => {
            let json_result = json!({
                "count": result.count,
                "updated": result.updated.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "failed": result.failed,
            });
            println!("{}", serde_json::to_string_pretty(&json_result)?);
        }
    }
    Ok(())
}

/// Format a simple message
pub fn format_message(format: OutputFormat, message: &str) -> String {
    match format {
        OutputFormat::Text => message.to_string(),
        OutputFormat::Json => serde_json::to_string(&json!({"message": message})).unwrap_or_default(),
    }
}

/// Format an error message
pub fn format_error(format: OutputFormat, error: &str) -> String {
    match format {
        OutputFormat::Text => format!("Error: {error}"),
        OutputFormat::Json => serde_json::to_string(&json!({"error": error})).unwrap_or_default(),
    }
}

/// Print invalid references error
pub fn print_invalid_references(
    format: OutputFormat,
    documents: &[(PathBuf, Vec<InvalidReference>)],
) -> Result<()> {
    match format {
        OutputFormat::Text => {
            eprintln!(
                "Error: Invalid references in {} document(s)",
                documents.len()
            );
            eprintln!();
            for (doc_path, invalid_refs) in documents {
                eprintln!("  {}", doc_path.display());
                for inv in invalid_refs {
                    eprintln!("    - `{}`: {}", inv.path, inv.reason);
                }
            }
        }
        OutputFormat::Json => {
            let json_docs: Vec<_> = documents
                .iter()
                .map(|(path, refs)| {
                    json!({
                        "document": path.display().to_string(),
                        "invalid": refs.iter().map(|r| {
                            json!({
                                "path": r.path,
                                "reason": r.reason.to_string(),
                            })
                        }).collect::<Vec<_>>(),
                    })
                })
                .collect();
            let output = json!({
                "error": "invalid_references",
                "count": documents.len(),
                "documents": json_docs,
            });
            eprintln!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}

/// Handle a ContextError, printing appropriate output
pub fn handle_error(format: OutputFormat, error: &ContextError) -> Result<()> {
    if let ContextError::InvalidReferences { documents, .. } = error {
        print_invalid_references(format, documents)
    } else {
        let msg = format_error(format, &error.to_string());
        eprintln!("{msg}");
        Ok(())
    }
}
