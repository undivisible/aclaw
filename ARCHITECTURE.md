# Subspace Runtime Architecture

## Overview

Subspace Runtime is a lightweight, trait-based agent runtime designed to be:
- **Minimal**: 3.9MB binary, <10ms startup
- **Composable**: Swap providers, channels, tools, memory, runtime without recompilation
- **Secure**: Container isolation (Docker), sandboxed execution, credential isolation
- **Extensible**: Simple trait interfaces for custom implementations

## Core Traits

### 1. Provider (LLM Backend)

**Interface:**
```rust
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;
    async fn chat(&self, request: &ChatRequest) -> anyhow::Result<ChatResponse>;
}
```

**Implementations:**
- `AnthropicProvider` — Claude (native tool calling)
- `OpenAiCompatProvider` — OpenAI, OpenRouter, Groq, Together (OpenAI API format)
- `OllamaProvider` — Local models (llama.cpp, MLX)

**Design Decision (ZeroClaw):**
Single trait with pluggable implementations. Each provider handles its own API format, authentication, and response parsing. Clients interact via unified `ChatRequest` / `ChatResponse` interface.

### 2. Channel (Messaging Platform)

**Interface:**
```rust
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&mut self) -> anyhow::Result<mpsc::Receiver<IncomingMessage>>;
    async fn send(&self, message: OutgoingMessage) -> anyhow::Result<()>;
    async fn stop(&mut self) -> anyhow::Result<()>;
}
```

**Implementations:**
- `CliChannel` — Interactive terminal (done)
- `TelegramChannel` — Telegram Bot API (planned)
- `DiscordChannel` — Discord.js (planned)
- `MatrixChannel` — Matrix protocol (planned)
- `WebSocketChannel` — Real-time WebSocket (planned)

**Design Decision (NanoClaw + HiClaw):**
Each channel manages its own message queue and protocol handling. Runtime is transport-agnostic. Channels emit `IncomingMessage` events to the agent loop via Tokio `mpsc`.

### 3. Tool (Agent Capability)

**Interface:**
```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn spec(&self) -> ToolSpec; // For LLM function calling
    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult>;
}
```

**Implementations:**
- `ShellTool` — Execute bash commands
- `FileReadTool` / `FileWriteTool` — File I/O
- `VibemaniaTool` — Autonomous code execution (Vibemania integration)

**Design Decision (ZeroClaw):**
Tools advertise their capabilities via `ToolSpec` (for LLM function calling). Execution is async and sandboxed. Error handling is tool-specific.

### 4. MemoryBackend (Persistent State)

**Interface:**
```rust
pub trait MemoryBackend: Send + Sync {
    async fn store(&self, namespace: &str, key: &str, value: &str, metadata: Option<Value>) -> anyhow::Result<()>;
    async fn recall(&self, namespace: &str, key: &str) -> anyhow::Result<Option<MemoryEntry>>;
    async fn search(&self, namespace: &str, query: &str, limit: usize) -> anyhow::Result<Vec<MemoryEntry>>;
    async fn forget(&self, namespace: &str, key: &str) -> anyhow::Result<()>;
    async fn list(&self, namespace: &str) -> anyhow::Result<Vec<MemoryEntry>>;
}
```

**Implementations:**
- `SqliteMemory` — SQLite-backed key-value with prefix search (done)
- `VectorMemory` — Vector embeddings with semantic search (planned, using pgvector like Pava)

**Design Decision (ZeroClaw + Pava):**
Namespaced memory for per-group/per-user isolation (like NanoClaw). Search for context injection during LLM calls. Vector embeddings coming for semantic retrieval.

### 5. RuntimeAdapter (Execution Environment)

**Interface:**
```rust
pub trait RuntimeAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn has_shell(&self) -> bool;
    fn has_filesystem(&self) -> bool;
    fn build_command(&self, command: &str, workspace: &Path) -> anyhow::Result<Command>;
}
```

**Implementations:**
- `NativeRuntime` — Direct shell execution (done)
- `DockerRuntime` — Isolated container per agent/workspace (done)
- `WasmRuntime` — WASM sandboxing (planned)

**Design Decision (NanoClaw):**
Each agent gets its own isolated environment (container or native). Filesystem and network are scoped. Memory limits enforced.

## Agent Loop (Core Execution)

```
┌─────────────────────────────────────────────┐
│ 1. Receive message from Channel             │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ 2. Fetch memory context (search)            │
│    - Relevant past interactions             │
│    - Knowledge base entries                 │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ 3. Build ChatRequest with:                  │
│    - System prompt                          │
│    - Memory context                         │
│    - Conversation history                   │
│    - Available tools (ToolSpecs)            │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ 4. Call Provider.chat()                     │
│    - Gets back text + tool_calls            │
└─────────────────────────────────────────────┘
                    ↓
            ┌───────┴────────┐
            │                │
      ┌─────▼────┐     ┌────▼──────┐
      │Has tool  │     │No tool    │
      │calls?    │     │calls?     │
      └─────┬────┘     └────┬──────┘
            │               │
         Yes│               │No
            │               │
      ┌─────▼──────────────▼──┐
      │ 5a. Execute each Tool │
      │     Get results       │
      │     Add to context    │
      │     Loop back to 4    │
      │ Max 10 rounds         │
      └──────────────────┬────┘
                         │
                    ┌────▼────┐
                    │ 5b. No   │
                    │ more     │
                    │ tools    │
                    └────┬─────┘
                         ↓
┌─────────────────────────────────────────────┐
│ 6. Extract final text response              │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ 7. Store interaction in memory              │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ 8. Send response via Channel.send()         │
└─────────────────────────────────────────────┘
```

