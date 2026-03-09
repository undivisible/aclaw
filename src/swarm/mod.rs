// Swarm coordination module
// Requires surrealdb feature for storage backend
pub mod task_queue;
pub mod agent_registry;

#[cfg(feature = "surrealdb")]
pub mod storage;
#[cfg(feature = "surrealdb")]
pub mod coordinator;

pub use task_queue::{Task, TaskStatus, TaskPriority};
pub use agent_registry::{AgentInfo, AgentCapability};

#[cfg(feature = "surrealdb")]
pub use coordinator::SwarmCoordinator;
#[cfg(feature = "surrealdb")]
pub use storage::{SwarmStorage, SurrealBackend, RocksCache};
