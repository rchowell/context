# Context

Context is a documentation cache system for AI-assisted development. It maintains project knowledge as markdown files with automatic invalidation when source files change.

## What context does.

- 

## What context does not do.

- Author documentation.


## Problem

AI coding assistants spend significant tokens exploring codebases repeatedly. Project documentation is often outdated or missing, compounding the problem. Context solves this by treating documentation as a cache layer with explicit dependencies on source files.

## How It Works

Documentation files declare their source dependencies in YAML frontmatter:

```yaml
---
slug: auth
description: Authentication system and JWT handling
references:
  src/auth/mod.rs: 8a3b2c1
  src/auth/jwt.rs: f4e5d6a
updated: 2025-01-21
---
```

When referenced files change, the documentation becomes **stale**. When referenced files are deleted, the documentation becomes **orphaned**. The `context status` command detects both conditions.

## Directory Structure

```
context/
├── index.md                # Project overview
├── guides/
│   ├── index.md            # Guide listing
│   └── {topic}.md          # Conceptual docs
└── references/
    ├── index.md            # Reference listing  
    └── {topic}.md          # Technical docs
```

**Guides** explain concepts, workflows, and architecture. They have broad references and invalidate slowly.

**References** document specific modules and components. They have narrow references and invalidate quickly.

**Index files** aggregate references from their children, invalidating when any child document's dependencies change.

## Cache Invalidation

Each reference maps a file path to a content hash. The cache is valid when all hashes match current file contents.

```
references:
  src/auth/mod.rs: 8a3b2c1   # hash of file contents
```

`context status` compares stored hashes against current files:
- **Valid**: all hashes match
- **Stale**: one or more files changed
- **Orphaned**: one or more files deleted

`context sync` recomputes hashes without changing content, marking documentation as reviewed.

## Session Integration

**Pre-session**: Run `context status` to identify stale docs. Inject this into the AI's context so it knows which documentation is untrusted.

**Post-session**: Run `context find` on modified files to identify affected documentation. The AI can review and update these docs before the session ends.

## Commands

| Command | Purpose |
|---------|---------|
| `context init [dir]` | Scaffold directory structure |
| `context status` | Report valid/stale/orphaned docs |
| `context search <term>` | Full-text search across docs |
| `context find <files...>` | Find docs referencing given files |
| `context validate <path>` | Check single doc validity |
| `context sync [path]` | Update hashes, mark as reviewed |

All commands support `-j, --json` for structured output.

## Design Principles

1. **Human-readable**: Standard markdown, familiar structure, no proprietary formats
2. **Self-describing**: Each file contains its own dependency metadata
3. **Git-friendly**: Distributed frontmatter diffs cleanly
4. **Explicit invalidation**: Dependencies declared, not inferred
5. **Layered detail**: Guides for concepts, references for specifics, source for implementation
