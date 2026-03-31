# TQS Architecture

## Overview

TQS is a Rust CLI for queue-based task management using Markdown files on disk. The codebase is organized around a small set of layers:

- CLI parsing and command dispatch
- application errors and orchestration
- domain types and search/filter logic
- filesystem-backed storage and config loading
- terminal I/O for prompts, pickers, and formatted output
- full-screen TUI dashboard (ratatui + crossterm)

## Module Layout

```text
src/
├── main.rs              # process entry point
├── lib.rs               # library entry point
├── cli/
│   ├── args.rs          # clap definitions for global options and commands
│   ├── fuzzy.rs         # command expansion for fuzzy input
│   ├── handlers.rs      # dispatch from parsed CLI to command handlers
│   └── commands/        # command implementations
├── app/
│   ├── service.rs       # top-level app runner and exit handling
│   ├── app_error.rs     # error model and exit codes
│   └── operations.rs    # shared task operations (mark_done) used by CLI and TUI
├── domain/
│   ├── task.rs          # Queue enum and Task model
│   ├── id.rs            # id generation and validation
│   └── filter.rs        # dashboard counts and search matching
├── storage/
│   ├── config.rs        # config loading and root resolution
│   ├── repo.rs          # repository for task files
│   ├── format.rs        # Markdown/frontmatter parsing and rendering
│   ├── id_state.rs      # shared generated-id allocator state and locking
│   ├── daily_notes.rs   # optional completion logging
│   ├── editor.rs        # editor resolution from VISUAL/EDITOR/vi
│   └── doctor.rs        # diagnostic checks for config and storage
├── tui/
│   ├── mod.rs           # terminal setup/teardown, main event loop, editor suspension
│   ├── app_state.rs     # TuiApp state, FocusedPanel, Mode, TriageSummary
│   ├── event.rs         # crossterm event polling, key→action dispatch per mode
│   ├── actions.rs       # task mutation actions (done, move, delete, add, triage)
│   ├── ui.rs            # top-level layout assembly for normal, triage, search views
│   └── widgets/
│       ├── sidebar.rs   # queue list with counts and focus highlight
│       ├── task_list.rs # task list for selected queue with selection
│       ├── detail.rs    # task body detail pane (scrollable)
│       ├── status_bar.rs# mode indicator and context-sensitive keybinding hints
│       ├── add_form.rs  # centered overlay for inline task creation
│       └── triage.rs    # triage mode: single task view with action prompts
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
Config + Repository + ID State (storage/config.rs, storage/repo.rs, storage/id_state.rs)
    ↓
Markdown task files (<tasks_root>/<queue-dir>/<id>.md)
```

The `add` command also uses `storage/id_state.rs` to allocate the next shared generated ID, and `done` may call `storage/daily_notes.rs` to append to today’s daily note when that integration is configured.

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
- `completed_at`
- `daily_note`
- `body`

New tasks are created in `inbox` with a default Markdown body containing just the title as a heading.

### Queue Transitions

- `add` creates a task in `inbox` unless `--queue` is supplied
- `move` changes queues and relocates the file
- `done` moves the task to `done` and sets `completed_at`
- editing preserves the task id and normalizes completion metadata so only `done` tasks keep `completed_at`

Transitions are idempotent where appropriate: moving to the current queue or running `done` on an already completed task succeeds with an informational message.

## Storage Model

### ID Generation

Auto-generated task IDs are bare lowercase Crockford-style codes such as `0f3` or `a7k9`.

- generation starts at width `3`
- once a width is exhausted, generation advances to `4`, then `5`, and so on
- each width uses a deterministic additive sequence over the full `32^width` space
- generated IDs are still checked against the repository so manual IDs remain authoritative

The sequence state is shared through a hidden `.tqs/` metadata directory and guarded with a local lock so concurrent `tqs add` processes on one machine do not allocate the same ID.

### Root Resolution

`storage/config.rs` resolves `tasks_root` in this order:

1. CLI `--root`
2. `TQS_ROOT`
3. config file at `$XDG_CONFIG_HOME/tqs/config.toml` or `~/.config/tqs/config.toml`

The same config file may also define:

- `obsidian_vault_dir`, which derives `<vault>/Tasks` and `<vault>/Daily Notes`
- `daily_notes_dir`
- queue directory overrides for `inbox`, `now`, `next`, `later`, and `done`

`obsidian_vault_dir` is a convenience alias over the same generic storage model, not a separate operating mode. It cannot be combined with the lower-level path or queue override settings.

Relative paths in the config file are resolved relative to the config file directory.

### Shared Metadata

TQS stores generator metadata separately from task Markdown files:

- `<vault>/.tqs/` when `obsidian_vault_dir` is configured
- `<tasks_root>/.tqs/` otherwise

The generated-ID allocator stores one state file per resolved `tasks_root` under that metadata directory. This keeps normal multi-computer use aligned when the task storage is synced, while leaving duplicate detection as the backstop for true offline concurrent edits.

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
completed_at:
daily_note:
---
```

The Markdown body follows the closing `---`. `storage/format.rs` is responsible for parsing, rendering, and validating this schema.

## TUI Dashboard

When `tqs` is invoked with no arguments on a TTY, `handlers.rs` launches the full-screen TUI instead of printing the text dashboard. The `--no-tui` flag or piped output falls back to the text dashboard.

### Architecture

The TUI is a separate frontend in `src/tui/` that reuses the same domain and storage layers as the CLI commands. It does not duplicate business logic.

```text
Event Loop (mod.rs)
    ↓
Key Mapping (event.rs) → dispatches based on Mode + FocusedPanel
    ↓
Actions (actions.rs) → calls TaskRepo, operations::mark_done, SharedIdAllocator
    ↓
State Update (app_state.rs) → TuiApp holds repo, config, task cache, selections
    ↓
Rendering (ui.rs + widgets/) → ratatui draws to the alternate screen
```

`TuiApp` owns a `TaskRepo` and `ResolvedConfig`. All mutations go through the repo, then `refresh()` reloads from disk. The action/update pattern returns `SideEffect` values (None, Quit, SuspendForEditor) that the main loop handles.

### Modes

The TUI operates in one of several modes: `Normal`, `AddForm`, `Search`, `Triage`, `MoveTarget`, and `ConfirmDelete`. Each mode has its own key mapping in `event.rs`. The `FocusedPanel` enum (`Sidebar`, `TaskList`, `Detail`) determines how `j/k` and arrow keys behave within `Normal` mode.

### Editor Suspension

When the user presses `e`, the TUI disables raw mode, leaves the alternate screen, spawns the editor as a blocking child process, then restores the TUI and refreshes task data.

## CLI Behavior

### Command Surface

The shipped lean-core commands are:

- `add`
- `list`
- `now`
- `inbox`
- `triage`
- `start`
- `move`
- `delete`
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
