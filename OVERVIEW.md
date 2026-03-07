# The New Subspace Stack (2026)

Three lightweight, modular projects that work together:

1. **subspace-runtime** — Agent execution engine
2. **subspace-cli (Vibemania)** — Task automation CLI
3. **subspace-editor** — Text editor with plugins

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Subspace Stack                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌───────────────────────────────────────────────────────────┐    │
│  │ Subspace Editor (Lightweight Text Editor)               │    │
│  │ - Core: < 2000 LOC (text buffer, keys, file I/O)       │    │
│  │ - Extensible: Plugin system via JSON-RPC              │    │
│  │ - UI: rngpui (React Native GPU)                        │    │
│  └────────────┬────────────────────────────────────────────┘    │
│               │                                                   │
│        ┌──────┴─────────┬──────────────┬─────────────┐           │
│        │                │              │             │           │
│  ┌─────▼────┐  ┌──────▼────┐  ┌─────▼──┐  ┌──────▼────┐       │
│  │ AI       │  │ Remote     │  │ Tools  │  │ Vibemania│       │
│  │ Plugin   │  │ Plugin     │  │ Plugin │  │ Plugin   │       │
│  │ (JSON    │  │(SSH/SFTP)  │  │(shell) │  │(planning)│       │
│  │ -RPC)    │  │            │  │        │  │          │       │
│  └─────┬────┘  └──────┬─────┘  └─────┬──┘  └──────┬────┘       │
│        │               │              │             │           │
│        └───────────────┴──────────────┴─────────────┘           │
│                        │                                         │
│  ┌─────────────────────▼──────────────────────────────┐        │
│  │ Subspace CLI (Command Orchestrator)               │        │
│  │ - Commands: init, plan, run, dream, tasks        │        │
│  │ - Backends: claude, subspace-rt, amp, codex      │        │
│  │ - Swarm: parallel execution, merging             │        │
│  │ - TUI: live dashboard + monitoring               │        │
│  └─────────────────────┬──────────────────────────────┘        │
│                        │                                         │
│  ┌─────────────────────▼──────────────────────────────┐        │
│  │ Subspace Runtime (Agent Execution)                │        │
│  │ - Traits: Provider, Channel, Tool, Memory,        │        │
│  │   RuntimeAdapter (all swappable)                  │        │
│  │ - Providers: Anthropic, OpenAI, Gemini, Ollama    │        │
│  │ - Channels: CLI, Telegram, Discord, Matrix        │        │
│  │ - Tools: Shell, File I/O, Vibemania               │        │
│  │ - Memory: SQLite + vector search                  │        │
│  │ - Runtime: Native or Docker isolation             │        │
│  │ - Gateway: HTTP/WebSocket for remote control      │        │
│  │ - Size: 3.9MB binary, <10ms startup               │        │
│  └──────────────────────────────────────────────────────┘        │
│                                                                   │
└─────────────────────────────────────────────────────────────────────┘
```

## 📦 Components

### Subspace Runtime (`undivisible/subspace-runtime`)

**Lightweight agent runtime** with composable traits:
- **Provider**: LLM backend (Anthropic Claude, OpenAI, Gemini, Ollama)
- **Channel**: Message transport (CLI, Telegram, Discord, Matrix, WebSocket)
- **Tool**: Agent capability (Shell, File I/O, Vibemania)
- **Memory**: Persistent state (SQLite + vector embeddings)
- **Runtime**: Execution environment (Native or Docker)

**Binary**: 3.9MB | **Startup**: <10ms | **RAM**: <5MB

**Usage**:
```bash
# Interactive chat
./subspace-rt chat --model claude-opus-4-6-20250514

# One-shot query
./subspace-rt ask "What's in this directory?"

# With Vibemania tool
./subspace-rt ask "use vibemania to add WebSocket support" --workspace .
```

**Gateway** (future):
```bash
# Start HTTP server
./subspace-rt gateway --addr 0.0.0.0:8080

