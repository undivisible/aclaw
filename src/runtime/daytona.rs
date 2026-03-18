//! Daytona runtime adapter scaffold.

use std::path::{Path, PathBuf};

use tokio::process::Command;

use super::traits::RuntimeAdapter;

pub struct DaytonaRuntime {
    workspace_id: String,
    storage: PathBuf,
}

impl DaytonaRuntime {
    pub fn new(workspace_id: impl Into<String>, storage: PathBuf) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            storage,
        }
    }
}

impl RuntimeAdapter for DaytonaRuntime {
    fn name(&self) -> &str {
        "daytona"
    }

    fn has_shell(&self) -> bool {
        true
    }

    fn has_filesystem(&self) -> bool {
        true
    }

    fn storage_path(&self) -> PathBuf {
        self.storage.clone()
    }

    fn build_command(&self, command: &str, workspace: &Path) -> anyhow::Result<Command> {
        let mut cmd = Command::new("daytona");
        cmd.args([
            "workspace",
            "exec",
            &self.workspace_id,
            "--",
            "bash",
            "-lc",
            command,
        ])
        .current_dir(workspace);
        Ok(cmd)
    }
}
