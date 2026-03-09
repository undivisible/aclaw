//! Tunnel tool — Cloudflare/Tailscale/ngrok tunnel management.

use async_trait::async_trait;
use serde::Deserialize;
use super::traits::*;

pub struct TunnelTool;

impl TunnelTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct TunnelArgs {
    action: String,
    port: Option<u16>,
    provider: Option<String>,
    name: Option<String>,
}

#[async_trait]
impl Tool for TunnelTool {
    fn name(&self) -> &str { "tunnel" }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "tunnel".to_string(),
            description: "Manage network tunnels. Providers: cloudflare (cloudflared), tailscale, ngrok. Actions: start, stop, status, url.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["start", "stop", "status", "url", "list"],
                        "description": "Tunnel action"
                    },
                    "port": {
                        "type": "integer",
                        "description": "Local port to expose (for start action)"
                    },
                    "provider": {
                        "type": "string",
                        "enum": ["cloudflare", "tailscale", "ngrok"],
                        "description": "Tunnel provider (default: cloudflare)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Tunnel name/hostname"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: TunnelArgs = serde_json::from_str(arguments)?;
        let provider = args.provider.as_deref().unwrap_or("cloudflare");

        match args.action.as_str() {
            "start" => {
                let port = args.port.unwrap_or(8080);
                let cmd = match provider {
                    "cloudflare" => {
                        // Check if cloudflared is installed
                        if !cmd_exists("cloudflared").await {
                            return Ok(ToolResult::error("cloudflared not installed. Run: curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /usr/local/bin/cloudflared && chmod +x /usr/local/bin/cloudflared"));
                        }
                        format!("cloudflared tunnel --url http://localhost:{} 2>&1 &", port)
                    }
                    "tailscale" => {
                        if !cmd_exists("tailscale").await {
                            return Ok(ToolResult::error("tailscale not installed. See: https://tailscale.com/download"));
                        }
                        format!("tailscale funnel {} 2>&1", port)
                    }
                    "ngrok" => {
                        if !cmd_exists("ngrok").await {
                            return Ok(ToolResult::error("ngrok not installed. See: https://ngrok.com/download"));
                        }
                        format!("ngrok http {} 2>&1 &", port)
                    }
                    _ => return Ok(ToolResult::error(format!("Unknown provider: {}", provider))),
                };
                let out = run_cmd(&cmd).await;
                Ok(ToolResult::success(format!("Started {} tunnel on port {}\n{}", provider, port, out)))
            }

            "status" => {
                let cmd = match provider {
                    "cloudflare" => "pgrep -la cloudflared 2>&1 || echo 'not running'",
                    "tailscale" => "tailscale status 2>&1",
                    "ngrok" => "curl -s http://localhost:4040/api/tunnels 2>&1 || echo 'ngrok not running'",
                    _ => "echo 'Unknown provider'",
                };
                Ok(ToolResult::success(run_cmd(cmd).await))
            }

            "url" => {
                let cmd = match provider {
                    "cloudflare" => "journalctl -u cloudflared --no-pager -n 20 2>&1 | grep -o 'https://[^ ]*trycloudflare.com[^ ]*' | tail -1",
                    "ngrok" => "curl -s http://localhost:4040/api/tunnels 2>&1 | grep -o 'https://[^\"]*ngrok[^\"]*' | head -1",
                    "tailscale" => "tailscale funnel status 2>&1",
                    _ => "echo 'Unknown provider'",
                };
                let out = run_cmd(cmd).await;
                if out.trim().is_empty() {
                    Ok(ToolResult::success("No tunnel URL found — is the tunnel running?".to_string()))
                } else {
                    Ok(ToolResult::success(out))
                }
            }

            "stop" => {
                let cmd = match provider {
                    "cloudflare" => "pkill cloudflared 2>&1 && echo 'stopped'",
                    "tailscale" => format!("tailscale funnel off {} 2>&1", args.port.unwrap_or(8080)).leak(),
                    "ngrok" => "pkill ngrok 2>&1 && echo 'stopped'",
                    _ => "echo 'Unknown provider'",
                };
                Ok(ToolResult::success(run_cmd(cmd).await))
            }

            "list" => {
                let mut out = String::new();
                for p in &["cloudflared", "tailscale", "ngrok"] {
                    let running = cmd_exists(p).await;
                    out.push_str(&format!("{}: {}\n", p, if running { "installed" } else { "not found" }));
                }
                Ok(ToolResult::success(out))
            }

            _ => Ok(ToolResult::error(format!("Unknown action: {}", args.action))),
        }
    }
}

async fn cmd_exists(cmd: &str) -> bool {
    tokio::process::Command::new("which")
        .arg(cmd)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn run_cmd(cmd: &str) -> String {
    let out = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await;
    match out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            let e = String::from_utf8_lossy(&o.stderr).to_string();
            format!("{}{}", s, e)
        }
        Err(e) => format!("Error: {}", e),
    }
}
