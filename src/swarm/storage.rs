use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::engine::local::RocksDb;
use surrealdb::Surreal;

/// Trait for swarm storage backends
#[async_trait]
pub trait SwarmStorage: Send + Sync {
    async fn store_task(&self, task: &super::Task) -> Result<()>;
    async fn get_task(&self, task_id: &str) -> Result<Option<super::Task>>;
    async fn list_pending_tasks(&self) -> Result<Vec<super::Task>>;
    async fn update_task_status(&self, task_id: &str, status: super::TaskStatus) -> Result<()>;
    
    async fn register_agent(&self, agent: &super::AgentInfo) -> Result<()>;
    async fn get_agent(&self, agent_id: &str) -> Result<Option<super::AgentInfo>>;
    async fn list_active_agents(&self) -> Result<Vec<super::AgentInfo>>;
    async fn update_agent_heartbeat(&self, agent_id: &str) -> Result<()>;
}

/// SurrealDB backend (distributed state)
pub struct SurrealBackend {
    db: Surreal<RocksDb>,
}

impl SurrealBackend {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(path.as_ref()).await?;
        db.use_ns("claw").use_db("swarm").await?;
        
        // Initialize schema
        db.query(r#"
            DEFINE TABLE tasks SCHEMAFULL;
            DEFINE FIELD task_id ON tasks TYPE string;
            DEFINE FIELD title ON tasks TYPE string;
            DEFINE FIELD description ON tasks TYPE string;
            DEFINE FIELD status ON tasks TYPE string;
            DEFINE FIELD priority ON tasks TYPE int;
            DEFINE FIELD assigned_to ON tasks TYPE option<string>;
            DEFINE FIELD created_at ON tasks TYPE datetime;
            DEFINE FIELD updated_at ON tasks TYPE datetime;
            DEFINE INDEX task_id_idx ON tasks FIELDS task_id UNIQUE;
            
            DEFINE TABLE agents SCHEMAFULL;
            DEFINE FIELD agent_id ON agents TYPE string;
            DEFINE FIELD name ON agents TYPE string;
            DEFINE FIELD capabilities ON agents TYPE array;
            DEFINE FIELD status ON agents TYPE string;
            DEFINE FIELD last_heartbeat ON agents TYPE datetime;
            DEFINE INDEX agent_id_idx ON agents FIELDS agent_id UNIQUE;
        "#).await?;
        
        Ok(Self { db })
    }
}

#[async_trait]
impl SwarmStorage for SurrealBackend {
    async fn store_task(&self, task: &super::Task) -> Result<()> {
        let _: Vec<super::Task> = self.db
            .create("tasks")
            .content(task)
            .await?;
        Ok(())
    }
    
    async fn get_task(&self, task_id: &str) -> Result<Option<super::Task>> {
        let mut result: Vec<super::Task> = self.db
            .select("tasks")
            .await?;
        Ok(result.into_iter().find(|t| t.task_id == task_id))
    }
    
    async fn list_pending_tasks(&self) -> Result<Vec<super::Task>> {
        let tasks: Vec<super::Task> = self.db
            .query("SELECT * FROM tasks WHERE status = 'pending' ORDER BY priority DESC, created_at ASC")
            .await?
            .take(0)?;
        Ok(tasks)
    }
    
    async fn update_task_status(&self, task_id: &str, status: super::TaskStatus) -> Result<()> {
        let _: Option<super::Task> = self.db
            .query("UPDATE tasks SET status = $status, updated_at = time::now() WHERE task_id = $task_id")
            .bind(("task_id", task_id))
            .bind(("status", status.to_string()))
            .await?
            .take(0)?;
        Ok(())
    }
    
    async fn register_agent(&self, agent: &super::AgentInfo) -> Result<()> {
        let _: Vec<super::AgentInfo> = self.db
            .create("agents")
            .content(agent)
            .await?;
        Ok(())
    }
    
    async fn get_agent(&self, agent_id: &str) -> Result<Option<super::AgentInfo>> {
        let mut result: Vec<super::AgentInfo> = self.db
            .select("agents")
            .await?;
        Ok(result.into_iter().find(|a| a.agent_id == agent_id))
    }
    
    async fn list_active_agents(&self) -> Result<Vec<super::AgentInfo>> {
        let agents: Vec<super::AgentInfo> = self.db
            .query("SELECT * FROM agents WHERE status = 'active' AND last_heartbeat > time::now() - 30s")
            .await?
            .take(0)?;
        Ok(agents)
    }
    
    async fn update_agent_heartbeat(&self, agent_id: &str) -> Result<()> {
        let _: Option<super::AgentInfo> = self.db
            .query("UPDATE agents SET last_heartbeat = time::now() WHERE agent_id = $agent_id")
            .bind(("agent_id", agent_id))
            .await?
            .take(0)?;
        Ok(())
    }
}

/// RocksDB cache (local hot data)
pub struct RocksCache {
    db: rocksdb::DB,
}

impl RocksCache {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let db = rocksdb::DB::open_cf(
            &opts,
            path,
            vec!["embeddings", "chunks", "sticker_cache"],
        )?;
        
        Ok(Self { db })
    }
    
    pub fn get(&self, cf: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let cf_handle = self.db.cf_handle(cf)
            .ok_or_else(|| anyhow::anyhow!("Column family {} not found", cf))?;
        Ok(self.db.get_cf(cf_handle, key)?)
    }
    
    pub fn put(&self, cf: &str, key: &[u8], value: &[u8]) -> Result<()> {
        let cf_handle = self.db.cf_handle(cf)
            .ok_or_else(|| anyhow::anyhow!("Column family {} not found", cf))?;
        self.db.put_cf(cf_handle, key, value)?;
        Ok(())
    }
    
    pub fn delete(&self, cf: &str, key: &[u8]) -> Result<()> {
        let cf_handle = self.db.cf_handle(cf)
            .ok_or_else(|| anyhow::anyhow!("Column family {} not found", cf))?;
        self.db.delete_cf(cf_handle, key)?;
        Ok(())
    }
}
