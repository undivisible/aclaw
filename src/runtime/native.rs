//! Native runtime — runs directly on the host.

use std::path::{Path, PathBuf};
use tokio::process::Command;

use super::traits::RuntimeAdapter;

pub struct NativeRuntime {
    storage: PathBuf,
}

impl NativeRuntime {
    pub fn new(storage: PathBuf) -> Self {
        Self { storage }
    }
}

impl RuntimeAdapter for NativeRuntime {
    fn name(&self) -> &str { "native" }
    fn has_shell(&self) -> bool { true }
    fn has_filesystem(&self) -> bool { true }
    fn storage_path(&self) -> PathBuf { self.storage.clone() }

    fn build_command(&self, command: &str, workspace: &Path) -> anyhow::Result<Command> {
        let mut cmd = Command::new("bash");
        cmd.arg("-c").arg(command).current_dir(workspace);
        Ok(cmd)
    }
}
