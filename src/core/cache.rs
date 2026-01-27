use crate::core::document::Document;
use crate::core::models::{SyncResult, Validation};
use crate::error::{ContextError, InvalidReference, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Create index.md files with empty frontmatter template
const INDEX_TEMPLATE: &str = r#"---
slug: index
description: ""
references: {}
updated: ""
---

"#;

/// Cache for managing context documentation
#[derive(Debug, Clone)]
pub struct Cache {
    /// Root directory (the context/ folder)
    root: PathBuf,
    /// Root index file, ./context/index.md
    index: Option<Document>,
    /// Guides index file, ./context/guides/index.md
    guides: Option<Document>,
    /// References index file, ./context/references/index.md
    references: Option<Document>,
    /// All documents in the cache
    documents: Vec<Document>,
}

impl Cache {
    /// Create a new Cache for the given context directory
    pub fn create(root: PathBuf) -> Result<Self> {
        Ok(Self {
            root,
            index: None,
            guides: None,
            references: None,
            documents: Vec::new(),
        })
    }

    /// Initialize a new context directory with template index files
    pub fn init(root: PathBuf) -> Result<Self> {
        // Create directory structure
        std::fs::create_dir_all(&root)?;
        std::fs::create_dir_all(root.join("guides"))?;
        std::fs::create_dir_all(root.join("references"))?;

        // Write template index files
        std::fs::write(root.join("index.md"), INDEX_TEMPLATE)?;
        std::fs::write(root.join("guides/index.md"), INDEX_TEMPLATE)?;
        std::fs::write(root.join("references/index.md"), INDEX_TEMPLATE)?;

        Self::create(root)
    }

    /// Load all documents from the cache directory
    pub fn load(&mut self) -> Result<()> {
        self.documents.clear();

        // Walk the context directory and find all .md files
        for entry in WalkDir::new(&self.root)
            .follow_links(true)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                let doc = Document::load(path)?;

                // Track special index files
                if path == self.root.join("index.md") {
                    self.index = Some(doc.clone());
                } else if path == self.root.join("guides/index.md") {
                    self.guides = Some(doc.clone());
                } else if path == self.root.join("references/index.md") {
                    self.references = Some(doc.clone());
                }

                self.documents.push(doc);
            }
        }

        Ok(())
    }

    /// Check the validity status of all documents
    pub fn status(&self) -> Result<Vec<Validation>> {
        let mut results = Vec::new();
        for doc in &self.documents {
            results.push(doc.validate()?);
        }
        Ok(results)
    }

    /// Sync (update hashes) for all or a specific document.
    ///
    /// This uses a two-phase approach for atomicity:
    /// 1. Validate all documents first, collecting any invalid references
    /// 2. Only if all documents are valid, write changes to all of them
    ///
    /// If any document has invalid references, no documents are modified.
    pub fn sync(&mut self, doc_path: Option<&Path>) -> Result<SyncResult> {
        // Determine which documents to sync
        let doc_indices: Vec<usize> = match doc_path {
            Some(p) => self
                .documents
                .iter()
                .enumerate()
                .filter(|(_, doc)| doc.path == p)
                .map(|(i, _)| i)
                .collect(),
            None => (0..self.documents.len()).collect(),
        };

        // Phase 1: Validate all documents, collect all errors
        let mut all_invalid: Vec<(PathBuf, Vec<InvalidReference>)> = Vec::new();

        for &idx in &doc_indices {
            let doc = &self.documents[idx];
            let invalid = doc.prepare_sync();
            if !invalid.is_empty() {
                all_invalid.push((doc.path.clone(), invalid));
            }
        }

        // If any documents have invalid references, fail the entire sync
        if !all_invalid.is_empty() {
            return Err(ContextError::InvalidReferences {
                count: all_invalid.len(),
                documents: all_invalid,
            });
        }

        // Phase 2: All documents valid, perform the actual sync
        let mut result = SyncResult::new();

        for &idx in &doc_indices {
            let doc = &mut self.documents[idx];
            match doc.sync() {
                Ok(()) => {
                    result.count += 1;
                    result.updated.push(doc.path.clone());
                }
                Err(e) => {
                    // This shouldn't happen since we validated, but handle it gracefully
                    result.failed.push(format!("{}: {}", doc.path.display(), e));
                }
            }
        }

        Ok(result)
    }
}
