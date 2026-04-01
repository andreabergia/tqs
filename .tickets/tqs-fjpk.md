---
id: tqs-fjpk
status: open
deps: []
links: []
created: 2026-04-01T07:53:45Z
type: task
priority: 3
assignee: andrea.bergia
tags: [ux]
---
# Restore getting-started guide for non-TUI path

handlers.rs:42 — the getting-started guide was removed entirely. A first-time user running tqs --no-tui (or piped) with no tasks gets an empty dashboard instead of setup instructions. Consider keeping the guide for the non-TUI code path when tasks are empty.

