#!/usr/bin/env bash
set -euo pipefail

# Run from the native crate directory
cd native/obscurax

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo clippy"
cargo clippy -- -D warnings
