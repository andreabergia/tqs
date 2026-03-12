---
id: tqs-azhx
status: open
deps: [tqs-sg15, tqs-t64b]
links: []
created: 2026-03-12T09:41:39Z
type: task
priority: 3
assignee: andrea.bergia
parent: tqs-oq3n
tags: [testing, coverage, cli]
---
# Add tests for CLI dispatch and exit-code handling

Add direct tests around src/cli/handlers.rs and src/app/service.rs for the remaining dispatch branches. Current integration coverage hits commands end to end but not the no-command usage path or the final exit-code translation in service::run().

## Acceptance Criteria

Tests cover handlers::handle returning a usage error when no command is provided and service::run translating application errors into the expected exit codes.

