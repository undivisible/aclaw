#!/usr/bin/env node
/**
 * unthinkclaw MCP launcher for Poke SDK.
 *
 * This script launches the unthinkclaw binary in MCP stdio mode,
 * bridging stdin/stdout so Poke can communicate with it directly.
 *
 * Usage (Poke registers this via):
 *   bunx poke@latest mcp add --name unthinkclaw stdio://node /path/to/poke-sdk/mcp-launcher.js
 *
 * Or run directly:
 *   node poke-sdk/mcp-launcher.js
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import path from 'path';
import fs from 'fs';
import { createRequire } from 'module';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '..');

// Resolve the unthinkclaw binary path
const BINARY = path.join(REPO_ROOT, 'target', 'release', 'unthinkclaw');
const CONFIG = path.join(REPO_ROOT, 'unthinkclaw.json');

if (!fs.existsSync(BINARY)) {
  process.stderr.write(
    `[unthinkclaw-mcp] Binary not found at ${BINARY}\n` +
    `[unthinkclaw-mcp] Build it first: cd ${REPO_ROOT} && cargo build --release\n`
  );
  process.exit(1);
}

const args = ['mcp', '--config', CONFIG];

// Pass through any extra args (e.g. --workspace, --model)
if (process.argv.length > 2) {
  args.push(...process.argv.slice(2));
}

process.stderr.write(`[unthinkclaw-mcp] Launching ${BINARY} ${args.join(' ')}\n`);

const child = spawn(BINARY, args, {
  stdio: ['inherit', 'inherit', 'inherit'],
  cwd: REPO_ROOT,
  env: {
    ...process.env,
    // Load .env values if not already set
    RUST_LOG: process.env.RUST_LOG || 'warn',
  },
});

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 0);
  }
});

process.on('SIGTERM', () => child.kill('SIGTERM'));
process.on('SIGINT', () => child.kill('SIGINT'));
