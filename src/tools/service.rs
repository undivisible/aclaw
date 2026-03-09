//! Service management tool — systemd/OpenRC install, start, stop, status.

use async_trait::async_trait;
use serde::Deserialize;
use super::traits::*;

pub struct ServiceTool;

impl ServiceTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ServiceArgs {
    action: String,
    name: Option<String>,
    unit_content: Option<String>,
}

#[async_trait]
impl Tool for ServiceTool {
    fn name(&self) -> &str { "service" }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "service".to_string(),
            description: "Manage system services (systemd/OpenRC). Actions: status, start, stop, restart, enable, disable, install (create unit file), logs.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["status", "start", "stop", "restart", "enable", "disable", "install", "logs", "list"],
                        "description": "Service action"
                    },
                    "name": {
                        "type": "string",
                        "description": "Service name (e.g. 'unthinkclaw', 'nginx')"
                    },
                    "unit_content": {
                        "type": "string",
                        "description": "Systemd unit file content (for install action)"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: ServiceArgs = serde_json::from_str(arguments)?;
        
        // Detect init system
        let has_systemd = std::path::Path::new("/run/systemd/private").exists()
            || std::path::Path::new("/sys/fs/cgroup/systemd").exists();

        match args.action.as_str() {
            "list" => {
                let cmd = if has_systemd {
                    "systemctl list-units --type=service --no-pager --plain 2>&1 | head -50"
                } else {
                    "rc-status 2>&1 || service --status-all 2>&1 | head -50"
                };
                let out = run_cmd(cmd).await;
                Ok(ToolResult::success(out))
            }

            "status" => {
                let name = args.name.as_deref().unwrap_or("unthinkclaw");
                let cmd = if has_systemd {
                    format!("systemctl status {} --no-pager 2>&1", name)
                } else {
                    format!("rc-service {} status 2>&1 || service {} status 2>&1", name, name)
                };
                Ok(ToolResult::success(run_cmd(&cmd).await))
            }

            "start" | "stop" | "restart" | "enable" | "disable" => {
                let name = match &args.name {
                    Some(n) => n.clone(),
                    None => return Ok(ToolResult::error("name required")),
                };
                let cmd = if has_systemd {
                    format!("sudo systemctl {} {} 2>&1", args.action, name)
                } else {
                    format!("sudo rc-service {} {} 2>&1", name, args.action)
                };
                Ok(ToolResult::success(run_cmd(&cmd).await))
            }

            "logs" => {
                let name = args.name.as_deref().unwrap_or("unthinkclaw");
                let cmd = if has_systemd {
                    format!("journalctl -u {} -n 100 --no-pager 2>&1", name)
                } else {
                    format!("tail -100 /var/log/{}.log 2>&1 || journalctl -u {} -n 100 --no-pager 2>&1", name, name)
                };
                Ok(ToolResult::success(run_cmd(&cmd).await))
            }

            "install" => {
                let name = match &args.name {
                    Some(n) => n.clone(),
                    None => return Ok(ToolResult::error("name required for install")),
                };
                let content = match &args.unit_content {
                    Some(c) => c.clone(),
                    None => {
                        // Generate default unit for unthinkclaw
                        let binary = std::env::current_exe()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|_| "/usr/local/bin/unthinkclaw".to_string());
                        format!(
                            "[Unit]\nDescription={name}\nAfter=network.target\n\n[Service]\nType=simple\nExecStart={binary}\nRestart=always\nRestartSec=5\nEnvironmentFile=/etc/{name}.env\n\n[Install]\nWantedBy=multi-user.target\n"
                        )
                    }
                };
                
                let unit_path = format!("/etc/systemd/system/{}.service", name);
                let write_cmd = format!(
                    "echo '{}' | sudo tee {} > /dev/null && sudo systemctl daemon-reload 2>&1",
                    content.replace("'", "'\\''"),
                    unit_path
                );
                let out = run_cmd(&write_cmd).await;
                Ok(ToolResult::success(format!("Installed {}\n{}", unit_path, out)))
            }

            _ => Ok(ToolResult::error(format!("Unknown action: {}", args.action))),
        }
    }
}

async fn run_cmd(cmd: &str) -> String {
    let out = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await;
    
    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            if stderr.is_empty() { stdout } else { format!("{}{}", stdout, stderr) }
        }
        Err(e) => format!("Error: {}", e),
    }
}
