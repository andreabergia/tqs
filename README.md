# TQS - Terminal Task Queue

A terminal-native task manager. Tasks are Markdown files organized in queues. Run `tqs` to get a full-screen dashboard, or use CLI commands for scripting and automation.

<!-- TODO: add a screenshot of the dashboard here -->

## The Dashboard

Just run `tqs`. You get a three-panel view: queues on the left, tasks in the middle, detail on the right.

```text
┌ Queues ──┬ Tasks in queue now (2) ──┬ Task detail: Ship v2 ──┐
│ > now  2 │ > 0f3  Ship v2           │ # Ship v2              │
│   next 1 │   a7k  Plan release      │                        │
│   later 3│                          │ Body text goes here    │
│   inbox 5│                          │                        │
│   done 12│                          │                        │
└──────────┴──────────────────────────┴────────────────────────┘
 [Normal] h/l:panel j/k:nav Tab:queue a:add d:done s:start q:quit
```

Navigate with `h/l` between panels, `j/k` within them. Everything else is a single keypress away:

| Key | Action |
|-----|--------|
| `a` | Add a task |
| `e` | Edit in your `$EDITOR` |
| `d` | Mark done |
| `s` | Start (move to now) |
| `m` | Move to another queue |
| `x` | Delete |
| `/` | Search across all queues |
| `t` | Triage inbox |
| `q` | Quit |

Use `tqs --no-tui` if you want the old plain-text output.

## Install

Homebrew:

```bash
brew tap andreabergia/homebrew-tap
brew install tqs
```

From source:

```bash
cargo install --path .
```

## Setup

Create `~/.config/tqs/config.toml`:

```toml
tasks_root = "/path/to/tasks"
```

Or, if you use Obsidian:

```toml
obsidian_vault_dir = "/path/to/My Vault"
```

That's it. Run `tqs config` to verify, `tqs doctor` to check for problems.

## How It Works

Tasks live in five queues: **inbox**, **now**, **next**, **later**, **done**. Each task is a Markdown file with YAML frontmatter, stored under `<tasks_root>/<queue>/<id>.md`.

The typical workflow:

```bash
tqs add "Reply to billing alert"    # lands in inbox
tqs                                  # open dashboard, triage, move tasks around
tqs done 0f3                         # mark it done
```

You can also do everything from the CLI:

```bash
tqs add "Plan rollout" --queue now
tqs start 0f3
tqs move a7k later
tqs find billing
tqs triage
```

Tasks are resolved by exact ID, unique ID prefix, or unique title substring. If ambiguous and you're on a TTY, you get an interactive picker.

## Optional Features

**Daily notes** -- if you set `daily_notes_dir`, completing a task appends a wiki-link entry to today's daily note.

**Obsidian integration** -- `obsidian_vault_dir` is a shortcut that sets `tasks_root` to `<vault>/Tasks` and `daily_notes_dir` to `<vault>/Daily Notes`.

**Custom queue directories** -- rename the on-disk folders without changing the queue names:

```toml
[queues]
now = "focus"
done = "archive"
```

## Learn More

- [USAGE.md](USAGE.md) -- full CLI and dashboard reference
- [ARCHITECTURE.md](ARCHITECTURE.md) -- code structure and data flow
- [CHANGELOG.md](CHANGELOG.md) -- release history

## Releasing

```bash
scripts/release.sh patch --execute
```

See [USAGE.md](USAGE.md) for preflight checks and detailed process.
