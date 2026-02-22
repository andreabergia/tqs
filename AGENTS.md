# AGENT.md

## Project
`tqs` - Rust terminal task queue CLI. Tasks stored as Markdown files with YAML frontmatter.

## Features and architecture
- CLI commands: `create`, `list`, `complete`, `reopen`, `info`, `delete`
- Interactive picker flows for selections
- Coherent CLI: output streams, exit codes, malformed-file handling
- File-backed only (no JSON, editor integration, or daemon)
- The codebase is organized into clear layers: CLI parsing, command handlers, domain logic, storage, and I/O

### Data Flow

```
CLI (args.rs)
    ↓
Handlers (handlers.rs)
    ↓
App Service (service.rs)
    ↓
Task Repository (repo.rs)
    ↓
Markdown Files (<root>/<id>.md)
```

## Commands
- `cargo fmt --check` - formatting
- `cargo clippy -- -D warnings` - lint
- `cargo test` - tests
- `cargo run -- <command>` - run

## Rules
- Conventional Commits format
- Keep commits short and focused
- Run tests, fmt, clippy before marking done
- Split work into logical commits
- Keep docs updated

