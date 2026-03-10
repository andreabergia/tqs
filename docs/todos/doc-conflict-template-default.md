# Doc Conflict Template Default

Priority: low
Status: open decision

## Conflict

The earlier docs disagree on task creation defaults:

- some describe `add` as using a minimal template
- others recommend the full `Context` and `Notes` template by default
- the template doc also suggests a future fast-capture mode as a separate option

## Current behavior

The implementation uses the fuller default body template with `## Context` and `## Notes`.

## Evidence

Docs:

- `tqs_v2_design.md`
- `tqs_design_refined.md`
- `tqs_v2_template_and_vault_layout.md`
- `tqs_v2_roadmap.md`

Code:

- `src/domain/task.rs`

## Recommended fix

Decide whether `add` should default to minimal capture or full-template capture, then keep only one documented answer.
