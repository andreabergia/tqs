# TQS v1 Rust Implementation Plan

## Summary
Implement `tqs` as a Rust CLI with coherent behavior across commands, backed by Markdown files with YAML frontmatter.

## Non-Negotiable Delivery Rule
Tests and quality gates are part of implementation from the first step and are run at every step.

- At each step, run:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
- No step is considered complete unless these pass (or failures are explicitly diagnosed and fixed within the same step).

## Scope
- Commands: `create`, `list`, `complete`, `reopen`, `info`, `delete`
- Storage: `<id>.md` files under resolved root
- Runtime: Rust (current stable)
- Out of scope for v1: JSON output, editor integration, daemon/service mode

## Architecture
Use layered modules:

1. `src/main.rs`
- CLI entrypoint, delegates to app layer, maps result to exit code.

2. `src/cli/`
- `args.rs`: `clap` command and flag definitions.
- `handlers.rs`: thin command handlers.

3. `src/domain/`
- `task.rs`: `Task`, `TaskStatus`, metadata structs.
- `filter.rs`: list and selection filters.

4. `src/storage/`
- `root.rs`: root resolution (`--root`, `TQS_ROOT`, git root, home fallback).
- `repo.rs`: filesystem repository.
- `format.rs`: frontmatter/body parse and serialize.

5. `src/io/`
- `picker.rs`: shared interactive picker.
- `output.rs`: stdout/stderr formatting.

6. `src/app/`
- `service.rs`: command use-cases.
- `app_error.rs`: central typed error model and exit-code mapping.

## Coherent Command Contract
- Exit codes:
  - `0` success
  - `1` operational/runtime error
  - `2` usage/argument error
- Output streams:
  - normal output to `stdout`
  - warnings/errors to `stderr`
- Interactive contract:
  - shared picker behavior across no-ID flows
  - no TTY for picker-required flow => clear error, require `id`, exit `1`
  - picker cancel => success `0` with cancellation message
- Malformed files are skipped with warning; processing continues.

## Command Behavior
1. `tqs create [summary] [--description <text>]`
- Creates new task.
- Missing summary => interactive prompt.
- Interactive description supports multiline input until EOF.

2. `tqs list [keywords...] [--all|--closed] [--verbose]`
- Default open tasks.
- `--all` open + closed.
- `--closed` closed only.
- AND keyword matching across `id`, `summary`, description body.
- Sort by `created_at` desc, tie-break `id` asc.
- Default columns: `id`, `summary`.
- `--verbose`: `id`, `status`, `created_at`, `summary`.
- No matches => `No tasks found`, exit `0`.

3. `tqs complete [id]`
- With `id`: close that task.
- Without `id`: picker over open tasks.
- Already closed => message, exit `0`.

4. `tqs reopen [id]`
- With `id`: reopen that task.
- Without `id`: picker over closed tasks.
- Already open => message, exit `0`.

5. `tqs info [id]`
- With `id`: detailed task view.
- Without `id`: picker over all tasks.
- Detailed view includes `id`, `status`, `created_at`, `summary`, full description markdown.

6. `tqs delete <id>`
- Hard delete, no confirmation.
- Missing task => not-found error, exit `1`.

## Data and Storage
- Task file path: `<storage-root>/<id>.md`
- Frontmatter required fields: `id`, `created_at`, `status`, `summary`
- Unknown extra frontmatter fields are ignored.
- Markdown body optional.

## ID Generation
- Format: `word-word-xxxx` (4-char lowercase hex suffix)
- Embedded fixed wordlists (~256 adjectives, ~256 nouns)
- Collision-safe retry with bounded attempts

## Dependencies
- `clap`
- `dialoguer`
- `serde`, `serde_yaml`
- `chrono` (or `time`, choose one consistently)
- `thiserror`
- Test deps: `assert_cmd`, `assert_fs`, `predicates`, `tempfile`

## Step-by-Step Execution
1. [x] Bootstrap crate and module skeleton.
- Added Rust crate scaffold plus module layout: `app`, `cli`, `domain`, `storage`, `io`.
- Added baseline unit/integration tests and CLI skeleton wiring.

2. [x] Implement domain types and central error model.
- Expanded domain model with serde-ready `Task`/`TaskStatus` behavior and transitions.
- Expanded `AppError` into a typed central error model with exit-code mapping.
- Added/expanded unit tests for task serde/status/transitions, keyword filtering, and error exit codes.

3. [x] Implement storage root resolver and file format parsing.
- Implemented root resolution precedence: `--root` > `TQS_ROOT` > `<git-root>/todos` > `~/.tqs/todos` (with cwd fallback).
- Implemented Markdown task format parser/renderer with YAML frontmatter validation and description body mapping.
- Added parser and root-resolution unit tests, including Git-root detection.

4. [x] Implement repository and ID generation.
- Implemented ID generator with `word-word-xxxx` format using embedded wordlists (~256 adjectives, ~256 nouns).
- Implemented TaskRepo with full CRUD operations (create, read, update, delete, list).
- Added collision-safe ID generation with bounded retry attempts.
- Added comprehensive tests for ID generation, collision handling, and repository behavior.
- Added FormatError to AppError mapping for proper error handling.

5. [x] Implement `create` handler with integration tests.
- Implemented `create` command with summary and optional description.
- Added CLI tests for create with/without description, missing summary error, and unique ID generation.

6. [x] Implement `list` handler with integration tests.

7. [x] Implement `complete`/`reopen` (no picker)

8. Implement `info` and `delete` (no picker)
- Add not-found and detail-output tests.

9. Implement shared picker
- Add interactive and non-TTY behavior tests.

10. Polish output and edge-case consistency.
- Add regression tests for exit codes/streams/malformed files.
- Run gates.

11. Final pass.
- Confirm no open behavioral gaps against `tsq.md`.

## Test Plan
Unit tests:
- ID format and collision retry
- parse/serialize roundtrip
- root resolution precedence
- keyword AND matching
- sort tie-break rules

Integration tests:
- `create` (arg and interactive)
- `list` default, `--all`, `--closed`, `--verbose`, no-match
- `complete`/`reopen` by ID and picker mode
- no-TTY failures for picker-required paths
- `info` by ID and picker mode
- `delete` success and missing-task error
- malformed-file warning + skip behavior
- stdout/stderr + exit-code consistency

## Defaults and Assumptions
- Command name is `tqs`.
- Timestamp is UTC ISO-8601.
- Rust current stable.
- `tsq.md` is the functional source of truth.