## Vibemania Integration

### Current State
- Vibemania is an autonomous code agent (4365 LOC Rust)
- Uses ACP (Agent Client Protocol) for execution
- **Issue**: Getting 0-byte responses from `claude-agent-acp`

### New Design
Vibemania becomes a **Tool** in the Subspace Runtime:

```rust
pub struct VibemaniaTool {
    workspace: PathBuf,
}

impl Tool for VibemaniaTool {
    async fn execute(&self, args: &str) -> anyhow::Result<ToolResult> {
        // Parse: {"goal": "add WebSocket support", "parallel": 3}
        // Invoke: subspace run --parallel 3 "goal"
        // Return: execution results as ToolResult
    }
}
```

**Workflow:**
1. Agent receives message asking for code work
2. Agent decides to call `vibemania` tool with goal
3. Vibemania runs autonomously, returns results
4. Agent incorporates results into response
5. Agent may call vibemania again if needed

**Benefits:**
- Decouples Vibemania from ACP issues
- Vibemania can use Subspace Runtime itself as agent backend
- Tool-based composition (Vibemania = one tool among many)

## subspace-editor Integration

### Current State
- subspace-editor is a VS Code-like IDE for Subspace
- Needs to control agents remotely
- Container management
- Real-time streaming

### New Design
HTTP/WebSocket gateway for remote control:

```
┌──────────────────┐
│ subspace-editor  │
│ (local or remote)│
└────────┬─────────┘
         │
         │ HTTP/WebSocket
         │
    ┌────▼────────────────────────────────┐
    │ Subspace Runtime Gateway             │
    ├─────────────────────────────────────┤
    │ POST /api/chat                      │
    │ GET  /api/status                    │
    │ GET  /api/containers                │
    │ POST /api/containers/{id}/stop      │
    │ WebSocket /ws                       │
    │ GET  /api/memory/{namespace}        │
    └────┬──────────────────────────────┬─┘
         │                              │
    ┌────▼────────────────┐  ┌────────▼──────┐
    │ Agent Loop          │  │ Docker API    │
    │ (tool execution)    │  │ (containers)  │
    └─────────────────────┘  └───────────────┘
```

**Endpoints:**

- `POST /api/chat` — Send message to agent
  - Request: `{text: string}`
  - Response: `{text: string, tool_calls: []}`

- `GET /api/status` — Runtime status
  - Response: `{name: string, version: string, provider: string, memory_bytes: u64}`

- `GET /api/containers` — List running containers
  - Response: `{containers: [{id, name, runtime, status, memory_used_mb, cpu_percent}]}`

- `POST /api/containers/{id}/stop` — Stop container
  - Response: `{stopped: bool}`

- `WebSocket /ws` — Real-time event stream
  - Incoming: `{kind: "chat" | "tool_call" | "status", payload: Value}`
  - Outgoing: `{id: string, kind: string, payload: Value, timestamp: DateTime}`

- `GET /api/memory/{namespace}` — List memories
  - Response: `{entries: [{key, value, metadata, created_at}]}`

- `POST /api/memory/{namespace}` — Store memory
  - Request: `{key: string, value: string, metadata?: Value}`

**Client (subspace-editor) Usage:**
```typescript
// Connect to runtime
const gateway = new SubspaceGatewayClient('http://localhost:8080');

// Send message to agent
const response = await gateway.chat({ text: 'add WebSocket support' });

// Listen for real-time events
gateway.onMessage = (msg) => {
  if (msg.kind === 'tool_call') {
    console.log(`Calling ${msg.payload.tool}...`);
  }
};

// Manage containers
const containers = await gateway.getContainers();
await gateway.stopContainer(containers[0].id);
```

## Manager/Worker Pattern (HiClaw)

### Future Design
For multi-agent coordination:

```
┌─────────────┐
│   Manager   │ (orchestrator, Claw)
│   Agent     │
└──────┬──────┘
       │
   ┌───┴────┬──────┬────────┐
   │        │      │        │
┌──▼──┐  ┌─▼──┐ ┌─▼──┐  ┌──▼──┐
│ W1  │  │W2  │ │W3  │  │ W4  │
│(FE) │  │(DB)│ │(API)  │(Test)
└─────┘  └────┘ └────┘  └─────┘
```

- Manager assigns tasks to Workers
- Workers execute in isolated containers
- All communication via message queue
- Results aggregated by Manager
- Memory is shared via MemoryBackend

## Security Model

1. **Process Isolation**: Each agent in its own container (Docker) or native process
2. **Credential Isolation**: Workers never hold real API keys (use consumer tokens)
3. **Filesystem Isolation**: Mount only necessary workspace
4. **Memory Limits**: Configurable per runtime
5. **Tool Allowlisting**: Runtime specifies available tools
6. **Input Validation**: All tool arguments are deserialized + validated

## Roadmap

**v0.2** (next sprint):
- [ ] HTTP/WebSocket gateway (axum)
- [ ] Telegram, Discord channels
- [ ] Vector memory backend

**v0.3** (ongoing):
- [ ] Manager/Worker swarms
- [ ] Claw migration
- [ ] Production hardening

**v0.4+** (future):
- [ ] WASM runtime
- [ ] Distributed agent network
- [ ] Advanced memory (RAG, semantic search)
