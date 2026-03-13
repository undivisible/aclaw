# unthinkclaw

Device-first Rust agent runtime. Unthink everything.

## Features

- Multi-provider (Anthropic, OpenAI, Ollama, OpenRouter, ...)
- Multi-channel (Telegram, Discord, CLI, ...)
- Agent swarms with parallel execution
- Local memory with SQLite and SurrealDB-backed storage paths
- Cron scheduling, cost tracking, plugin system
- WAL-mode SQLite, all DB ops on blocking thread pool
- Focused branch for running the bot directly on your own device

## Build

```bash
cargo build --release
```

## Run

```bash
./target/release/unthinkclaw chat --config unthinkclaw.json
```

## Config

Create `unthinkclaw.json` with `unthinkclaw init`, then set `ANTHROPIC_API_KEY`
(or the relevant provider key) in your environment.

The hosted web/app stack lives on the `codex/full-platform` branch. This branch
is kept focused on the local bot runtime.
