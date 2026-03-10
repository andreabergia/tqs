# TQS v2 CLI Contract

This document defines the initial command-line contract for `tqs` v2.

The goal is to keep the CLI:

-   small
-   predictable
-   fast for terminal-heavy workflows
-   consistent with the markdown + Obsidian storage model

This is not an implementation document. It is a behavioral contract for
the user-facing CLI.

------------------------------------------------------------------------

# Design Principles

The CLI should follow these principles:

-   **fast capture first**
-   **simple queue-oriented operations**
-   **minimal mandatory arguments**
-   **human-friendly task references**
-   **safe defaults**
-   **filesystem-transparent behavior**

The CLI should optimize for the most common workflow:

1.  capture quickly
2.  review inbox
3.  move tasks to a queue
4.  work from `now`
5.  complete tasks
6.  let completion update the daily note

------------------------------------------------------------------------

# Global Assumptions

## Storage model

Tasks are stored as markdown files inside an Obsidian vault:

``` text
<Vault>/Tasks/<queue>/<id>.md
```

Queues:

-   `inbox`
-   `now`
-   `next`
-   `later`
-   `done`

## Task references

Commands that accept `<task>` should support:

-   exact task id
-   unique filename prefix
-   title substring
-   interactive selection if ambiguous

Examples:

``` bash
tqs show 20260309-103412-reply-aws-billing
tqs edit reply-aws
tqs done "reply to aws"
```

Resolution rules:

1.  exact id match wins
2.  unique filename prefix match
3.  unique title substring match
4.  if multiple matches exist, prompt user to choose
5.  if no match exists, return a clear error

------------------------------------------------------------------------

# Command Summary

Initial v2 command set:

-   `add`
-   `list`
-   `move`
-   `done`
-   `edit`
-   `show`
-   `find`
-   `now`
-   `inbox`
-   `doctor`
-   `config`

This is intentionally small.

------------------------------------------------------------------------

# Command Details

## `tqs add`

Create a new task.

### Syntax

``` bash
tqs add <title>
tqs add <title> --source <source>
tqs add <title> --tags <tag1,tag2,...>
tqs add <title> --project <project>
tqs add <title> --queue <queue>
tqs add <title> --edit
```

### Examples

``` bash
tqs add "reply to AWS billing alert"
tqs add "check failed deploy" --source teams --tags ops,prod
tqs add "prepare sprint retro" --project team-process --edit
```

### Behavior

Default behavior:

-   generate a task id
-   create a markdown task file
-   place it in `inbox`
-   populate frontmatter
-   create a minimal body template
-   print the created task id and path

If `--queue` is provided, the task is created directly in that queue.

If `--edit` is provided, the task is opened in the configured editor
after creation.

### Options

  Option                  Meaning
  ----------------------- ------------------------------------
  `--source <source>`     set task source
  `--tags <a,b,c>`        comma-separated tags
  `--project <project>`   set project field
  `--queue <queue>`       override default queue
  `--edit`                open editor after creation
  `--template <name>`     optional future template selection

### Output

Suggested output:

``` text
Created task 20260309-103412-reply-aws-billing in inbox
```

------------------------------------------------------------------------

## `tqs list`

List tasks.

### Syntax

``` bash
tqs list
tqs list <queue>
tqs list --tag <tag>
tqs list --source <source>
tqs list --project <project>
tqs list --limit <n>
```

### Examples

``` bash
tqs list
tqs list now
tqs list inbox --limit 20
tqs list --tag aws
tqs list --source teams
```

### Behavior

Default `tqs list` should show a compact dashboard-like view, for
example:

-   tasks in `now`
-   recent or pending tasks in `inbox`
-   queue counts

When a queue is specified, only that queue is shown.

Filtering flags apply across relevant queues unless a queue is
explicitly specified.

### Output

Output should be compact and terminal-friendly.

Example:

``` text
NOW
[1] Reply to AWS billing alert
[2] Review deployment logs

INBOX
[3] Reply on prod issue thread
[4] Update incident note

COUNTS
inbox: 4  now: 2  next: 9  later: 21  done: 57
```

------------------------------------------------------------------------

## `tqs now`

Convenience command for current focus.

### Syntax

``` bash
tqs now
```

### Behavior

Equivalent to:

``` bash
tqs list now
```

This command exists because it is expected to be used frequently.

------------------------------------------------------------------------

## `tqs inbox`

Convenience command for inbox review.

### Syntax

``` bash
tqs inbox
```

### Behavior

Equivalent to:

``` bash
tqs list inbox
```

This command exists because inbox review is part of the core workflow.

------------------------------------------------------------------------

## `tqs move`

Move a task between queues.

### Syntax

``` bash
tqs move <task> <queue>
```

### Examples

``` bash
tqs move reply-aws now
tqs move "update incident note" next
tqs move 20260309-111020-reply-slack-prod-issue later
```

### Behavior

-   resolve the task reference
-   move the file into the target queue folder
-   update `queue`
-   update `updated_at`

If the target queue is the same as the current queue, the command should
do nothing and report that no change was needed.

### Output

Suggested output:

``` text
Moved 20260309-103412-reply-aws-billing to now
```

------------------------------------------------------------------------

## `tqs done`

Mark a task as completed.

### Syntax

``` bash
tqs done <task>
```

### Examples

``` bash
tqs done reply-aws
tqs done "review deployment logs"
```

### Behavior

-   resolve task reference
-   move task to `done`
-   set `queue: done`
-   set `completed_at`
-   update `updated_at`
-   append completion entry to the daily note if configured

