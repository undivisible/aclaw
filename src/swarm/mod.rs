//! Swarm coordination module — multi-agent delegation, teams, handoffs, and evaluation loops.
//!
//! Requires `plugin-swarm` for SurrealDB + RocksDB storage backend.
//! Without the feature, only basic task queue and agent registry types are available.

pub mod agent_registry;
pub mod models;
pub mod task_queue;

#[cfg(feature = "plugin-swarm")]
pub mod coordinator;
#[cfg(feature = "plugin-swarm")]
pub mod delegation;
#[cfg(feature = "plugin-swarm")]
pub mod evaluate;
#[cfg(feature = "plugin-swarm")]
pub mod handoff;
#[cfg(feature = "plugin-swarm")]
pub mod scheduler;
#[cfg(feature = "plugin-swarm")]
pub mod storage;
#[cfg(feature = "plugin-swarm")]
pub mod team;

pub use agent_registry::{AgentCapability, AgentInfo, AgentRegistry, AgentStatus};
pub use models::*;
pub use task_queue::{Task, TaskPriority, TaskStatus};

#[cfg(feature = "plugin-swarm")]
pub use coordinator::SwarmCoordinator;
#[cfg(feature = "plugin-swarm")]
pub use delegation::DelegationManager;
#[cfg(feature = "plugin-swarm")]
pub use evaluate::{evaluate_loop, EvalResult, EvaluateConfig};
#[cfg(feature = "plugin-swarm")]
pub use handoff::HandoffManager;
#[cfg(feature = "plugin-swarm")]
pub use scheduler::{ConcurrencyScheduler, ExecutionSlot, Lane};
#[cfg(feature = "plugin-swarm")]
pub use storage::{RocksCache, SurrealBackend, SwarmStorage};
#[cfg(feature = "plugin-swarm")]
pub use team::TeamManager;
