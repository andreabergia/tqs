# TQS v2 Task Template and Example Vault Layout

This document defines a practical **task file template** and an
**example Obsidian vault layout** for `tqs` v2.

The goal is to keep the format:

-   easy to create from the terminal
-   pleasant to edit in Obsidian
-   easy to parse from code
-   friendly to LLM-based tooling
-   flexible enough for both tiny and medium-sized tasks

------------------------------------------------------------------------

# Example Vault Layout

A simple default layout inside an existing Obsidian vault:

``` text
<Vault>/
  Tasks/
    inbox/
    now/
    next/
    later/
    done/
  Daily/
  Projects/
```

## Directory purposes

### `Tasks/`

Main task storage area.

### `Tasks/inbox/`

Fast capture destination for newly created tasks.

### `Tasks/now/`

Tasks intended for immediate focus.

### `Tasks/next/`

Short-term tasks for the next few days or the current week.

### `Tasks/later/`

Backlog and medium-term work.

### `Tasks/done/`

Completed tasks archive.

### `Daily/`

Daily notes used for journaling and completion logging.

### `Projects/`

Optional project notes that tasks may reference.

------------------------------------------------------------------------

# Task File Naming

Each task is stored as one markdown file.

Recommended filename format:

``` text
<id>.md
```

Example:

``` text
20260309-103412-reply-aws-billing.md
```

This keeps filenames:

-   stable
-   readable
-   sortable
-   easy to reference from the terminal

The filename should not depend on later title edits.

------------------------------------------------------------------------

# Task Frontmatter Template

Recommended frontmatter:

``` yaml
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

## Field meanings

  -----------------------------------------------------------------------
  Field                               Meaning
  ----------------------------------- -----------------------------------
  `id`                                Stable unique task identifier

  `title`                             Human-readable task title

  `queue`                             Current queue (`inbox`, `now`,
                                      `next`, `later`, `done`)

  `created_at`                        Creation timestamp

  `updated_at`                        Last update timestamp

  `tags`                              Optional list of tags

  `source`                            Origin of the task, such as
                                      `manual`, `email`, `teams`,
                                      `meeting`, `idea`

  `project`                           Optional project reference

  `completed_at`                      Completion timestamp, set when task
                                      is done

  `daily_note`                        Optional reference to the daily
                                      note where completion was logged
  -----------------------------------------------------------------------

------------------------------------------------------------------------

# Recommended Markdown Body Template

A minimal but useful default body:

``` markdown
# Reply to AWS billing alert

## Context

## Notes
```

This works well because:

-   tiny tasks can remain almost empty
-   medium tasks have obvious places for context and notes
-   Obsidian editing remains natural
-   the structure is easy to scan in the terminal

------------------------------------------------------------------------

# Minimal Task Example

A very small interrupt-driven task:

``` markdown
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

This should be the normal output of a very fast capture command.

------------------------------------------------------------------------

# Richer Task Example

A medium-sized task with notes:

``` markdown
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

This shows how the same task format can support richer working notes.

------------------------------------------------------------------------

# Queue-Specific Expectations

The task format stays the same in every queue, but expectations differ.

## Inbox

Characteristics:

-   minimal information is acceptable
-   title may be enough
-   body may be empty

## Now

Characteristics:

-   usually has enough context to act immediately
-   often benefits from some notes or next steps

## Next

Characteristics:

-   may be less detailed than `now`
-   should still be clear enough to promote later

## Later

Characteristics:

-   may represent backlog items or ideas
-   often useful to keep brief context

## Done

Characteristics:

-   should keep the final body intact
-   should have `completed_at` populated
-   may have `daily_note` populated if logged

------------------------------------------------------------------------

# Example Daily Note Layout

An example daily note:

``` markdown
# 2026-03-09

## Completed Tasks

- [x] [[Tasks/done/20260309-103412-reply-aws-billing|Reply to AWS billing alert]]
- [x] [[Tasks/done/20260309-111020-reply-slack-prod-issue|Reply on prod issue thread]]
```

## Daily note recommendations

Recommended behavior for `tqs done`:

1.  locate today's daily note
2.  create it if missing
3.  ensure the `## Completed Tasks` heading exists
4.  append the task link if it is not already present

------------------------------------------------------------------------

# Example Project Note Linkage

If `Projects/` is used, a task may reference a project in metadata:

``` yaml
project: platform-costs
```

And the task body may optionally include a wiki-link:

``` markdown
Related: [[Projects/Platform Costs]]
```

This keeps project handling lightweight while still making it useful in
Obsidian.

------------------------------------------------------------------------

# Default Template Recommendation

Recommended task template for `tqs add`:

``` markdown
---
id: <generated-id>
title: <task-title>
queue: inbox
created_at: <timestamp>
updated_at: <timestamp>
tags: []
source: manual
project:
completed_at:
daily_note:
---

# <task-title>

## Context

## Notes
```

This is a strong default because it is structured without being heavy.

------------------------------------------------------------------------

# Capture Modes

It may be useful for `tqs` to support two creation styles.

## Fast capture

Used for the quickest possible capture.

Output example:

``` markdown
---
id: <generated-id>
title: <task-title>
queue: inbox
created_at: <timestamp>
updated_at: <timestamp>
tags: []
source: manual
project:
completed_at:
daily_note:
---

# <task-title>
```

Best for:

-   messages
-   interruptions
-   quick follow-ups

## Full template capture

Used when the task is expected to need notes immediately.

Output example:

``` markdown
---
id: <generated-id>
title: <task-title>
queue: inbox
created_at: <timestamp>
updated_at: <timestamp>
tags: []
source: manual
project:
completed_at:
daily_note:
---

# <task-title>

## Context

## Notes
```

Best for:

-   medium-sized work
-   tasks created from meetings
-   tasks expected to be edited soon

------------------------------------------------------------------------

# Practical Recommendations

Recommended defaults for v1:

-   store all tasks under `Tasks/<queue>/`
-   use `<id>.md` as the filename
-   keep queue both in path and in frontmatter
-   use the full template by default
-   allow a fast capture mode later if needed
-   keep daily note logging limited to completion events
-   keep project support lightweight

------------------------------------------------------------------------

# Summary

The proposed task format and vault layout aim to make `tqs` v2:

-   simple to reason about
-   efficient from the terminal
-   pleasant to use in Obsidian
-   easy to automate
-   robust for future LLM integrations

The most important design choice is to keep **one task per markdown
file**, stored directly in queue folders inside the Obsidian vault.
