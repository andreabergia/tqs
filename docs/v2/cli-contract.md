# TQS v2 CLI Contract

This document defines the lean-core command contract for `tqs` v2.

## Design Principles

- fast capture first
- simple queue-oriented operations
- minimal mandatory arguments
- human-friendly task references
- safe defaults
- filesystem-transparent behavior

## Global Assumptions

### Storage Model

Tasks are stored as Markdown files under:

```text
<tasks_root>/<queue>/<id>.md
```

Valid queues:

- `inbox`
- `now`
- `next`
- `later`
- `done`

### Task References

Commands that accept `<task>` support:

- exact task id
- unique filename prefix
- unique title substring
- interactive selection if ambiguous

Resolution rules:

1. exact id match wins
2. unique filename prefix match
3. unique title substring match
4. if multiple matches exist, prompt the user to choose
5. if no match exists, return a clear error

## Command Set

The first implementation milestone includes:

- `add`
- `list`
- `move`
- `done`
- `edit`
- `show`
- `find`

Deferred commands:

- `now`
- `inbox`
- `doctor`
- `config`

## Commands

### `tqs add`

```bash
tqs add <title>
tqs add <title> --source <source>
tqs add <title> --tags <tag1,tag2,...>
tqs add <title> --project <project>
tqs add <title> --queue <queue>
tqs add <title> --edit
```

Behavior:

- generate a task id
- create a Markdown task file
- default to `inbox`
- populate frontmatter
- create the default body template
- print the created task id and path

### `tqs list`

```bash
tqs list
tqs list <queue>
```

Behavior:

- `tqs list` shows a compact default dashboard
- default recommendation: show `now`, `inbox`, and queue counts
- `tqs list <queue>` shows only that queue

Advanced filters remain out of scope for the first milestone.

### `tqs move`

```bash
tqs move <task> <queue>
```

Behavior:

- resolve the task reference
- move the file into the target queue directory
- update `queue`
- update `updated_at`
- if the task is already in that queue, report that no change was needed

### `tqs done`

```bash
tqs done <task>
```

Behavior:

- resolve the task reference
- move the task to `done`
- set `queue: done`
- set `completed_at`
- update `updated_at`
- report idempotently if the task is already done

Daily-note append behavior is documented separately and deferred from the first implementation milestone.

### `tqs edit`

```bash
tqs edit <task>
```

Behavior:

- resolve the task reference
- open the task file in the configured editor

### `tqs show`

```bash
tqs show <task>
```

Behavior:

- display key metadata
- display the resolved file path
- display the Markdown body

### `tqs find`

```bash
tqs find <query>
```

Behavior:

- search across tasks in all queues
- simple text-based search is acceptable for v1
- search should cover the title and body at minimum
- searching tags, source, and project is recommended if implemented in the first milestone

## Exit Behavior

- success: exit code `0`
- failure: non-zero exit code with concise stderr output

Typical failures:

- task not found
- ambiguous task reference with no interactive selection available
- invalid queue
- invalid configuration
- unreadable or malformed task file
