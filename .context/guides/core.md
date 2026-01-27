---
slug: core
description: ''
references:
  src/core/document.rs: fd7d7d8
  src/core/models.rs: f66b4da
  src/core/cache.rs: 154d6cb
updated: 2026-01-27
hash: aedfac4
---

# Core Module

The core module provides the foundational types and operations for managing context documentation.

## Components

- **`discovery`**: Locates the `.context` directory by walking up from a given path
- **`frontmatter`**: Parses and serializes YAML frontmatter from markdown documents
- **`document`**: Represents a single document with validation and sync operations
- **`cache`**: Manages the entire context directory, loading and operating on all documents
- **`models`**: Shared data structures (Status, Validation, SearchResult, etc.)

## Key Patterns

- Documents declare source dependencies via `references` in frontmatter (file path → content hash)
- Validation compares stored hashes against current file contents
- Sync updates hashes without changing document content
- All paths in references are relative to the project root (parent of `.context`)

## Adding Features

When extending core functionality:
- Keep document operations in `src/core/document.rs`
- Cache-level operations belong in `src/core/cache.rs`
- Use `Result<T>` from `crate::error` for error handling
- Add corresponding models to `src/core/models.rs` if needed
