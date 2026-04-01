# TQS Usage

## Quick Reference

```bash
# Capture tasks
tqs add <title>
tqs add <title> --queue <queue>
tqs add <title> --edit

# Review work
tqs list
tqs list <queue>
tqs now
tqs inbox
tqs find <query>
tqs show <task>

# Move work forward
tqs triage
tqs start <task>
tqs move <task> <queue>
tqs done <task>
tqs delete <task>
tqs edit <task>

# Global storage override
tqs --root <path> <command>
tqs config
tqs doctor
```

## Interactive Dashboard

Running `tqs` with no arguments on a TTY launches a full-screen interactive TUI. Use `--no-tui` to get the plain text dashboard instead.

### Layout

The dashboard has three panels: a queue sidebar, a task list, and a task detail pane. The focused panel is highlighted with a cyan border.

The sidebar groups queues into three sections separated by dividers:
- **Active work**: now, next, later
- **Triage**: inbox
- **Archive & overview**: done, all

The "all" view shows every task across all queues, with a `[queue]` tag on each entry.

### Navigation

| Key | Action |
|-----|--------|
| `h` / `l` / Left / Right | Move focus between panels |
| `j` / `k` / Up / Down | Navigate within focused panel (queues, tasks, or scroll detail) |
| `Tab` / `Shift-Tab` | Cycle to next / previous queue (from any panel) |
| `1`–`6` | Jump directly to a queue or all (from any panel) |

### Task Actions

| Key | Action |
|-----|--------|
| `a` | Add a task (inline form: type title, Tab/Shift-Tab to pick queue, Enter to create) |
| `e` | Edit selected task in `$EDITOR` (suspends and restores the TUI) |
| `d` | Mark selected task as done |
| `s` | Start selected task (move to now) |
| `m` | Move selected task (then press `i`/`n`/`x`/`l` for inbox/now/next/later) |
| `x` | Delete selected task (press `y` to confirm) |
| `r` | Refresh task data from disk |

### Modes

| Key | Action |
|-----|--------|
| `/` | Enter search mode — type to filter tasks across all queues, Enter to jump to result |
| `t` | Enter triage mode — cycle through inbox tasks (same keys as normal mode, plus `Space` to skip) |
| `q` / `Esc` | Quit dashboard (or exit current mode) |

## Global Options

### `--root <path>`

Override the configured tasks root for a single command.

```bash
tqs --root /tmp/tasks list
```

### `TQS_ROOT`

Set a default tasks root via environment variable.

```bash
export TQS_ROOT=/path/to/tasks
tqs list
```

Storage precedence:

1. `--root <path>`
2. `TQS_ROOT`
3. config file at `$XDG_CONFIG_HOME/tqs/config.toml` or `~/.config/tqs/config.toml`

## Exit Codes

- `0` - success
- `1` - runtime error
- `2` - usage or argument error

## Queues

Valid queue names:

- `inbox`
- `now`
- `next`
- `later`
- `done`

These names are the command-line and frontmatter values even when queue directory names are overridden in config.

## Task Resolution

Commands that accept `<task>` resolve it in this order:

1. exact id match
2. unique id prefix
3. unique title substring
4. interactive picker if multiple matches remain and a TTY is available

If the match is ambiguous without a TTY, the command fails with an ambiguity error. If no tasks exist, interactive commands print `No tasks available` and exit successfully.

## Commands

### `add`

```bash
tqs add [title] [--queue <queue>] [--edit]
```

Creates a new task. If `title` is omitted, TQS prompts for it interactively.

Flags:

- `--queue <queue>` creates the task directly in a queue other than `inbox`
- `--edit` opens the created file in the configured editor immediately after creation

Behavior:

- generates a unique lowercase Crockford-style task id
- starts with 3-character ids and grows to wider ids only as needed
- stores allocator state in `<vault>/.tqs/` when using `obsidian_vault_dir`, otherwise in `<tasks_root>/.tqs/`
- creates a Markdown file with the default task template
- prints `Created task: <id> (<path>)`

