# Daily Note Link Format Mismatch

Priority: high
Status: open

## Expected behavior

The earlier v2 docs describe daily-note completion entries as Obsidian wiki-links such as `- [x] [[Tasks/done/<id>|<title>]]` under `## Completed Tasks`.

## Current behavior

The implementation appends plain Markdown text in the form `- [x] Title (id)`.

## Evidence

Docs:

- `tqs_v2_cli_contract.md`
- `tqs_v2_template_and_vault_layout.md`
- `tqs_v2_design.md`

Code:

- `src/storage/daily_notes.rs`
- `src/cli/commands/done.rs`

Tests:

- `tests/cli_smoke.rs`

## Recommended fix

If Obsidian integration remains a requirement, switch daily-note rendering to the wiki-link format and keep duplicate detection idempotent for that new representation.
