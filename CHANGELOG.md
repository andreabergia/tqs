# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project uses SemVer tags (`vX.Y.Z`).

## [0.1.2] - 2026-02-25

### Fixed

- Fuzzy subcommand expansion now works when boolean global flags (for example `-g` / `--global`) appear before the command shorthand, so `tqs -g l` resolves to `list`.

## [0.1.1] - 2026-02-24

### Added

- Homebrew formula.

## [0.1.0] - 2026-02-24

### Added

- GitHub Actions release automation with `cargo-dist` and tag-driven publishing.
- Initial release.
