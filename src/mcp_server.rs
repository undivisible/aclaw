//! MCP server mode — exposes unthinkclaw as an MCP server over stdio or HTTP.
//! Other AI clients can connect to prompt unthinkclaw or use its tools.

use axum::{extract::State, response::IntoResponse, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::tools::traits::{Tool, ToolSpec};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcErrorObj>,
}

#[derive(Debug, Serialize)]
struct JsonRpcErrorObj {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcResponse {
    fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcErrorObj {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

fn tool_to_mcp_schema(spec: &ToolSpec) -> Value {
    serde_json::json!({
        "name": spec.name,
        "description": spec.description,
        "inputSchema": spec.parameters
    })
}

#[derive(Clone)]
struct McpState {
    tools: Arc<Vec<Arc<dyn Tool>>>,
    provider: Option<Arc<dyn crate::providers::traits::Provider>>,
    model: Option<String>,
}

/// Run unthinkclaw as an MCP server over stdio.
pub async fn run_mcp_server(
    tools: Vec<Arc<dyn Tool>>,
    provider: Option<Arc<dyn crate::providers::traits::Provider>>,
    model: Option<String>,
) -> anyhow::Result<()> {
    let stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut lines = stdin.lines();

    let tools = Arc::new(tools);

    tracing::info!("MCP server started on stdio");

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => {
                let resp = JsonRpcResponse::error(None, -32700, "Parse error");
                write_response(&mut stdout, &resp).await?;
                continue;
            }
        };

        let response = handle_request(&request, &tools, &provider, &model).await;
        write_response(&mut stdout, &response).await?;
    }

    Ok(())
}

/// Run unthinkclaw as an MCP server over HTTP (for Cloudflare Container deployment).
pub async fn run_mcp_server_http(
    tools: Vec<Arc<dyn Tool>>,
    provider: Option<Arc<dyn crate::providers::traits::Provider>>,
    model: Option<String>,
    port: u16,
) -> anyhow::Result<()> {
    let state = McpState {
        tools: Arc::new(tools),
        provider,
        model,
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/mcp", post(handle_http_mcp))
        .route("/", post(handle_http_mcp))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("MCP HTTP server listening on {}", addr);
    eprintln!("MCP HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_http_mcp(
    State(state): State<McpState>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let response = handle_request(&request, &state.tools, &state.provider, &state.model).await;
    Json(response)
}

async fn write_response(
    stdout: &mut tokio::io::Stdout,
    response: &JsonRpcResponse,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(response)?;
    stdout.write_all(json.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}

async fn handle_request(
    req: &JsonRpcRequest,
    tools: &[Arc<dyn Tool>],
    provider: &Option<Arc<dyn crate::providers::traits::Provider>>,
    model: &Option<String>,
) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => handle_initialize(req.id.clone()),
        "notifications/initialized" => JsonRpcResponse::success(req.id.clone(), Value::Null),
        "tools/list" => handle_tools_list(req.id.clone(), tools, provider.is_some()),
        "tools/call" => {
            handle_tools_call(req.id.clone(), req.params.as_ref(), tools, provider, model).await
        }
        "shutdown" => {
            tracing::info!("MCP server shutting down");
            JsonRpcResponse::success(req.id.clone(), Value::Null)
        }
        _ => JsonRpcResponse::error(req.id.clone(), -32601, "Method not found"),
    }
}

fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::success(
        id,
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "unthinkclaw",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

fn handle_tools_list(
    id: Option<Value>,
    tools: &[Arc<dyn Tool>],
    has_provider: bool,
) -> JsonRpcResponse {
    let mut mcp_tools: Vec<Value> = tools.iter().map(|t| tool_to_mcp_schema(&t.spec())).collect();

    // Add "ask" tool if we have a provider (allows other AIs to prompt unthinkclaw)
    if has_provider {
        mcp_tools.push(serde_json::json!({
            "name": "ask",
            "description": "Send a message to the unthinkclaw AI agent and get a response. Use this to prompt unthinkclaw for complex tasks like coding, research, or analysis.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message/prompt to send to unthinkclaw"
                    }
                },
                "required": ["message"]
            }
        }));
    }

    JsonRpcResponse::success(id, serde_json::json!({ "tools": mcp_tools }))
}

async fn handle_tools_call(
    id: Option<Value>,
    params: Option<&Value>,
    tools: &[Arc<dyn Tool>],
    provider: &Option<Arc<dyn crate::providers::traits::Provider>>,
    model: &Option<String>,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(id, -32602, "Missing params");
        }
    };

    let name = match params.get("name").and_then(|n| n.as_str()) {
        Some(n) => n,
        None => {
            return JsonRpcResponse::error(id, -32602, "Missing tool name");
        }
    };

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    // Handle the "ask" meta-tool
    if name == "ask" {
        if let Some(provider) = provider {
            let message = arguments
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("");
            let model_name = model.as_deref().unwrap_or("claude-sonnet-4-5");

            match provider.simple_chat(message, model_name).await {
                Ok(response) => {
                    return JsonRpcResponse::success(
                        id,
                        serde_json::json!({
                            "content": [{"type": "text", "text": response}]
                        }),
                    );
                }
                Err(e) => {
                    return JsonRpcResponse::success(
                        id,
                        serde_json::json!({
                            "content": [{"type": "text", "text": format!("Error: {}", e)}],
                            "isError": true
                        }),
                    );
                }
            }
        } else {
            return JsonRpcResponse::error(id, -32602, "No provider configured for ask tool");
        }
    }

    // Find and execute the matching tool
    let tool = match tools.iter().find(|t| t.name() == name) {
        Some(t) => t,
        None => {
            return JsonRpcResponse::error(id, -32602, format!("Unknown tool: {}", name));
        }
    };

    let args_str = serde_json::to_string(&arguments).unwrap_or_default();
    match tool.execute(&args_str).await {
        Ok(result) => JsonRpcResponse::success(
            id,
            serde_json::json!({
                "content": [{"type": "text", "text": result.output}],
                "isError": result.is_error
            }),
        ),
        Err(e) => JsonRpcResponse::success(
            id,
            serde_json::json!({
                "content": [{"type": "text", "text": format!("Tool error: {}", e)}],
                "isError": true
            }),
        ),
    }
}
