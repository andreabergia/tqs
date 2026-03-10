# List Missing Filter Flags

Priority: medium
Status: open

## Expected behavior

The earlier CLI contract describes `tqs list --tag`, `--source`, `--project`, and `--limit`, in addition to dashboard and single-queue views.

## Current behavior

The implementation supports only `tqs list` and `tqs list <queue>`.

## Evidence

Docs:

- `tqs_v2_cli_contract.md`

Code:

- `src/cli/commands/list.rs`

## Recommended fix

Either implement the filtering and limiting flags or make the reduced `list` surface the only supported contract.