Examples:

```bash
tqs add "Reply to AWS billing alert"
tqs add "Plan release notes" --queue now
tqs add "Draft incident summary" --edit
```

### `list`

```bash
tqs list
tqs list <queue>
```

Behavior:

- `tqs list` prints queue counts for all built-in queues, then the `now` section, then the `inbox` section
- `tqs list <queue>` prints that queue header and one line per task: `<id>  <title>`
- empty queue output prints `No tasks found`

Examples:

```bash
tqs list
tqs list now
tqs list done
```

### `now`

```bash
tqs now
```

Prints the `now` queue view.

Behavior:

- equivalent to `tqs list now`
- prints the `now` header and one line per task: `<id>  <title>`
- empty queue output prints `No tasks found`

### `inbox`

```bash
tqs inbox
```

Prints the `inbox` queue view.

Behavior:

- equivalent to `tqs list inbox`
- prints the `inbox` header and one line per task: `<id>  <title>`
- empty queue output prints `No tasks found`

### `triage`

```bash
tqs triage
```

Interactively walk through each inbox task and decide what to do with it.

Behavior:

- requires a TTY — fails with `NoTty` if not connected to a terminal
- if the inbox is empty, prints a message and exits
- shows a header with the number of inbox tasks
- for each task, displays `<id>  <title>` and prompts for an action:
  - **move to now / next / later** — moves the task to that queue
  - **mark done** — moves to `done` and writes a daily-note entry if configured
  - **edit** — opens the task in the editor, then re-shows the same task
  - **delete** — removes the task file
  - **skip** — leaves the task in inbox and advances to the next one
  - **quit** (or Esc) — stops triaging immediately
- prints a summary of actions taken at the end of the session

Example:

```bash
tqs triage
# Triaging inbox (3 tasks)
#
# 0f3  Reply to AWS billing alert
# ? Action: move to now
# a7k  Plan release notes
# ? Action: skip
# b2x  Draft incident summary
# ? Action: mark done
#
# 2 to now, 1 done, 1 skipped
```

### `start`

```bash
tqs start <task>
```

Moves a task to the `now` queue. Equivalent to `tqs move <task> now`.

Behavior:

- resolves `<task>` using standard task resolution
- if the task is already in `now`, prints `Task <id> is already in now` and exits successfully
- otherwise moves the task to `now`, updates `updated_at`, and prints `Started task: <id> (<path>)`

Examples:

```bash
tqs start 0f3
tqs start "billing alert"
```

### `move`

```bash
tqs move <task> <queue>
```

Moves a task into a different queue.

Behavior:

- updates `queue`
- updates `updated_at`
- moves the file into the target queue directory
- if the task is already in the target queue, prints `Task <id> is already in <queue>` and exits successfully

Examples:

```bash
tqs move 20260309-aws now
tqs move billing later
```

### `done`

```bash
tqs done <task>
```

Marks a task as done by moving it to the `done` queue.

Behavior:

- updates `queue` to `done`
- sets `completed_at`
- updates `updated_at`
- prints `Completed task: <id> (<path>)`
- if the task is already done, prints `Task <id> is already done` and exits successfully
- if `daily_notes_dir` is configured, appends a completion line to today’s daily note and stores the note name in `daily_note`

Examples:

```bash
tqs done 20260309-aws
tqs done "billing alert"
```

### `delete`

```bash
tqs delete <task>
tqs delete <task> --interactive
```

Permanently deletes a task file.

Flags:

- `--interactive` / `-i` prompts for confirmation before deleting

Behavior:

- resolves `<task>` using standard task resolution
- removes the task Markdown file from disk
- prints `Deleted task: <id>`

Examples:

```bash
tqs delete 0f3
tqs delete "billing alert"
tqs delete 0f3 -i
```

### `edit`

```bash
tqs edit <task>
```

Opens the resolved task file in the configured editor, then validates and re-saves the edited task.

