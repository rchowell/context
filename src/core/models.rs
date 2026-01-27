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

/// A single match from a find operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindMatch {
    /// Path to the document that contains the reference
    pub document: PathBuf,
    /// The reference path as stored in the document
    pub reference: String,
    /// Validation status of the document
    pub status: Status,
}

/// Result of a find operation for a single query path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindResult {
    /// The source file path that was queried
    pub query: String,
    /// Documents that reference this file
    pub matches: Vec<FindMatch>,
}
