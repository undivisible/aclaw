# unthinkclaw

unthinkclaw is a local-first Rust agent runtime for people who want the bot on
their own machine, not hidden behind a hosted control plane.

It is small, async-first, and moving toward SurrealDB + RocksDB as the primary
state layer, while still keeping a lightweight SQLite fallback for local/dev
paths that have not been fully migrated yet.

## What This Branch Is

- `main` is the device-first branch.
- `codex/full-platform` is where the hosted gateway, web UI, and deployment work
  belong.

If you want the bot running on your laptop, desktop, server, or box at home,
this is the branch.

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
- SQLite fallback memory with conversation history, FTS, chunk/file indexing,
  and sticker cache tables
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
- [docs/CLAW-ADOPTION-TODO.md](docs/CLAW-ADOPTION-TODO.md)

## Storage Direction

The repo should not treat SQLite as the end state.

- SurrealDB + RocksDB is the target backend for memory, session state, and
  swarm/coordinator data.
- SQLite is still present as a compatibility and local-dev fallback while the
  remaining SQLite-only paths are moved over.
- If you explicitly set `"storage.backend": "surreal"` without building the
  Surreal feature set, startup now fails fast instead of quietly pretending the
  storage layer matches production intent.

## Rough Edges Still On Deck

- the storage migration is only partially complete; cron/scheduler and some
  memory flows still assume SQLite-shaped behavior
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
