//! Handoff system — transfer conversation control between agents.
//!
//! When an agent hands off to another:
//! 1. A routing override is set for the channel+chat_id
//! 2. Future messages go to the new agent
//! 3. Context is transferred (conversation summary)
//! 4. The handoff can be reversed

use anyhow::{bail, Result};
use std::sync::Arc;

use super::models::HandoffRoute;
use super::storage::SwarmStorage;

/// Handoff manager — routes conversations between agents
pub struct HandoffManager {
    storage: Arc<dyn SwarmStorage>,
}

impl HandoffManager {
    pub fn new(storage: Arc<dyn SwarmStorage>) -> Self {
        Self { storage }
    }

    /// Transfer conversation control from one agent to another
    pub async fn handoff(
        &self,
        channel: &str,
        chat_id: &str,
        from_agent: &str,
        to_agent: &str,
        context: Option<String>,
    ) -> Result<HandoffRoute> {
        // Verify agents exist (by name)
        let from = self
            .storage
            .get_agent_by_name(from_agent)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Source agent '{}' not found", from_agent))?;
        let to = self
            .storage
            .get_agent_by_name(to_agent)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Target agent '{}' not found", to_agent))?;

        // Verify target agent is active
        if to.status.to_string() != "active" {
            bail!(
                "Target agent '{}' is not active (status: {:?})",
                to_agent,
                to.status
            );
        }

        let mut route = HandoffRoute::new(
            channel.to_string(),
            chat_id.to_string(),
            from.name.clone(),
            to.name.clone(),
        );
        if let Some(ctx) = context {
            route = route.with_context(ctx);
        }

        self.storage.set_handoff_route(&route).await?;
        Ok(route)
    }

    /// Check if there's an active handoff route for this conversation
    pub async fn get_route(&self, channel: &str, chat_id: &str) -> Result<Option<HandoffRoute>> {
        self.storage.get_handoff_route(channel, chat_id).await
    }

    /// Resolve which agent should handle this conversation
    pub async fn resolve_agent(
        &self,
        channel: &str,
        chat_id: &str,
        default_agent: &str,
    ) -> Result<String> {
        match self.storage.get_handoff_route(channel, chat_id).await? {
            Some(route) => Ok(route.to_agent_key),
            None => Ok(default_agent.to_string()),
        }
    }

    /// Return conversation control to the original agent
    pub async fn return_control(&self, channel: &str, chat_id: &str) -> Result<()> {
        self.storage.clear_handoff_route(channel, chat_id).await
    }

    /// List all active handoff routes
    pub async fn list_routes(&self) -> Result<Vec<HandoffRoute>> {
        self.storage.list_handoff_routes().await
    }
}