Behavior:

- resolves the editor from `VISUAL`, then `EDITOR`, then `vi`
- rejects edits that make the file empty
- rejects edits that change the task id
- normalizes timestamps and completion metadata after a valid edit
- restores the original file if validation fails

Examples:

```bash
export VISUAL="nvim"
tqs edit 20260309-aws
```

### `show`

```bash
tqs show <task>
```

Displays task metadata and the Markdown body.

Output includes:

- id
- queue
- resolved file path
- created timestamp
- updated timestamp
- title
- completed timestamp when present
- full body

Example:

```bash
tqs show 20260309-aws
```

### `find`

```bash
tqs find <query>
```

Searches across tasks in all queues.

The search matches case-insensitively against:

- id
- title
- body
`find` does not support structured filter flags in v2. Use a single free-text query.

Output format:

```text
[<queue>] <id>  <title>
```

Example:

```bash
tqs find billing
tqs find finance
```

### `config`

```bash
tqs config
```

Displays the effective configuration values used by the CLI.

Output includes:

- `obsidian_vault_dir` when configured
- `tasks_root`
- `daily_notes_dir` or `<unset>`
- queue directory mappings for `inbox`, `now`, `next`, `later`, and `done`

Behavior:

- resolves config with the current precedence: `--root`, `TQS_ROOT`, then config file
- prints effective values only
- does not modify config files or validate beyond normal command startup

### `doctor`

```bash
tqs doctor
```

Runs a read-only diagnostic pass over effective config and on-disk task storage.

Checks include:

- config resolution and queue directory overlap
- whether `tasks_root` and `daily_notes_dir` exist as directories
- which editor command would be used from `VISUAL`, `EDITOR`, or the `vi` fallback
- whether the editor executable is discoverable on `PATH`
- malformed Markdown task files
- task files whose frontmatter queue does not match their containing queue directory
- duplicate task ids across queue directories

Behavior:

- prints each finding with `ok`, `warn`, or `error`
- exits successfully when no errors are found
- exits non-zero when any error is found

## File Format

Tasks are stored as Markdown files under:

```text
<tasks_root>/<queue-dir>/<id>.md
```

Example task:

```markdown
---
id: 20260309-103412-reply-aws-billing
title: Reply to AWS billing alert
queue: now
created_at: 2026-03-09T10:34:12Z
updated_at: 2026-03-09T11:20:07Z
completed_at:
daily_note:
---
# Reply to AWS billing alert

## Context

Finance asked for an explanation of an unexpected cost spike.

## Notes
```

## Configuration

Minimal config:

```toml
tasks_root = "/path/to/tasks"
daily_notes_dir = "/path/to/daily-notes"

[queues]
inbox = "inbox"
now = "focus"
next = "next"
later = "later"
done = "archive"
```

Obsidian convenience config:

```toml
obsidian_vault_dir = "/path/to/My Vault"
```

Notes:

- `tasks_root` is required unless supplied via `--root` or `TQS_ROOT`
- `daily_notes_dir` is optional
- `obsidian_vault_dir` derives `tasks_root = <vault>/Tasks` and `daily_notes_dir = <vault>/Daily Notes`
- `obsidian_vault_dir` cannot be combined with `tasks_root`, `daily_notes_dir`, or queue overrides
- queue overrides change directory names only
- relative config paths are resolved relative to the config file directory
- queue directory overrides must be a single path segment

## Obsidian Layout

TQS remains a generic filesystem-backed task manager. If you use Obsidian, the recommended vault layout is:

```text
<vault>/
  Tasks/
    inbox/
    now/
    next/
    later/
    done/
  Daily Notes/
    YYYY-MM-DD.md
```

This layout matches `obsidian_vault_dir` directly. Daily-note completion entries are written as wiki-links to the completed task file, such as `- [x] [[Tasks/done/task-1|Ship v2]]`. For other supported layouts, TQS derives the link target from the configured task and daily-note directories.
