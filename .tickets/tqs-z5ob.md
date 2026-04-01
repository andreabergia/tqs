---
id: tqs-z5ob
status: open
deps: []
links: []
created: 2026-04-01T07:53:40Z
type: bug
priority: 2
assignee: andrea.bergia
tags: [tui]
---
# Fix area_too_narrow ignoring actual terminal width

status_bar.rs:81-83 — area_too_narrow compares total span width to hardcoded 120 but never checks the actual Rect width. On narrow terminals the status bar overflows; on wide terminals it truncates unnecessarily. Should pass area.width and compare against it.

