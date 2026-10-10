#!/usr/bin/env bash
set -euo pipefail
mode="${1:?Choose test or bench}"
case "$mode" in
  test)
    cargo llvm-cov --all-features --lib --tests \
      --ignore-filename-regex '/tests/|/examples/|/benches/' \
      --summary-only --fail-under-lines 95
    ;;
  bench)
    cargo llvm-cov --all-features --bench crosshair \
      --ignore-filename-regex '/tests/|/examples/|/benches/' \
      --summary-only --fail-under-lines 100 -- --test
    ;;
  *)
    echo 'Choose test or bench.' >&2
    exit 2
    ;;
esac
