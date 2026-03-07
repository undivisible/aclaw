//! OpenAI-compatible provider — works with OpenAI, OpenRouter, Groq, Together, etc.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::*;
use crate::tools::ToolSpec;

pub struct OpenAiCompatProvider {
    api_key: String,
    base_url: String,
    provider_name: String,
}

impl OpenAiCompatProvider {
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            provider_name: name.into(),
        }
    }

    /// OpenAI
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self::new(api_key, "https://api.openai.com/v1", "openai")
    }

    /// OpenRouter
    pub fn openrouter(api_key: impl Into<String>) -> Self {
        Self::new(api_key, "https://openrouter.ai/api/v1", "openrouter")
    }

    /// Groq
    pub fn groq(api_key: impl Into<String>) -> Self {
        Self::new(api_key, "https://api.groq.com/openai/v1", "groq")
    }

    fn build_tools_payload(&self, tools: &[ToolSpec]) -> Vec<Value> {
        tools.iter().map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }
            })
        }).collect()
    }
}

#[async_trait]
impl Provider for OpenAiCompatProvider {
    fn name(&self) -> &str { &self.provider_name }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            native_tools: true,
            streaming: true,
            vision: true,
            max_context: 128_000,
        }
    }

    async fn chat(&self, request: &ChatRequest) -> anyhow::Result<ChatResponse> {
        let client = reqwest::Client::new();

        let messages: Vec<Value> = request.messages.iter()
            .map(|m| serde_json::json!({ "role": &m.role, "content": &m.content }))
            .collect();

        let mut body = serde_json::json!({
            "model": &request.model,
            "messages": messages,
            "temperature": request.temperature,
        });

        if let Some(max) = request.max_tokens {
            body["max_tokens"] = Value::Number(max.into());
        }

        if let Some(tools) = &request.tools {
            if !tools.is_empty() {
                body["tools"] = Value::Array(self.build_tools_payload(tools));
            }
        }

        let resp = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("{} API error {}: {}", self.provider_name, status, &text[..text.len().min(200)]);
        }

        let data: Value = resp.json().await?;
        let choice = &data["choices"][0];

        let text = choice["message"]["content"].as_str().map(String::from);

        let tool_calls = choice["message"]["tool_calls"]
            .as_array()
            .map(|calls| {
                calls.iter().map(|tc| ToolCall {
                    id: tc["id"].as_str().unwrap_or("").to_string(),
                    name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                    arguments: tc["function"]["arguments"].as_str().unwrap_or("{}").to_string(),
                }).collect()
            })
            .unwrap_or_default();

        let usage = data["usage"].as_object().map(|u| Usage {
            input_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            output_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        });

        Ok(ChatResponse { text, tool_calls, usage })
    }
}
