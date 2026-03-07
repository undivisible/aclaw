# Subspace Runtime

**Lightweight agent runtime** — hybrid architecture combining the best of ZeroClaw, NanoClaw, and HiClaw.

- **3.9MB binary** — smaller than ZeroClaw
- **<10ms startup** — instant agent deployment
- **Trait-based architecture** — swap providers, channels, tools, memory without recompiling
- **Container isolation** — inspired by NanoClaw (Docker + native)
- **Manager/Worker pattern** — inspired by HiClaw (multi-agent coordination)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Subspace Runtime (3.9MB binary)                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Provider │  │ Channel  │  │   Tool   │  │  Memory  │    │
│  │  Traits  │  │  Traits  │  │  Traits  │  │  Traits  │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
│       │            │              │              │           │
│  ┌────┴────┐  ┌────┴────┐  ┌────┴────┐  ┌────┴────┐        │
│  │Anthropic│  │   CLI   │  │ Shell   │  │ SQLite  │        │
│  │ OpenAI  │  │Telegram │  │ File I/O│  │ Vector  │        │
│  │ Gemini  │  │ Discord │  │Web HTTP │  │ Memory  │        │
│  │ Ollama  │  │ Matrix  │  │Vibemania│  │ Search  │        │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘        │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Agent Loop (with tool execution + memory integration)  │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Gateway (HTTP/WebSocket for remote management)         │ │
│  │ - /api/chat — message to agent                         │ │
│  │ - /api/containers — list/manage Docker instances       │ │
│  │ - /ws — real-time streaming                            │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Runtime Adapter (native or Docker)                     │ │
│  │ - Native: direct shell execution                       │ │
│  │ - Docker: isolated containers per workspace            │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Build
cargo build --release

# Interactive chat
./target/release/subspace-rt chat --workspace .

# Ask a question
./target/release/subspace-rt ask "What's in the current directory?"

# Initialize config
./target/release/subspace-rt init --provider anthropic --api-key sk-...
```

## Configuration

Create `subspace-rt.json`:

```json
{
  "provider": {
    "name": "anthropic",
    "api_key": "sk-ant-..."
  },
  "model": "claude-sonnet-4-5-20250514",
  "system_prompt": "You are a helpful AI assistant with access to shell, files, and code tools.",
  "workspace": ".",
  "runtime": {
    "kind": "native",
    "docker_image": null,
    "memory_limit_mb": null
  },
  "channel": {
    "kind": "cli",
    "token": null
  }
}
```

## Integration with Subspace/Vibemania

```bash
# Subspace can invoke the runtime as a backend
subspace run "add WebSocket support" --runtime subspace-rt

# Vibemania is available as a tool within the agent
# Agent can call: {"tool": "vibemania", "goal": "..."}
```

## Integration with subspace-editor

The HTTP gateway enables remote control:

```typescript
// subspace-editor can connect to the runtime
const ws = new WebSocket('ws://localhost:8080/ws');
ws.send(JSON.stringify({
  kind: 'chat',
  payload: { text: 'ls -la' }
}));

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log(msg.payload); // Agent response
};
```

## Traits (Composable Architecture)

### Provider
Implement to support any LLM:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;
    async fn chat(&self, request: &ChatRequest) -> anyhow::Result<ChatResponse>;
}
```

Included: Anthropic, OpenAI, OpenRouter, Groq, Ollama, custom OpenAI-compatible.

### Channel
Implement to support any messaging platform:

```rust
#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&mut self) -> anyhow::Result<mpsc::Receiver<IncomingMessage>>;
    async fn send(&self, message: OutgoingMessage) -> anyhow::Result<()>;
}
```

Included: CLI, (Telegram, Discord, Matrix coming soon).

### Tool
Implement agent capabilities:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn spec(&self) -> ToolSpec;
    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult>;
}
```

Included: Shell, File I/O, Vibemania (code execution).

### MemoryBackend
Implement persistent state:

```rust
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    async fn store(&self, namespace: &str, key: &str, value: &str, metadata: Option<Value>) -> anyhow::Result<()>;
    async fn recall(&self, namespace: &str, key: &str) -> anyhow::Result<Option<MemoryEntry>>;
    async fn search(&self, namespace: &str, query: &str, limit: usize) -> anyhow::Result<Vec<MemoryEntry>>;
}
```

Included: SQLite (simple key-value + search).

### RuntimeAdapter
Implement execution environments:

```rust
pub trait RuntimeAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn has_shell(&self) -> bool;
    fn has_filesystem(&self) -> bool;
    fn build_command(&self, command: &str, workspace: &Path) -> anyhow::Result<Command>;
}
```

Included: Native (direct), Docker (isolated).

## What's Next

- [ ] HTTP/WebSocket gateway implementation (axum-based)
- [ ] More channels: Telegram, Discord, Matrix, Slack, WhatsApp
- [ ] Vector embeddings for semantic memory search
- [ ] Manager/Worker agent swarms
- [ ] Remote container orchestration dashboard
- [ ] Integration tests
- [ ] Subspace/Vibemania deep integration
- [ ] Claw migration onto Subspace Runtime

## License

MIT
