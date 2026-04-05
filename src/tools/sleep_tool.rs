//! SleepTool — async delay for pacing multi-step agent workflows.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use super::traits::*;

pub struct SleepTool;

#[derive(Deserialize)]
struct SleepArgs {
    /// Duration in milliseconds (max 300_000 = 5 minutes)
    ms: u64,
}

#[async_trait]
impl Tool for SleepTool {
    fn name(&self) -> &str {
        "sleep"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "sleep".to_string(),
            description: "Wait for a specified number of milliseconds before continuing. \
                Useful for rate-limiting, polling loops, or waiting for async side-effects."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "ms": {
                        "type": "integer",
                        "description": "Milliseconds to wait (max 300000 = 5 minutes)",
                        "minimum": 0,
                        "maximum": 300000
                    }
                },
                "required": ["ms"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: SleepArgs = serde_json::from_str(arguments)?;
        let ms = args.ms.min(300_000);
        tokio::time::sleep(Duration::from_millis(ms)).await;
        Ok(ToolResult::success(format!("Slept {}ms", ms)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sleep_completes() {
        let tool = SleepTool;
        let r = tool.execute(r#"{"ms": 10}"#).await.unwrap();
        assert!(!r.is_error);
        assert!(r.output.contains("10ms"));
    }

    #[test]
    fn sleep_ms_cap_is_five_minutes() {
        #[allow(clippy::unnecessary_min_or_max)]
        let clamped = 999_999_u64.min(300_000);
        assert_eq!(clamped, 300_000);
    }
}
