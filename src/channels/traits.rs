//! Core Channel trait — messaging interface.
//! Inspired by ZeroClaw's channel abstraction + NanoClaw's group isolation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Incoming message from a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub chat_id: String,
    pub text: String,
    pub is_group: bool,
    pub reply_to: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Outgoing message to a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingMessage {
    pub chat_id: String,
    pub text: String,
    pub reply_to: Option<String>,
}

/// The core Channel trait.
/// Implement for each messaging platform (Telegram, Discord, CLI, WebSocket, etc.)
#[async_trait]
pub trait Channel: Send + Sync {
    /// Channel name (e.g., "telegram", "discord", "cli")
    fn name(&self) -> &str;

    /// Start receiving messages. Returns a receiver for incoming messages.
    async fn start(&mut self) -> anyhow::Result<mpsc::Receiver<IncomingMessage>>;

    /// Send a message through this channel.
    async fn send(&self, message: OutgoingMessage) -> anyhow::Result<()>;

    /// Stop the channel gracefully.
    async fn stop(&mut self) -> anyhow::Result<()>;
}
