use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent_id: String,
    pub name: String,
    pub capabilities: Vec<AgentCapability>,
    pub status: AgentStatus,
    pub last_heartbeat: DateTime<Utc>,
    /// LLM model this agent uses
    pub model: Option<String>,
    /// Tool names this agent has access to
    pub tools: Option<Vec<String>>,
    /// Maximum concurrent incoming delegations
    pub max_concurrent: Option<i32>,
    /// Agent-specific settings
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Active,
    Idle,
    Busy,
    Dead,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Active => write!(f, "active"),
            AgentStatus::Idle => write!(f, "idle"),
            AgentStatus::Busy => write!(f, "busy"),
            AgentStatus::Dead => write!(f, "dead"),
        }
    }
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
    Security,
    DataAnalysis,
    Communication,
}

impl std::fmt::Display for AgentCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentCapability::Coding => write!(f, "coding"),
            AgentCapability::Research => write!(f, "research"),
            AgentCapability::Review => write!(f, "review"),
            AgentCapability::Testing => write!(f, "testing"),
            AgentCapability::Documentation => write!(f, "documentation"),
            AgentCapability::Design => write!(f, "design"),
            AgentCapability::DevOps => write!(f, "devops"),
            AgentCapability::Security => write!(f, "security"),
            AgentCapability::DataAnalysis => write!(f, "data-analysis"),
            AgentCapability::Communication => write!(f, "communication"),
        }
    }
}

impl AgentInfo {
    pub fn new(name: String, capabilities: Vec<AgentCapability>) -> Self {
        Self {
            agent_id: uuid::Uuid::new_v4().to_string(),
            name,
            capabilities,
            status: AgentStatus::Active,
            last_heartbeat: Utc::now(),
            model: None,
            tools: None,
            max_concurrent: Some(5),
            settings: None,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_max_concurrent(mut self, max: i32) -> Self {
        self.max_concurrent = Some(max);
        self
    }
}

#[cfg(feature = "swarm")]
use crate::swarm::storage::SwarmStorage;
use std::sync::Arc;

pub struct AgentRegistry {
    #[cfg(feature = "swarm")]
    storage: Arc<dyn SwarmStorage>,
}

impl AgentRegistry {
    #[cfg(feature = "swarm")]
    pub fn new(storage: Arc<dyn SwarmStorage>) -> Self {
        Self { storage }
    }

    pub async fn register(&self, agent: AgentInfo) -> anyhow::Result<String> {
        #[cfg(feature = "swarm")]
        {
            self.storage.upsert_agent(&agent).await?;
            Ok(agent.agent_id)
        }
        #[cfg(not(feature = "swarm"))]
        {
            let _ = agent;
            anyhow::bail!("Swarm storage requires 'swarm' feature")
        }
    }

    pub async fn get_agent(&self, id: &str) -> anyhow::Result<Option<AgentInfo>> {
        #[cfg(feature = "swarm")]
        {
            self.storage.get_agent(id).await
        }
        #[cfg(not(feature = "swarm"))]
        {
            let _ = id;
            Ok(None)
        }
    }

    pub async fn find_by_capability(&self, cap: AgentCapability) -> anyhow::Result<Vec<AgentInfo>> {
        #[cfg(feature = "swarm")]
        {
            let agents = self.storage.list_all_agents().await?;
            Ok(agents
                .into_iter()
                .filter(|a| a.capabilities.contains(&cap) && a.status != AgentStatus::Dead)
                .collect())
        }
        #[cfg(not(feature = "swarm"))]
        {
            let _ = cap;
            Ok(Vec::new())
        }
    }

    pub async fn heartbeat(&self, id: &str) -> anyhow::Result<()> {
        #[cfg(feature = "swarm")]
        {
            if let Some(mut agent) = self.storage.get_agent(id).await? {
                agent.last_heartbeat = Utc::now();
                agent.status = AgentStatus::Active;
                self.storage.upsert_agent(&agent).await?;
            }
            Ok(())
        }
        #[cfg(not(feature = "swarm"))]
        {
            let _ = id;
            Ok(())
        }
    }
}
