---
id: tqs-txb9
status: closed
deps: []
links: []
created: 2026-03-12T09:41:11Z
type: task
priority: 1
assignee: andrea.bergia
parent: tqs-oq3n
---
# Add tests for repository filesystem invariants

Cover untested safety and correctness paths in src/storage/repo.rs. Focus on delete behavior, duplicate ID detection, filename and task ID mismatch handling, path traversal protection, and the repo-level edit finalization helpers.

## Acceptance Criteria

Tests exercise delete, duplicate-ID lookup failures, filename mismatch rejection, path traversal rejection, and replace_edited/finalize_added_edit validation paths.


## Notes

**2026-03-12T10:43:14Z**

Added repository invariant coverage in src/storage/repo.rs for delete behavior, duplicate-ID lookup failures across queues, filename mismatch rejection, path traversal protection in finalize_added_edit, and replace_edited/finalize_added_edit validation failures. Verified with cargo test storage::repo::tests.
