//! Launches the pinned app-server with a freshly generated `config.toml`
//! (providers, Aura MCP, user MCP servers) and the per-run secrets in its
//! environment. Regenerating on every launch means settings changes take
//! effect on the next app-server start without extra plumbing.

use aura_codex::home::{BaseConfig, ConfigContributor, GATEWAY_TOKEN_ENV, MCP_TOKEN_ENV, prepare};
use aura_codex::launcher::{Connection, Launcher, ProcessLauncher};
use aura_core::credentials::CredentialStore;
use aura_core::secret::Secret;
use aura_extensions::mcp_config::{McpConfigContributor, McpServerSpec, secret_env};
use aura_gateway::codex_config::GatewayConfigContributor;
use aura_gateway::registry::Provider;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub type SpawnHook = Arc<dyn Fn(u32) + Send + Sync>;

/// What the launcher reads at every launch.
#[derive(Default)]
pub struct LaunchState {
    pub base: BaseConfig,
    pub providers: Vec<Provider>,
    pub mcp_servers: Vec<McpServerSpec>,
}

pub struct AuraLauncher {
    /// Set once the pinned app-server is installed and verified.
    pub program: Arc<RwLock<Option<PathBuf>>>,
    pub codex_home: PathBuf,
    pub state: Arc<RwLock<LaunchState>>,
    pub vault: Arc<dyn CredentialStore>,
    pub gateway_token: Secret<String>,
    pub mcp_token: Secret<String>,
    pub on_spawn: Option<SpawnHook>,
}

impl Launcher for AuraLauncher {
    fn launch(&self) -> std::io::Result<Connection> {
        let program = self.program.read().unwrap().clone().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "o motor do agente (Codex app-server) ainda está sendo instalado",
            )
        })?;
        let st = self.state.read().unwrap();
        let gateway = GatewayConfigContributor {
            providers: st.providers.clone(),
            port: st.base.gateway_port,
        };
        let mcp = McpConfigContributor {
            specs: st.mcp_servers.clone(),
        };
        let contributors: [&dyn ConfigContributor; 2] = [&gateway, &mcp];
        prepare(&self.codex_home, &st.base, &contributors)?;

        let mut env = vec![
            (
                GATEWAY_TOKEN_ENV.to_string(),
                self.gateway_token.expose().clone(),
            ),
            (MCP_TOKEN_ENV.to_string(), self.mcp_token.expose().clone()),
        ];
        // User MCP secrets: a missing one disables only that server.
        for spec in st.mcp_servers.iter().filter(|s| s.enabled) {
            match secret_env(std::slice::from_ref(spec), self.vault.as_ref()) {
                Ok(vars) => env.extend(vars.into_iter().map(|(k, v)| (k, v.expose().clone()))),
                Err(e) => {
                    tracing::warn!(server = %spec.name, "MCP server secrets unavailable: {e}")
                }
            }
        }
        drop(st);
        let mut p = ProcessLauncher::app_server(program, self.codex_home.clone(), env);
        p.on_spawn = self.on_spawn.clone();
        p.launch()
    }

    fn describe(&self) -> String {
        match self.program.read().unwrap().as_ref() {
            Some(p) => format!("app-server {}", p.display()),
            None => "app-server (not installed yet)".into(),
        }
    }
}
