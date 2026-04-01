---
id: tqs-8pl4
status: open
deps: []
links: []
created: 2026-04-01T07:53:41Z
type: chore
priority: 3
assignee: andrea.bergia
tags: [refactor]
---
# Remove unnecessary .clone() in done.rs

done.rs:25 — resolved.tasks_root and resolved.queue_dirs are cloned just because resolved is also passed to operations::mark_done. Consider having mark_done take individual config fields by reference, or making TaskRepo::new take references.

