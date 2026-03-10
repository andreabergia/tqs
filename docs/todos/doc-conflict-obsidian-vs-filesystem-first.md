# Doc Conflict Obsidian Vs Filesystem First

Priority: high
Status: open decision

## Conflict

The earlier top-level v2 docs consistently describe an Obsidian-first product, but the shipped implementation behaves like a generic filesystem-first task tool.

## Current behavior

Core storage and commands work without any Obsidian-specific concepts beyond optional daily-note integration.

## Evidence

Docs:

- `tqs_v2_design.md`
- `tqs_design_refined.md`
- `tqs_v2_cli_contract.md`
- `tqs_v2_template_and_vault_layout.md`

Code:

- `src/storage/config.rs`
- `src/storage/repo.rs`
- `src/storage/daily_notes.rs`

## Recommended fix

Make one direction authoritative. Either restore Obsidian-first concepts in config, paths, and note linking, or rewrite the remaining product docs to describe the current generic model.
