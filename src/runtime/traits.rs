//! Core RuntimeAdapter trait.

use std::path::{Path, PathBuf};

/// Runtime adapter — abstracts where agent code executes.
/// Native = same process. Docker = isolated container. WASM = sandboxed.
pub trait RuntimeAdapter: Send + Sync {
    /// Runtime name
    fn name(&self) -> &str;

    /// Whether shell access is available
    fn has_shell(&self) -> bool;

    /// Whether filesystem access is available
    fn has_filesystem(&self) -> bool;

    /// Base storage path
    fn storage_path(&self) -> PathBuf;

    /// Maximum memory budget in bytes (0 = unlimited)
    fn memory_budget(&self) -> u64 { 0 }

    /// Build a shell command for this runtime
    fn build_command(&self, command: &str, workspace: &Path) -> anyhow::Result<tokio::process::Command>;
}
