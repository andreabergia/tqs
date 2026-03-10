# Find Missing Structured Flags

Priority: medium
Status: open

## Expected behavior

The earlier CLI contract describes `tqs find --tag`, `--source`, `--project`, and `--queue` in addition to plain free-text search.

## Current behavior

The implementation accepts only one positional query and performs a broad substring match across task fields.

## Evidence

Docs:

- `tqs_v2_cli_contract.md`

Code:

- `src/cli/commands/find.rs`
- `src/domain/filter.rs`

## Recommended fix

Either implement the missing structured flags or explicitly freeze `find` as free-text only and update the authoritative docs to match.
