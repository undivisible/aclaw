#!/usr/bin/env node
/**
 * Start the unthinkclaw MCP HTTP server and print the URL for poke registration.
 *
 * Usage:
 *   node start.js [--port 3333] [--config unthinkclaw.json]
 *
 * After starting, register with poke:
 *   bunx poke@latest mcp add http://localhost:<PORT>/mcp
 */
import { spawn } from "child_process";
import { existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Parse args
const args = process.argv.slice(2);
const portIdx = args.indexOf("--port");
const PORT = portIdx !== -1 ? parseInt(args[portIdx + 1], 10) : 3333;
const configIdx = args.indexOf("--config");
const CONFIG = configIdx !== -1 ? args[configIdx + 1] : "unthinkclaw.json";

// Locate the binary: prefer release build next to this repo, then PATH
const candidates = [
  resolve(__dirname, "../target/release/unthinkclaw"),
  resolve(__dirname, "../target/debug/unthinkclaw"),
  "unthinkclaw",
];

const binary = candidates.find(
  (c) => c === "unthinkclaw" || existsSync(c)
) ?? "unthinkclaw";

const mcpArgs = ["mcp", "--port", String(PORT), "--config", CONFIG];

console.log(`Starting unthinkclaw MCP server on port ${PORT}…`);
console.log(`Binary: ${binary}`);
console.log(`Args:   ${mcpArgs.join(" ")}`);
console.log();

const proc = spawn(binary, mcpArgs, {
  stdio: "inherit",
  env: { ...process.env },
});

proc.on("error", (err) => {
  if (err.code === "ENOENT") {
    console.error(`Error: '${binary}' not found.`);
    console.error("Build it first:  cargo build --release");
  } else {
    console.error("Failed to start unthinkclaw:", err.message);
  }
  process.exit(1);
});

proc.on("spawn", () => {
  // Give the server a moment to bind, then print the registration command
  setTimeout(() => {
    console.log();
    console.log("─".repeat(60));
    console.log("MCP server ready. Register with poke:");
    console.log();
    console.log(`  bunx poke@latest mcp add http://localhost:${PORT}/mcp --name unthinkclaw`);
    console.log();
    console.log("─".repeat(60));
  }, 800);
});

// Forward signals so the server shuts down cleanly
for (const sig of ["SIGINT", "SIGTERM"]) {
  process.on(sig, () => {
    proc.kill(sig);
    process.exit(0);
  });
}