This command should be idempotent enough to avoid duplicate daily note
entries.

If the task is already in `done`, the command should either:

-   report that it is already done, or
-   repair missing completion metadata if necessary

### Output

Suggested output:

``` text
Completed 20260309-103412-reply-aws-billing
Logged completion to Daily/2026-03-09.md
```

------------------------------------------------------------------------

## `tqs edit`

Open a task in the configured editor.

### Syntax

``` bash
tqs edit <task>
```

### Examples

``` bash
tqs edit reply-aws
tqs edit 20260309-103412-reply-aws-billing
```

### Behavior

-   resolve task reference
-   open the task file using the configured editor

No task metadata should be modified unless the editor changes the file
and the implementation later decides to update `updated_at`.

------------------------------------------------------------------------

## `tqs show`

Display a task in the terminal.

### Syntax

``` bash
tqs show <task>
```

### Examples

``` bash
tqs show reply-aws
tqs show "review deployment logs"
```

### Behavior

Display:

-   key metadata
-   path
-   markdown body

Suggested layout:

``` text
ID: 20260309-103412-reply-aws-billing
Title: Reply to AWS billing alert
Queue: now
Created: 2026-03-09T10:34:12+01:00
Updated: 2026-03-09T11:20:07+01:00
Tags: aws, finance
Source: email
Project: platform-costs
Path: Tasks/now/20260309-103412-reply-aws-billing.md

# Reply to AWS billing alert

## Context

Finance asked for an explanation of an unexpected cost spike.
```

------------------------------------------------------------------------

## `tqs find`

Search tasks by metadata or text.

### Syntax

``` bash
tqs find <query>
tqs find --tag <tag>
tqs find --source <source>
tqs find --project <project>
tqs find --queue <queue>
```

### Examples

``` bash
tqs find aws
tqs find --tag finance
tqs find --source teams
tqs find --queue now
```

### Behavior

Search should operate across:

-   title
-   tags
-   source
-   project
-   markdown body

A simple text-based implementation is acceptable for v1.

### Output

Search output should be compact and referenceable.

Example:

``` text
[1] now   Reply to AWS billing alert
[2] done  Add billing notes to incident doc
[3] later Review AWS budget thresholds
```

------------------------------------------------------------------------

## `tqs doctor`

Validate configuration and storage layout.

### Syntax

``` bash
tqs doctor
```

### Behavior

Validate:

-   vault path exists
-   tasks directory exists or can be created
-   queue directories exist or can be created
-   daily notes directory is valid
-   editor is configured or discoverable

Example output:

``` text
Vault path: OK
Tasks directory: OK
Queue directories: OK
Daily notes directory: OK
Editor: OK
```

This command is intended to help users diagnose setup problems.

------------------------------------------------------------------------

## `tqs config`

Show current configuration.

### Syntax

``` bash
tqs config
```

### Behavior

Display effective configuration, such as:

-   vault path
-   tasks directory
-   daily notes directory
-   daily note date format
-   whether completion logging is enabled

This command is read-only in v1.

Example output:

``` text
vault_path = /Users/user/Obsidian/MyVault
tasks_dir = Tasks
daily_notes_dir = Daily
append_completed_to_daily_note = true
daily_note_completed_section = Completed Tasks
```

------------------------------------------------------------------------

# Exit Behavior

Commands should follow normal CLI conventions.

## Success

-   exit code `0`

## Failure

-   non-zero exit code
-   concise error message printed to stderr

Examples of failure cases:

-   task not found
-   ambiguous task reference with no interactive mode available
-   invalid queue
-   invalid configuration
-   unreadable task file

------------------------------------------------------------------------

# Queue Names

Valid queue names in v1:

-   `inbox`
-   `now`
-   `next`
-   `later`
-   `done`

Any other queue name should be rejected with a clear error.

------------------------------------------------------------------------

# Task Creation Defaults

Default `tqs add` behavior:

-   queue: `inbox`
-   source: `manual`
-   tags: `[]`
-   project: empty
-   body template:

``` markdown
# <task-title>

## Context

## Notes
```

This provides enough structure without adding too much friction.

------------------------------------------------------------------------

# Daily Note Integration Rules

When `tqs done` logs to the daily note, the implementation should:

1.  locate today's daily note
2.  create it if missing, if configured to do so
3.  ensure the target section exists
4.  append one completion line
5.  avoid duplicates for the same task

Recommended completion entry format:

``` markdown
- [x] [[Tasks/done/<task-id>|<task-title>]]
```

Recommended section heading:

``` markdown
## Completed Tasks
```

------------------------------------------------------------------------

# Minimal Command Set Recommendation

If command scope must stay very tight, the truly essential commands are:

-   `add`
-   `list`
-   `move`
-   `done`
-   `edit`
-   `show`
-   `find`

The following are convenience commands and can be added after the
essentials:

-   `now`
-   `inbox`
-   `doctor`
-   `config`

------------------------------------------------------------------------

# Future Extensions

The following may be added later, but are not part of the initial CLI
contract:

-   `refile`
-   `reopen`
-   `archive`
-   `stats`
-   `projects`
-   `templates`
-   `mcp`
-   `obsidian open`

These should only be added if they support real workflow needs.

------------------------------------------------------------------------

# Summary

The `tqs` v2 CLI should remain small and centered on the queue-based
workflow:

-   capture quickly
-   inspect easily
-   move tasks between horizons
-   complete tasks cleanly
-   keep everything as markdown inside the Obsidian vault

This contract is intended to stabilize the user-facing behavior before
implementation begins.
