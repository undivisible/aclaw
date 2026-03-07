//! Ollama provider — local model support via Ollama API.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::*;

pub struct OllamaProvider {
    base_url: String,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into() }
    }

    pub fn default() -> Self {
        Self::new("http://localhost:11434")
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str { "ollama" }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            native_tools: false, // Most Ollama models don't support native tools
            streaming: true,
            vision: false,
            max_context: 32_000,
        }
    }

    async fn chat(&self, request: &ChatRequest) -> anyhow::Result<ChatResponse> {
        let client = reqwest::Client::new();

        let messages: Vec<Value> = request.messages.iter()
            .map(|m| serde_json::json!({ "role": &m.role, "content": &m.content }))
            .collect();

        let body = serde_json::json!({
            "model": &request.model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": request.temperature,
            }
        });

        let resp = client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama error: {}", &text[..text.len().min(200)]);
        }

        let data: Value = resp.json().await?;
        let text = data["message"]["content"].as_str().map(String::from);

        Ok(ChatResponse {
            text,
            tool_calls: vec![],
            usage: None,
        })
    }
}
