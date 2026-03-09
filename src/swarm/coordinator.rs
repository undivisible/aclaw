use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::{SwarmStorage, Task, TaskStatus, TaskPriority, AgentInfo, AgentCapability};

/// Swarm coordinator - manages task distribution and agent lifecycle
pub struct SwarmCoordinator {
    storage: Arc<dyn SwarmStorage>,
    message_queue: Arc<RwLock<Vec<String>>>,
}

impl SwarmCoordinator {
    pub fn new(storage: Arc<dyn SwarmStorage>) -> Self {
        Self {
            storage,
            message_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Submit a task to the swarm
    pub async fn submit_task(&self, title: String, description: String, priority: TaskPriority) -> Result<String> {
        let task = Task::new(title, description, priority);
        let task_id = task.task_id.clone();
        self.storage.store_task(&task).await?;
        Ok(task_id)
    }
    
    /// Assign task to an agent based on capabilities
    pub async fn assign_task(&self, task_id: &str) -> Result<Option<String>> {
        let task = match self.storage.get_task(task_id).await? {
            Some(t) => t,
            None => return Ok(None),
        };
        
        // Find available agent
        let agents = self.storage.list_active_agents().await?;
        if let Some(agent) = agents.first() {
            self.storage.update_task_status(task_id, TaskStatus::Running).await?;
            return Ok(Some(agent.agent_id.clone()));
        }
        
        Ok(None)
    }
    
    /// Register a new agent
    pub async fn register_agent(&self, name: String, capabilities: Vec<AgentCapability>) -> Result<String> {
        let agent = AgentInfo::new(name, capabilities);
        let agent_id = agent.agent_id.clone();
        self.storage.register_agent(&agent).await?;
        Ok(agent_id)
    }
    
    /// Update agent heartbeat
    pub async fn heartbeat(&self, agent_id: &str) -> Result<()> {
        self.storage.update_agent_heartbeat(agent_id).await
    }
    
    /// Queue a message for steering (interrupts current execution)
    pub async fn queue_message(&self, message: String) {
        let mut queue = self.message_queue.write().await;
        queue.push(message);
    }
    
    /// Get next queued message (for steering)
    pub async fn pop_message(&self) -> Option<String> {
        let mut queue = self.message_queue.write().await;
        if !queue.is_empty() {
            Some(queue.remove(0))
        } else {
            None
        }
    }
    
    /// List all pending tasks
    pub async fn list_pending_tasks(&self) -> Result<Vec<Task>> {
        self.storage.list_pending_tasks().await
    }
    
    /// List all active agents
    pub async fn list_active_agents(&self) -> Result<Vec<AgentInfo>> {
        self.storage.list_active_agents().await
    }
}
