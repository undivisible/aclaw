//! Vibemania tool — autonomous agent coding & codebase exploration.
//! Integrates Subspace's code-first agent system as a composable tool.

use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;

use super::traits::*;

pub struct VibemaniaTool {
    workspace: PathBuf,
}

impl VibemaniaTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[derive(Deserialize)]
struct VibemaiaArgs {
    goal: String,
    #[serde(default)]
    parallel: usize,
}

#[async_trait]
impl Tool for VibemaniaTool {
    fn name(&self) -> &str { "vibemania" }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "vibemania".to_string(),
            description: "Run Vibemania for autonomous code exploration, implementation, and codebase analysis. Supports parallel execution.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "goal": {
                        "type": "string",
                        "description": "What to build/fix/analyze (e.g., 'add WebSocket support', 'fix auth bug', 'explore database schema')"
                    },
                    "parallel": {
                        "type": "integer",
                        "description": "Number of parallel workers (1-8, default 2)",
                        "minimum": 1,
                        "maximum": 8,
                        "default": 2
                    }
                },
                "required": ["goal"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: VibemaiaArgs = serde_json::from_str(arguments)?;
        
        // Check if vibemania is available
        let vibemania_bin = self.workspace.parent()
            .and_then(|p| Some(p.join("vibemania/target/release/vibemania")))
            .filter(|p| p.exists());

        if vibemania_bin.is_none() {
            return Ok(ToolResult::error(
                "Vibemania not found. Clone/build from atechnology-company/vibemania first."
            ));
        }

        let parallel = args.parallel.max(1).min(8);

        // Spawn vibemania process
        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(format!(
                "cd {} && {} run \"{}\" --parallel {}",
                self.workspace.display(),
                vibemania_bin.unwrap().display(),
                &args.goal,
                parallel
            ))
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result = if !stderr.is_empty() {
            format!("stdout:\n{}\n\nstderr:\n{}", stdout, stderr)
        } else {
            stdout.to_string()
        };

        Ok(if output.status.success() {
            ToolResult::success(result)
        } else {
            ToolResult::error(result)
        })
    }
}
