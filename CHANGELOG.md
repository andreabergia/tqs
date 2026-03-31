# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project uses SemVer tags (`vX.Y.Z`).

## [0.2.3] - 2026-03-31

### Added

- `delete` command — permanently removes a task file. Supports `--interactive` (`-i`) flag to prompt for confirmation before deleting.

### Changed

- Dashboard (`list` without a queue) now shows **now**, **next**, a separator, then **inbox** (previously: now, inbox, next).

## [0.2.2] - 2026-03-24

### Added

- `triage` command — interactively walk through inbox tasks and dispatch them to queues, mark done, edit, or delete.
- `doctor` now detects orphaned ID-generator state files in `.tqs/id-generator/` and warns about them.
- `doctor --fix` removes orphaned state files automatically.

### Fixed

- Config file paths starting with `~` (e.g. `tasks_root = "~/o/tasks"`) are now correctly expanded to the user's home directory instead of being treated as relative to the config file location.

## [0.2.1] - 2026-03-23

### Added

- `start` command — moves a task to the now queue (shortcut for `move <task> now`).
- Running `tqs` with no arguments now shows the task dashboard if a config and tasks exist, or a getting-started guide otherwise.

## [0.2.0] - 2026-03-18

Complete rework, changed totally the design - it went from a "per project task queue" to a "personal todo list with some optional Obsidian integration". I'm not going to document the changes because it makes little sense; go read the README or ARCHITECTURE if you are interested in the new system.

## [0.1.2] - 2026-02-25

### Fixed

- Fuzzy subcommand expansion now works when global options with values appear before the command shorthand, so `tqs --root /tmp l` resolves to `list`.

## [0.1.1] - 2026-02-24

### Added

- Homebrew formula.

## [0.1.0] - 2026-02-24

### Added

- GitHub Actions release automation with `cargo-dist` and tag-driven publishing.
- Initial release.
