use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::{self, EnvFilter};

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ServerHandler,
};

use crate::core::{find_context_root_from_cwd, Cache, FindResult, Status, SyncResult, Validation};
use crate::error::ContextError;

// ============================================================================
// Request types for MCP tools
// ============================================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StatusRequest {
    #[schemars(description = "If true, only return stale or orphaned documents")]
    pub invalid_only: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SyncRequest {
    #[schemars(description = "Path to a specific document to sync. If omitted, syncs all documents.")]
    pub path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindRequest {
    #[schemars(description = "Source file paths to search for (e.g., [\"src/core/models.rs\"])")]
    pub paths: Vec<String>,
}

// ============================================================================
// Response types for MCP tools
// ============================================================================

#[derive(Debug, serde::Serialize)]
struct StatusItem {
    path: String,
    status: String,
    changed: Vec<String>,
    missing: Vec<String>,
}

impl From<Validation> for StatusItem {
    fn from(v: Validation) -> Self {
        Self {
            path: v.path.display().to_string(),
            status: v.status.to_string(),
            changed: v.changed,
            missing: v.missing,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct SyncResponse {
    count: usize,
    updated: Vec<String>,
    failed: Vec<String>,
}

impl From<SyncResult> for SyncResponse {
    fn from(r: SyncResult) -> Self {
        Self {
            count: r.count,
            updated: r.updated.iter().map(|p| p.display().to_string()).collect(),
            failed: r.failed,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct FindMatchItem {
    document: String,
    reference: String,
    status: String,
}

#[derive(Debug, serde::Serialize)]
struct FindResultItem {
    query: String,
    matches: Vec<FindMatchItem>,
}

impl From<FindResult> for FindResultItem {
    fn from(r: FindResult) -> Self {
        Self {
            query: r.query,
            matches: r
                .matches
                .into_iter()
                .map(|m| FindMatchItem {
                    document: m.document.display().to_string(),
                    reference: m.reference,
                    status: m.status.to_string(),
                })
                .collect(),
        }
    }
}

// ============================================================================
// MCP Server implementation
// ============================================================================

#[derive(Debug, Clone)]
pub struct ContextServer {
    tool_router: ToolRouter<Self>,
}

impl ContextServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Load the cache from the current working directory
    fn load_cache() -> std::result::Result<Cache, String> {
        let root = find_context_root_from_cwd().map_err(|e| match e {
            ContextError::NotARepository => {
                "Not a context repository (no .context directory found)".to_string()
            }
            _ => format!("Failed to find context root: {e}"),
        })?;

        let mut cache = Cache::create(root).map_err(|e| format!("Failed to create cache: {e}"))?;
        cache
            .load()
            .map_err(|e| format!("Failed to load cache: {e}"))?;

        Ok(cache)
    }
}

impl Default for ContextServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl ContextServer {
    #[tool(description = "Validate all context documents and return their status (valid, stale, or orphaned)")]
    #[allow(clippy::unused_self)]
    fn context_status(&self, Parameters(req): Parameters<StatusRequest>) -> String {
        let cache = match Self::load_cache() {
            Ok(c) => c,
            Err(e) => return format!("Error: {e}"),
        };

        let validations = match cache.status() {
            Ok(v) => v,
            Err(e) => return format!("Error: {e}"),
        };

        let invalid_only = req.invalid_only.unwrap_or(false);

        let items: Vec<StatusItem> = validations
            .into_iter()
            .filter(|v| !invalid_only || v.status != Status::Valid)
            .map(StatusItem::from)
            .collect();

        match serde_json::to_string_pretty(&items) {
            Ok(json) => json,
            Err(e) => format!("Error serializing response: {e}"),
        }
    }

    #[tool(description = "Update reference hashes for context documents, marking them as reviewed")]
    #[allow(clippy::unused_self)]
    fn context_sync(&self, Parameters(req): Parameters<SyncRequest>) -> String {
        let mut cache = match Self::load_cache() {
            Ok(c) => c,
            Err(e) => return format!("Error: {e}"),
        };

        let doc_path = match &req.path {
            Some(p) => {
                let path = std::path::Path::new(p);
                match cache.resolve_doc_path(path) {
                    Ok(resolved) => Some(resolved),
                    Err(e) => return format!("Error: {e}"),
                }
            }
            None => None,
        };

        let result = match cache.sync(doc_path.as_deref()) {
            Ok(r) => r,
            Err(ContextError::InvalidReferences { count, documents }) => {
                // Format a detailed error message for invalid references
                use std::fmt::Write;
                let mut msg = format!("Error: Invalid references in {count} document(s):\n");
                for (doc_path, refs) in documents {
                    let _ = write!(msg, "\n{}:\n", doc_path.display());
                    for r in refs {
                        let _ = writeln!(msg, "  - {}: {}", r.path, r.reason);
                    }
                }
                return msg;
            }
            Err(e) => return format!("Error: {e}"),
        };

        let response = SyncResponse::from(result);
        match serde_json::to_string_pretty(&response) {
            Ok(json) => json,
            Err(e) => format!("Error serializing response: {e}"),
        }
    }

    #[tool(description = "Find all context documents that reference the given source file path(s)")]
    #[allow(clippy::unused_self)]
    fn context_find(&self, Parameters(req): Parameters<FindRequest>) -> String {
        let cache = match Self::load_cache() {
            Ok(c) => c,
            Err(e) => return format!("Error: {e}"),
        };

        let mut results: Vec<FindResultItem> = Vec::new();

        for path in &req.paths {
            match cache.find_by_reference(path) {
                Ok(r) => results.push(FindResultItem::from(r)),
                Err(e) => return format!("Error searching for '{path}': {e}"),
            }
        }

        match serde_json::to_string_pretty(&results) {
            Ok(json) => json,
            Err(e) => format!("Error serializing response: {e}"),
        }
    }
}

#[tool_handler]
impl ServerHandler for ContextServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Context documentation cache server. Use context_status to check document validity, \
                 context_find to locate documents referencing source files, and context_sync to \
                 update hashes after reviewing documentation."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// Start the Context MCP server over stdio
pub async fn run_server() -> Result<()> {
    // Initialize the tracing subscriber with stderr logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Context MCP server");

    let service = ContextServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
