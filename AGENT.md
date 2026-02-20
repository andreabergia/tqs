# AGENT.md

## Project
`tqs` is a Rust terminal task queue CLI.  
It manages tasks stored as Markdown files with YAML frontmatter.

## Scope (v1)
- Implement CLI commands: `create`, `list`, `complete`, `reopen`, `info`, `delete`.
- Support interactive picker flows where command behavior requires selection.
- Enforce coherent CLI behavior for output streams, exit codes, and malformed-file handling.
- Keep v1 focused on file-backed task management (no JSON output, editor integration, or daemon mode).

## Core Commands
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo run -- <command>`

## Rules
- We use Conventional Commits format.
- Keep commit messages short and focused.
- Always run tests, formatter, and linter before declaring a task "done".
- Treat `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` as required quality gates for each implementation step.
- When implementing a plan, split the work into multiple logical git commits.
