use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent_id: String,
    pub name: String,
    pub capabilities: Vec<AgentCapability>,
    pub status: AgentStatus,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Active,
    Idle,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentCapability {
    Coding,
    Research,
    Review,
    Testing,
    Documentation,
    Design,
    DevOps,
}

impl AgentInfo {
    pub fn new(name: String, capabilities: Vec<AgentCapability>) -> Self {
        Self {
            agent_id: uuid::Uuid::new_v4().to_string(),
            name,
            capabilities,
            status: AgentStatus::Active,
            last_heartbeat: Utc::now(),
        }
    }
}
