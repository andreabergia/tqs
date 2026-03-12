---
id: tqs-sg15
status: open
deps: [tqs-txb9, tqs-apgz]
links: []
created: 2026-03-12T09:41:07Z
type: task
priority: 1
assignee: andrea.bergia
parent: tqs-oq3n
tags: [testing, coverage, cli]
---
# Add tests for task reference resolution branches

Cover the untested control paths in src/cli/commands/helpers.rs that resolve task references. This includes exact ID precedence, unique ID-prefix matches, no-task behavior, and cancellation/ambiguity handling.

## Acceptance Criteria

Tests cover exact ID matches taking precedence over title matches, unique prefix resolution, the empty-repo path, and cancellation or ambiguous selection behavior without a TTY.

