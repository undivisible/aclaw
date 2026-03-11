//! Plugin system for aclaw
//! JSON-RPC 2.0 based plugin interface (subspace-editor compatible)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;

use crate::policy::ExecutionPolicy;

/// Plugin trait — implement this to extend aclaw
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name (e.g., "ai", "remote", "tools", "vibemania", "git")
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// List available methods this plugin provides
    fn methods(&self) -> Vec<MethodSpec>;

    /// Execute a method (JSON-RPC style)
    async fn call(&self, method: &str, params: Value) -> Result<Value, PluginError>;
}

/// Method specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSpec {
    pub name: String,
    pub description: String,
    pub params: HashMap<String, String>, // param_name -> type
    pub returns: String,
}

/// Plugin error
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl PluginError {
    pub fn new(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}

/// Plugin registry — manage installed plugins
pub struct PluginRegistry {
    plugins: HashMap<String, std::sync::Arc<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: std::sync::Arc<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    /// Call a plugin method
    pub async fn call(
        &self,
        plugin: &str,
        method: &str,
        params: Value,
    ) -> Result<Value, PluginError> {
        let p = self
            .plugins
            .get(plugin)
            .ok_or_else(|| PluginError::new(-32601, "Plugin not found"))?;
        p.call(method, params).await
    }

    /// List all plugins
    pub fn list(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get plugin info
    pub fn info(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.get(name).map(|p| PluginInfo {
            name: p.name().to_string(),
            version: p.version().to_string(),
            methods: p.methods(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub methods: Vec<MethodSpec>,
}

// Official builtin plugins
pub struct AiPlugin;

#[async_trait]
impl Plugin for AiPlugin {
    fn name(&self) -> &str {
        "ai"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn methods(&self) -> Vec<MethodSpec> {
        vec![
            MethodSpec {
                name: "explain".to_string(),
                description: "Explain code".to_string(),
                params: {
                    let mut m = HashMap::new();
                    m.insert("code".to_string(), "string".to_string());
                    m
                },
                returns: "string".to_string(),
            },
            MethodSpec {
                name: "refactor".to_string(),
                description: "Refactor code".to_string(),
                params: {
                    let mut m = HashMap::new();
                    m.insert("code".to_string(), "string".to_string());
                    m
                },
                returns: "string".to_string(),
            },
        ]
    }

    async fn call(&self, method: &str, _params: Value) -> Result<Value, PluginError> {
        match method {
            "explain" => Ok(json!({ "result": "Code explanation would go here" })),
            "refactor" => Ok(json!({ "result": "Refactored code would go here" })),
            _ => Err(PluginError::new(-32601, "Method not found")),
        }
    }
}

pub struct ToolsPlugin {
    policy: Arc<ExecutionPolicy>,
}

impl ToolsPlugin {
    pub fn new(policy: Arc<ExecutionPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl Plugin for ToolsPlugin {
    fn name(&self) -> &str {
        "tools"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn methods(&self) -> Vec<MethodSpec> {
        vec![MethodSpec {
            name: "shell".to_string(),
            description: "Execute shell command".to_string(),
            params: {
                let mut m = HashMap::new();
                m.insert("cmd".to_string(), "string".to_string());
                m
            },
            returns: "string".to_string(),
        }]
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, PluginError> {
        match method {
            "shell" => {
                if !self.policy.allow_plugin_shell {
                    return Err(PluginError::new(
                        -32604,
                        "Plugin shell execution is disabled by policy",
                    ));
                }

                let cmd = params
                    .get("cmd")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::new(-32602, "Invalid params"))?;

                match tokio::time::timeout(
                    Duration::from_secs(120),
                    Command::new("sh").arg("-c").arg(cmd).output(),
                )
                .await
                {
                    Ok(Ok(output)) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(json!({ "stdout": stdout }))
                    }
                    Ok(Err(e)) => Err(PluginError::new(-32000, &e.to_string())),
                    Err(_) => Err(PluginError::new(-32000, "Command timed out")),
                }
            }
            _ => Err(PluginError::new(-32601, "Method not found")),
        }
    }
}

pub struct VibemaniaPlugin;

#[async_trait]
impl Plugin for VibemaniaPlugin {
    fn name(&self) -> &str {
        "vibemania"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn methods(&self) -> Vec<MethodSpec> {
        vec![
            MethodSpec {
                name: "run".to_string(),
                description: "Run task with goal".to_string(),
                params: {
                    let mut m = HashMap::new();
                    m.insert("goal".to_string(), "string".to_string());
                    m.insert("parallel".to_string(), "number".to_string());
                    m
                },
                returns: "object".to_string(),
            },
            MethodSpec {
                name: "dream".to_string(),
                description: "Generate ideas".to_string(),
                params: {
                    let mut m = HashMap::new();
                    m.insert("prompt".to_string(), "string".to_string());
                    m
                },
                returns: "object".to_string(),
            },
        ]
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, PluginError> {
        match method {
            "run" => {
                let goal = params
                    .get("goal")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Build something great");
                Ok(json!({ "status": "running", "goal": goal }))
            }
            "dream" => Ok(json!({ "ideas": vec!["feature1", "feature2", "feature3"] })),
            _ => Err(PluginError::new(-32601, "Method not found")),
        }
    }
}

pub struct GitPlugin {
    policy: Arc<ExecutionPolicy>,
}

impl GitPlugin {
    pub fn new(policy: Arc<ExecutionPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl Plugin for GitPlugin {
    fn name(&self) -> &str {
        "git"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn methods(&self) -> Vec<MethodSpec> {
        vec![
            MethodSpec {
                name: "diff".to_string(),
                description: "Show git diff".to_string(),
                params: HashMap::new(),
                returns: "string".to_string(),
            },
            MethodSpec {
                name: "commit".to_string(),
                description: "Make a commit".to_string(),
                params: {
                    let mut m = HashMap::new();
                    m.insert("message".to_string(), "string".to_string());
                    m
                },
                returns: "object".to_string(),
            },
        ]
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, PluginError> {
        match method {
            "diff" => {
                if !self.policy.allow_plugin_git {
                    return Err(PluginError::new(
                        -32604,
                        "Plugin git execution is disabled by policy",
                    ));
                }

                match tokio::time::timeout(
                    Duration::from_secs(120),
                    Command::new("git").arg("diff").output(),
                )
                .await
                {
                    Ok(Ok(output)) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(json!({ "diff": stdout }))
                    }
                    Ok(Err(e)) => Err(PluginError::new(-32000, &e.to_string())),
                    Err(_) => Err(PluginError::new(-32000, "git diff timed out")),
                }
            }
            "commit" => {
                if !self.policy.allow_plugin_git {
                    return Err(PluginError::new(
                        -32604,
                        "Plugin git execution is disabled by policy",
                    ));
                }

                let msg = params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::new(-32602, "Invalid params"))?;

                match tokio::time::timeout(
                    Duration::from_secs(120),
                    Command::new("git").args(["commit", "-m", msg]).output(),
                )
                .await
                {
                    Ok(Ok(_)) => Ok(json!({ "committed": true })),
                    Ok(Err(e)) => Err(PluginError::new(-32000, &e.to_string())),
                    Err(_) => Err(PluginError::new(-32000, "git commit timed out")),
                }
            }
            _ => Err(PluginError::new(-32601, "Method not found")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_call() {
        let mut registry = PluginRegistry::new();
        registry.register(std::sync::Arc::new(AiPlugin));

        let result = registry
            .call("ai", "explain", json!({ "code": "fn main() {}" }))
            .await
            .unwrap();

        assert!(result.get("result").is_some());
    }
}
