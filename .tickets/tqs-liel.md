---
id: tqs-liel
status: open
deps: []
links: []
created: 2026-04-01T07:53:43Z
type: bug
priority: 3
assignee: andrea.bergia
tags: [tui]
---
# active_filter returns All for Separator sidebar entry

app_state.rs:251 — active_filter silently returns QueueFilter::All when the sidebar index points to a Separator. The comment says 'shouldn't happen' but there's no enforcement. Consider using unreachable!() or a debug_assert to catch bugs early.

