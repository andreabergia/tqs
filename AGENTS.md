# AGENT.md

## Project
`tqs` is a Rust CLI for managing tasks stored as Markdown files with YAML frontmatter.

Tasks are organized in queues by means of folders. The location of the queues can be customized by a config file. Optionally, a "daily note" markdown file can be configured to be updated as tasks are closed.

## Development Commands
- `cargo fmt --check` - formatting
- `cargo clippy -- -D warnings` - lint
- `cargo test` - tests
- `cargo run -- <command>` - run the CLI locally

## Rules
- Use Conventional Commits
- Keep commits short and focused
- Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` before marking work done
- Split work into logical commits
- Keep documentation updated when behavior changes
