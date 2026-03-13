# TQS - Terminal Task Queue

TQS is a Rust CLI for managing queue-based tasks stored as Markdown files with YAML frontmatter.

## Quick Reference

```bash
tqs add "Reply to AWS billing alert"
tqs add "Plan rollout" --queue now
tqs list
tqs now
tqs inbox
tqs config
tqs doctor
tqs move a7k now
tqs done a7k
tqs edit a7k
tqs show a7k
tqs find billing
```

## Queue Model

Tasks live in one of five built-in queues:

- `inbox`
- `now`
- `next`
- `later`
- `done`

`tqs list` prints a compact dashboard with queue counts plus the `now` and `inbox` sections. `tqs list <queue>` prints just that queue. `tqs now` and `tqs inbox` are direct shortcuts for the two most common queue views.

## Installation

Homebrew (macOS/Linux):

```bash
brew tap andreabergia/homebrew-tap
brew install tqs
```

Build from source:

```bash
cargo build --release
```

The binary will be at `target/release/tqs`.

## Quick Start

```bash
# Capture a task in inbox
tqs add "Write v2 release notes"

# Review the default dashboard
tqs list
tqs now
tqs inbox

# Focus a task
tqs move 0f3 now

# Inspect or edit the full Markdown file
tqs show 0f3
tqs edit 0f3

# Complete the task
tqs done 0f3

# Search across all queues
tqs find latency

# Inspect effective configuration
tqs config

# Run storage and editor diagnostics
tqs doctor
```

New tasks created by `tqs add` start with this default body:

```markdown
# <title>

## Context

## Notes
```

Task arguments are resolved in this order:

1. exact id
2. unique id prefix
3. unique title substring
4. interactive picker if a TTY is available

## Storage and Configuration

TQS stores tasks as Markdown files under:

```text
<tasks_root>/<queue>/<id>.md
```

The logical queue names are always `inbox`, `now`, `next`, `later`, and `done`. The directory names under `tasks_root` can be overridden in config, so a task may be stored at `<tasks_root>/<configured-queue-dir>/<id>.md`.

`tasks_root` is resolved in this order:

1. `--root <path>`
2. `TQS_ROOT`
3. config file at `$XDG_CONFIG_HOME/tqs/config.toml` or `~/.config/tqs/config.toml`

Minimal config example:

```toml
tasks_root = "/path/to/tasks"
daily_notes_dir = "/path/to/daily-notes"

[queues]
inbox = "inbox"
now = "focus"
next = "next"
later = "later"
done = "archive"
```

Obsidian convenience config:

```toml
obsidian_vault_dir = "/path/to/My Vault"
```

When `obsidian_vault_dir` is set, TQS derives:

- `tasks_root = <vault>/Tasks`
- `daily_notes_dir = <vault>/Daily Notes`

TQS also stores allocator metadata in a hidden `.tqs/` directory:

- `<vault>/.tqs/` when `obsidian_vault_dir` is configured
- `<tasks_root>/.tqs/` otherwise

Auto-generated IDs are bare lowercase Crockford-style codes. TQS starts with 3-character IDs such as `0f3` and grows to 4, 5, and more characters only as earlier widths are exhausted.

`obsidian_vault_dir` is a shortcut for the generic filesystem model. It cannot be combined with `tasks_root`, `daily_notes_dir`, or queue directory overrides.

If the config file uses relative paths, they are resolved relative to the config file directory. Queue directory overrides must be a single path segment.

Task frontmatter uses the current v2 schema:

```yaml
---
id: 20260309-103412-reply-aws-billing
title: Reply to AWS billing alert
queue: inbox
created_at: 2026-03-09T10:34:12Z
updated_at: 2026-03-09T10:34:12Z
completed_at:
daily_note:
---
```

## Daily Notes

If `daily_notes_dir` is configured, `tqs done` appends a completion entry to today’s Markdown daily note and records the note name in `daily_note`. Re-running `tqs done` for an already completed task is idempotent.

## Obsidian

TQS is filesystem-first: it works with plain Markdown task files in any directory layout that matches the configured paths. Obsidian is supported as a friendly workflow, not a separate storage mode.

Recommended vault layout:

```text
<vault>/
  Tasks/
    inbox/
    now/
    next/
    later/
    done/
  Daily Notes/
    YYYY-MM-DD.md
```

Using `obsidian_vault_dir` configures exactly this layout. Daily-note completion entries are written as wiki-links to the completed task file, for example `- [x] [[Tasks/done/task-1|Ship v2]]`. For other supported layouts, the link target is derived from the configured task and daily-note directories.

## Learn More

- [USAGE.md](USAGE.md) - Current CLI reference
- [ARCHITECTURE.md](ARCHITECTURE.md) - Current code structure and data flow

## Maintainer Release Process

Releases are built and published by GitHub Actions from version tags.

### One-time setup

Install the maintainer tools:

```bash
cargo install cargo-dist cargo-release
```

### Preflight checks

Run the standard checks locally before cutting a release:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
dist plan
```

### Cut a release

Update `CHANGELOG.md` (`Unreleased` section), then run:

```bash
scripts/release.sh patch --execute
```

Use `minor`, `major`, or an exact version instead of `patch` as needed.
