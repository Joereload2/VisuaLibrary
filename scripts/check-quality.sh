#!/usr/bin/env bash
# Visual Library — quality gate (Unix)
# Usage: ./scripts/check-quality.sh [--strict] [--skip-tests]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

STRICT=0
SKIP_TESTS=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    --skip-tests) SKIP_TESTS=1 ;;
  esac
done

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo clippy"
if [[ "$STRICT" -eq 1 ]]; then
  cargo clippy --workspace --all-targets -- -D warnings
else
  cargo clippy --workspace --all-targets -- -W clippy::correctness
fi

if [[ "$SKIP_TESTS" -eq 0 ]]; then
  echo "==> cargo test --workspace"
  cargo test --workspace
fi

echo "==> pnpm ui tsc"
pnpm --filter @visual-library/ui exec tsc --noEmit

if [[ "$SKIP_TESTS" -eq 0 ]]; then
  echo "==> pnpm ui vitest"
  pnpm --filter @visual-library/ui test
fi

echo "Quality check OK. Rules: docs/reglas-calidad-codigo.md"
