---
id: tqs-apgz
status: closed
deps: []
links: []
created: 2026-03-12T09:41:18Z
type: task
priority: 1
assignee: andrea.bergia
parent: tqs-oq3n
---
# Add tests for malformed task markdown parsing

Expand src/storage/format.rs test coverage for malformed input and parse-time validation. The current tests do not pin down missing frontmatter delimiters, invalid YAML, or parsing a non-done task with completed_at set.

## Acceptance Criteria

Tests cover missing frontmatter start, missing frontmatter end, invalid YAML frontmatter, and parse-time rejection of completed_at on non-done tasks.


## Notes

**2026-03-12T10:42:07Z**

Added malformed markdown parsing coverage in src/storage/format.rs for missing frontmatter start/end delimiters, invalid YAML frontmatter, and parse-time rejection of completed_at outside the done queue. Verified with cargo test storage::format::tests.
