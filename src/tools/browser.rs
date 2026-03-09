//! Browser tool — open URLs, take screenshots, extract content.
//! Lightweight implementation using headless chromium via CLI.

use async_trait::async_trait;
use serde::Deserialize;

use super::traits::*;

pub struct BrowserTool;

impl BrowserTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct BrowserArgs {
    /// Action: open, screenshot, extract, status
    action: String,
    /// URL to open/screenshot/extract
    url: Option<String>,
    /// CSS selector for extraction
    selector: Option<String>,
}

#[async_trait]
impl Tool for BrowserTool {
    fn name(&self) -> &str { "browser" }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "browser".to_string(),
            description: "Control a browser. Actions: open (open URL), screenshot (capture page), extract (get text content from selector).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["open", "screenshot", "extract", "status"],
                        "description": "Browser action"
                    },
                    "url": {
                        "type": "string",
                        "description": "URL to open/screenshot/extract"
                    },
                    "selector": {
                        "type": "string",
                        "description": "CSS selector for extraction (default: body)"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: BrowserArgs = serde_json::from_str(arguments)?;

        match args.action.as_str() {
            "status" => {
                // Check if chromium/chrome is available
                let output = tokio::process::Command::new("which")
                    .arg("chromium-browser")
                    .output()
                    .await;
                let has_chromium = output.map(|o| o.status.success()).unwrap_or(false);

                let output = tokio::process::Command::new("which")
                    .arg("google-chrome")
                    .output()
                    .await;
                let has_chrome = output.map(|o| o.status.success()).unwrap_or(false);

                Ok(ToolResult::success(format!(
                    "Browser status:\n  chromium: {}\n  chrome: {}",
                    if has_chromium { "available" } else { "not found" },
                    if has_chrome { "available" } else { "not found" },
                )))
            }

            "open" | "extract" => {
                let url = match &args.url {
                    Some(u) => u.clone(),
                    None => return Ok(ToolResult::error("url required for open/extract action")),
                };

                // Use web_fetch as fallback since we may not have a display
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .user_agent("unthinkclaw/0.1")
                    .build()?;

                let resp = match client.get(&url).send().await {
                    Ok(r) => r,
                    Err(e) => return Ok(ToolResult::error(format!("Failed to fetch: {}", e))),
                };

                if !resp.status().is_success() {
                    return Ok(ToolResult::error(format!("HTTP {}", resp.status())));
                }

                let html = resp.text().await.unwrap_or_default();

                // Simple text extraction
                let text = strip_tags(&html);
                let truncated = if text.len() > 30_000 {
                    format!("{}...\n[truncated at 30k chars]", &text[..30_000])
                } else {
                    text
                };

                Ok(ToolResult::success(truncated))
            }

            "screenshot" => {
                let url = match &args.url {
                    Some(u) => u.clone(),
                    None => return Ok(ToolResult::error("url required for screenshot")),
                };

                // Try headless chrome screenshot
                let output = tokio::process::Command::new("google-chrome")
                    .args(&[
                        "--headless", "--disable-gpu", "--no-sandbox",
                        "--screenshot=/tmp/unthinkclaw_screenshot.png",
                        &format!("--window-size=1280,720"),
                        &url,
                    ])
                    .output()
                    .await;

                match output {
                    Ok(o) if o.status.success() => {
                        Ok(ToolResult::success("Screenshot saved to /tmp/unthinkclaw_screenshot.png"))
                    }
                    _ => {
                        Ok(ToolResult::error("Chrome not available for screenshots. Use 'extract' action instead."))
                    }
                }
            }

            other => Ok(ToolResult::error(format!("Unknown action: {}. Use: open, screenshot, extract, status", other))),
        }
    }
}

/// Strip HTML tags (basic)
fn strip_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            // Crude script detection
            let lower = html.to_lowercase();
            if lower.contains("<script") { in_script = true; }
            if lower.contains("</script") { in_script = false; }
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag && !in_script {
            result.push(ch);
        }
    }

    // Collapse whitespace
    let mut prev_newline = false;
    let mut cleaned = String::new();
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_newline {
                cleaned.push('\n');
                prev_newline = true;
            }
        } else {
            cleaned.push_str(trimmed);
            cleaned.push('\n');
            prev_newline = false;
        }
    }

    cleaned.trim().to_string()
}
