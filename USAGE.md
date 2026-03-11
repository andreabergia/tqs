# TQS Usage

## Quick Reference

```bash
# Capture tasks
tqs add <title>
tqs add <title> --queue <queue>
tqs add <title> --source <source> --tags <tag1,tag2> --project <project>
tqs add <title> --edit

# Review work
tqs list
tqs list <queue>
tqs now
tqs inbox
tqs find <query>
tqs show <task>

# Move work forward
tqs move <task> <queue>
tqs done <task>
tqs edit <task>

# Global storage override
tqs --root <path> <command>
tqs config
tqs doctor
```

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
tqs add [title] [--source <source>] [--tags <tag1,tag2>] [--project <project>] [--queue <queue>] [--edit]
```

Creates a new task. If `title` is omitted, TQS prompts for it interactively.

Flags:

- `--source <source>` sets the optional source field
- `--tags <tag1,tag2>` sets comma-separated tags
- `--project <project>` sets the optional project field
- `--queue <queue>` creates the task directly in a queue other than `inbox`
- `--edit` opens the created file in the configured editor immediately after creation

Behavior:

- generates a unique task id
- creates a Markdown file with the default task template
- prints `Created task: <id> (<path>)`

Examples:

```bash
tqs add "Reply to AWS billing alert"
tqs add "Plan release notes" --queue now --project docs
tqs add "Follow up with finance" --tags billing,aws --source email
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
- tags, source, project when present
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
- tags
- source
- project

`find` does not support structured filter flags in v2. Use a single free-text query.

Output format:

```text
[<queue>] <id>  <title>
```

Example:

```bash
tqs find billing
tqs find platform-costs
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
tags:
  - aws
  - finance
source: email
project: platform-costs
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

This layout matches `obsidian_vault_dir` directly. Daily-note completion entries stay in plain Markdown checklist form rather than Obsidian wiki-link syntax.
