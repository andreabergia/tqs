---
id: tqs-l6w2
status: closed
deps: [tqs-apgz]
links: []
created: 2026-03-12T09:41:26Z
type: task
priority: 2
assignee: andrea.bergia
parent: tqs-oq3n
---
# Add tests for doctor diagnostic edge cases

Cover remaining diagnostic branches in src/storage/doctor.rs, including non-directory paths, filename mismatch diagnostics, and the warning path when queue mappings overlap and task scanning is skipped.

## Acceptance Criteria

Tests cover non-directory tasks_root or daily_notes_dir handling, non-directory queue paths, filename mismatch diagnostics, and the overlap warning that skips task scanning.


## Notes

**2026-03-12T10:47:00Z**

Added doctor edge-case coverage in src/storage/doctor.rs for non-directory tasks_root and daily_notes_dir paths, non-directory queue paths, filename mismatch diagnostics, and the overlap warning path that skips task scanning. Verified with cargo test storage::doctor::tests.
