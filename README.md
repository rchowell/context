# Context

Context is a documentation cache system for coding agents. It maintains project
knowledge as markdown files with automatic invalidation when source files
change. It can be used as a CLI tool or MCP server.

## Install

```sh
brew install rchowell/tap/context
```

## Usage

Documentation is stored in `.context/` as markdown files. You (and agents) can view and edit with any tools. You can use `context sync` to mark documentation
as fresh, and you can use `context status` to get a list of stale documents. This information can be fed to agents (and you) to determine which documentation
should be updated.

```sh
# Initialize .context/ within the project root
context init
```

**Claude Code**

```sh
# Adds a 'context' MCP server with stdio
claude mcp add --transport stdio --scope project context context serve

# Add the skill (optional, in-progress)
.claude/skills/using-documentation/SKILL.md
```

**via @**

```sh
# It's just markdown
You can do @.context/guides/doc.md
```

**via MCP**

```sh
# MCP via stdio protocol
context serve
```

**via CLI**

```sh
# You can use it manually or within a bash tool call
context --help
```

| Command               | Purpose                               |
|-----------------------|---------------------------------------|
| `context init [dir]`  | Scaffold directory structure          |
| `context status`      | Report valid/stale/orphaned docs      |
| `context sync [path]` | Update hashes, mark as reviewed       |
| `context find [path]` | Find all references to the given path |


## How It Works

Each project gets a root `.context` directory somewhat like the `.git` directory.

```sh
context init  # initialiazes .context/
```

Author markdown documentation within `./context` and reference other files in backticks. Here is an example doc:

```
---
slug: auth
description: Authentication and JWT handling
---

The auth system lives in `src/auth/mod.rs` and `src/auth/jwt.rs`.
```

Then run `context sync` to generate all reference hashes in the frontmatter.

```yaml
references:
  src/auth/mod.rs: 8a3b2c1
  src/auth/jwt.rs: f4e5d6a
```

As you write code, the documentation can become stale (bad) — you can find
invalidated documents by doing `context status`, ex:

```sh
context status

modified:  .context/guides/auth.md
```

## Directory Structure

The idea here is that an index is like a layered cache, and agents should
try to read index > guides > references. 

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

* **Guides**: explain concepts, workflows, and architecture. They have broad references and invalidate slowly.
* **References**: document specific modules and components. They have narrow references and invalidate quickly.
* **Index files**: aggregate references from their children, invalidating when any child document's dependencies change.

## Release

I will be manually releasing prebuilt Apple Silicon binaries for early versions (~1MB).

```sh
# Define the version and tag
export VERSION="0.1.0"
export TAG="v$VERSION"

# Build a release
cargo build --release --target aarch64-apple-darwin

# Bundle as a tar.gz
tar -czvf context-$VERSION-aarch64-apple-darwin.tar.gz -C target/aarch64-apple-darwin/release context

# Upload the release
gh release create $TAG context-$VERSION-aarch64-apple-darwin.tar.gz

# Compute the new sha245
shasum -a 256 context-$VERSION-*.tar.gz
```