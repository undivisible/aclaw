//! Runtime abstraction — execution environments.
//! Inspired by ZeroClaw's RuntimeAdapter + NanoClaw's container isolation.

pub mod traits;
pub mod native;
pub mod docker;

pub use traits::RuntimeAdapter;
