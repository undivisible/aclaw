use async_trait::async_trait;
use super::traits::*;
use serde_json::json;
use reqwest::Client;
use anyhow::{anyhow, Context};
use std::time::Duration;

pub struct ComposioTool {
    client: Client,
    api_key: String,
}

impl ComposioTool {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            api_key,
        }
    }

    async fn make_request(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value> {
        // Try v3 first
        let v3_url = format!("https://backend.composio.dev/api/v3{}", endpoint);
        let v3_result = self.try_request(method, &v3_url, body.clone()).await;
        
        if v3_result.is_ok() {
            return v3_result;
        }

        // Fallback to v2
        let v2_url = format!("https://backend.composio.dev/api/v2{}", endpoint);
        self.try_request(method, &v2_url, body).await
    }

    async fn try_request(
        &self,
        method: &str,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value> {
        let mut request = match method.to_uppercase().as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };

        request = request.header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json");

        if let Some(body_value) = body {
            request = request.json(&body_value);
        }

        let response = request.send().await
            .context("Failed to send request to Composio API")?;

        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Composio API error ({}): {}", status, error_text));
        }

        let json_response: serde_json::Value = response.json().await
            .context("Failed to parse Composio API response")?;

        Ok(json_response)
    }

    async fn list_actions(&self, tool_name: Option<&str>) -> anyhow::Result<String> {
        let endpoint = if let Some(tool) = tool_name {
            format!"/tools?name={}", tool)
        } else {
            "/tools".to_string()
        };

        let response = self.make_request("GET", &endpoint, None).await?;
        Ok(serde_json::to_string_pretty(&response)?)
    }

    async fn execute_action(
        &self,
        action_name: &str,
        params: serde_json::Value,
        entity_id: Option<&str>,
    ) -> anyhow::Result<String> {
        let endpoint = format!("/actions/{}/execute", action_name);
        
        let mut body = json!({
            "input": params,
        });

        if let Some(entity) = entity_id {
            body["entityId"] = json!(entity);
        }

        let response = self.make_request("POST", &endpoint, Some(body)).await?;
        Ok(serde_json::to_string_pretty(&response)?)
    }
}

#[async_trait]
impl Tool for ComposioTool {
    fn name(&self) -> &str {
        "composio"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "composio".to_string(),
            description: "Integrate with Composio API to list and execute actions. Automatically tries v3 API first, falls back to v2.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["list_actions", "execute_action"],
                        "description": "Operation to perform"
                    },
                    "tool_name": {
                        "type": "string",
                        "description": "Tool name to filter actions (for list_actions)"
                    },
                    "action_name": {
                        "type": "string",
                        "description": "Action name to execute (for execute_action)"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Parameters for the action (for execute_action)",
                        "default": {}
                    },
                    "entity_id": {
                        "type": "string",
                        "description": "Entity ID for the action (for execute_action)"
                    }
                },
                "required": ["operation"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .context("Failed to parse arguments")?;

        let operation = args["operation"].as_str()
            .ok_or_else(|| anyhow!("Missing operation"))?;

        let result = match operation {
            "list_actions" => {
                let tool_name = args["tool_name"].as_str();
                self.list_actions(tool_name).await
            },
            "execute_action" => {
                let action_name = args["action_name"].as_str()
                    .ok_or_else(|| anyhow!("Missing action_name for execute_action"))?;
                
                let params = args["parameters"].clone();
                let params = if params.is_null() { json!({}) } else { params };
                
                let entity_id = args["entity_id"].as_str();
                
                self.execute_action(action_name, params, entity_id).await
            },
            _ => Err(anyhow!("Unknown operation: {}", operation)),
        };

        match result {
            Ok(output) => Ok(ToolResult::success(output)),
            Err(e) => Ok(ToolResult::error(format!("Composio operation failed: {}", e))),
        }
    }
}