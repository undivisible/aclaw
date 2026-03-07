//! Subspace Runtime — Lightweight agent runtime
//!
//! Architecture inspired by:
//! - ZeroClaw: Trait-based pluggable providers, channels, tools, memory
//! - NanoClaw: Container isolation for agent execution
//! - HiClaw: Manager/Worker coordination pattern
//!
//! Core traits:
//! - `Provider` — LLM backend (OpenAI, Anthropic, Gemini, Ollama, etc.)
//! - `Channel` — Communication interface (Telegram, Discord, CLI, WebSocket, etc.)
//! - `Tool` — Agent capability (shell, file I/O, web, memory, etc.)
//! - `MemoryBackend` — Persistent state (SQLite, vector, file-based)
//! - `RuntimeAdapter` — Execution environment (native, Docker, WASM)

pub mod agent;
pub mod channels;
pub mod config;
pub mod gateway;
pub mod memory;
pub mod providers;
pub mod runtime;
pub mod tools;