# Connect from subspace-editor or external tools
curl http://localhost:8080/api/chat -d '{"text": "hello"}'
```

### Subspace CLI (`atechnology-company/vibemania`, branch `subspace-cli`)

**Task automation & autonomous coding orchestrator**:
- Commands: `init`, `plan`, `run`, `dream`, `tasks`, `status`, `swarm`
- Pluggable backends: Claude Code, Subspace Runtime, AMP, Codex
- Parallel execution: Swarm agents work in parallel, merge results
- Live TUI dashboard with monitoring
- Notifications (OpenClaw integration)

**Usage**:
```bash
# Plan only (audit project, show what would be built)
subspace plan "add WebSocket support"

# Full autonomous run with Claude Code
subspace run "add WebSocket support" --parallel 3 --tui

# With Subspace Runtime backend
subspace run "add WebSocket support" --tool subspace-rt --parallel 4

# Autonomous feature generation
subspace dream --parallel 2 --tui

# Manage tasks in .subspace/subspace.md
subspace tasks --add "fix auth bug" --priority high
```

**Key difference from old Vibemania:**
- No longer tied to ACP (broken/unstable)
- **Now**: Pluggable backends (claude, subspace-rt, amp, codex)
- Each agent backend can be swapped without changing the CLI
- Focus: orchestration & swarm coordination

### Subspace Editor (`undivisible/subspace-editor`)

**Lightweight text editor** with plugin architecture:
- Core: < 2000 LOC (buffer, keybindings, file I/O)
- UI: rngpui (React Native + GPU acceleration)
- Extensible: Everything else is a plugin (JSON-RPC)

**Official Plugins** (coming soon):
1. **AI Plugin** — Explain, refactor, docstring, test, fix
   - Supports: Claude, OpenAI, Gemini, Ollama, Subspace Runtime
2. **Remote Plugin** — SSH editing with local cache
   - SFTP + rsync for performance
3. **Tools Plugin** — Shell + file operations
   - Uses Subspace Runtime tools
4. **Vibemania Plugin** — Plan, run, dream, status
   - Delegates to Subspace Runtime or Claude Code
5. **Git Plugin** — Staging, commits, diffs, blame

**Plugin Architecture**:
```
Editor → JSON-RPC → Plugin (separate process)
```

Each plugin is an independent binary that communicates with the editor via JSON-RPC 2.0.

**Usage**:
```bash
# Launch editor
subspace-editor

# Press Cmd+K (AI menu)
# Select: "AI: Explain"
# → calls AI plugin → Claude/Gemini/Ollama
# → displays explanation in sidebar

# Editor menu → "Tools: Build"
# → calls Tools plugin → shell execution in sandbox

# Editor menu → "Vibemania: Plan"
# → calls Vibemania plugin → Subspace Runtime or Claude Code
```

## 🔄 Workflows

### Workflow 1: Interactive Coding (subspace-editor)

```
1. Open file in Subspace Editor
2. Cmd+K → "AI: Refactor this function"
3. AI Plugin calls Claude
4. Refactored code appears inline
5. Press Enter to accept
6. Git Plugin auto-commits
```

### Workflow 2: Autonomous Tasks (subspace-cli)

```
1. cd my-project
2. subspace plan "add OAuth2 support"
   → Shows plan (no execution yet)
3. subspace run "add OAuth2 support" --tool subspace-rt --parallel 3
   → 3 agents plan → execute → merge
   → Results in PRs/commits
4. subspace dream --auto-approve
   → AI invents features forever, auto-builds top ideas
