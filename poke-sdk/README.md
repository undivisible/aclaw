# unthinkclaw — Poke SDK integration

Register unthinkclaw's tools with [Poke](https://poke.ai) via the MCP HTTP server.

## Quick start

### 1. Build the binary

```bash
cargo build --release
```

### 2. Start the MCP server

```bash
node poke-sdk/start.js
# or with a custom port:
node poke-sdk/start.js --port 3333
```

This starts `unthinkclaw mcp --port 3333` and prints the poke registration command.

### 3. Register with Poke

```bash
bunx poke@latest mcp add http://localhost:3333/mcp --name unthinkclaw
```

---

## Manual (no Node.js)

Run the server directly:

```bash
./target/release/unthinkclaw mcp --port 3333
```

Then register:

```bash
bunx poke@latest mcp add http://localhost:3333/mcp --name unthinkclaw
```

---

## Endpoints

| Endpoint | Description |
|----------|-------------|
| `POST /mcp` | MCP JSON-RPC 2.0 endpoint (for poke) |
| `POST /chat` | HTTP chat endpoint |
| `GET /health` | Health check |

## Available tools (via MCP)

- `shell` — execute shell commands
- `file_ops` — read/write files
- `web_search` — search the web
- `web_fetch` — fetch URL content
- `edit` — surgical file edits
- `ask` — prompt the unthinkclaw AI agent directly

## Stdio mode (other MCP clients)

`mcp-launcher.js` launches the binary in stdio MCP mode for clients that support
it (not poke — poke requires a URL):

```bash
node poke-sdk/mcp-launcher.js
```

## Configuration

The server reads `unthinkclaw.json` by default. Pass `--config <path>` to override.
