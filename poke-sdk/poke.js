#!/usr/bin/env node
/**
 * poke.js — primary Poke SDK entry point for unthinkclaw.
 *
 * Starts the unthinkclaw binary in MCP HTTP mode so Poke can connect to it.
 *
 * Usage:
 *   node poke-sdk/poke.js [--port 3333] [--config unthinkclaw.json]
 *
 * After starting, register with Poke:
 *   bunx poke@latest mcp add http://localhost:<PORT>/mcp --name unthinkclaw
 *
 * One-shot: start + register automatically:
 *   node poke-sdk/poke.js && node poke-sdk/register.js
 *
 * For stdio-based MCP clients (not Poke), use mcp-launcher.js instead.
 */

import { spawn } from "child_process";
import { existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Parse CLI args
const args = process.argv.slice(2);
const portIdx = args.indexOf("--port");
const PORT = portIdx !== -1 ? parseInt(args[portIdx + 1], 10) : 3333;
const configIdx = args.indexOf("--config");
const CONFIG = configIdx !== -1 ? args[configIdx + 1] : "unthinkclaw.json";

// Locate binary: prefer release build, fall back to PATH
const binary =
  [
    resolve(__dirname, "../target/release/unthinkclaw"),
    resolve(__dirname, "../target/debug/unthinkclaw"),
  ].find(existsSync) ?? "unthinkclaw";

const mcpArgs = ["mcp", "--port", String(PORT), "--config", CONFIG];

process.stderr.write(`Starting unthinkclaw MCP server on port ${PORT}...\n`);
process.stderr.write(`Binary: ${binary}\n`);

const proc = spawn(binary, mcpArgs, {
  stdio: "inherit",
  env: { ...process.env },
});

proc.on("error", (err) => {
  if (err.code === "ENOENT") {
    process.stderr.write(`Error: '${binary}' not found.\n`);
    process.stderr.write("Build it first:  cargo build --release\n");
  } else {
    process.stderr.write(`Failed to start unthinkclaw: ${err.message}\n`);
  }
  process.exit(1);
});

proc.on("spawn", () => {
  setTimeout(() => {
    process.stderr.write("\n");
    process.stderr.write("MCP server ready. Register with Poke:\n\n");
    process.stderr.write(
      `  bunx poke@latest mcp add http://localhost:${PORT}/mcp --name unthinkclaw\n\n`
    );
  }, 800);
});

for (const sig of ["SIGINT", "SIGTERM"]) {
  process.on(sig, () => {
    proc.kill(sig);
    process.exit(0);
  });
}
