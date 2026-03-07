//! Docker runtime — isolated container execution (inspired by NanoClaw).
//! Agents run in their own Linux containers with filesystem isolation.

use std::path::{Path, PathBuf};
use tokio::process::Command;

use super::traits::RuntimeAdapter;

pub struct DockerRuntime {
    image: String,
    storage: PathBuf,
    memory_limit: u64, // bytes
}

impl DockerRuntime {
    pub fn new(image: impl Into<String>, storage: PathBuf) -> Self {
        Self {
            image: image.into(),
            storage,
            memory_limit: 512 * 1024 * 1024, // 512MB default
        }
    }

    pub fn with_memory_limit(mut self, bytes: u64) -> Self {
        self.memory_limit = bytes;
        self
    }
}

impl RuntimeAdapter for DockerRuntime {
    fn name(&self) -> &str { "docker" }
    fn has_shell(&self) -> bool { true }
    fn has_filesystem(&self) -> bool { true }
    fn storage_path(&self) -> PathBuf { self.storage.clone() }
    fn memory_budget(&self) -> u64 { self.memory_limit }

    fn build_command(&self, command: &str, workspace: &Path) -> anyhow::Result<Command> {
        let mut cmd = Command::new("docker");
        cmd.args([
            "run", "--rm",
            "--memory", &format!("{}m", self.memory_limit / (1024 * 1024)),
            "--cpus", "2",
            "-v", &format!("{}:/workspace", workspace.display()),
            "-w", "/workspace",
            &self.image,
            "bash", "-c", command,
        ]);
        Ok(cmd)
    }
}
