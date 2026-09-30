//! How the app-server process is started. Two real adapters occupy this seam:
//! [`ProcessLauncher`] (production) and [`InMemoryLauncher`] (tests and UI
//! development without a Codex account).

use aura_core::jsonrpc::{Incoming, Peer};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::mpsc::UnboundedReceiver;

pub struct Connection {
    pub peer: Peer,
    pub incoming: UnboundedReceiver<Incoming>,
    pub child: Option<Child>,
}

pub trait Launcher: Send + Sync {
    fn launch(&self) -> std::io::Result<Connection>;
    fn describe(&self) -> String;
}

type SpawnHook = Arc<dyn Fn(u32) + Send + Sync>;

pub struct ProcessLauncher {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<PathBuf>,
    /// Called with the child PID right after spawn (the host adopts it into
    /// its Job Object so it dies with Aura; OT-004 of 001).
    pub on_spawn: Option<SpawnHook>,
}

impl ProcessLauncher {
    pub fn app_server(program: PathBuf, codex_home: PathBuf, env: Vec<(String, String)>) -> Self {
        let mut all_env = vec![
            (
                "CODEX_HOME".to_string(),
                codex_home.to_string_lossy().into_owned(),
            ),
            ("RUST_LOG".to_string(), "warn".to_string()),
        ];
        all_env.extend(env);
        let args = app_server_args(&program);
        Self {
            program,
            args,
            env: all_env,
            cwd: None,
            on_spawn: None,
        }
    }
}

/// The pinned release ships a standalone `codex-app-server` executable that
/// takes no subcommand (it rejects `app-server`); the `codex` CLI (e.g. via
/// `AURA_CODEX_BIN`) needs `codex app-server`.
pub fn app_server_args(program: &std::path::Path) -> Vec<String> {
    let stem = program
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if stem.starts_with("codex-app-server") {
        vec![]
    } else {
        vec!["app-server".into()]
    }
}

impl Launcher for ProcessLauncher {
    fn launch(&self) -> std::io::Result<Connection> {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = cmd.spawn()?;
        if let (Some(hook), Some(pid)) = (&self.on_spawn, child.id()) {
            hook(pid);
        }
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| std::io::Error::other("no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("no stdout"))?;
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut lines = tokio::io::BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!(target: "codex_app_server", "{}", aura_core::logging::redact(&line));
                }
            });
        }
        let (peer, incoming) = Peer::spawn(stdout, stdin);
        Ok(Connection {
            peer,
            incoming,
            child: Some(child),
        })
    }

    fn describe(&self) -> String {
        format!("process {}", self.program.display())
    }
}

type ServerFactory = Arc<dyn Fn(tokio::io::DuplexStream) + Send + Sync>;

/// Runs a server function on an in-memory pipe for each launch.
pub struct InMemoryLauncher {
    factory: ServerFactory,
}

impl InMemoryLauncher {
    pub fn new(factory: impl Fn(tokio::io::DuplexStream) + Send + Sync + 'static) -> Self {
        Self {
            factory: Arc::new(factory),
        }
    }
}

impl Launcher for InMemoryLauncher {
    fn launch(&self) -> std::io::Result<Connection> {
        let (client, server) = tokio::io::duplex(1 << 20);
        (self.factory)(server);
        let (r, w) = tokio::io::split(client);
        let (peer, incoming) = Peer::spawn(r, w);
        Ok(Connection {
            peer,
            incoming,
            child: None,
        })
    }

    fn describe(&self) -> String {
        "in-memory".into()
    }
}

#[cfg(test)]
mod tests {
    use super::app_server_args;
    use std::path::Path;

    #[test]
    fn standalone_binary_takes_no_subcommand() {
        assert!(
            app_server_args(Path::new(
                "C:/Aura/bin/codex/rust-v0.159.0/codex-app-server.exe"
            ))
            .is_empty()
        );
        assert!(
            app_server_args(Path::new("/opt/codex-app-server-x86_64-unknown-linux-musl"))
                .is_empty()
        );
        assert_eq!(
            app_server_args(Path::new("/usr/local/bin/codex")),
            vec!["app-server".to_string()]
        );
    }
}
