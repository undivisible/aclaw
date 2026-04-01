//! CronTool — schedule, list, and manage recurring agent tasks.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use super::traits::*;
use crate::scheduler::Scheduler;

pub struct CronTool {
    scheduler: Arc<Scheduler>,
}

impl CronTool {
    pub fn new(scheduler: Arc<Scheduler>) -> Self {
        Self { scheduler }
    }
}

#[derive(Deserialize)]
struct CronArgs {
    /// Action: "schedule", "list", "enable", "disable", "delete"
    action: String,
    /// Cron expression (required for schedule)
    #[serde(default)]
    cron: String,
    /// Goal/task description (required for schedule)
    #[serde(default)]
    goal: String,
    /// Priority 1-10 (default 5)
    #[serde(default = "default_priority")]
    priority: u8,
    /// Schedule ID (required for enable/disable/delete)
    #[serde(default)]
    id: String,
}

fn default_priority() -> u8 {
    5
}

#[async_trait]
impl Tool for CronTool {
    fn name(&self) -> &str {
        "cron"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "cron".to_string(),
            description: "Schedule recurring agent tasks using cron expressions. \
                Actions: schedule (create), list, enable, disable, delete."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["schedule", "list", "enable", "disable", "delete"],
                        "description": "Operation to perform"
                    },
                    "cron": {
                        "type": "string",
                        "description": "Cron expression (e.g. '0 9 * * MON' = every Monday at 9am)"
                    },
                    "goal": {
                        "type": "string",
                        "description": "What the agent should do when this schedule fires"
                    },
                    "priority": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 10,
                        "description": "Task priority 1-10 (default 5)"
                    },
                    "id": {
                        "type": "string",
                        "description": "Schedule ID (required for enable/disable/delete)"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: CronArgs = serde_json::from_str(arguments)?;

        match args.action.as_str() {
            "schedule" => {
                if args.cron.is_empty() {
                    return Ok(ToolResult::error("cron expression is required"));
                }
                if args.goal.is_empty() {
                    return Ok(ToolResult::error("goal is required"));
                }
                match self.scheduler.schedule(&args.cron, &args.goal, args.priority).await {
                    Ok(id) => Ok(ToolResult::success(format!(
                        "Scheduled '{}' with id={} (cron: {})",
                        args.goal, id, args.cron
                    ))),
                    Err(e) => Ok(ToolResult::error(format!("Failed to schedule: {}", e))),
                }
            }

            "list" => {
                let schedules = self.scheduler.list().await;
                if schedules.is_empty() {
                    return Ok(ToolResult::success("No schedules configured."));
                }
                let lines: Vec<String> = schedules
                    .iter()
                    .map(|s| {
                        format!(
                            "- [{}] id={} cron='{}' goal='{}' enabled={}",
                            if s.enabled { "✓" } else { "✗" },
                            &s.id[..8],
                            s.cron_expression,
                            s.task_goal,
                            s.enabled
                        )
                    })
                    .collect();
                Ok(ToolResult::success(lines.join("\n")))
            }

            "enable" => {
                if args.id.is_empty() {
                    return Ok(ToolResult::error("id is required"));
                }
                match self.scheduler.enable(&args.id).await {
                    Ok(_) => Ok(ToolResult::success(format!("Enabled {}", args.id))),
                    Err(e) => Ok(ToolResult::error(e.to_string())),
                }
            }

            "disable" => {
                if args.id.is_empty() {
                    return Ok(ToolResult::error("id is required"));
                }
                match self.scheduler.disable(&args.id).await {
                    Ok(_) => Ok(ToolResult::success(format!("Disabled {}", args.id))),
                    Err(e) => Ok(ToolResult::error(e.to_string())),
                }
            }

            "delete" => {
                if args.id.is_empty() {
                    return Ok(ToolResult::error("id is required"));
                }
                match self.scheduler.delete(&args.id).await {
                    Ok(_) => Ok(ToolResult::success(format!("Deleted {}", args.id))),
                    Err(e) => Ok(ToolResult::error(e.to_string())),
                }
            }

            other => Ok(ToolResult::error(format!("Unknown action: {}", other))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn schedule_and_list() {
        let scheduler = Arc::new(Scheduler::new());
        let tool = CronTool::new(scheduler);

        let r = tool
            .execute(r#"{"action":"schedule","cron":"0 9 * * *","goal":"daily standup"}"#)
            .await
            .unwrap();
        assert!(!r.is_error, "{}", r.output);

        let l = tool.execute(r#"{"action":"list"}"#).await.unwrap();
        assert!(l.output.contains("daily standup"));
    }

    #[tokio::test]
    async fn invalid_cron_fails() {
        let scheduler = Arc::new(Scheduler::new());
        let tool = CronTool::new(scheduler);
        let r = tool
            .execute(r#"{"action":"schedule","cron":"not-valid","goal":"test"}"#)
            .await
            .unwrap();
        assert!(r.is_error);
    }
}
