//! Tool abstraction — agent capabilities matching OpenClaw's tool set.
//!
//! Core tools (OpenClaw parity):
//!   group:runtime  — exec (shell commands)
//!   group:fs       — Read, Write, Edit
//!   group:web      — web_search, web_fetch
//!   group:memory   — memory_search, memory_get
//!   group:sessions — session_status, list_models
//!   group:messaging — message (Telegram send/edit/delete)

pub mod browser;
pub mod claude_usage;
pub mod computer_use;
pub mod doctor;
pub mod dynamic;
pub mod edit;
pub mod embeddings;
pub mod file_ops;
pub mod mcp;
pub mod message;
pub mod network;
pub mod sandbox;
pub mod session;
pub mod shell;
pub mod skill_manager;
pub mod toolsets;
pub mod traits;
pub mod vibemania;
pub mod web_fetch;
pub mod web_search;

pub use traits::{Tool, ToolResult, ToolSpec};
