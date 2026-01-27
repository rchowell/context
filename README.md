# Context

Context is a documentation cache system for coding agents. It maintains project
knowledge as markdown files with automatic invalidation when source files
change. 

## Problem

Each coding agent session is a new contributor to your project.
These agents waste time and tokens exploring entire source code files and
navigating incomplete, outdated, or missing documentation. Context addresses
this by currating agent-oriented documentation as a cache-like layer for 
the disovery phase.

## How It Works

Each project gets a `.context` directory via `context init` — humans or agents
then author markdown documentation in this directory. 

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

**Example**

```sh
context status
valid /Users/rch/Projects/Context/.context/references/index.md
valid /Users/rch/Projects/Context/.context/index.md
valid /Users/rch/Projects/Context/.context/guides/index.md
stale /Users/rch/Projects/Context/.context/guides/core.md
               changed: src/core/models.rs
```

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

| Command               | Purpose                               |
|-----------------------|---------------------------------------|
| `context init [dir]`  | Scaffold directory structure          |
| `context status`      | Report valid/stale/orphaned docs      |
| `context sync [path]` | Update hashes, mark as reviewed       |
| `context find [path]` | Find all references to the given path |

All commands support `-j, --json` for structured output.

## Install

**Development Install*

```sh
claude mcp add --transport stdio --scope project context ./target/debug/context serve
```
