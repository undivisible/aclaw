//! Agent — the autonomous agent loop.
//! Receives messages, uses tools, responds via channels.
//! Inspired by HiClaw's Manager/Worker pattern.

pub mod hooks;
pub mod loop_runner;
pub mod mode;
pub mod streaming;

pub use loop_runner::AgentRunner;
pub use mode::{AgentMode, NullChannel, PendingPlan};
pub use streaming::{stream_channel, StreamChunk, StreamReceiver, StreamSender};
