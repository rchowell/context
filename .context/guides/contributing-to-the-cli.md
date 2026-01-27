---
slug: contributing-to-the-cli
description: This document is a short guide for how to contribute to the CLI. It should be used for any changes to related to CLI commands or configuration.
references:
  src/cli/mod.rs: ad6283a
  src/cli/args.rs: 4de433c
  src/cli/commands.rs: d55bacd
  src/cli/console.rs: 4fa60ca
updated: 2026-01-27
hash: 469f548
---

# CLI Architecture

The CLI module provides a clean async architecture for command handling using clap and tokio.

## Structure

```
src/
├── main.rs           # Async entry point with #[tokio::main]
└── cli/
    ├── mod.rs        # Public exports
    ├── args.rs       # Command definitions and argument types
    ├── commands.rs   # Async command handlers
    └── console.rs    # Output formatting (text/JSON)
```

## Command Definition Pattern

Each command has a dedicated args struct and a corresponding async handler:

```rust
// In args.rs
#[derive(Args)]
pub struct InitArgs {
    pub path: PathBuf,
    pub create: bool,
}

pub enum Commands {
    Init(InitArgs),
    // ...
}

// In commands.rs
async fn init(args: InitArgs) -> Result<i32> {
    // Implementation
}
```

## Adding a New Command

1. **Define args struct** in `src/cli/args.rs`:
   ```rust
   #[derive(Args, Debug)]
   pub struct MyCommandArgs {
       #[arg(short, long)]
       pub flag: bool,
   }
   ```

2. **Add enum variant** to `Commands`:
   ```rust
   #[command(about = "Description")]
   MyCommand(MyCommandArgs),
   ```

3. **Implement handler** in `src/cli/commands.rs`:
   ```rust
   async fn my_command(args: MyCommandArgs, output: OutputFormat) -> Result<i32> {
       // Implementation
       Ok(0)
   }
   ```

4. **Add dispatch case** in `execute()`:
   ```rust
   Commands::MyCommand(args) => my_command(args, cli.output).await,
   ```

5. **Export args type** in `src/cli/mod.rs` if needed externally.

## Design Decisions

- **Async throughout**: All handlers are async for consistency, even if some don't currently await
- **Typed arguments**: Each command has its own args struct for type safety and extensibility
- **Thin dispatcher**: The `execute()` function is a simple match that delegates to handlers
- **Exit codes**: Handlers return `Result<i32>` where the i32 is the process exit code
- **Output format**: Commands that produce output receive `OutputFormat` to support text/JSON modes

## Output Handling

Commands use `src/cli/console.rs` for formatted output. The `OutputFormat` enum supports:
- `Text` - Human-readable output
- `Json` - Machine-parseable JSON

Pass the format to print functions: `console::print_status(format, &data)?`
