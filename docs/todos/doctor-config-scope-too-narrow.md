# Doctor And Config Scope Too Narrow

Priority: medium
Status: open

## Expected behavior

The earlier docs expect `config` and `doctor` to expose and validate vault-oriented settings, daily-note settings, and editor availability.

## Current behavior

`config` prints `tasks_root`, `daily_notes_dir`, and queue-dir mappings. `doctor` validates generic storage health and task-file integrity, but not editor discoverability or vault-specific settings.

## Evidence

Docs:

- `tqs_v2_cli_contract.md`

Code:

- `src/io/output.rs`
- `src/storage/doctor.rs`
- `src/storage/config.rs`

## Recommended fix

If vault/editor settings remain part of the product contract, expand resolved config and diagnostics accordingly. Otherwise narrow the docs to the current generic behavior.
