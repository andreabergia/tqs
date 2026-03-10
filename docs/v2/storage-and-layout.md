# TQS v2 Storage and Layout

This document defines the filesystem layout, task schema, and Markdown conventions for `tqs` v2.

## Directory Layout

`tqs` is configured around directories rather than a specific editor or notes application.

Primary directories:

- `tasks_root`: required directory containing queue subdirectories
- `daily_notes_dir`: optional directory containing Markdown daily notes

Example:

```text
<tasks_root>/
  inbox/
  now/
  next/
  later/
  done/

<daily_notes_dir>/
  2026-03-09.md
```

## Task File Naming

Each task is stored as:

```text
<id>.md
```

The filename must remain stable after creation and must not depend on later title edits.

## Task Frontmatter Template

```yaml
---
id: 20260309-103412-reply-aws-billing
title: Reply to AWS billing alert
queue: inbox
created_at: 2026-03-09T10:34:12+01:00
updated_at: 2026-03-09T10:34:12+01:00
tags: [aws, finance]
source: manual
project:
completed_at:
daily_note:
---
```

Field meanings:

- `id`: stable unique task identifier
- `title`: human-readable title
- `queue`: current queue
- `created_at`: creation timestamp
- `updated_at`: last update timestamp
- `tags`: optional list of tags
- `source`: optional origin, such as `manual`, `email`, `meeting`, `idea`
- `project`: optional project reference
- `completed_at`: completion timestamp
- `daily_note`: optional reference to the daily note where completion was logged

## Default Markdown Body

Recommended default body:

```markdown
# <task-title>

## Context

## Notes
```

Tiny interrupt-driven tasks may remain almost empty. Richer tasks may grow into full Markdown working notes.

## Example Minimal Task

```markdown
---
id: 20260309-111020-reply-slack-prod-issue
title: Reply on prod issue thread
queue: inbox
created_at: 2026-03-09T11:10:20+01:00
updated_at: 2026-03-09T11:10:20+01:00
tags: [prod]
source: teams
project:
completed_at:
daily_note:
---

# Reply on prod issue thread
```

## Example Rich Task

```markdown
---
id: 20260309-103412-reply-aws-billing
title: Reply to AWS billing alert
queue: now
created_at: 2026-03-09T10:34:12+01:00
updated_at: 2026-03-09T11:20:07+01:00
tags: [aws, finance]
source: email
project: platform-costs
completed_at:
daily_note:
---

# Reply to AWS billing alert

## Context

Finance asked for an explanation of an unexpected cost spike.

## Notes

Possible relation to the new log ingestion pipeline.

## Next steps

- check Cost Explorer
- verify recent infrastructure changes
- reply with findings
```

## Daily Notes

Daily-note integration is generic Markdown integration.

When `daily_notes_dir` is configured, `tqs done`:

1. locates today’s note in `daily_notes_dir`
2. creates it if it does not exist
3. ensures the completed-tasks section exists
4. appends one completion line
5. avoids duplicates for the same task

Recommended default section heading:

```markdown
## Completed Tasks
```

Recommended tool-agnostic completion entry:

```markdown
- [x] Reply to AWS billing alert (20260309-103412-reply-aws-billing)
```

Users may choose directory layouts that fit editors such as Obsidian, but the core format remains generic Markdown on disk.
