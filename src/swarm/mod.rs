//! Swarm coordination module — multi-agent delegation, teams, handoffs, and evaluation loops.
//!
//! Requires `swarm` feature for SurrealDB + RocksDB storage backend.
//! Without the feature, only basic task queue and agent registry types are available.

pub mod agent_registry;
pub mod models;
pub mod task_queue;

#[cfg(feature = "swarm")]
pub mod coordinator;
#[cfg(feature = "swarm")]
pub mod delegation;
#[cfg(feature = "swarm")]
pub mod evaluate;
#[cfg(feature = "swarm")]
pub mod handoff;
#[cfg(feature = "swarm")]
pub mod scheduler;
#[cfg(feature = "swarm")]
pub mod storage;
#[cfg(feature = "swarm")]
pub mod team;

pub use agent_registry::{AgentCapability, AgentInfo, AgentStatus};
pub use models::*;
pub use task_queue::{Task, TaskPriority, TaskStatus};

#[cfg(feature = "swarm")]
pub use coordinator::SwarmCoordinator;
#[cfg(feature = "swarm")]
pub use delegation::DelegationManager;
#[cfg(feature = "swarm")]
pub use evaluate::{evaluate_loop, EvalResult, EvaluateConfig};
#[cfg(feature = "swarm")]
pub use handoff::HandoffManager;
#[cfg(feature = "swarm")]
pub use scheduler::{ConcurrencyScheduler, ExecutionSlot, Lane};
#[cfg(feature = "swarm")]
pub use storage::{RocksCache, SurrealBackend, SwarmStorage};
#[cfg(feature = "swarm")]
pub use team::TeamManager;
