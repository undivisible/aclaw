//! Doctor tool — system diagnostics, dependency checks, health report.

use async_trait::async_trait;
use serde::Deserialize;
use super::traits::*;

pub struct DoctorTool;

impl DoctorTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct DoctorArgs {
    #[serde(default)]
    verbose: bool,
}

#[async_trait]
impl Tool for DoctorTool {
    fn name(&self) -> &str { "doctor" }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "doctor".to_string(),
            description: "Run system diagnostics — check deps, env vars, disk space, memory, network, and bot health. Like 'zeroclaw doctor'.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "verbose": {
                        "type": "boolean",
                        "description": "Show detailed output (default false)"
                    }
                }
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: DoctorArgs = serde_json::from_str(arguments).unwrap_or(DoctorArgs { verbose: false });
        
        let mut report = vec!["🩺 unthinkclaw doctor\n".to_string()];
        
        // Binary deps
        let bins = [
            ("v", "V compiler"),
            ("cargo", "Rust toolchain"),
            ("git", "Git"),
            ("cloudflared", "Cloudflare tunnel"),
            ("tailscale", "Tailscale"),
            ("ffmpeg", "FFmpeg (voice)"),
            ("faster-whisper", "Whisper (transcription)"),
            ("docker", "Docker"),
            ("node", "Node.js"),
        ];
        
        report.push("📦 Dependencies:".to_string());
        for (bin, label) in &bins {
            let found = check_cmd(bin).await;
            let icon = if found { "✅" } else { "❌" };
            if found || args.verbose {
                report.push(format!("  {} {}", icon, label));
            }
        }

        // Env vars
        report.push("\n🔑 Environment:".to_string());
        let envs = [
            ("ANTHROPIC_API_KEY", "Anthropic"),
            ("TELEGRAM_BOT_TOKEN", "Telegram"),
            ("OPENAI_API_KEY", "OpenAI"),
            ("PERPLEXITY_API_KEY", "Perplexity"),
            ("GEMINI_API_KEY", "Gemini (embeddings)"),
            ("DISCORD_TOKEN", "Discord"),
        ];
        for (key, label) in &envs {
            let set = std::env::var(key).is_ok();
            let icon = if set { "✅" } else { "⚠️ " };
            if set || args.verbose {
                report.push(format!("  {} {}", icon, label));
            }
        }

        // System resources
        report.push("\n💻 System:".to_string());
        let disk = run_cmd("df -h / 2>&1 | tail -1 | awk '{print $4\" free of \"$2}'").await;
        let mem = run_cmd("free -h 2>&1 | grep Mem | awk '{print $7\" free of \"$2}'").await;
        let uptime = run_cmd("uptime -p 2>&1").await;
        let load = run_cmd("cat /proc/loadavg 2>&1 | awk '{print $1,$2,$3}'").await;
        report.push(format!("  💾 Disk: {}", disk.trim()));
        report.push(format!("  🧠 RAM: {}", mem.trim()));
        report.push(format!("  ⏱️  Uptime: {}", uptime.trim()));
        report.push(format!("  📊 Load: {}", load.trim()));

        // Bot process
        report.push("\n🤖 Bot:".to_string());
        let bot_pid = run_cmd("pgrep -f unthinkclaw 2>&1").await;
        if bot_pid.trim().is_empty() {
            report.push("  ❌ unthinkclaw not running".to_string());
        } else {
            report.push(format!("  ✅ Running (PID: {})", bot_pid.trim()));
        }

        // DB health
        let db_path = dirs::home_dir()
            .map(|h| h.join(".aclaw/memory.db").display().to_string())
            .unwrap_or_default();
        if std::path::Path::new(&db_path).exists() {
            let size = run_cmd(&format!("du -sh {} 2>&1 | cut -f1", db_path)).await;
            report.push(format!("  ✅ DB: {} ({})", db_path, size.trim()));
        } else {
            report.push(format!("  ⚠️  DB not found at {}", db_path));
        }

        // Network
        report.push("\n🌐 Network:".to_string());
        let ping = run_cmd("curl -s -o /dev/null -w '%{http_code}' --max-time 5 https://api.anthropic.com/health 2>&1").await;
        let icon = if ping.trim() == "200" || ping.trim() == "404" { "✅" } else { "❌" };
        report.push(format!("  {} Anthropic API reachable ({})", icon, ping.trim()));

        Ok(ToolResult::success(report.join("\n")))
    }
}

async fn check_cmd(cmd: &str) -> bool {
    tokio::process::Command::new("which")
        .arg(cmd)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn run_cmd(cmd: &str) -> String {
    tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}
