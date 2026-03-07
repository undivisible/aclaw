# Integration Guide

This document explains how to integrate Subspace Runtime with:
1. Vibemania (code execution tool)
2. subspace-editor (IDE control)
3. Claw (agent migration)

## 1. Vibemania Integration

### Setup

```bash
# Build subspace-runtime
cd ~/subspace-runtime
cargo build --release

# Clone/build Vibemania nearby (so it can be found)
cd ..
git clone https://github.com/atechnology-company/vibemania.git
cd vibemania && cargo build --release

# Test integration
cd ~/subspace-runtime
./target/release/subspace-rt ask "build vibemania" --workspace ../vibemania
```

### How It Works

The `VibemaniaTool` works like any other tool:

```rust
// Agent decides to call Vibemania
ChatRequest {
    messages: [
        system_prompt,
        "Add WebSocket support to the API"
    ],
    tools: [
        ShellTool.spec(),
        FileReadTool.spec(),
        FileWriteTool.spec(),
        VibemaniaTool.spec(),  // ← Here
    ],
    ...
}

// LLM response includes tool call:
ToolCall {
    name: "vibemania",
    arguments: r#"{"goal": "add WebSocket support", "parallel": 2}"#
}

// Tool execution:
VibemaniaTool.execute(arguments)
  → spawn `vibemania run "add WebSocket support" --parallel 2`
  → return stdout/stderr as ToolResult
  → Agent incorporates results, may iterate
```

### Example: Autonomous Code Fix

```bash
./target/release/subspace-rt ask "
I have a bug in my authentication module. 
Use vibemania to explore the codebase and fix it.
" --workspace ../atmosphere/cupboard --model claude-sonnet-4-5
```

Agent will:
1. Understand the task
2. Call `vibemania` tool with goal "fix authentication module"
3. Vibemania runs autonomously (exploring, testing, iterating)
4. Returns results to agent
5. Agent summarizes findings

## 2. subspace-editor Integration

### Setup (Future - Gateway not yet implemented)

```bash
# Start the gateway (once axum server is implemented)
./target/release/subspace-rt gateway --addr 0.0.0.0:8080

# In subspace-editor, connect to:
# wss://localhost:8080/ws
# or
# http://localhost:8080/api/*
```

### API Usage

```typescript
// Connect to runtime
const client = new SubspaceRuntimeClient('http://localhost:8080');

// Send message
const response = await client.chat({
  text: 'implement a REST API endpoint',
  context: { file: 'src/routes.rs' }
});

// Listen for real-time updates
client.on('message', (msg) => {
  switch (msg.kind) {
    case 'tool_call':
      console.log(`Agent calling ${msg.payload.tool_name}...`);
      break;
    case 'text':
      console.log(`Agent: ${msg.payload.text}`);
      break;
    case 'status':
      console.log(`Status: ${msg.payload.status}`);
      break;
  }
});

// Manage containers
const containers = await client.getContainers();
await client.exec('container-id', 'ls -la');
```

### Implementation Checklist

- [ ] Implement axum HTTP server
- [ ] Add WebSocket endpoint with tokio-tungstenite
- [ ] Proxy requests to agent loop
- [ ] Docker API integration (via bollard)
- [ ] Real-time streaming
- [ ] Authentication (bearer token or mTLS)
- [ ] Rate limiting
- [ ] CORS support for web clients

## 3. Claw Migration

### Current Architecture
- Claw runs on OpenClaw (Node.js)
- Uses ACP for agent execution (has issues)
- Configuration in SOUL.md, AGENTS.md, MEMORY.md

### New Architecture
- Claw runs on Subspace Runtime (Rust)
- Uses traits for provider/channel/tool/memory
- Configuration in subspace-rt.json

### Migration Steps

**Phase 1: Prepare**
```bash
# Fork subspace-runtime (already done: undivisible/subspace-runtime)
# Build locally
cd ~/subspace-runtime && cargo build --release
```

**Phase 2: Adapter Layer**
Create `src/adapters/claw.rs` to bridge OpenClaw config → Subspace Runtime:

```rust
pub struct ClawAdapter {
    // Read from OpenClaw SOUL.md, AGENTS.md
    // Map to Subspace Runtime traits
    // Preserve personality, memory, skills
}

impl ClawAdapter {
    pub async fn load_from_openclaw() -> anyhow::Result<Self> {
        // 1. Parse SOUL.md for personality
        // 2. Parse AGENTS.md for capabilities
        // 3. Load memory from MEMORY.md
        // 4. Create AgentRunner with these configs
    }
}
```

**Phase 3: Test**
```bash
# Test Claw on Subspace Runtime
cd ~/subspace-runtime
./target/release/subspace-rt chat --config claw-config.json

# Verify:
# - Personality matches SOUL.md
# - Memory loads and works
# - Tools available (shell, file I/O, web, vibemania)
# - Channels work (CLI, Telegram, Discord when ready)
```

**Phase 4: Migrate**
```bash
# Once stable, update AGENTS.md to point to subspace-runtime
# Move SOUL.md, MEMORY.md to subspace-rt structure
# Shut down OpenClaw session
# Start subspace-rt with Claw config
```

### Configuration Format

**Old (OpenClaw):**
```json
{
  "runtime": "openclaw",
  "agent": "acp",
  "model": "claude-opus-4-6"
}
```

**New (Subspace Runtime):**
```json
{
  "provider": {
    "name": "anthropic",
    "api_key": "sk-ant-..."
  },
  "model": "claude-opus-4-6-20250514",
  "system_prompt": "You are Claw, a smartass best friend...",
  "workspace": "~/.openclaw/workspace",
  "runtime": {
    "kind": "native"
  },
  "channel": {
    "kind": "cli"
  }
}
```

### Memory Migration

**Old (OpenClaw):**
- `MEMORY.md` (text file)
- `memory/YYYY-MM-DD.md` (daily notes)
- Loaded manually each session

**New (Subspace Runtime):**
- SQLite backend with namespaces
- `store(namespace, key, value)` interface
- Search + retrieval built-in
- Persists across sessions
- Optional vector embeddings for semantic search

**Migration Process:**
```bash
# Parse existing MEMORY.md and daily files
# Convert to SQLite entries
# Load via MemoryBackend interface
# Extend with new capabilities (search, metadata)
```

## Testing

```bash
# Unit tests
cargo test

# Integration test (chat loop)
echo "Hello, what's the current directory?" | \
  ./target/release/subspace-rt chat

# Vibemania integration test
./target/release/subspace-rt ask "
Run Vibemania to analyze src/ directory structure
" --workspace . --model claude-opus-4-6-20250514

# Load test (many concurrent messages)
for i in {1..10}; do
  ./target/release/subspace-rt ask "Summarize $(date)" &
done
wait
```

## Troubleshooting

### Gateway won't start
- Check port 8080 is available: `lsof -i :8080`
- Ensure axum dependencies are correct: `cargo update`

### Vibemania tool not found
- Verify vibemania binary exists at `../vibemania/target/release/vibemania`
- Check workspace path is correct in config

### Memory not persisting
- Ensure `~/.subspace-rt/memory.db` exists and is writable
- Check SQLite doesn't have locking issues: `lsof ~/.subspace-rt/memory.db`

### 0-byte responses from provider
- Check API key is valid
- Verify network connectivity
- Enable logging: `RUST_LOG=debug ./target/release/subspace-rt chat`

## Support

- Issues: https://github.com/undivisible/subspace-runtime/issues
- Architecture questions: See ARCHITECTURE.md
- Integration help: Check examples/ directory (coming soon)
