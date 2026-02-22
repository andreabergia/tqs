# TQS Architecture

## Overview

TQS is a simple Rust CLI application that manages tasks as Markdown files. The codebase is organized into clear layers: CLI parsing, command handlers, domain logic, storage, and I/O.

## Module Organization

```
src/
├── main.rs          # Entry point
├── lib.rs           # Library exports
├── cli/             # Command-line interface
│   ├── args.rs      # Clap CLI definitions
│   ├── commands/    # Per-command structs
│   └── handlers.rs  # Command routing
├── app/             # Application layer
│   ├── service.rs   # Main app runner
│   └── app_error.rs # Error types
├── domain/          # Domain logic
│   ├── task.rs      # Task model and status
│   ├── id.rs        # ID generation
│   └── filter.rs    # Filtering logic
├── storage/         # Storage layer
│   ├── repo.rs      # Task repository
│   ├── format.rs    # Markdown serialization
│   └── root.rs      # Storage root resolution
└── io/              # Input/output
    ├── input.rs     # Interactive input
    ├── output.rs    # Formatted output
    └── picker.rs    # Fuzzy-select picker
```

## Data Flow

```
CLI (args.rs)
    ↓
Handlers (handlers.rs)
    ↓
App Service (service.rs)
    ↓
Task Repository (repo.rs)
    ↓
Markdown Files (<root>/<id>.md)
```

1. `clap` parses command-line arguments into `Cli` struct
2. `handlers::handle()` routes to the appropriate command handler
3. Command handlers call domain logic and storage operations
4. `TaskRepo` manages file I/O and serialization
5. Tasks are persisted as Markdown with YAML frontmatter

## Task Lifecycle

- Tasks are created in `Open` status
- `complete()` transitions Open → Closed
- `reopen()` transitions Closed → Open
- `delete()` removes the task file entirely

State transitions are idempotent: closing an already-closed task succeeds with a message.

## Storage Architecture

### Repository Pattern

`TaskRepo` provides CRUD operations:
- `create()` - Write new task file
- `read()` - Parse task from file
- `update()` - Modify and rewrite task file
- `delete()` - Remove task file
- `list()` - Scan directory, parse all files, sort

### Storage Root Resolution

```
Priority:
1. --root <path> flag
2. TQS_ROOT environment variable
3. <git-repo>/todos (detected via `git rev-parse --show-toplevel`)
4. ~/.tqs/todos (fallback)
```

Root resolution is in `storage/root.rs::resolve_root()`.

### File Format

Tasks use Markdown with YAML frontmatter:

```yaml
---
id: cobalt-urial-7f3a
created_at: 2026-02-20T22:15:00Z
status: open
summary: Short task summary
---

Markdown body follows...
```

Serialization is in `storage/format.rs`:
- `parse_task_markdown()` - Parse file into Task
- `render_task_markdown()` - Render Task to file

## Key Design Decisions

### File-based Storage
- Simple and version control friendly
- No database dependencies
- Easy to inspect and edit manually
- Git-friendly (stored in repo todos/ directory)

### Markdown Format
- Human-readable and editable
- Supports rich descriptions
- YAML frontmatter for structured metadata
- Skips malformed files gracefully (with warnings)

### ID Generation
- Random 4-character words (e.g., "cobalt-urial-7f3a")
- Uniqueness guaranteed via collision detection
- Easy to reference and type

### Error Handling
- Custom `AppError` type with exit codes
- Runtime errors → exit code 1
- Usage errors → exit code 2
- Warnings to stderr for recoverable issues (malformed files)

### Interactive vs Non-interactive
- Commands work in scripts (non-interactive mode)
- Interactive features require TTY
- Graceful fallback with clear error messages

## Extensibility

### Adding a New Command

1. Define struct in `cli/commands/` with `clap::Parser`
2. Implement handler function returning `Result<(), AppError>`
3. Add variant to `Command` enum in `cli/args.rs`
4. Route in `handlers.rs`

### Modifying Storage

- Add methods to `TaskRepo` in `storage/repo.rs`
- Update `Task` model in `domain/task.rs` for schema changes
- Update serialization in `storage/format.rs`

### Customizing Output

- Modify `io/output.rs` formatting functions
- `print_tasks_simple()` - Default list output
- `print_tasks_verbose()` - Verbose list output
- `print_task_detail()` - Task info output
