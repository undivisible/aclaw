//! Agent loop — the core execution engine.
//! Processes incoming messages, calls LLM, executes tools, sends responses.

use std::sync::Arc;

use crate::channels::{Channel, IncomingMessage, OutgoingMessage};
use crate::memory::MemoryBackend;
use crate::providers::{ChatMessage, ChatRequest, Provider};
use crate::tools::Tool;

const MAX_TOOL_ROUNDS: usize = 10;

pub struct AgentRunner {
    provider: Arc<dyn Provider>,
    tools: Vec<Arc<dyn Tool>>,
    memory: Arc<dyn MemoryBackend>,
    system_prompt: String,
    model: String,
}

impl AgentRunner {
    pub fn new(
        provider: Arc<dyn Provider>,
        tools: Vec<Arc<dyn Tool>>,
        memory: Arc<dyn MemoryBackend>,
        system_prompt: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            tools,
            memory,
            system_prompt: system_prompt.into(),
            model: model.into(),
        }
    }

    /// Run the agent loop on a channel.
    pub async fn run(&self, channel: &mut dyn Channel) -> anyhow::Result<()> {
        let mut rx = channel.start().await?;

        tracing::info!("Agent started on channel: {}", channel.name());

        while let Some(msg) = rx.recv().await {
            match self.handle_message(&msg).await {
                Ok(response) => {
                    channel.send(OutgoingMessage {
                        chat_id: msg.chat_id.clone(),
                        text: response,
                        reply_to: Some(msg.id.clone()),
                    }).await?;
                }
                Err(e) => {
                    tracing::error!("Error handling message: {}", e);
                    channel.send(OutgoingMessage {
                        chat_id: msg.chat_id,
                        text: format!("Error: {}", e),
                        reply_to: Some(msg.id),
                    }).await?;
                }
            }
        }

        channel.stop().await?;
        Ok(())
    }

    /// Handle a single message — LLM call with tool loop.
    async fn handle_message(&self, msg: &IncomingMessage) -> anyhow::Result<String> {
        // Build conversation with system prompt + memory context
        let mut messages = vec![ChatMessage::system(&self.system_prompt)];

        // Add memory context if available
        if let Ok(memories) = self.memory.search("chat", &msg.text, 5).await {
            if !memories.is_empty() {
                let context: Vec<String> = memories.iter()
                    .map(|m| format!("- {}: {}", m.key, m.value))
                    .collect();
                messages.push(ChatMessage::system(format!(
                    "Relevant past context:\n{}",
                    context.join("\n")
                )));
            }
        }

        messages.push(ChatMessage::user(&msg.text));

        // Tool specs for function calling
        let tool_specs: Vec<crate::tools::ToolSpec> = self.tools.iter()
            .map(|t| t.spec())
            .collect();

        // Agent loop: LLM → tool calls → LLM → ... → final text
        for _round in 0..MAX_TOOL_ROUNDS {
            let request = ChatRequest {
                messages: messages.clone(),
                tools: if tool_specs.is_empty() { None } else { Some(tool_specs.clone()) },
                model: self.model.clone(),
                temperature: 0.7,
                max_tokens: None,
            };

            let response = self.provider.chat(&request).await?;

            if !response.has_tool_calls() {
                // No more tool calls — return the text response
                let text = response.text.unwrap_or_default();

                // Store the interaction in memory
                let _ = self.memory.store(
                    "chat",
                    &format!("msg_{}", msg.id),
                    &format!("User: {} | Assistant: {}", msg.text, &text[..text.len().min(200)]),
                    None,
                ).await;

                return Ok(text);
            }

            // Add assistant message with tool calls
            if let Some(text) = &response.text {
                messages.push(ChatMessage::assistant(text));
            }

            // Execute each tool call
            for tc in &response.tool_calls {
                let result = if let Some(tool) = self.tools.iter().find(|t| t.name() == tc.name) {
                    match tool.execute(&tc.arguments).await {
                        Ok(r) => r,
                        Err(e) => crate::tools::ToolResult::error(format!("Tool error: {}", e)),
                    }
                } else {
                    crate::tools::ToolResult::error(format!("Unknown tool: {}", tc.name))
                };

                messages.push(ChatMessage::tool_result(&tc.id, &result.output));
            }
        }

        Ok("Reached maximum tool rounds. Please try a simpler request.".to_string())
    }
}
