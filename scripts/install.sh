#!/usr/bin/env bash
set -euo pipefail

REPO_URL="${SSHDECK_REPO_URL:-https://github.com/PLASMA-FR/sshdeck}"
BRANCH="${SSHDECK_BRANCH:-}"
TAG="${SSHDECK_TAG:-}"
REV="${SSHDECK_REV:-}"
DRY_RUN=0

usage() {
  cat <<'USAGE'
Install SSHDeck from source with Cargo.

Usage:
  scripts/install.sh [--dry-run]
  curl -fsSL https://raw.githubusercontent.com/PLASMA-FR/sshdeck/main/scripts/install.sh | bash

Environment:
  SSHDECK_REPO_URL   Git repository URL to install from when not in a checkout
  SSHDECK_BRANCH     Install a specific branch
  SSHDECK_TAG        Install a specific tag
  SSHDECK_REV        Install a specific git revision

The script uses `cargo install --locked` so Cargo.lock is respected.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown option: %s\n\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Missing required command: %s\n' "$1" >&2
    exit 1
  fi
}

run() {
  printf '+'
  printf ' %q' "$@"
  printf '\n'
  if [ "$DRY_RUN" -eq 0 ]; then
    "$@"
  fi
}

need cargo

SCRIPT_DIR=""
if [ "${BASH_SOURCE[0]:-}" != "" ]; then
  SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd -P || true)"
fi

CHECKOUT_ROOT=""
if [ "$SCRIPT_DIR" != "" ] && [ -f "$SCRIPT_DIR/../Cargo.toml" ]; then
  CHECKOUT_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
elif [ -f "Cargo.toml" ] && grep -q '^name = "sshdeck"$' Cargo.toml; then
  CHECKOUT_ROOT="$(pwd -P)"
fi

printf 'SSHDeck installer\n'

if [ "$CHECKOUT_ROOT" != "" ]; then
  printf 'Installing from local checkout: %s\n' "$CHECKOUT_ROOT"
  run cargo install --locked --path "$CHECKOUT_ROOT"
else
  args=(cargo install --locked --git "$REPO_URL")
  if [ "$BRANCH" != "" ]; then
    args+=(--branch "$BRANCH")
  fi
  if [ "$TAG" != "" ]; then
    args+=(--tag "$TAG")
  fi
  if [ "$REV" != "" ]; then
    args+=(--rev "$REV")
  fi
  printf 'Installing from git: %s\n' "$REPO_URL"
  run "${args[@]}"
fi

if [ "$DRY_RUN" -eq 0 ]; then
  printf '\nInstalled. Try:\n  sshdeck doctor\n  sshdeck\n'
fi
