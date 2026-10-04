//! On-demand diagnosis of a stdio MCP server (008 AC-005): Aura starts the
//! same command the app-server would, sends `initialize` and keeps the last
//! stderr lines, so a failing server shows its own error output. The
//! app-server logs server stderr without naming the server, so Aura cannot
//! attribute those lines reliably.

use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Lines of server output kept for the user.
pub const LOG_LINES: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDiagnosis {
    /// The server answered `initialize`.
    pub connected: bool,
    /// Exit code when the process ended by itself.
    pub exit_code: Option<i32>,
    /// Last stderr lines (newest last).
    pub log: Vec<String>,
}

pub struct StdioLaunch {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<PathBuf>,
}

pub async fn diagnose(launch: StdioLaunch, timeout: Duration) -> std::io::Result<McpDiagnosis> {
    let mut cmd = Command::new(&launch.command);
    cmd.args(&launch.args)
        .envs(launch.env.iter().map(|(k, v)| (k, v)))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    if let Some(cwd) = &launch.cwd {
        cmd.current_dir(cwd);
    }
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000);
    let mut child = cmd.spawn()?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let init = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "aura-diagnostics", "version": env!("CARGO_PKG_VERSION")}}});
    let _ = stdin.write_all(format!("{init}\n").as_bytes()).await;
    let _ = stdin.flush().await;

    let errors = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        let mut kept = std::collections::VecDeque::new();
        while let Ok(Some(line)) = lines.next_line().await {
            if kept.len() == LOG_LINES {
                kept.pop_front();
            }
            kept.push_back(line.trim_end().to_string());
        }
        kept.into_iter().collect::<Vec<_>>()
    });
    let answered = async {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if serde_json::from_str::<serde_json::Value>(&line)
                .is_ok_and(|v| v["id"] == 1 && v.get("result").is_some())
            {
                return true;
            }
        }
        false
    };
    let connected = tokio::time::timeout(timeout, answered)
        .await
        .unwrap_or(false);
    drop(stdin);
    // Give a failing server a moment to exit with its code, then stop it.
    let exit_code = match tokio::time::timeout(Duration::from_millis(500), child.wait()).await {
        Ok(Ok(status)) => status.code(),
        _ => {
            let _ = child.kill().await;
            None
        }
    };
    let log = tokio::time::timeout(Duration::from_secs(2), errors)
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or_default();
    Ok(McpDiagnosis {
        connected,
        exit_code,
        log,
    })
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_failing_server_reports_its_stderr_and_exit_code() {
        let d = diagnose(
            StdioLaunch {
                command: "cmd".into(),
                args: vec![
                    "/C".into(),
                    "echo starting 1>&2 & echo missing QA_TOKEN 1>&2 & exit 3".into(),
                ],
                env: vec![],
                cwd: None,
            },
            Duration::from_secs(5),
        )
        .await
        .unwrap();
        assert!(!d.connected);
        assert_eq!(d.exit_code, Some(3));
        assert_eq!(d.log, ["starting", "missing QA_TOKEN"]);
    }

    #[tokio::test]
    async fn a_missing_command_is_an_error() {
        assert!(
            diagnose(
                StdioLaunch {
                    command: "aura-no-such-mcp-server".into(),
                    args: vec![],
                    env: vec![],
                    cwd: None
                },
                Duration::from_secs(1)
            )
            .await
            .is_err()
        );
    }
}
