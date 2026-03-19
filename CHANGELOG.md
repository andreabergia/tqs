# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project uses SemVer tags (`vX.Y.Z`).

## [Unreleased]

### Added

- `start` command — moves a task to the now queue (shortcut for `move <task> now`).

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