```

### Workflow 3: Remote Editing (subspace-editor + Remote Plugin)

```
1. Open subspace-editor
2. Cmd+O → "Connect to SSH server"
3. Remote Plugin caches files locally
4. Edit files with full power (AI, tools, etc.)
5. Changes sync back via SFTP
6. Zero latency (local editing)
```

### Workflow 4: Agent Swarm (subspace-runtime Gateway)

```
1. Start gateway: ./subspace-rt gateway
2. External tool connects: curl http://localhost:8080/api/chat
3. Agent processes request with tools
4. Response streamed via WebSocket
5. Can manage multiple containers simultaneously
```

## 🔌 Integrations

### With Claude Code
```bash
subspace run "add feature" --tool claude
```
Claude Code becomes a tool that Vibemania orchestrates.

### With Ollama (Local LLM)
```bash
subspace-rt chat --provider ollama --base-url http://localhost:11434
```
Run agents entirely locally, no API keys needed.

### With Telegram
```bash
# Configure Telegram bot token in ~/.subspace/config.json
subspace-rt chat --channel telegram
```
Chat with agents via Telegram.

### With Git
```bash
subspace run "feature" --notify
# Auto-commits, creates PRs (via Git plugin)
```

### With Docker
```bash
# Runtime adapter for isolation
./subspace-rt run --runtime docker --docker-image rust:latest
```
Each agent gets its own container.

## 📊 Comparison: Old vs New

| Aspect | Old (OpenClaw) | New (Subspace Stack) |
|--------|---|---|
| **Agent Runtime** | ACP (broken) | Subspace Runtime (trait-based, 3.9MB) |
| **Coding Orchestration** | Vibemania + ACP | Vibemania + pluggable backends |
| **LLM Providers** | ACP only | Anthropic, OpenAI, Gemini, Ollama, custom |
| **Channels** | Telegram only | CLI, Telegram, Discord, Matrix, WebSocket, SSH |
| **Editor** | None | Subspace Editor (lightweight) + plugins |
| **Extensibility** | Monolithic | Composable traits + plugin RPC |
| **Isolation** | Process | Process + Docker + WASM (planned) |
| **Binary Size** | N/A | 3.9MB (runtime) + plugins (small) |

## 🚀 Getting Started

### Install Subspace Runtime
```bash
git clone https://github.com/undivisible/subspace-runtime.git
cd subspace-runtime
cargo build --release
./target/release/subspace-rt --version
```

### Install Vibemania (with new backend support)
```bash
git clone https://github.com/atechnology-company/vibemania.git
cd vibemania && cargo build --release
./target/release/subspace run "your goal" --tool subspace-rt
```

### Install Subspace Editor
```bash
git clone https://github.com/undivisible/subspace-editor.git
cd subspace-editor
npm install && npm run build
# Launch with: npm run start
```

### Configure
```bash
mkdir -p ~/.subspace
cat > ~/.subspace/config.json << 'EOF'
{
  "provider": "anthropic",
  "api_key": "sk-ant-...",
  "editor": "subspace-editor",
  "plugins": [
    {"name": "ai", "enabled": true},
    {"name": "remote", "enabled": true},
    {"name": "tools", "enabled": true}
  ]
}
EOF
```

## 📚 Documentation

- **Subspace Runtime**: https://github.com/undivisible/subspace-runtime
  - Architecture: ARCHITECTURE.md
  - Integration: INTEGRATION.md
- **Vibemania**: https://github.com/atechnology-company/vibemania
  - Now a CLI orchestrator with pluggable backends
- **Subspace Editor**: https://github.com/undivisible/subspace-editor
  - Plugin Architecture: PLUGIN_ARCHITECTURE.md

## 🎯 Roadmap

**Phase 1** (Done):
- [x] Subspace Runtime (trait-based architecture)
- [x] Vibemania refactor (pluggable backends)
- [x] Subspace Editor (plugin system skeleton)

**Phase 2** (Next):
- [ ] HTTP/WebSocket gateway (axum)
- [ ] Official plugins (AI, Remote, Tools, Vibemania, Git)
- [ ] Telegram, Discord channels
- [ ] Vector memory search

**Phase 3** (Ongoing):
- [ ] Agent swarms (Manager/Worker)
- [ ] Claw migration onto Subspace Runtime
- [ ] Production hardening
- [ ] Community plugins

## 🛠️ Contributing

All three repos are open source (MIT). Contributions welcome!

- Find a good first issue
- Read the ARCHITECTURE or PLUGIN_ARCHITECTURE docs
- Build + test locally
- Open a PR

## 📞 Support

- Issues: GitHub issue tracker on each repo
- Docs: ARCHITECTURE.md, PLUGIN_ARCHITECTURE.md, README.md
- Discord: [TBD]

---

**Built by**: undivisible (Claw)  
**Year**: 2026  
**License**: MIT
