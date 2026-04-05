//! Optional Poke MCP bridge: runs `poke-sdk/start.js` as a subprocess (`plugin-poke`).

use std::path::Path;

use crate::config::Config;

/// Spawn Node/Bun helper that starts `unthinkclaw mcp` and prints the Poke registration URL.
pub async fn spawn_poke_tunnel(
    workspace: &Path,
    config_file: &str,
    cfg: &Config,
) -> anyhow::Result<Option<tokio::process::Child>> {
    if !cfg.plugin_layer.poke_tunnel {
        return Ok(None);
    }

    let script = workspace.join("poke-sdk/start.js");
    if !script.is_file() {
        anyhow::bail!(
            "plugin poke_tunnel enabled but {} is missing",
            script.display()
        );
    }

    let port = cfg.plugin_layer.poke_mcp_port.to_string();
    let mut cmd = tokio::process::Command::new("node");
    cmd.arg(&script)
        .arg("--port")
        .arg(&port)
        .arg("--config")
        .arg(config_file)
        .current_dir(workspace)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true);

    match cmd.spawn() {
        Ok(c) => Ok(Some(c)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut b = tokio::process::Command::new("bun");
            b.arg("run")
                .arg(&script)
                .arg("--port")
                .arg(&port)
                .arg("--config")
                .arg(config_file)
                .current_dir(workspace)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .kill_on_drop(true);
            Ok(Some(b.spawn()?))
        }
        Err(e) => Err(e.into()),
    }
}
