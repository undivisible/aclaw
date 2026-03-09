//! Swarm coordination module — multi-agent delegation, teams, handoffs, and evaluation loops.
//!
//! Requires `swarm` feature for SurrealDB + RocksDB storage backend.
//! Without the feature, only basic task queue and agent registry types are available.

pub mod models;
pub mod task_queue;
pub mod agent_registry;

#[cfg(feature = "swarm")]
pub mod storage;
#[cfg(feature = "swarm")]
pub mod coordinator;
#[cfg(feature = "swarm")]
pub mod delegation;
#[cfg(feature = "swarm")]
pub mod team;
#[cfg(feature = "swarm")]
pub mod handoff;
#[cfg(feature = "swarm")]
pub mod evaluate;
#[cfg(feature = "swarm")]
pub mod scheduler;

pub use task_queue::{Task, TaskStatus, TaskPriority};
pub use agent_registry::{AgentInfo, AgentCapability, AgentStatus};
pub use models::*;

#[cfg(feature = "swarm")]
pub use coordinator::SwarmCoordinator;
#[cfg(feature = "swarm")]
pub use storage::{SwarmStorage, SurrealBackend, RocksCache};
#[cfg(feature = "swarm")]
pub use delegation::DelegationManager;
#[cfg(feature = "swarm")]
pub use team::TeamManager;
#[cfg(feature = "swarm")]
pub use handoff::HandoffManager;
#[cfg(feature = "swarm")]
pub use evaluate::{evaluate_loop, EvaluateConfig, EvalResult};
#[cfg(feature = "swarm")]
pub use scheduler::{ConcurrencyScheduler, Lane, ExecutionSlot};
