# TQS v2 Implementation Plan

Phases 1 through 5 of the original rewrite plan are complete. This document tracks only the remaining work needed before the v2 rewrite can be treated as fully consolidated.

## Current State

Implemented:

- canonical `docs/v2` design set
- queue-based task model and filesystem layout
- lean v2 command set: `add`, `list`, `now`, `inbox`, `move`, `done`, `edit`, `show`, `find`, `config`
- minimal config loading for `tasks_root`, optional `daily_notes_dir`, and queue directory overrides
- optional daily-note completion logging
- top-level repository docs aligned with the shipped v2 CLI, storage model, and architecture

Validated:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`

Still missing:

- `doctor` remains deferred for a future milestone
- phase 9 query/filter decisions remain open

## Phase 6: Consolidate Documentation

Status: complete

Goal:

Make the repository documentation describe the current v2 product rather than the removed v1 surface.

Deliverables:

- rewrite `README.md` around the v2 command set and queue model
- rewrite `USAGE.md` as the current CLI reference for `add`, `list`, `move`, `done`, `edit`, `show`, and `find`
- rewrite `ARCHITECTURE.md` to reflect the queue-based domain model, current config resolution, and current file layout
- update any references that still point readers at superseded behavior or terminology

Acceptance criteria:

- no top-level doc instructs users to run removed commands such as `create`, `complete`, `reopen`, `info`, or `delete`
- storage documentation matches the current `<tasks_root>/<queue>/<id>.md` layout and current frontmatter schema
- config documentation matches the current precedence: `--root`, `TQS_ROOT`, then config file
- architecture documentation matches the current module layout and queue workflow

## Phase 7: Close Lean-Core Test Gaps

Status: complete

Goal:

Bring CLI-level acceptance coverage in line with the lean-core contract and the original test plan.

Deliverables:

- add a smoke test for `list <queue>`
- add an end-to-end test for `edit` using a non-interactive editor command
- add an end-to-end test for `add --edit`
- add a test for `move` when the task is already in the target queue
- add a CLI-level test confirming repeated `done` runs do not duplicate the daily-note entry
- add or tighten end-to-end coverage for the original scenario list where it is still only covered indirectly

Recommended scenario coverage to close:

- promotion from `inbox` to `now`
- editing a task body without renaming the file
- search coverage for tags, source, and project if those fields remain part of the intended search surface

Acceptance criteria:

- every lean-core command has at least one direct CLI smoke test for its primary happy path
- lean-core idempotency and no-op behaviors are covered at the CLI layer
- the remaining end-to-end scenarios from the original plan are directly covered by CLI tests

## Phase 8: Decide and Implement Remaining Deferred Commands

Status: complete

Goal:

Resolve the command-level deferred items from the original plan and keep only the ones that still fit the product.

Outcome:

- implemented `now` as a convenience alias for `list now`
- implemented `inbox` as a convenience alias for `list inbox`
- implemented `config` as a read-only effective-config view
- kept `doctor` explicitly deferred for a future milestone because it is additive diagnostics rather than core queue workflow

Acceptance criteria:

- each deferred command is either implemented with tests and docs, or explicitly deferred/dropped from the active roadmap

## Phase 9: Decide and Implement Deferred Query Features

Status: pending

Goal:

Resolve the deferred filtering and search work from the original plan.

Deferred items carried over from the original plan:

- advanced list filters
- richer search filters

Work required:

- define whether `list` should remain queue-only or grow additional filters
- define whether `find` should remain plain text or gain structured filtering
- implement only after the CLI contract is clear and test coverage is added

Acceptance criteria:

- the active roadmap no longer contains undefined filter work
- any adopted filtering or search features are documented and tested end to end

## Completion Criteria

This plan is complete when:

- repository-facing docs are fully aligned with the shipped v2 behavior
- the lean-core contract has complete acceptance coverage where behavior matters at the CLI layer
- every deferred item from the original plan has been implemented or explicitly removed from the roadmap
