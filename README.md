# TQS - Terminal Task Queue

## Quick Reference

```
tqs create [summary]         Create a new task
tqs create --id <id>         Create with custom ID
tqs list [keywords]          List open tasks
tqs list --all               List all tasks
tqs list --closed            List closed tasks
tqs complete [id]            Mark task as closed
tqs reopen [id]              Mark task as open
tqs info [id]                Show task details
tqs edit [id]                Edit task in $EDITOR
tqs move [old_id] [new_id]   Change task ID
tqs delete <id>              Delete a task

# Aliases work too!
tqs new [summary]            Alias for create
tqs show <id>                Alias for info
tqs done <id>                Alias for complete
tqs open <id>                Alias for reopen
tqs modify <id>              Alias for edit
tqs remove <id>              Alias for delete
tqs rename <old> <new>       Alias for move

# Fuzzy commands work too!
tqs cr [summary]             Create (shorter)
tqs l                        List
tqs i <id>                   Info
tqs ed [id]                  Edit
tqs c [summary]              Create (create > complete)
```

## What is TQS?

TQS is a simple command-line task manager that stores tasks as Markdown files. Perfect for tracking work in Git repositories.

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

Download release binaries:

- GitHub Releases publishes `tqs` archives for Linux x86_64 and macOS arm64.
- Stable releases also update the Homebrew formula in `andreabergia/homebrew-tap`.
- Each release includes a checksum file for verification.

## Quick Start

```bash
# Create a task
tqs create "Write documentation"

# List open tasks
tqs list

# View task details
tqs info <task-id>

# Edit a task in your editor
tqs edit <task-id>

# Mark task as complete
tqs complete <task-id>

# List completed tasks
tqs list --closed

# Or use fuzzy commands!
tqs cr "Write documentation"
tqs l
tqs i <task-id>
tqs ed <task-id>

# Or aliases (shell-style)
tqs new "Write documentation"
tqs show <task-id>
tqs done <task-id>
tqs modify <task-id>
```

## Storage Location

Tasks are stored as Markdown files with YAML frontmatter. The storage location follows this precedence:

1. `--root <path>` flag
2. `TQS_ROOT` environment variable
3. `<git-repo>/todos` (if in a Git repository)
4. `$XDG_DATA_HOME/tqs/todos` (defaults to `~/.local/share/tqs/todos`)

Each task is saved as `<storage-root>/<task-id>.md`.

## Learn More

- [USAGE.md](USAGE.md) - Complete command reference
- [ARCHITECTURE.md](ARCHITECTURE.md) - How it works internally

## Maintainer Release Process

Releases are built and published by GitHub Actions from version tags.

### One-time setup

Install the maintainer tools:

```bash
cargo install cargo-dist cargo-release
```

Or use the helper script in this repo (still requires those tools).

### Preflight checks

Run the standard checks locally before cutting a release:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
dist plan
```

### Cut a release

Update `CHANGELOG.md` (`Unreleased` section), then run one command:

```bash
scripts/release.sh patch --execute
```

Use `minor`, `major`, or an exact version instead of `patch` as needed.

This will:

- bump `Cargo.toml` version
- create a `vX.Y.Z` git tag
- push the release commit and tag to `origin`

Pushing the tag triggers `.github/workflows/release.yml`, which builds archives and checksums and publishes them to GitHub Releases.
For stable releases, the same workflow also updates the Homebrew formula in
`andreabergia/homebrew-tap`. Prereleases (`alpha`, `beta`, `rc`) do not update Homebrew.

To preview a release without making changes:

```bash
scripts/release.sh patch
```
