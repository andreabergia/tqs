---
id: tqs-2ssh
status: open
deps: []
links: []
created: 2026-04-01T07:53:47Z
type: chore
priority: 3
assignee: andrea.bergia
tags: [tui, refactor]
---
# TUI editor bypass of TaskRepo for file I/O

tui/mod.rs run_editor reads/writes task files directly (fs::read_to_string, fs::write) instead of going through TaskRepo. If TaskRepo ever adds validation or locking, the TUI editor path would bypass it. Consider routing the empty-file check and restore through the repo.

