# Edit Updates Metadata Without Content Change

Priority: medium
Status: open

## Expected behavior

The earlier CLI contract says task metadata should not change unless the edited file content actually changes.

## Current behavior

After every successful edit, the implementation reparses and normalizes the task, which updates `updated_at` even when the editor makes no content changes.

## Evidence

Docs:

- `tqs_v2_cli_contract.md`

Code:

- `src/cli/commands/edit.rs`
- `src/storage/repo.rs`
- `src/domain/task.rs`

## Recommended fix

Detect no-op edits before rewriting the task, or compare parsed task content and skip normalization when nothing changed.
