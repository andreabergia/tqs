---
id: tqs-t64b
status: closed
deps: []
links: []
created: 2026-03-12T09:41:31Z
type: task
priority: 2
assignee: andrea.bergia
parent: tqs-oq3n
---
# Add tests for task normalization and add-command metadata parsing

Fill low-cost gaps in src/domain/task.rs and src/cli/commands/add.rs. Target move_to no-op behavior and normalize on done tasks without completed_at.

## Acceptance Criteria

Tests cover move_to returning false for same-queue moves and normalize populating completed_at for done tasks.


## Notes

**2026-03-12T10:44:32Z**

Added coverage for Task::move_to no-op behavior and Task::normalize populating completed_at for done tasks. Verified with cargo test domain::task::tests and cargo test cli::commands::add::tests.
