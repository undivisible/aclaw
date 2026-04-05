#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"
cargo build --release --locked
exec ./target/release/unthinkclaw-install install --binary "$ROOT/target/release/unthinkclaw"
