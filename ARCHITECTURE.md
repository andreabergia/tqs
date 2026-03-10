# TQS Architecture

## Overview

TQS is a Rust CLI for queue-based task management using Markdown files on disk. The codebase is organized around a small set of layers:

- CLI parsing and command dispatch
- application errors and orchestration
- domain types and search/filter logic
- filesystem-backed storage and config loading
- terminal I/O for prompts, pickers, and formatted output

## Module Layout

```text
src/
├── main.rs              # process entry point
├── lib.rs               # library entry point
├── cli/
│   ├── args.rs          # clap definitions for global flags and commands
│   ├── fuzzy.rs         # command expansion for fuzzy input
│   ├── handlers.rs      # dispatch from parsed CLI to command handlers
│   └── commands/        # command implementations
├── app/
│   ├── service.rs       # top-level app runner and exit handling
│   └── app_error.rs     # error model and exit codes
├── domain/
│   ├── task.rs          # Queue enum and Task model
│   ├── id.rs            # id generation and validation
│   └── filter.rs        # dashboard counts and search matching
├── storage/
│   ├── config.rs        # config loading and root resolution
│   ├── repo.rs          # repository for task files
│   ├── format.rs        # Markdown/frontmatter parsing and rendering
│   └── daily_notes.rs   # optional completion logging
└── io/
    ├── input.rs         # interactive text prompts
    ├── output.rs        # CLI output formatting
    └── picker.rs        # interactive task selection
```

## Data Flow

```text
CLI (args.rs + fuzzy.rs)
    ↓
Handlers (handlers.rs)
    ↓
Command implementation (cli/commands/*)
    ↓
Config + Repository (storage/config.rs, storage/repo.rs)
    ↓
Markdown task files (<tasks_root>/<queue-dir>/<id>.md)
```

The `done` command may also call `storage/daily_notes.rs` to append to today’s daily note when that integration is configured.

## Domain Model

### Queues

The task workflow is centered on five built-in logical queues:

- `inbox`
- `now`
- `next`
- `later`
- `done`

The queue value is stored in frontmatter and used by CLI parsing, filtering, and output. Configurable queue directory names affect only on-disk folder names.

### Task Schema

`Task` in `src/domain/task.rs` stores:

- `id`
- `title`
- `queue`
- `created_at`
- `updated_at`
- `tags`
- `source`
- `project`
- `completed_at`
- `daily_note`
- `body`

New tasks are created in `inbox` with a default Markdown body template:

```markdown
# <title>

## Context

## Notes
```

### Queue Transitions

- `add` creates a task in `inbox` unless `--queue` is supplied
- `move` changes queues and relocates the file
- `done` moves the task to `done` and sets `completed_at`
- editing preserves the task id and normalizes completion metadata so only `done` tasks keep `completed_at`

Transitions are idempotent where appropriate: moving to the current queue or running `done` on an already completed task succeeds with an informational message.

## Storage Model

### Root Resolution

`storage/config.rs` resolves `tasks_root` in this order:

1. CLI `--root`
2. `TQS_ROOT`
3. config file at `$XDG_CONFIG_HOME/tqs/config.toml` or `~/.config/tqs/config.toml`

The same config file may also define:

- `daily_notes_dir`
- queue directory overrides for `inbox`, `now`, `next`, `later`, and `done`

Relative paths in the config file are resolved relative to the config file directory.

### Repository Behavior

`TaskRepo` in `storage/repo.rs` provides the filesystem-backed operations used by commands:

- `create` writes a new task file
- `read` and `find_by_id` resolve stored tasks
- `update` rewrites a task and moves its file if the queue changed
- `move_to_queue` applies queue transitions
- `replace_edited` reparses and validates editor changes
- `scan_all` walks all queue directories, skipping malformed Markdown files with warnings

Tasks are stored as:

```text
<tasks_root>/<queue-dir>/<id>.md
```

### File Format

Task files are Markdown with YAML frontmatter:

```yaml
---
id: 20260309-103412-reply-aws-billing
title: Reply to AWS billing alert
queue: now
created_at: 2026-03-09T10:34:12Z
updated_at: 2026-03-09T11:20:07Z
tags: [aws, finance]
source: email
project: platform-costs
completed_at:
daily_note:
---
```

The Markdown body follows the closing `---`. `storage/format.rs` is responsible for parsing, rendering, and validating this schema.

## CLI Behavior

### Command Surface

The shipped lean-core commands are:

- `add`
- `list`
- `move`
- `done`
- `edit`
- `show`
- `find`
- `config`
- `doctor`

### Task Reference Resolution

Commands that accept a task reference use the same resolution rules:

1. exact id
2. unique id prefix
3. unique title substring
4. picker when ambiguous and a TTY is available

If no TTY is available for an ambiguous match, the command returns an error instead of guessing.

### Output

`io/output.rs` owns the text UI:

- dashboard and queue listings for `list`
- detailed task rendering for `show`
- search result formatting for `find`
- diagnostic report formatting for `doctor`
- informational messages for create, move, done, and edit flows

## Error Handling

`AppError` in `src/app/app_error.rs` drives user-visible failures and exit codes:

- `0` for success
- `1` for runtime failures
- `2` for usage and argument errors

Recoverable malformed task files are reported as warnings during repository scans and skipped rather than aborting the whole command.
