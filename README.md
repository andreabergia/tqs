# TQS - Terminal Task Queue

## Quick Reference

```
tqs create [summary]         Create a new task
tqs create --id <id>         Create with custom ID
tqs list [keywords]          List open tasks
tqs list --all               List all tasks
tqs list --closed            List closed tasks
tqs complete [id]            Mark task as closed
tqs reopen [id]              Mark task as open
tqs info [id]                Show task details
tqs move <old_id> <new_id>   Change task ID
tqs delete <id>              Delete a task
```

## What is TQS?

TQS is a simple command-line task manager that stores tasks as Markdown files. Perfect for tracking work in Git repositories.

## Installation

Build from source:

```bash
cargo build --release
```

The binary will be at `target/release/tqs`.

## Quick Start

```bash
# Create a task
tqs create "Write documentation"

# List open tasks
tqs list

# View task details
tqs info <task-id>

# Mark task as complete
tqs complete <task-id>

# List completed tasks
tqs list --closed
```

## Storage Location

Tasks are stored as Markdown files with YAML frontmatter. The storage location follows this precedence:

1. `--root <path>` flag
2. `TQS_ROOT` environment variable
3. `<git-repo>/todos` (if in a Git repository)
4. `~/.tqs/todos` (fallback)

Each task is saved as `<storage-root>/<task-id>.md`.

## Learn More

- [USAGE.md](USAGE.md) - Complete command reference
- [ARCHITECTURE.md](ARCHITECTURE.md) - How it works internally
