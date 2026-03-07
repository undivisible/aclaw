//! Agent — the autonomous agent loop.
//! Receives messages, uses tools, responds via channels.
//! Inspired by HiClaw's Manager/Worker pattern.

pub mod loop_runner;

pub use loop_runner::AgentRunner;
