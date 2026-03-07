//! LLM Provider abstraction — swap backends without changing agent logic.
//! Supports: Anthropic (OAuth + API), OpenAI, GitHub Copilot, Ollama,
//!           OpenRouter, Groq, Together, Mistral, DeepSeek, Fireworks,
//!           Perplexity, xAI, Moonshot, Venice, HuggingFace, SiliconFlow,
//!           Cerebras, MiniMax, Vercel, Cloudflare

pub mod traits;
pub mod anthropic;
pub mod openai_compat;
pub mod ollama;
pub mod oauth;
pub mod copilot;

pub use traits::{ChatMessage, ChatRequest, ChatResponse, Provider, ToolCall};
