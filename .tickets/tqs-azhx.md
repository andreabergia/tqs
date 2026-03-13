---
id: tqs-azhx
status: closed
deps: [tqs-sg15, tqs-t64b]
links: []
created: 2026-03-12T09:41:39Z
type: task
priority: 3
assignee: andrea.bergia
parent: tqs-oq3n
---
# Add tests for CLI dispatch and exit-code handling

Add direct tests around src/cli/handlers.rs and src/app/service.rs for the remaining dispatch branches. Current integration coverage hits commands end to end but not the no-command usage path or the final exit-code translation in service::run().

## Acceptance Criteria

Tests cover handlers::handle returning a usage error when no command is provided and service::run translating application errors into the expected exit codes.


## Notes

**2026-03-12T10:47:14Z**

Added direct dispatch coverage in src/cli/handlers.rs for the no-command usage path and in src/app/service.rs for exit-code translation of success, usage errors, and operational errors. Verified with cargo test handle_returns_usage_error_when_no_command_is_specified and cargo test exit_code_for_.
