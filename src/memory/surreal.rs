//! SurrealDB-backed memory storage using the local RocksDB engine.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::engine::local::RocksDb;
use surrealdb::Surreal;

use super::traits::*;

#[derive(Clone)]
pub struct SurrealMemory {
    db: Surreal<surrealdb::engine::local::Db>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryRow {
    namespace: String,
    key: String,
    value: String,
    metadata: Option<serde_json::Value>,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConversationRow {
    chat_id: String,
    sender_id: String,
    role: String,
    content: String,
    seq: i64,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StickerRow {
    sticker_id: String,
    file_id: String,
    description: String,
    analyzed_at: String,
}

impl SurrealMemory {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(path.as_ref()).await?;
        db.use_ns("claw").use_db("memory").await?;
        db.query(SCHEMA_SQL).await?;
        Ok(Self { db })
    }

    fn memory_id(namespace: &str, key: &str) -> String {
        format!("{namespace}::{key}")
    }
}

const SCHEMA_SQL: &str = r#"
    DEFINE TABLE IF NOT EXISTS memories SCHEMALESS;
    DEFINE FIELD IF NOT EXISTS namespace ON memories TYPE string;
    DEFINE FIELD IF NOT EXISTS key ON memories TYPE string;
    DEFINE FIELD IF NOT EXISTS value ON memories TYPE string;
    DEFINE FIELD IF NOT EXISTS metadata ON memories TYPE option<object>;
    DEFINE FIELD IF NOT EXISTS created_at ON memories TYPE string;
    DEFINE INDEX IF NOT EXISTS memory_lookup_idx ON memories FIELDS namespace, key UNIQUE;
    DEFINE INDEX IF NOT EXISTS memory_namespace_idx ON memories FIELDS namespace;

    DEFINE TABLE IF NOT EXISTS conversations SCHEMALESS;
    DEFINE FIELD IF NOT EXISTS chat_id ON conversations TYPE string;
    DEFINE FIELD IF NOT EXISTS sender_id ON conversations TYPE string;
    DEFINE FIELD IF NOT EXISTS role ON conversations TYPE string;
    DEFINE FIELD IF NOT EXISTS content ON conversations TYPE string;
    DEFINE FIELD IF NOT EXISTS seq ON conversations TYPE int;
    DEFINE FIELD IF NOT EXISTS created_at ON conversations TYPE string;
    DEFINE INDEX IF NOT EXISTS conversation_chat_idx ON conversations FIELDS chat_id, seq;

    DEFINE TABLE IF NOT EXISTS sticker_cache SCHEMALESS;
    DEFINE FIELD IF NOT EXISTS sticker_id ON sticker_cache TYPE string;
    DEFINE FIELD IF NOT EXISTS file_id ON sticker_cache TYPE string;
    DEFINE FIELD IF NOT EXISTS description ON sticker_cache TYPE string;
    DEFINE FIELD IF NOT EXISTS analyzed_at ON sticker_cache TYPE string;
    DEFINE INDEX IF NOT EXISTS sticker_id_idx ON sticker_cache FIELDS sticker_id UNIQUE;
"#;

fn parse_timestamp(value: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

#[async_trait]
impl MemoryBackend for SurrealMemory {
    async fn store(
        &self,
        namespace: &str,
        key: &str,
        value: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let created_at = chrono::Utc::now().to_rfc3339();
        let row = MemoryRow {
            namespace: namespace.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            metadata,
            created_at,
        };
        let _: Option<MemoryRow> = self
            .db
            .upsert(("memories", Self::memory_id(namespace, key)))
            .content(row)
            .await?;
        Ok(())
    }

    async fn recall(&self, namespace: &str, key: &str) -> Result<Option<MemoryEntry>> {
        let row: Option<MemoryRow> = self
            .db
            .select(("memories", Self::memory_id(namespace, key)))
            .await?;
        Ok(row.map(|entry| MemoryEntry {
            key: entry.key,
            value: entry.value,
            metadata: entry.metadata,
            created_at: parse_timestamp(&entry.created_at),
        }))
    }

    async fn search(&self, namespace: &str, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        let query_lower = query.to_lowercase();
        let mut result = self.db
            .query(
                "SELECT key, value, metadata, created_at FROM memories
                 WHERE namespace = $namespace
                   AND (string::lowercase(key) CONTAINS $query OR string::lowercase(value) CONTAINS $query)
                 ORDER BY created_at DESC
                 LIMIT $limit"
            )
            .bind(("namespace", namespace.to_string()))
            .bind(("query", query_lower))
            .bind(("limit", limit as i64))
            .await?;
        let rows: Vec<MemoryRow> = result.take(0)?;
        Ok(rows
            .into_iter()
            .map(|entry| MemoryEntry {
                key: entry.key,
                value: entry.value,
                metadata: entry.metadata,
                created_at: parse_timestamp(&entry.created_at),
            })
            .collect())
    }

    async fn forget(&self, namespace: &str, key: &str) -> Result<()> {
        let _: Option<MemoryRow> = self
            .db
            .delete(("memories", Self::memory_id(namespace, key)))
            .await?;
        Ok(())
    }

    async fn list(&self, namespace: &str) -> Result<Vec<MemoryEntry>> {
        let mut result = self.db
            .query("SELECT key, value, metadata, created_at FROM memories WHERE namespace = $namespace ORDER BY created_at DESC")
            .bind(("namespace", namespace.to_string()))
            .await?;
        let rows: Vec<MemoryRow> = result.take(0)?;
        Ok(rows
            .into_iter()
            .map(|entry| MemoryEntry {
                key: entry.key,
                value: entry.value,
                metadata: entry.metadata,
                created_at: parse_timestamp(&entry.created_at),
            })
            .collect())
    }

    async fn store_conversation(
        &self,
        chat_id: &str,
        sender_id: &str,
        role: &str,
        content: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now();
        let row = ConversationRow {
            chat_id: chat_id.to_string(),
            sender_id: sender_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            seq: now.timestamp_millis(),
            created_at: now.to_rfc3339(),
        };
        let _: Option<ConversationRow> = self.db.create("conversations").content(row).await?;
        Ok(())
    }

    async fn store_conversation_batch(&self, entries: &[(&str, &str, &str, &str)]) -> Result<()> {
        for (offset, (chat_id, sender_id, role, content)) in entries.iter().enumerate() {
            let now = chrono::Utc::now();
            let row = ConversationRow {
                chat_id: (*chat_id).to_string(),
                sender_id: (*sender_id).to_string(),
                role: (*role).to_string(),
                content: (*content).to_string(),
                seq: now.timestamp_millis() + offset as i64,
                created_at: now.to_rfc3339(),
            };
            let _: Option<ConversationRow> = self.db.create("conversations").content(row).await?;
        }
        Ok(())
    }

    async fn get_conversation_history(
        &self,
        chat_id: &str,
        limit: usize,
    ) -> Result<Vec<(String, String)>> {
        let mut result = self
            .db
            .query(
                "SELECT role, content FROM conversations
                 WHERE chat_id = $chat_id
                 ORDER BY seq DESC
                 LIMIT $limit",
            )
            .bind(("chat_id", chat_id.to_string()))
            .bind(("limit", limit as i64))
            .await?;
        let mut rows: Vec<ConversationRow> = result.take(0)?;
        rows.reverse();
        Ok(rows
            .into_iter()
            .map(|row| (row.role, row.content))
            .collect())
    }

    async fn get_sticker_cache(&self, sticker_id: &str) -> Result<Option<String>> {
        let row: Option<StickerRow> = self.db.select(("sticker_cache", sticker_id)).await?;
        Ok(row.map(|entry| entry.description))
    }

    async fn store_sticker_cache(
        &self,
        sticker_id: &str,
        file_id: &str,
        description: &str,
    ) -> Result<()> {
        let row = StickerRow {
            sticker_id: sticker_id.to_string(),
            file_id: file_id.to_string(),
            description: description.to_string(),
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        };
        let _: Option<StickerRow> = self
            .db
            .upsert(("sticker_cache", sticker_id))
            .content(row)
            .await?;
        Ok(())
    }
}
