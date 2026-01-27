//! Integration tests for the sync command

use context::core::document::Document;
use context::core::Cache;
use std::fs;
use tempfile::TempDir;

/// Set up a test project with a .context directory
fn setup_project() -> TempDir {
    let dir = TempDir::new().unwrap();

    // Create project structure
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("src/lib.rs"), "// lib").unwrap();

    // Create .context directory
    fs::create_dir_all(dir.path().join(".context/guides")).unwrap();
    fs::create_dir_all(dir.path().join(".context/references")).unwrap();

    dir
}

#[test]
fn test_sync_valid_references() {
    let dir = setup_project();

    // Create a document with valid references in the body
    let doc_content = r#"---
slug: main
description: ""
references: {}
updated: ""
---

# Main Module

The entry point is in `src/main.rs`.
"#;
    let doc_path = dir.path().join(".context/guides/main.md");
    fs::write(&doc_path, doc_content).unwrap();

    // Load and sync
    let mut doc = Document::load(&doc_path).unwrap();
    doc.sync().unwrap();

    // Verify the references were updated
    assert!(doc.references.contains_key("src/main.rs"));
    assert!(!doc.updated.is_empty());

    // Verify the file was saved correctly
    let reloaded = Document::load(&doc_path).unwrap();
    assert!(reloaded.references.contains_key("src/main.rs"));
}

#[test]
fn test_sync_missing_file_fails() {
    let dir = setup_project();

    // Create a document referencing a non-existent file
    let doc_content = r#"---
slug: missing
description: ""
references: {}
updated: ""
---

# Missing

See `src/nonexistent.rs` for details.
"#;
    let doc_path = dir.path().join(".context/guides/missing.md");
    fs::write(&doc_path, doc_content).unwrap();

    // Load and try to sync - should fail
    let mut doc = Document::load(&doc_path).unwrap();
    let result = doc.sync();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid references"));
}

#[test]
fn test_sync_parent_traversal_fails() {
    let dir = setup_project();

    // Create a document with parent traversal path
    let doc_content = r#"---
slug: escape
description: ""
references: {}
updated: ""
---

# Escape Attempt

This references `../outside.rs`.
"#;
    let doc_path = dir.path().join(".context/guides/escape.md");
    fs::write(&doc_path, doc_content).unwrap();

    // Load and try to sync - should fail
    let mut doc = Document::load(&doc_path).unwrap();
    let result = doc.sync();

    assert!(result.is_err());
}

#[test]
fn test_sync_no_frontmatter_creates_default() {
    let dir = setup_project();

    // Create a document without frontmatter
    let doc_content = r#"# No Frontmatter

This document references `src/main.rs`.
"#;
    let doc_path = dir.path().join(".context/guides/nofm.md");
    fs::write(&doc_path, doc_content).unwrap();

    // Load and sync
    let mut doc = Document::load(&doc_path).unwrap();
    assert_eq!(doc.slug, "nofm");
    assert!(doc.description.is_empty());

    doc.sync().unwrap();

    // Verify frontmatter was generated
    assert!(doc.references.contains_key("src/main.rs"));
    assert!(!doc.updated.is_empty());

    // Reload and verify persistence
    let reloaded = Document::load(&doc_path).unwrap();
    assert_eq!(reloaded.slug, "nofm");
    assert!(reloaded.references.contains_key("src/main.rs"));
}

#[test]
fn test_sync_empty_body_clears_references() {
    let dir = setup_project();

    // Create a document with no paths in the body
    let doc_content = r#"---
slug: empty
description: "An empty document"
references:
  src/main.rs: abc1234
updated: "2025-01-01"
---

# Empty

No file references here.
"#;
    let doc_path = dir.path().join(".context/guides/empty.md");
    fs::write(&doc_path, doc_content).unwrap();

    // Load and sync
    let mut doc = Document::load(&doc_path).unwrap();
    assert!(!doc.references.is_empty()); // Has existing references

    doc.sync().unwrap();

    // References should now be empty
    assert!(doc.references.is_empty());
}

#[test]
fn test_cache_sync_atomic_failure() {
    let dir = setup_project();

    // Initialize the cache
    let context_dir = dir.path().join(".context");

    // Create two documents - one valid, one invalid
    let valid_content = r#"---
slug: valid
description: ""
references: {}
updated: ""
---

Uses `src/main.rs`.
"#;
    fs::write(context_dir.join("guides/valid.md"), valid_content).unwrap();

    let invalid_content = r#"---
slug: invalid
description: ""
references: {}
updated: ""
---

Uses `src/missing.rs`.
"#;
    fs::write(context_dir.join("guides/invalid.md"), invalid_content).unwrap();

    // Load cache
    let mut cache = Cache::create(context_dir).unwrap();
    cache.load().unwrap();

    // Sync should fail
    let result = cache.sync(None);
    assert!(result.is_err());

    // Verify valid document was NOT modified (atomic failure)
    let valid_doc = Document::load(dir.path().join(".context/guides/valid.md")).unwrap();
    assert!(valid_doc.references.is_empty()); // Should still be empty
}

#[test]
fn test_sync_deduplicates_references() {
    let dir = setup_project();

    // Create a document that mentions the same file multiple times
    let doc_content = r#"---
slug: dedup
description: ""
references: {}
updated: ""
---

# Deduplication

First mention: `src/main.rs`
Second mention: `src/main.rs`
Third mention: `./src/main.rs`
"#;
    let doc_path = dir.path().join(".context/guides/dedup.md");
    fs::write(&doc_path, doc_content).unwrap();

    // Load and sync
    let mut doc = Document::load(&doc_path).unwrap();
    doc.sync().unwrap();

    // Should have exactly one reference
    assert_eq!(doc.references.len(), 1);
    assert!(doc.references.contains_key("src/main.rs"));
}

#[test]
fn test_sync_ignores_code_blocks() {
    let dir = setup_project();

    // Create a document with paths inside code blocks
    let doc_content = r#"---
slug: codeblock
description: ""
references: {}
updated: ""
---

# Code Block Test

Real reference: `src/main.rs`

```rust
// This should be ignored
let path = `src/nonexistent.rs`;
```

```
`another/fake/path.rs`
```
"#;
    let doc_path = dir.path().join(".context/guides/codeblock.md");
    fs::write(&doc_path, doc_content).unwrap();

    // Load and sync
    let mut doc = Document::load(&doc_path).unwrap();
    doc.sync().unwrap();

    // Should only have the real reference, not the ones in code blocks
    assert_eq!(doc.references.len(), 1);
    assert!(doc.references.contains_key("src/main.rs"));
}

#[test]
fn test_sync_multiple_valid_references() {
    let dir = setup_project();

    // Create a document with multiple valid references
    let doc_content = r#"---
slug: multi
description: ""
references: {}
updated: ""
---

# Multiple References

See `src/main.rs` for the entry point.
The library code is in `src/lib.rs`.
"#;
    let doc_path = dir.path().join(".context/guides/multi.md");
    fs::write(&doc_path, doc_content).unwrap();

    // Load and sync
    let mut doc = Document::load(&doc_path).unwrap();
    doc.sync().unwrap();

    // Should have both references
    assert_eq!(doc.references.len(), 2);
    assert!(doc.references.contains_key("src/main.rs"));
    assert!(doc.references.contains_key("src/lib.rs"));
}
