//! Memory abstraction — persistent agent state.
//! Inspired by ZeroClaw's pluggable memory + NanoClaw's per-group isolation.

pub mod embeddings;
pub mod search;
pub mod sqlite;
#[cfg(feature = "swarm")]
pub mod surreal;
pub mod traits;

pub use traits::MemoryBackend;
