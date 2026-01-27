use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;


/// Validity status of a document relative to its source file references
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// All referenced files exist and hashes match
    Valid,
    /// One or more referenced files have changed (hash mismatch)
    Stale,
    /// One or more referenced files no longer exist
    Orphaned,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid => write!(f, "valid"),
            Self::Stale => write!(f, "stale"),
            Self::Orphaned => write!(f, "orphaned"),
        }
    }
}

/// Status information for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    /// Path to the document file
    pub path: PathBuf,
    /// Validity status
    pub status: Status,
    /// Files that changed (hash mismatch)
    pub changed: Vec<String>,
    /// Files that are missing
    pub missing: Vec<String>,
}

impl Validation {
    /// Create a new DocumentStatus
    pub fn new(path: PathBuf, status: Status) -> Self {
        Self {
            path,
            status,
            changed: vec![],
            missing: vec![],
        }
    }

    /// Add a changed file
    pub fn add_changed(&mut self, file: String) {
        self.changed.push(file);
    }

    /// Add a missing file
    pub fn add_missing(&mut self, file: String) {
        self.missing.push(file);
    }
}

/// Search result for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Path to the document
    pub path: PathBuf,

    /// Document description
    pub description: String,

    /// Matched text snippet (if available)
    pub snippet: Option<String>,
}

impl SearchResult {
    /// Create a new SearchResult
    pub fn new(path: PathBuf, description: String, snippet: Option<String>) -> Self {
        Self {
            path,
            description,
            snippet,
        }
    }
}

/// Find result for a document that references given sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindResult {
    /// Path to the document
    pub path: PathBuf,
    /// Description of the document
    pub description: String,
    /// Source files this document references
    pub references: Vec<String>,
}

impl FindResult {
    /// Create a new FindResult
    pub fn new(path: PathBuf, description: String, references: Vec<String>) -> Self {
        Self {
            path,
            description,
            references,
        }
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Number of documents synced
    pub count: usize,
    /// Documents that were updated
    pub updated: Vec<PathBuf>,
    /// Documents that failed (orphaned or had errors)
    pub failed: Vec<String>,
}

impl SyncResult {
    /// Create a new SyncResult
    pub fn new() -> Self {
        Self {
            count: 0,
            updated: vec![],
            failed: vec![],
        }
    }
}

impl Default for SyncResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Frontmatter metadata for documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub slug: String,
    pub description: String,
    pub references: HashMap<String, String>,
    pub updated: String,
}
