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
tqs edit [id]                                 Edit in $EDITOR
tqs move [old_id] [new_id]                    Change task ID
tqs delete <id>                               Delete task

# Aliases (examples)
tqs new [summary]                             Alias for create
tqs show [id]                                 Alias for info
tqs done [id]                                 Alias for complete
tqs open [id]                                 Alias for reopen
tqs modify [id]                               Alias for edit
tqs remove <id>                               Alias for delete
tqs rename [old_id] [new_id]                  Alias for move

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
- `tqs ed` or `tqs edi` → `tqs edit`
- `tqs cmp` → `tqs complete`
- `tqs opn` → `tqs reopen`
- `tqs d` or `tqs del` → `tqs delete`
- `tqs m` or `tqs mov` → `tqs move`

Commands also support exact aliases (shell-style synonyms):

- `tqs new` or `tqs add` → `tqs create`
- `tqs ls` → `tqs list`
- `tqs show` or `tqs view` → `tqs info`
- `tqs modify` → `tqs edit`
- `tqs done`, `tqs finish`, or `tqs close` → `tqs complete`
- `tqs open` → `tqs reopen`
- `tqs remove`, `tqs rm`, or `tqs del` → `tqs delete`
- `tqs rename` or `tqs mv` → `tqs move`

**Resolution behavior** (important for short inputs like `tqs c`):
1. Exact canonical command (e.g. `create`)
2. Exact alias (e.g. `new`)
3. Prefix/fuzzy match on canonical commands
4. Prefix/fuzzy match on aliases

If a fuzzy/alias match is ambiguous across different commands, TQS leaves it unchanged and shows the normal unknown-command error so you can type a clearer command.

**Examples:**
```bash
# Shortest possible commands
tqs l                    # List open tasks
tqs cr "Buy groceries"   # Create a task
tqs c <task-id>          # Create a task (create > complete)
tqs i <task-id>          # Show task info
tqs ed <task-id>         # Edit a task

# Aliases
tqs new "Buy groceries"  # Create a task
tqs show <task-id>       # Show task info
tqs modify <task-id>     # Edit a task
tqs done <task-id>       # Complete a task
tqs rename old-id new-id # Move/rename task ID

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

### `edit` - Edit task in editor

```
tqs edit [id]
```

**Arguments:**
- `id` - Task ID (optional, opens interactive picker if omitted)

**Examples:**
```bash
tqs edit cobalt-urial-7f3a
tqs edit                             # Interactive picker
```

**Interactive:**
Without an ID, opens a fuzzy-select picker of all tasks. Requires a TTY.

**Behavior:**
- Opens the task file in `$EDITOR` (defaults to `vi`)
- Validates the edited file after the editor closes
- Shows recovery options if the file is empty or invalid
- Validates that the ID in the file matches the filename
- Provides options to restore, rename, or abort on ID mismatch

**Recovery options:**

1. **Empty file:** Three options
   - "Restore original content" - Restore the file before editing
   - "Delete task" - Remove the task entirely
   - "Abort" - Leave the file empty and return an error

2. **Invalid format:** Two options
   - "Restore original content" - Restore the file before editing
   - "Abort" - Leave the file malformed and return an error

3. **ID mismatch:** Three options
   - "Restore original ID in file" - Keep user's edits, fix ID field
   - "Rename file to match new ID" - Move the file to the new ID
   - "Abort" - Leave the file as-is and return an error

**Editor configuration:**
Set the `EDITOR` environment variable to use your preferred editor:
```bash
export EDITOR=nvim
tqs edit <task-id>
```

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
- `edit` - Picker for all tasks
- `move` - Picker for old ID, prompt for new ID
- `create` - Prompts for summary and description (optional)

**TTY Requirement:**
Interactive features require a TTY. Without a TTY, you must provide the required arguments explicitly.

**Picker Behavior:**
- Fuzzy-select interface
- Cancel with Ctrl+C or Esc
- Success with confirmation message
