# TQS v2 Implementation Plan

## Summary

`tqs` v2 is a clean rewrite built in this repository with no backward-compatibility constraints.

Defaults for the rewrite:

- the current codebase is reference material only
- the first delivery is the lean core command set
- the system is defined in terms of configured directories, not a specific notes application
- daily notes are generic Markdown integration and are deferred from the first delivery

## Phase 1: Canonicalize the Design

Status: completed

Delivered:

- canonical `docs/v2` document set
- filesystem-first product language
- generic daily-note model
- older top-level design drafts marked as superseded

## Phase 2: Replace the Domain Model and Storage

Status: completed

Goal:

Replace the old `open` and `closed` task model with the new queue-based model and filesystem layout.

Deliverables:

- add a `Queue` type with `inbox`, `now`, `next`, `later`, and `done`
- replace the old task struct with the v2 schema
- rewrite Markdown parse/render for the new frontmatter plus freeform body
- implement repository paths at `<tasks_root>/<queue>/<id>.md`
- implement scanning across all queue directories
- implement malformed-file handling during scans

Key rules:

- filenames are always `<id>.md`
- title edits must not rename files
- moving a task means moving the file and updating metadata
- `completed_at` is only set when the task is done

Acceptance criteria:

- a task can be created, read, updated, and moved across queues
- queue scans find tasks across all queue directories
- invalid queue names are rejected cleanly
- parse/render roundtrips preserve the task schema and body

Delivered:

- replaced the old status model with a queue-centric `Task` schema
- implemented queue-backed storage at `<tasks_root>/<queue>/<id>.md`
- implemented whole-repo scanning across the built-in queue directories
- preserved malformed-file warnings during scans
- centralized queue enumeration in the domain layer so a future configurable-queues feature has a single integration point

## Phase 3: Replace the CLI Surface

Status: completed

Goal:

Replace the old command set with the lean v2 workflow.

Deliverables:

- remove `create`, `complete`, `reopen`, `info`, old `move`, and related aliases
- implement `add`, `list`, `move`, `done`, `edit`, `show`, and `find`
- implement shared task-reference resolution
- keep editor integration generic and file-based

Command expectations:

- `add` creates a task in `inbox` by default using the default body template
- `list` shows a compact default dashboard and supports `list <queue>`
- `move` changes queue, file location, and `updated_at`
- `done` moves to `done`, sets `completed_at`, and updates `updated_at`
- `edit` opens the resolved file in the configured editor
- `show` prints metadata, resolved path, and body
- `find` performs simple search across all queues

Task-reference resolution rules:

1. exact id match
2. unique filename prefix
3. unique title substring
4. interactive picker if ambiguous
5. clear error if unresolved

Acceptance criteria:

- all lean-core commands work end to end against the new storage layout
- ambiguous references behave correctly in interactive and non-interactive contexts
- already-done tasks are handled idempotently by `done`

Delivered:

- removed the old `create`, `complete`, `reopen`, `info`, and `delete` command paths
- implemented `add`, `list`, `move`, `done`, `edit`, `show`, and `find`
- removed legacy command aliases from the v2 CLI surface
- implemented shared task resolution with exact id, unique id prefix, unique title substring matching, and picker fallback
- clarified non-interactive ambiguity handling with a dedicated task-reference error

## Phase 4: Add Minimal Configuration

Status: completed

Goal:

Resolve runtime directories without expanding the public surface unnecessarily.

Deliverables:

- resolve `tasks_root` from CLI flag, environment, and config file
- resolve optional `daily_notes_dir`
- make queue definitions configurable without changing the on-disk task schema
- keep configuration small and implementation-focused

Rules:

- `tasks_root` is the only required effective location
- `daily_notes_dir` is optional in the first delivery
- built-in queues remain the default until explicit configuration support is added
- no `config` command is included in the first delivery
- no `doctor` command is included in the first delivery

Acceptance criteria:

- CLI commands operate predictably with configured paths
- missing required configuration fails with concise errors
- optional daily-note configuration can be absent without affecting the core workflow

Delivered:

- added a small config loader using `--root`, `TQS_ROOT`, then `~/.config/tqs/config.toml`
- resolved optional `daily_notes_dir` from config without affecting the current core workflow
- added configurable queue directory names while preserving canonical queue values in task frontmatter
- removed the implicit git/XDG data fallback so missing `tasks_root` now fails explicitly

## Phase 5: Add Daily-Note Completion Logging

Status: completed

Goal:

Implement the deferred generic Markdown daily-note integration for task completion.

Deliverables:

- append completed tasks to today’s note when `daily_notes_dir` is configured
- create today’s note and the completed-tasks section when missing
- avoid duplicate completion entries for the same task
- persist the referenced daily note back into task frontmatter

Rules:

- daily-note integration remains optional and inactive when `daily_notes_dir` is unset
- completion logging stays tool-agnostic Markdown rather than editor-specific metadata
- `done` remains idempotent for already-completed tasks
- daily-note references are stored as note filenames in task frontmatter

Acceptance criteria:

- `done` still completes tasks correctly when no daily-note directory is configured
- `done` appends a completion entry to `YYYY-MM-DD.md` when daily-note integration is configured
- the completed-tasks section is created automatically when absent
- repeated completion logging does not duplicate the same task entry

Delivered:

- added daily-note completion logging to `done` using `daily_notes_dir` when configured
- create or update today’s `YYYY-MM-DD.md` note with a `## Completed Tasks` section
- prevent duplicate completion lines for the same task in the target note
- record the linked daily-note filename in task frontmatter after successful completion logging

## Deferred Features

The following remain out of scope until the lean core is complete and stable:

- `now`
- `inbox`
- `doctor`
- `config`
- advanced list filters
- richer search filters

## Test Plan

Model and storage:

- task schema parse/render roundtrip
- queue parsing and invalid queue rejection
- repository create/read/update/move across queue directories
- whole-repo scans across all queues
- malformed-file handling during scans
- stable filenames when titles change

CLI:

- `add` creates under `<tasks_root>/inbox/`
- `list` works for the default dashboard and explicit queue view
- `move` updates both file location and metadata
- `done` moves to `done` and is idempotent enough for repeated runs
- `done` appends to the configured daily note and records `daily_note`
- `edit` opens the resolved file path
- `show` prints metadata, path, and body
- `find` matches title and body at minimum
- task-reference resolution works for exact ids, prefixes, substrings, and ambiguities

End-to-end scenarios:

- fast capture of a minimal task into `inbox`
- promotion from `inbox` to `now`
- completion into `done`
- editing a task body without renaming the file
- searching across multiple queues after moves and completion

## Assumptions

- no compatibility layer will be built for the old storage format or command set
- old tests may be deleted or rewritten freely
- daily-note integration is intentionally deferred from the first delivery
- queue configurability is a future extension; the current implementation keeps queue definitions centralized to make that migration local
- Obsidian compatibility remains a usage pattern, not a core architectural dependency
