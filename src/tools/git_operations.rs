use async_trait::async_trait;
use super::traits::*;
use serde_json::json;
use tokio::process::Command;
use anyhow::{anyhow, Context};

pub struct GitOperationsTool {
    repo_path: String,
}

impl GitOperationsTool {
    pub fn new(repo_path: String) -> Self {
        Self { repo_path }
    }

    fn sanitize_args(&self, args: &[&str]) -> anyhow::Result<()> {
        for arg in args {
            // Block dangerous git options
            if arg.starts_with("--exec=") || arg.starts_with("--upload-pack=") ||
               arg.starts_with("-c") || arg == "-c" {
                return Err(anyhow!("Blocked dangerous git option: {}", arg));
            }
            
            // Block shell metacharacters
            if arg.contains('|') || arg.contains(';') || arg.contains('>') ||
               arg.contains('<') || arg.contains('`') || arg.contains('$') ||
               arg.contains('(') || arg.contains(')') {
                return Err(anyhow!("Blocked shell metacharacters in argument: {}", arg));
            }
        }
        Ok(())
    }

    async fn run_git(&self, args: &[&str]) -> anyhow::Result<String> {
        self.sanitize_args(args)?;
        
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(args)
            .output()
            .await
            .context("Failed to execute git command")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("Git command failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    async fn git_status(&self) -> anyhow::Result<String> {
        let output = self.run_git(&["status", "--porcelain", "-b"]).await?;
        let lines: Vec<&str> = output.lines().collect();
        
        let mut branch = "unknown";
        let mut modified = Vec::new();
        let mut added = Vec::new();
        let mut deleted = Vec::new();
        let mut untracked = Vec::new();
        
        for line in lines {
            if line.starts_with("##") {
                branch = line.trim_start_matches("## ").split("...").next().unwrap_or("unknown");
            } else if line.len() >= 3 {
                let status = &line[0..2];
                let file = line[3..].trim();
                match status {
                    " M" | "M " | "MM" => modified.push(file),
                    "A " | "AM" => added.push(file),
                    " D" | "D " => deleted.push(file),
                    "??" => untracked.push(file),
                    _ => modified.push(file),
                }
            }
        }
        
        Ok(json!({
            "branch": branch,
            "modified": modified,
            "added": added,
            "deleted": deleted,
            "untracked": untracked
        }).to_string())
    }
}

#[async_trait]
impl Tool for GitOperationsTool {
    fn name(&self) -> &str {
        "git_operations"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "git_operations".to_string(),
            description: "Execute git operations on a repository. Supports status, diff, log, show, commit, add, checkout, branch, stash, reset, and rev-parse.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["status", "diff", "log", "show", "commit", "add", "checkout", "branch", "stash", "reset", "rev-parse"],
                        "description": "Git operation to perform"
                    },
                    "args": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Additional arguments for the git command",
                        "default": []
                    }
                },
                "required": ["operation"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .context("Failed to parse arguments")?;

        let operation = args["operation"].as_str()
            .ok_or_else(|| anyhow!("Missing operation"))?;
        
        let extra_args: Vec<&str> = args["args"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let result = match operation {
            "status" => self.git_status().await,
            "diff" => {
                let mut cmd_args = vec!["diff"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            "log" => {
                let mut cmd_args = vec!["log", "--oneline", "-n", "20"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            "show" => {
                let mut cmd_args = vec!["show"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            "commit" => {
                let mut cmd_args = vec!["commit"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            "add" => {
                let mut cmd_args = vec!["add"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            "checkout" => {
                let mut cmd_args = vec!["checkout"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            "branch" => {
                let mut cmd_args = vec!["branch"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            "stash" => {
                let mut cmd_args = vec!["stash"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            "reset" => {
                let mut cmd_args = vec!["reset"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            "rev-parse" => {
                let mut cmd_args = vec!["rev-parse"];
                cmd_args.extend(&extra_args);
                self.run_git(&cmd_args).await
            },
            _ => Err(anyhow!("Unknown operation: {}", operation)),
        };

        match result {
            Ok(output) => Ok(ToolResult::success(output)),
            Err(e) => Ok(ToolResult::error(format!("Git operation failed: {}", e))),
        }
    }
}