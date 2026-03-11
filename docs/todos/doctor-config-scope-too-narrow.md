# Doctor And Config Scope Too Narrow

Priority: medium
Status: open

## Expected behavior

The earlier docs expect `config` and `doctor` to expose and validate vault-oriented settings, daily-note settings, and editor availability.

## Current behavior

`config` prints `obsidian_vault_dir` when configured, along with the resolved generic paths and queue-dir mappings. `doctor` validates generic storage health and task-file integrity, but not editor discoverability.

## Evidence

Docs:

- `tqs_v2_cli_contract.md`

Code:

- `src/io/output.rs`
- `src/storage/doctor.rs`
- `src/storage/config.rs`

## Recommended fix

Vault-specific configuration is no longer a separate contract, so this gap is now limited to editor discoverability. If editor checks remain a requirement, narrow this todo to that behavior explicitly.
