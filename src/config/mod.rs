//! Configuration management.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub provider: ProviderConfig,
    pub model: String,
    pub system_prompt: String,
    pub workspace: PathBuf,
    pub runtime: RuntimeConfig,
    pub channel: ChannelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub kind: String, // "native", "docker"
    pub docker_image: Option<String>,
    pub memory_limit_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub kind: String, // "cli", "telegram", "discord", "websocket"
    pub token: Option<String>,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn default_config() -> Self {
        Self {
            provider: ProviderConfig {
                name: "anthropic".to_string(),
                api_key: None,
                base_url: None,
            },
            model: "claude-sonnet-4-5-20250514".to_string(),
            system_prompt: "You are a helpful AI assistant.".to_string(),
            workspace: PathBuf::from("."),
            runtime: RuntimeConfig {
                kind: "native".to_string(),
                docker_image: None,
                memory_limit_mb: None,
            },
            channel: ChannelConfig {
                kind: "cli".to_string(),
                token: None,
            },
        }
    }
}
