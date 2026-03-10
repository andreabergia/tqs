# Add Default Source Is Not Manual

Priority: low
Status: open

## Expected behavior

Several earlier v2 docs show `source: manual` as the default task frontmatter value produced by `tqs add`.

## Current behavior

The implementation leaves `source` unset unless the user passes `--source`.

## Evidence

Docs:

- `tqs_v2_cli_contract.md`
- `tqs_v2_template_and_vault_layout.md`
- `tqs_design_refined.md`
- `tqs_v2_design.md`

Code:

- `src/domain/task.rs`
- `src/cli/commands/add.rs`

## Recommended fix

Choose one default explicitly. Either set `source` to `manual` in new tasks or remove that expectation from the docs.
