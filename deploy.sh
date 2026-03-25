#!/usr/bin/env bash
set -e

echo "→ Cross-compiling for Linux..."
cargo zigbuild --target x86_64-unknown-linux-gnu --release

echo "→ Deploying to Cloudflare..."
npx wrangler deploy
