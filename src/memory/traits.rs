//! Core MemoryBackend trait.

use async_trait::async_trait;
use serde_json::Value;

/// Memory entry
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub metadata: Option<Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// The core MemoryBackend trait.
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    /// Store a key-value memory
    async fn store(&self, namespace: &str, key: &str, value: &str, metadata: Option<Value>) -> anyhow::Result<()>;

    /// Recall a specific memory by key
    async fn recall(&self, namespace: &str, key: &str) -> anyhow::Result<Option<MemoryEntry>>;

    /// Search memories by query (semantic or keyword)
    async fn search(&self, namespace: &str, query: &str, limit: usize) -> anyhow::Result<Vec<MemoryEntry>>;

    /// Delete a memory
    async fn forget(&self, namespace: &str, key: &str) -> anyhow::Result<()>;

    /// List all memories in a namespace
    async fn list(&self, namespace: &str) -> anyhow::Result<Vec<MemoryEntry>>;
}
