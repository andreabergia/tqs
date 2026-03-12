---
id: tqs-oq3n
status: closed
deps: []
links: []
created: 2026-03-12T09:41:02Z
type: epic
priority: 1
assignee: andrea.bergia
tags: [testing, coverage, review]
---
# Improve missing test coverage from coverage review

Track the missing-test work identified during the March 12, 2026 codebase review. Focus on negative paths, invariants, and command-selection behavior that are not pinned down by the current suite.

## Acceptance Criteria

All agreed follow-up test tasks are captured as child tickets with clear scope and acceptance criteria.


## Notes

**2026-03-12T10:48:16Z**

All follow-up coverage tickets are implemented and committed: malformed markdown parsing, repository invariants, task normalization/add metadata, task reference resolution, doctor edge cases, and CLI dispatch/exit-code handling. Verified with cargo fmt --check, cargo clippy -- -D warnings, and cargo test on 2026-03-12.
