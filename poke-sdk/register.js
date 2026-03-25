#!/usr/bin/env node
/**
 * Register the running unthinkclaw MCP server with Poke.
 *
 * Requires the server to already be running (start.js) on the given port.
 *
 * Usage:
 *   node register.js [--port 3333] [--name unthinkclaw]
 */
import { spawn } from "child_process";

const args = process.argv.slice(2);

const portIdx = args.indexOf("--port");
const PORT = portIdx !== -1 ? parseInt(args[portIdx + 1], 10) : 3333;

const nameIdx = args.indexOf("--name");
const NAME = nameIdx !== -1 ? args[nameIdx + 1] : "unthinkclaw";

const url = `http://localhost:${PORT}/mcp`;

console.log(`Registering ${url} with poke as "${NAME}"…`);

const proc = spawn("bunx", ["poke@latest", "mcp", "add", url, "--name", NAME], {
  stdio: "inherit",
});

proc.on("error", (err) => {
  console.error("Failed to run poke:", err.message);
  console.error("Make sure bun/bunx is installed: https://bun.sh");
  process.exit(1);
});

proc.on("exit", (code) => {
  process.exit(code ?? 0);
});
