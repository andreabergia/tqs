# TQS Usage

## Quick Reference

```
# Creating tasks
tqs create [summary]                          Create with summary
tqs create "Task name" --description "..."    Create with description
tqs create --id <id> "Task name"              Create with custom ID

# Listing tasks
tqs list [keywords]                           List open tasks (default)
tqs list --all                                List all tasks
tqs list --closed                             List closed tasks
tqs list --verbose                            Show extra columns
tqs list bug fix                              Filter by keywords

# Managing tasks
tqs complete [id]                             Mark as closed
tqs reopen [id]                               Mark as open
tqs info [id]                                 Show details
tqs move [old_id] [new_id]                    Change task ID
tqs delete <id>                               Delete task

# Global options
--root <path>                                 Override storage directory
TQS_ROOT                                      Environment variable for storage
```

## Global Options

### `--root <path>`

Override the default storage location.

```bash
tqs --root /custom/path create "My task"
```

### `TQS_ROOT` Environment Variable

Set a default storage location via environment variable.

```bash
export TQS_ROOT=/custom/path
tqs list
```

Storage precedence: `--root` → `TQS_ROOT` → `<git-repo>/todos` → `~/.tqs/todos`

## Exit Codes

- `0` - Success
- `1` - Runtime error
- `2` - Usage/argument error

## Commands

### Fuzzy Command Matching

Commands support fuzzy matching - enter a subset of characters in order to match:

- `tqs cr` or `tqs crte` → `tqs create`
- `tqs l` or `tqs ls` → `tqs list`
- `tqs i` or `tqs inf` → `tqs info`
- `tqs cmp` → `tqs complete`
- `tqs opn` → `tqs reopen`
- `tqs d` or `tqs del` → `tqs delete`
- `tqs m` or `tqs mov` → `tqs move`

**Priority order** for ambiguous matches (e.g., `tqs c`):
1. create
2. list
3. info
4. complete
5. reopen
6. delete
7. move

**Examples:**
```bash
# Shortest possible commands
tqs l                    # List open tasks
tqs cr "Buy groceries"   # Create a task
tqs c <task-id>          # Create a task (create > complete)
tqs i <task-id>          # Show task info

# Works with flags and arguments
tqs l --all              # List all tasks
tqs l bug fix            # Filter by keywords
tqs cr "Task" --id my-id # Create with custom ID
```

---

### `create` - Create a new task

```
tqs create [summary] [--description <text>] [--id <id>]
```

**Arguments:**
- `summary` - Task summary (optional, prompts interactively if omitted)

**Flags:**
- `--description <text>` - Task description
- `--id <id>` - Custom task ID (auto-generated if omitted)

**Examples:**
```bash
tqs create "Write documentation"
tqs create "Fix bug" --description "The login page fails"
tqs create --id my-task-id "Important task"
```

**Interactive:**
Without a TTY, you must provide the summary. With a TTY, TQS prompts for summary and optionally a multi-line description.

---

### `list` - List tasks

```
tqs list [keywords...] [--all|--closed] [--verbose]
```

**Arguments:**
- `keywords...` - Filter by keywords (AND semantics, matches id, summary, description)

**Flags:**
- `--all` - List all tasks
- `--closed` - List closed tasks only
- `--verbose` - Show status and created_at columns

**Examples:**
```bash
tqs list                              # Open tasks only
tqs list --all                        # All tasks
tqs list --closed                     # Closed tasks only
tqs list bug                          # Tasks containing "bug"
tqs list bug urgent                   # Tasks containing "bug" AND "urgent"
tqs list --verbose                    # Show extra columns
```

**Default columns:** id, summary

**Verbose columns:** id, status, created_at, summary

**Sort order:** Newest first (by created_at), then by id

---

### `complete` - Mark task as closed

```
tqs complete [id]
```

**Arguments:**
- `id` - Task ID (optional, opens interactive picker if omitted)

**Examples:**
```bash
tqs complete cobalt-urial-7f3a
tqs complete                            # Interactive picker
```

**Interactive:**
Without an ID, opens a fuzzy-select picker of open tasks. Requires a TTY.

**Behavior:**
- Already closed: Prints info message, exits successfully
- No open tasks: Prints "No open tasks available"

---

### `reopen` - Mark task as open

```
tqs reopen [id]
```

**Arguments:**
- `id` - Task ID (optional, opens interactive picker if omitted)

**Examples:**
```bash
tqs reopen cobalt-urial-7f3a
tqs reopen                             # Interactive picker
```

**Interactive:**
Without an ID, opens a fuzzy-select picker of closed tasks. Requires a TTY.

**Behavior:**
- Already open: Prints info message, exits successfully
- No closed tasks: Prints "No closed tasks available"

---

### `info` - Show task details

```
tqs info [id]
```

**Arguments:**
- `id` - Task ID (optional, opens interactive picker if omitted)

**Examples:**
```bash
tqs info cobalt-urial-7f3a
tqs info                               # Interactive picker
```

**Interactive:**
Without an ID, opens a fuzzy-select picker of all tasks. Requires a TTY.

**Output:**
Displays id, status, created_at, summary, and full markdown description.

---

### `delete` - Delete a task

```
tqs delete <id>
```

**Arguments:**
- `id` - Task ID (required)

**Examples:**
```bash
tqs delete cobalt-urial-7f3a
```

**Behavior:**
- Hard-deletes the task file
- No confirmation prompt
- Fails if task not found

---

### `move` - Change task ID

```
tqs move [old_id] [new_id]
```

**Arguments:**
- `old_id` - Current task ID (optional, opens interactive picker if omitted)
- `new_id` - New task ID (optional, prompts for input if omitted)

**Examples:**
```bash
tqs move old-id new-id
tqs move cobalt-urial-7f3a better-name-1234
tqs move                              # Interactive picker and prompt
```

**Interactive:**
Without arguments, opens a fuzzy-select picker of all tasks for the old ID, then prompts for the new ID. Requires a TTY.

**Behavior:**
- Renames the task file and updates the ID in the task
- Fails if old task ID does not exist
- Fails if new task ID already exists
- Preserves all other task properties (status, created_at, summary, description)

---

## Storage Format

Tasks are stored as Markdown files with YAML frontmatter:

```yaml
---
id: cobalt-urial-7f3a
created_at: 2026-02-20T22:15:00Z
status: open
summary: Short task summary
---

Markdown description body follows here.
```

- Files are named `<id>.md`
- Status is `open` or `closed` (lowercase)
- Description is optional
- Malformed files are skipped with a warning

## Interactive Features

Several commands support interactive mode when no ID is provided:
- `complete` - Picker for open tasks
- `reopen` - Picker for closed tasks
- `info` - Picker for all tasks
- `move` - Picker for old ID, prompt for new ID
- `create` - Prompts for summary and description (optional)

**TTY Requirement:**
Interactive features require a TTY. Without a TTY, you must provide the required arguments explicitly.

**Picker Behavior:**
- Fuzzy-select interface
- Cancel with Ctrl+C or Esc
- Success with confirmation message
