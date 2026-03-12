---
id: tqs-t64b
status: open
deps: []
links: []
created: 2026-03-12T09:41:31Z
type: task
priority: 2
assignee: andrea.bergia
parent: tqs-oq3n
tags: [testing, coverage, domain, cli]
---
# Add tests for task normalization and add-command metadata parsing

Fill low-cost gaps in src/domain/task.rs and src/cli/commands/add.rs. Target move_to no-op behavior, normalize on done tasks without completed_at, tag parsing normalization, and add-command persistence for tags and project metadata.

## Acceptance Criteria

Tests cover move_to returning false for same-queue moves, normalize populating completed_at for done tasks, parse_tags trimming and dropping empties, and add persisting tags and project values.

