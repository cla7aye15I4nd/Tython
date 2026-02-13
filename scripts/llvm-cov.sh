#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IGNORE_REGEX='.*/src/bin/cargo-cmin\.rs$'

cd "$REPO_ROOT"

if ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "error: cargo-llvm-cov is not installed."
  echo "install: cargo install cargo-llvm-cov"
  exit 1
fi

exec cargo +nightly llvm-cov --ignore-filename-regex "$IGNORE_REGEX" "$@" --html --branch
