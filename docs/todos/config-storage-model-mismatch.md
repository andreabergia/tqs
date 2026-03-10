# Config Storage Model Mismatch

Priority: high
Status: open

## Expected behavior

The earlier v2 docs describe an Obsidian-first configuration model built around `vault_path`, `tasks_dir`, and `daily_notes_dir`, with task files stored under `<Vault>/Tasks/<queue>/<id>.md`.

## Current behavior

The implementation resolves a generic `tasks_root`, an optional `daily_notes_dir`, and optional queue-directory overrides. There is no `vault_path` or `tasks_dir` concept.

## Evidence

Docs:

- `tqs_v2_cli_contract.md`
- `tqs_design_refined.md`
- `tqs_v2_roadmap.md`

Code:

- `src/storage/config.rs`
- `src/storage/repo.rs`

## Recommended fix

Decide whether the product is Obsidian-first or generic filesystem-first. Then align config shape, storage path construction, and user-facing docs around that single model.
