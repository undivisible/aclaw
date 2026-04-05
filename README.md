# unthinkclaw

unthinkclaw is a local-first Rust agent runtime for people who want the bot on
their own machine, not hidden behind a hosted control plane.

It is small, async-first, and uses SurrealDB + RocksDB as the primary state
layer.

## What This Branch Is

- `main` is the device-first branch.
- `codex/full-platform` is where the hosted gateway, web UI, and deployment work
  belong.

If you want the bot running on your laptop, desktop, server, or box at home,
this is the branch.

## v2 architecture (minimal core + plugins)

- **Default build** is intentionally small: `core` + `channel-cli` + `provider-ollama`. Tool packs such as web, browser, skills, advanced MCP/vibemania, swarm, Poke, and local ONNX embeddings are **Cargo features** (`plugin-web`, `plugin-browser`, `plugin-skills`, `plugin-advanced`, `plugin-swarm`, `plugin-poke`, `plugin-fastembed`). Use `desktop` for a typical Telegram + Anthropic + tool-pack setup, or `full` for everything.
- **Plugin manifest**: optional `.unthinkclaw/plugins/manifest.json` merges package names into toolset allowlists and can append a `system_prompt_suffix` (see `src/plugins/manifest.rs`).
- **Poke**: with `plugin-poke`, set `plugin_layer.poke_tunnel` in config to spawn `poke-sdk/start.js`, which starts the MCP HTTP server for Poke registration.
- **Native plugins**: see [`docs/NATIVE_PLUGINS.md`](docs/NATIVE_PLUGINS.md) and the optional `vendor/equilibrium` submodule for equilibrium-based FFI.
- **Install helper**: `./install.sh` builds release binaries and runs `unthinkclaw-install install` (copies `unthinkclaw` into `~/.local/bin` by default).

## Current Status

As of March 18, 2026:

- core validation is green:
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- GitHub issues `#2` through `#6` are resolved and closed
- Hermes-inspired groundwork is in:
  - toolset allowlists
  - session search
  - managed skill persistence
  - Daytona runtime scaffolding
- `CLAUDE.md` is present as a symlink to `AGENTS.md`, so the Claude-facing repo
  instructions are in sync with the main agent protocol

## What Already Works

- Anthropic-first provider flow, plus OpenAI-compatible and other provider hooks
- Telegram, CLI, Discord, Slack, WhatsApp, Matrix, Signal, IRC, Google Chat, and
  MS Teams channel modules
- Tool execution for shell, files, web fetch/search, browser, doctor, MCP,
  dynamic tools, and messaging
- SurrealDB + RocksDB memory backend for the long-term storage path
- SurrealDB memory with conversation history, FTS, chunk/file indexing, and
  sticker cache tables
- Cron scheduling, diagnostics, execution policy, and swarm coordination
- Toolset-based tool exposure control, session search, and managed skills

## Quick Start

Build it:

```bash
cargo build --release
```

Initialize config:

```bash
./target/release/unthinkclaw init
```

Run the bot:

```bash
./target/release/unthinkclaw chat --config unthinkclaw.json
```

Ask one question without starting a full chat loop:

```bash
./target/release/unthinkclaw ask "summarize this repo" --config unthinkclaw.json
```

## Useful Commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
./target/release/unthinkclaw doctor --config unthinkclaw.json
./target/release/unthinkclaw audit --config unthinkclaw.json
./target/release/unthinkclaw self-update --config unthinkclaw.json
```

## Automatic Self-Update

If this checkout is a git repo, unthinkclaw can poll its own repository,
fast-forward to new commits, rebuild itself, and optionally restart the user
service.

```json
{
  "runtime": {
    "self_update": {
      "enabled": true,
      "interval_secs": 900,
      "remote": "origin",
      "branch": "main",
      "restart_service": "unthinkclaw"
    }
  }
}
```

Notes:
- it only auto-updates a clean worktree
- it uses fast-forward git updates, not destructive resets
- if service restart fails, it still rebuilds and logs the restart failure

## Docs

- [docs/README.md](docs/README.md)
- [docs/TODO.md](docs/TODO.md)
- [docs/ROADMAP.md](docs/ROADMAP.md)
- [docs/SWARM.md](docs/SWARM.md)

## Storage Direction

- SurrealDB + RocksDB is the backend for memory, session state, and
  swarm/coordinator data.
- `storage.backend` is fixed to `surreal`.
- startup fails fast if a config still requests any other storage mode.

## Rough Edges Still On Deck

- cron/scheduler and some memory flows still need a bit more cleanup around the
  Surreal-backed storage contract
- Daytona is scaffolded, but not yet threaded through tool execution/runtime
  selection
- managed skills exist as a tool and persistence layer, but the agent does not
  auto-author/update them yet
- non-Telegram channels still have less real-world depth than the Telegram path
- gateway hardening, observability, and swarm operator UX still have room to
  grow

## Markdown Policy

The repo docs are being kept intentionally simple:

- root Markdown should stay short and user-facing
- current planning and migration notes live in `docs/`
- stale placeholder Markdown should be removed or clearly marked as generated
- speculative design dumps should be rewritten into short migration notes
