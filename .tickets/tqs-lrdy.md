---
id: tqs-lrdy
status: open
deps: []
links: []
created: 2026-04-01T07:53:51Z
type: task
priority: 3
assignee: andrea.bergia
tags: [tui, testing]
---
# Add unit tests for TUI event handlers

The TUI event handlers (event.rs handle_key and friends) are pure functions taking TuiApp + KeyEvent and returning SideEffect. These are straightforward to unit test without a terminal. Would improve confidence in mode transitions and key dispatch.

