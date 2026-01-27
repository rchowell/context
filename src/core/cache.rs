use crate::core::document::Document;
use crate::core::models::{FindResult, SearchResult, SyncResult, Validation};
use crate::error::Result;
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

    /// Search for documents matching the given query
    pub fn search(&self, _query: &str) -> Result<Vec<SearchResult>> {
        // Deferred to later implementation
        Ok(Vec::new())
    }

    /// Find documents that reference the given source files
    pub fn find(&self, _references: &[&str]) -> Result<Vec<FindResult>> {
        // Deferred to later implementation
        Ok(Vec::new())
    }

    /// Validate a single document or all documents
    pub fn validate(&self, path: Option<&Path>) -> Result<Vec<Validation>> {
        match path {
            Some(p) => {
                // Validate specific document
                for doc in &self.documents {
                    if doc.path == p {
                        return Ok(vec![doc.validate()?]);
                    }
                }
                Err(crate::error::ContextError::DocumentNotFound(
                    p.display().to_string(),
                ))
            }
            None => self.status(),
        }
    }

    /// Sync (update hashes) for all or a specific document
    pub fn sync(&mut self, doc_path: Option<&Path>) -> Result<SyncResult> {
        let mut result = SyncResult::new();

        match doc_path {
            Some(p) => {
                // Sync specific document
                for doc in &mut self.documents {
                    if doc.path == p {
                        match doc.sync() {
                            Ok(()) => {
                                result.count += 1;
                                result.updated.push(doc.path.clone());
                            }
                            Err(e) => {
                                result.failed.push(format!("{}: {}", doc.path.display(), e));
                            }
                        }
                        break;
                    }
                }
            }
            None => {
                // Sync all documents
                for doc in &mut self.documents {
                    match doc.sync() {
                        Ok(()) => {
                            result.count += 1;
                            result.updated.push(doc.path.clone());
                        }
                        Err(e) => {
                            result.failed.push(format!("{}: {}", doc.path.display(), e));
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}
