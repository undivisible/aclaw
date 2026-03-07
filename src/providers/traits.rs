//! Core Provider trait — defines the interface for LLM backends.
//! Inspired by ZeroClaw's trait system with simplifications.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::ToolSpec;

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into() }
    }
    pub fn tool_result(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: "tool".into(), content: format!("[{}] {}", id.into(), content.into()) }
    }
}

/// A tool call requested by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// LLM response — text, tool calls, or both.
#[derive(Debug, Clone, Default)]
pub struct ChatResponse {
    pub text: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Default)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl ChatResponse {
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
    pub fn text_or_empty(&self) -> &str {
        self.text.as_deref().unwrap_or("")
    }
}

/// Chat request payload.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub tools: Option<Vec<ToolSpec>>,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: Option<u32>,
}

/// Provider capabilities
#[derive(Debug, Clone, Default)]
pub struct ProviderCapabilities {
    /// Supports native tool calling (not prompt-injection)
    pub native_tools: bool,
    /// Supports streaming responses
    pub streaming: bool,
    /// Supports vision/image input
    pub vision: bool,
    /// Maximum context window
    pub max_context: u32,
}

/// The core Provider trait.
/// Implement this for each LLM backend (Anthropic, OpenAI, Gemini, Ollama, etc.)
#[async_trait]
pub trait Provider: Send + Sync {
    /// Provider name (e.g., "anthropic", "openai", "ollama")
    fn name(&self) -> &str;

    /// Query capabilities
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }

    /// Send a chat request and get a response.
    async fn chat(&self, request: &ChatRequest) -> anyhow::Result<ChatResponse>;

    /// Simple one-shot message (convenience wrapper)
    async fn simple_chat(&self, message: &str, model: &str) -> anyhow::Result<String> {
        let request = ChatRequest {
            messages: vec![ChatMessage::user(message)],
            tools: None,
            model: model.to_string(),
            temperature: 0.7,
            max_tokens: None,
        };
        let response = self.chat(&request).await?;
        Ok(response.text.unwrap_or_default())
    }
}
