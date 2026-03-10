# TQS - Terminal Task Queue

TQS is a Rust CLI for managing queue-based tasks stored as Markdown files with YAML frontmatter.

## Quick Reference

```bash
tqs add "Reply to AWS billing alert"
tqs add "Plan rollout" --queue now --tags ops,release --project platform
tqs list
tqs now
tqs inbox
tqs config
tqs doctor
tqs move 20260309-aws now
tqs done 20260309-aws
tqs edit 20260309-aws
tqs show 20260309-aws
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

# Add metadata at creation time
tqs add "Investigate API latency" --tags api,perf --source pager --project platform

# Review the default dashboard
tqs list
tqs now
tqs inbox

# Focus a task
tqs move 20260309-api now

# Inspect or edit the full Markdown file
tqs show 20260309-api
tqs edit 20260309-api

# Complete the task
tqs done 20260309-api

# Search across all queues
tqs find latency

# Inspect effective configuration
tqs config

# Run storage diagnostics
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

If the config file uses relative paths, they are resolved relative to the config file directory. Queue directory overrides must be a single path segment.

Task frontmatter uses the current v2 schema:

```yaml
---
id: 20260309-103412-reply-aws-billing
title: Reply to AWS billing alert
queue: inbox
created_at: 2026-03-09T10:34:12Z
updated_at: 2026-03-09T10:34:12Z
tags: [aws, finance]
source: email
project: platform-costs
completed_at:
daily_note:
---
```

## Daily Notes

If `daily_notes_dir` is configured, `tqs done` appends a completion entry to today’s Markdown daily note and records the note name in `daily_note`. Re-running `tqs done` for an already completed task is idempotent.

## Learn More

- [USAGE.md](USAGE.md) - Current CLI reference
- [ARCHITECTURE.md](ARCHITECTURE.md) - Current code structure and data flow
- [docs/todos/README.md](docs/todos/README.md) - Open gaps and unresolved product decisions

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
