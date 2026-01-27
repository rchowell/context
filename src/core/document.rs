use crate::core::frontmatter;
use crate::core::models::{Status, Validation};
use crate::error::Result;
use chrono::Local;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, path::{Path, PathBuf}};

/// A document in the context cache
#[derive(Debug, Clone)]
pub struct Document {
    /// File path of this document within the context directory
    pub path: PathBuf,
    /// Identifier from frontmatter, matches filename without extension
    pub slug: String,
    /// Brief summary of the document
    pub description: String,
    /// Map of source file paths to their content hashes (short SHA)
    pub references: HashMap<String, String>,
    /// Last update date (ISO 8601 format: YYYY-MM-DD)
    pub updated: String,
    /// Document body content (after frontmatter)
    pub body: String,
}

impl Document {
    /// Create a new Document
    pub fn new(
        path: PathBuf,
        slug: String,
        description: String,
        references: HashMap<String, String>,
        updated: String,
        body: String,
    ) -> Self {
        Self {
            path,
            slug,
            description,
            references,
            updated,
            body,
        }
    }
}

impl Document {
    /// Load a document from the given path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        frontmatter::parse(path.to_path_buf(), &content)
    }

    /// Save the document to disk
    pub fn save(&self) -> Result<()> {
        let content = frontmatter::serialize(self)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    /// Get the project root directory (parent of context/)
    fn project_root(&self) -> Option<PathBuf> {
        // Walk up the path to find the "context" directory
        let mut current = self.path.parent();
        while let Some(dir) = current {
            if dir.file_name().is_some_and(|n| n == "context") {
                return dir.parent().map(Path::to_path_buf);
            }
            current = dir.parent();
        }
        None
    }

    /// Resolve a reference path relative to the project root
    fn resolve_ref_path(&self, ref_path: &str) -> PathBuf {
        if let Some(root) = self.project_root() {
            root.join(ref_path)
        } else {
            PathBuf::from(ref_path)
        }
    }

    /// Updates the document's references and saves it to disk
    pub fn sync(&mut self) -> Result<()> {
        // Compute new hashes for all references
        let project_root = self.project_root();
        let mut new_hashes: HashMap<String, String> = HashMap::new();

        for (ref_path, stored_hash) in &self.references {
            let resolved_path = if let Some(ref root) = project_root {
                root.join(ref_path)
            } else {
                PathBuf::from(ref_path)
            };

            if resolved_path.exists() {
                let content = std::fs::read(&resolved_path)?;
                let new_hash = hash(&content);
                new_hashes.insert(ref_path.clone(), new_hash);
            } else {
                // Keep old hash if file doesn't exist (will be detected as orphaned)
                new_hashes.insert(ref_path.clone(), stored_hash.clone());
            }
        }

        self.references = new_hashes;

        // Update the updated date
        self.updated = Local::now().format("%Y-%m-%d").to_string();

        // Save to disk
        self.save()
    }

    /// Validate the document's references
    pub fn validate(&self) -> Result<Validation> {
        let mut validation = Validation::new(self.path.clone(), Status::Valid);

        for (ref_path, stored_hash) in &self.references {
            let resolved_path = self.resolve_ref_path(ref_path);

            if resolved_path.exists() {
                let content = std::fs::read(&resolved_path)?;
                let current_hash = hash(&content);

                if current_hash != *stored_hash {
                    validation.add_changed(ref_path.clone());
                    if validation.status != Status::Orphaned {
                        validation.status = Status::Stale;
                    }
                }
            } else {
                validation.add_missing(ref_path.clone());
                validation.status = Status::Orphaned;
            }
        }

        Ok(validation)
    }
}


/// Compute SHA-256 hash of content, returning the first 7 characters of the hash
fn hash(content: &[u8]) -> String {
    let hash = Sha256::digest(content);
    format!("{hash:x}")[..7].to_string()
}
