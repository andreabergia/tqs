# TQS v1 Functional Specification

## Summary
`tqs` is a terminal task queue CLI.

- Command name is `tqs`.
- Implementation language is Rust.
- Tasks are stored as Markdown files with YAML frontmatter.
- Default storage root is `<git-root>/todos`; if no Git repository is found, fallback is `~/.tqs/todos`.
- Optional overrides are supported via `--root <path>` and `TQS_ROOT`.

## Task Model
- Lifecycle states: `open`, `closed`.
- Task fields:
  - `id`
  - `created_at`
  - `status`
  - `summary`
  - optional markdown description body
- File path format: `<storage-root>/<id>.md`.

## Commands
1. `tqs create [summary] [--description <text>]`
- Creates a new task.
- If `summary` is omitted, prompt for it interactively.
- If description is entered interactively, allow multiline input until EOF.

2. `tqs list [keywords...] [--all|--closed] [--verbose]`
- Default shows open tasks only.
- `--all` shows open and closed.
- `--closed` shows closed only.
- Keyword matching uses AND semantics across `id`, `summary`, and description body.
- Sort order is newest first by `created_at`; ties are sorted by `id` ascending.
- Default output columns: `id`, `summary`.
- `--verbose` output columns: `id`, `status`, `created_at`, `summary`.
- If no tasks match, print `No tasks found`.

3. `tqs complete [id]`
- Marks task as closed.
- With `id`, targets that task.
- Without `id`, opens interactive picker of open tasks.
- If task is already closed, return success with an informational message.

4. `tqs reopen [id]`
- Marks task as open.
- With `id`, targets that task.
- Without `id`, opens interactive picker of closed tasks.
- If task is already open, return success with an informational message.

5. `tqs info [id]`
- Shows detailed view for a single task.
- With `id`, shows that task.
- Without `id`, opens interactive picker across all tasks and shows the selected one.
- Detailed view includes: `id`, `status`, `created_at`, `summary`, full description markdown.

6. `tqs delete <id>`
- Hard-deletes the task file.
- No confirmation prompt.

## Coherent Behavior Rules (All Commands)
- Exit codes:
  - `0`: success
  - `1`: operational/runtime error
  - `2`: usage/argument error
- Output streams:
  - normal results/messages to `stdout`
  - warnings/errors to `stderr`
- Interactive behavior:
  - Shared picker behavior across commands that support no-ID mode.
  - If no TTY is available for a picker-required flow, return a clear error and require `id`.
  - If user cancels picker, exit successfully with a cancellation message.
- Malformed task files:
  - skip malformed files, emit warning, continue processing valid files.

## File Format
```yaml
---
id: cobalt-urial-7f3a
created_at: 2026-02-20T22:15:00Z
status: open
summary: Short task summary
---
```

Markdown body follows frontmatter and may be empty.

## Validation of Completeness (Functional Scope)
This specification is functionally complete for v1 because it defines:
- commands and their inputs
- expected outputs and behavior
- data model and storage format
- interactive/non-interactive behavior
- error and exit-code contract
- consistency rules across all commands
