//! Runtime abstraction — execution environments.

pub mod daytona;
#[cfg(feature = "docker")]
pub mod docker;
pub mod native;
pub mod traits;

pub use traits::RuntimeAdapter;
