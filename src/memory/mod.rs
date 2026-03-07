//! Memory abstraction — persistent agent state.
//! Inspired by ZeroClaw's pluggable memory + NanoClaw's per-group isolation.

pub mod traits;
pub mod sqlite;

pub use traits::MemoryBackend;
