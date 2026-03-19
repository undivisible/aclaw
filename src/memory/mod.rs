//! Memory abstraction — persistent agent state.
//! Inspired by ZeroClaw's pluggable memory + NanoClaw's per-group isolation.

pub mod embeddings;
pub mod hnsw;
pub mod search;
pub mod surreal;
pub mod traits;

pub use traits::MemoryBackend;
