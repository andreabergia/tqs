#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/release.sh <patch|minor|major|release|rc|beta|alpha|VERSION> [options] [-- <cargo-release args...>]

Options:
  --execute         Actually perform the release (default is dry-run)
  --skip-checks     Skip fmt/clippy/test/dist preflight checks
  -y, --yes         Skip confirmation prompt
  -h, --help        Show this help

Examples:
  scripts/release.sh patch
  scripts/release.sh minor --execute
  scripts/release.sh 0.2.0 --execute -- --no-verify
EOF
}

die() {
  echo "error: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

if [[ $# -eq 0 ]]; then
  usage
  exit 1
fi

level_or_version=""
execute=false
skip_checks=false
assume_yes=false
extra_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    --execute)
      execute=true
      shift
      ;;
    --skip-checks)
      skip_checks=true
      shift
      ;;
    -y|--yes)
      assume_yes=true
      shift
      ;;
    --)
      shift
      extra_args=("$@")
      break
      ;;
    -*)
      die "unknown option: $1"
      ;;
    *)
      if [[ -n "$level_or_version" ]]; then
        die "multiple release levels/versions provided: '$level_or_version' and '$1'"
      fi
      level_or_version="$1"
      shift
      ;;
  esac
done

[[ -n "$level_or_version" ]] || die "missing release level/version"

need_cmd git
need_cmd cargo
need_cmd dist
need_cmd cargo-release

repo_root="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not inside a git repository"
cd "$repo_root"

branch="$(git branch --show-current)"
[[ "$branch" == "main" ]] || die "releases must be cut from 'main' (current: '${branch:-detached}')"

if [[ -n "$(git status --porcelain)" ]]; then
  die "working tree is not clean; commit or stash changes before releasing"
fi

if [[ ! -f "release.toml" ]]; then
  die "release.toml not found at repo root"
fi

if [[ "$skip_checks" == false ]]; then
  echo "Running preflight checks..."
  cargo fmt --check
  cargo clippy -- -D warnings
  cargo test
  dist plan
fi

echo
echo "Release target: $level_or_version"
echo "Mode: $([[ "$execute" == true ]] && echo "execute" || echo "dry-run")"
echo "Config: release.toml"
echo "Reminder: update CHANGELOG.md (Unreleased section) before the final release commit/tag."
echo

if [[ "$assume_yes" == false ]]; then
  read -r -p "Continue with cargo-release? [y/N] " reply
  case "$reply" in
    y|Y|yes|YES) ;;
    *) echo "Aborted."; exit 1 ;;
  esac
fi

cmd=(cargo release "$level_or_version" -c release.toml --no-confirm)
if [[ "$execute" == true ]]; then
  cmd+=(--execute)
fi
if [[ ${#extra_args[@]} -gt 0 ]]; then
  cmd+=("${extra_args[@]}")
fi

printf 'Running:'
printf ' %q' "${cmd[@]}"
printf '\n'

"${cmd[@]}"
