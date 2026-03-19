//! Web search tool — search the web via Perplexity or Brave API.

use async_trait::async_trait;
use serde::Deserialize;

use super::traits::*;
use crate::text::truncate_chars;

pub struct WebSearchTool {
    api_key: Option<String>,
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self {
            api_key: std::env::var("PERPLEXITY_API_KEY").ok(),
        }
    }

    pub fn with_api_key(mut self, key: String) -> Self {
        self.api_key = Some(key);
        self
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct SearchArgs {
    query: String,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "web_search".to_string(),
            description: "Search the web for information. Returns relevant results with snippets."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: SearchArgs = serde_json::from_str(arguments)?;

        let api_key = match &self.api_key {
            Some(k) => k.clone(),
            None => return Ok(ToolResult::error("No PERPLEXITY_API_KEY set")),
        };

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.perplexity.ai/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "sonar",
                "messages": [
                    {"role": "user", "content": args.query}
                ]
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Ok(ToolResult::error(format!(
                "Search API error {}: {}",
                status,
                truncate_chars(&text, 200)
            )));
        }

        let data: serde_json::Value = resp.json().await?;
        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("No results found")
            .to_string();

        Ok(ToolResult::success(content))
    }
}
