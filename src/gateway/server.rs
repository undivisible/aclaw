//! HTTP/WebSocket gateway for remote container management and chat integration.
//! Enables subspace-editor and other clients to control agents over the network.

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMessage {
    pub id: String,
    pub kind: String, // "chat", "tool_call", "status", "error"
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    pub id: String,
    pub name: String,
    pub runtime: String, // "docker", "native"
    pub status: String,  // "running", "stopped", "error"
    pub memory_used_mb: u64,
    pub cpu_percent: f64,
}

pub struct Gateway {
    addr: SocketAddr,
    tx: mpsc::Sender<GatewayMessage>,
    containers: Arc<RwLock<Vec<ContainerStatus>>>,
}

impl Gateway {
    pub fn new(addr: SocketAddr) -> (Self, mpsc::Receiver<GatewayMessage>) {
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                addr,
                tx,
                containers: Arc::new(RwLock::new(Vec::new())),
            },
            rx,
        )
    }

    pub async fn start(self) -> anyhow::Result<()> {
        println!("🚀 Gateway listening on {}", self.addr);
        
        // TODO: Implement axum HTTP/WebSocket server
        // - POST /api/chat — send message to agent
        // - GET /api/status — runtime status + container list
        // - GET /api/containers — list managed containers
        // - POST /api/containers/{id}/stop — stop a container
        // - WebSocket /ws — real-time message stream
        // - POST /api/agent/config — update agent configuration
        // - GET /api/memory/{namespace} — retrieve memories
        
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        Ok(())
    }

    pub async fn broadcast(&self, msg: GatewayMessage) -> anyhow::Result<()> {
        self.tx.send(msg).await?;
        Ok(())
    }
}
