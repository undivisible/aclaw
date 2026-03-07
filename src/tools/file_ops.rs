//! File operations tool — read, write, list files.

use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;

use super::traits::*;

pub struct FileReadTool {
    workspace: PathBuf,
}

impl FileReadTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[derive(Deserialize)]
struct ReadArgs {
    path: String,
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str { "file_read" }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "file_read".to_string(),
            description: "Read a file's contents".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path relative to workspace" }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: ReadArgs = serde_json::from_str(arguments)?;
        let full_path = self.workspace.join(&args.path);

        // Prevent path traversal
        if !full_path.starts_with(&self.workspace) {
            return Ok(ToolResult::error("Path traversal not allowed"));
        }

        match tokio::fs::read_to_string(&full_path).await {
            Ok(content) => {
                let truncated = if content.len() > 50_000 {
                    format!("{}...\n[truncated]", &content[..50_000])
                } else {
                    content
                };
                Ok(ToolResult::success(truncated))
            }
            Err(e) => Ok(ToolResult::error(format!("Failed to read: {}", e))),
        }
    }
}

pub struct FileWriteTool {
    workspace: PathBuf,
}

impl FileWriteTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[derive(Deserialize)]
struct WriteArgs {
    path: String,
    content: String,
}

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str { "file_write" }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "file_write".to_string(),
            description: "Write content to a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path relative to workspace" },
                    "content": { "type": "string", "description": "Content to write" }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: WriteArgs = serde_json::from_str(arguments)?;
        let full_path = self.workspace.join(&args.path);

        if !full_path.starts_with(&self.workspace) {
            return Ok(ToolResult::error("Path traversal not allowed"));
        }

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        match tokio::fs::write(&full_path, &args.content).await {
            Ok(_) => Ok(ToolResult::success(format!("Wrote {} bytes to {}", args.content.len(), args.path))),
            Err(e) => Ok(ToolResult::error(format!("Failed to write: {}", e))),
        }
    }
}
