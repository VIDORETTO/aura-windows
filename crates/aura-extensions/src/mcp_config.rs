//! MCP server specs → Codex `[mcp_servers.<name>]` config (FR-002).
//!
//! Secrets never reach `config.toml` or logs (OT-001): secret env vars and
//! bearer tokens live in the Windows vault (`Aura/mcp/<server>/<VAR>`); the
//! host injects them into the app-server process environment and the TOML
//! only whitelists the variable names (`env_vars`) or points to them
//! (`bearer_token_env_var`).

use aura_codex::home::{ConfigContributor, section};
use aura_core::credentials::{CredentialError, CredentialStore, target};
use aura_core::secret::Secret;
use aura_store::{Store, StoreError, now_secs};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use toml::{Table, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EnvValue {
    Plain {
        value: String,
    },
    /// Stored in the vault under [`secret_target`].
    Secret,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Transport {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: BTreeMap<String, EnvValue>,
        #[serde(default)]
        cwd: Option<String>,
    },
    Http {
        url: String,
        /// A static bearer token kept in the vault (otherwise OAuth or none).
        #[serde(default)]
        bearer_secret: bool,
        #[serde(default)]
        headers: BTreeMap<String, String>,
    },
}

/// When Aura asks before the agent calls a tool of this server (AC-008).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApprovalMode {
    AlwaysAsk,
    #[default]
    AskForWrites,
    Auto,
}

/// MCP tool annotations (`readOnlyHint`, `destructiveHint`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ToolHints {
    pub read_only: Option<bool>,
    pub destructive: Option<bool>,
}

/// Destructive tools always ask; unknown hints count as "may write".
pub fn needs_approval(mode: ApprovalMode, hints: ToolHints) -> bool {
    if hints.destructive == Some(true) {
        return true;
    }
    match mode {
        ApprovalMode::AlwaysAsk => true,
        ApprovalMode::AskForWrites => hints.read_only != Some(true),
        ApprovalMode::Auto => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerSpec {
    pub name: String,
    pub transport: Transport,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// Tools the user switched off (AC-007).
    #[serde(default)]
    pub disabled_tools: Vec<String>,
    #[serde(default)]
    pub approval_mode: ApprovalMode,
    #[serde(default)]
    pub startup_timeout_sec: Option<u32>,
    #[serde(default)]
    pub tool_timeout_sec: Option<u32>,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum McpConfigError {
    #[error("nome inválido: use letras, números, _ e - (até 64)")]
    BadName,
    #[error("o nome \"aura\" é reservado")]
    Reserved,
    #[error("informe o comando")]
    MissingCommand,
    #[error("URL inválida: use http(s)://")]
    BadUrl,
    #[error("nome de variável inválido: {0}")]
    BadEnvName(String),
    #[error("a variável secreta {var} é usada por \"{a}\" e \"{b}\"; renomeie em um deles")]
    SecretCollision { var: String, a: String, b: String },
    #[error("segredo ausente no cofre: {0}")]
    MissingSecret(String),
    #[error("cofre: {0}")]
    Vault(String),
    #[error("store: {0}")]
    Store(String),
}

impl From<CredentialError> for McpConfigError {
    fn from(e: CredentialError) -> Self {
        McpConfigError::Vault(e.to_string())
    }
}

impl From<StoreError> for McpConfigError {
    fn from(e: StoreError) -> Self {
        McpConfigError::Store(e.to_string())
    }
}

pub fn secret_target(server: &str, var: &str) -> String {
    target("mcp", &format!("{server}/{var}"))
}

/// Env var through which the host passes an HTTP server's bearer token.
pub fn bearer_env_var(server: &str) -> String {
    let up: String = server
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    format!("AURA_MCP_{up}_TOKEN")
}

const BEARER_VAR: &str = "BEARER_TOKEN";

fn valid_env_name(n: &str) -> bool {
    !n.is_empty()
        && !n.starts_with(|c: char| c.is_ascii_digit())
        && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl McpServerSpec {
    pub fn validate(&self) -> Result<(), McpConfigError> {
        let ok = !self.name.is_empty()
            && self.name.len() <= 64
            && self
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if !ok {
            return Err(McpConfigError::BadName);
        }
        if self.name.eq_ignore_ascii_case("aura") {
            return Err(McpConfigError::Reserved);
        }
        match &self.transport {
            Transport::Stdio { command, env, .. } => {
                if command.trim().is_empty() {
                    return Err(McpConfigError::MissingCommand);
                }
                if let Some(bad) = env.keys().find(|k| !valid_env_name(k)) {
                    return Err(McpConfigError::BadEnvName(bad.clone()));
                }
            }
            Transport::Http { url, .. } => {
                if !(url.starts_with("https://") || url.starts_with("http://")) || url.len() < 10 {
                    return Err(McpConfigError::BadUrl);
                }
            }
        }
        Ok(())
    }

    /// `(env var, vault target)` pairs the host must inject.
    pub fn secret_vars(&self) -> Vec<(String, String)> {
        match &self.transport {
            Transport::Stdio { env, .. } => env
                .iter()
                .filter(|(_, v)| matches!(v, EnvValue::Secret))
                .map(|(k, _)| (k.clone(), secret_target(&self.name, k)))
                .collect(),
            Transport::Http {
                bearer_secret: true,
                ..
            } => vec![(
                bearer_env_var(&self.name),
                secret_target(&self.name, BEARER_VAR),
            )],
            Transport::Http { .. } => vec![],
        }
    }

    pub fn bearer_target(&self) -> String {
        secret_target(&self.name, BEARER_VAR)
    }

    /// The `[mcp_servers.<name>]` table. Keys follow the Codex config
    /// reference at the pinned version (see HANDOFF: verify on upgrade).
    pub fn to_toml(&self) -> Table {
        let mut t = Table::new();
        match &self.transport {
            Transport::Stdio {
                command,
                args,
                env,
                cwd,
            } => {
                t.insert("command".into(), Value::String(command.clone()));
                if !args.is_empty() {
                    t.insert(
                        "args".into(),
                        Value::Array(args.iter().cloned().map(Value::String).collect()),
                    );
                }
                let plain: Table = env
                    .iter()
                    .filter_map(|(k, v)| match v {
                        EnvValue::Plain { value } => {
                            Some((k.clone(), Value::String(value.clone())))
                        }
                        EnvValue::Secret => None,
                    })
                    .collect();
                if !plain.is_empty() {
                    t.insert("env".into(), Value::Table(plain));
                }
                let secret: Vec<Value> = self
                    .secret_vars()
                    .into_iter()
                    .map(|(k, _)| Value::String(k))
                    .collect();
                if !secret.is_empty() {
                    t.insert("env_vars".into(), Value::Array(secret));
                }
                if let Some(cwd) = cwd {
                    t.insert("cwd".into(), Value::String(cwd.clone()));
                }
            }
            Transport::Http {
                url,
                bearer_secret,
                headers,
            } => {
                t.insert("url".into(), Value::String(url.clone()));
                if *bearer_secret {
                    t.insert(
                        "bearer_token_env_var".into(),
                        Value::String(bearer_env_var(&self.name)),
                    );
                }
                if !headers.is_empty() {
                    t.insert(
                        "http_headers".into(),
                        Value::Table(
                            headers
                                .iter()
                                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                                .collect(),
                        ),
                    );
                }
            }
        }
        t.insert("enabled".into(), Value::Boolean(self.enabled));
        if !self.disabled_tools.is_empty() {
            t.insert(
                "disabled_tools".into(),
                Value::Array(
                    self.disabled_tools
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }
        if let Some(s) = self.startup_timeout_sec {
            t.insert("startup_timeout_sec".into(), Value::Integer(s as i64));
        }
        if let Some(s) = self.tool_timeout_sec {
            t.insert("tool_timeout_sec".into(), Value::Integer(s as i64));
        }
        t
    }
}

/// Fails when two servers need the same secret variable name (they would
/// share one process environment).
pub fn check_collisions(specs: &[McpServerSpec]) -> Result<(), McpConfigError> {
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for s in specs.iter().filter(|s| s.enabled) {
        for (var, _) in s.secret_vars() {
            if let Some(prev) = seen.insert(var.clone(), s.name.clone()) {
                return Err(McpConfigError::SecretCollision {
                    var,
                    a: prev,
                    b: s.name.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Environment the host adds to the app-server process.
pub fn secret_env(
    specs: &[McpServerSpec],
    vault: &dyn CredentialStore,
) -> Result<Vec<(String, Secret<String>)>, McpConfigError> {
    check_collisions(specs)?;
    let mut out = Vec::new();
    for s in specs.iter().filter(|s| s.enabled) {
        for (var, tgt) in s.secret_vars() {
            let value = vault
                .get(&tgt)?
                .ok_or_else(|| McpConfigError::MissingSecret(format!("{}/{var}", s.name)))?;
            out.push((var, value));
        }
    }
    Ok(out)
}

/// Adds all user servers to the generated `config.toml`.
pub struct McpConfigContributor {
    pub specs: Vec<McpServerSpec>,
}

impl ConfigContributor for McpConfigContributor {
    fn contribute(&self, config: &mut Table) {
        // OAuth tokens obtained by Codex go to the OS keyring (Windows
        // Credential Manager), never to a file in CODEX_HOME.
        config.insert(
            "mcp_oauth_credentials_store".into(),
            Value::String("keyring".into()),
        );
        let servers = section(config, "mcp_servers");
        for s in &self.specs {
            if s.validate().is_ok() {
                servers.insert(s.name.clone(), Value::Table(s.to_toml()));
            }
        }
    }
}

/// Persistence of specs (`mcp_servers_meta`) + their secrets in the vault.
pub struct McpServersRepo {
    store: Store,
}

impl McpServersRepo {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn list(&self) -> Result<Vec<McpServerSpec>, McpConfigError> {
        Ok(self.store.with_conn(|c| {
            let mut st = c.prepare("SELECT spec_json FROM mcp_servers_meta ORDER BY name")?;
            let rows = st
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows
                .into_iter()
                .filter_map(|j| serde_json::from_str(&j).ok())
                .collect())
        })?)
    }

    /// Saves the spec and the given secrets (`var → value`; for HTTP bearer
    /// use [`McpServerSpec::bearer_target`] via `bearer`).
    pub fn save(
        &self,
        spec: &McpServerSpec,
        secrets: &[(String, Secret<String>)],
        bearer: Option<&Secret<String>>,
        vault: &dyn CredentialStore,
    ) -> Result<(), McpConfigError> {
        spec.validate()?;
        let mut others = self.list()?;
        others.retain(|s| s.name != spec.name);
        others.push(spec.clone());
        check_collisions(&others)?;
        for (var, value) in secrets {
            vault.put(&secret_target(&spec.name, var), value)?;
        }
        if let Some(b) = bearer {
            vault.put(&spec.bearer_target(), b)?;
        }
        let json = serde_json::to_string(spec).expect("json");
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO mcp_servers_meta(name, spec_json, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(name) DO UPDATE SET spec_json = excluded.spec_json, updated_at = excluded.updated_at",
                params![spec.name, json, now_secs()],
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn delete(&self, name: &str, vault: &dyn CredentialStore) -> Result<(), McpConfigError> {
        for t in vault.list(&target("mcp", &format!("{name}/")))? {
            vault.delete(&t)?;
        }
        self.store.with_conn(|c| {
            Ok(c.execute(
                "DELETE FROM mcp_servers_meta WHERE name = ?1",
                params![name],
            )?)
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_core::credentials::MemoryCredentialStore;

    pub(crate) fn github() -> McpServerSpec {
        let mut env = BTreeMap::new();
        env.insert("GITHUB_TOKEN".into(), EnvValue::Secret);
        env.insert(
            "LOG_LEVEL".into(),
            EnvValue::Plain {
                value: "info".into(),
            },
        );
        McpServerSpec {
            name: "github".into(),
            transport: Transport::Stdio {
                command: "npx".into(),
                args: vec!["-y".into(), "@modelcontextprotocol/server-github".into()],
                env,
                cwd: None,
            },
            enabled: true,
            disabled_tools: vec!["delete_repository".into()],
            approval_mode: ApprovalMode::AskForWrites,
            startup_timeout_sec: Some(20),
            tool_timeout_sec: None,
        }
    }

    #[test]
    fn toml_never_contains_secrets() {
        let spec = github();
        let rendered = toml::to_string(&spec.to_toml()).unwrap();
        assert_eq!(
            rendered,
            "args = [\"-y\", \"@modelcontextprotocol/server-github\"]\ncommand = \"npx\"\ndisabled_tools = [\"delete_repository\"]\nenabled = true\nenv_vars = [\"GITHUB_TOKEN\"]\nstartup_timeout_sec = 20\n\n[env]\nLOG_LEVEL = \"info\"\n"
        );
        let http = McpServerSpec {
            name: "linear-app".into(),
            transport: Transport::Http {
                url: "https://mcp.linear.app/mcp".into(),
                bearer_secret: true,
                headers: BTreeMap::new(),
            },
            ..github()
        };
        let t = http.to_toml();
        assert_eq!(
            t["bearer_token_env_var"].as_str(),
            Some("AURA_MCP_LINEAR_APP_TOKEN")
        );
        let mut cfg = Table::new();
        McpConfigContributor {
            specs: vec![spec, http],
        }
        .contribute(&mut cfg);
        assert_eq!(cfg["mcp_servers"].as_table().unwrap().len(), 2);
        assert_eq!(cfg["mcp_oauth_credentials_store"].as_str(), Some("keyring"));
    }

    #[test]
    fn approval_rules() {
        let ro = ToolHints {
            read_only: Some(true),
            destructive: None,
        };
        let destructive = ToolHints {
            read_only: None,
            destructive: Some(true),
        };
        assert!(!needs_approval(ApprovalMode::AskForWrites, ro));
        assert!(needs_approval(
            ApprovalMode::AskForWrites,
            ToolHints::default()
        ));
        assert!(needs_approval(ApprovalMode::AlwaysAsk, ro));
        assert!(!needs_approval(ApprovalMode::Auto, ToolHints::default()));
        assert!(needs_approval(ApprovalMode::Auto, destructive));
    }

    #[test]
    fn validation_and_collisions() {
        let mut s = github();
        s.name = "aura".into();
        assert_eq!(s.validate(), Err(McpConfigError::Reserved));
        s.name = "bad name".into();
        assert_eq!(s.validate(), Err(McpConfigError::BadName));
        let mut other = github();
        other.name = "github2".into();
        assert!(matches!(
            check_collisions(&[github(), other.clone()]),
            Err(McpConfigError::SecretCollision { .. })
        ));
        other.enabled = false;
        assert!(check_collisions(&[github(), other]).is_ok());
    }

    #[test]
    fn repo_keeps_secrets_in_the_vault() {
        let vault = MemoryCredentialStore::default();
        let repo = McpServersRepo::new(Store::open_in_memory().unwrap());
        let spec = github();
        repo.save(
            &spec,
            &[("GITHUB_TOKEN".into(), Secret::new("ghp_x".into()))],
            None,
            &vault,
        )
        .unwrap();
        assert_eq!(repo.list().unwrap(), vec![spec.clone()]);
        let env = secret_env(&[spec], &vault).unwrap();
        assert_eq!(env[0].0, "GITHUB_TOKEN");
        assert_eq!(env[0].1.expose(), "ghp_x");
        repo.delete("github", &vault).unwrap();
        assert!(repo.list().unwrap().is_empty());
        assert!(vault.list("Aura/mcp/").unwrap().is_empty());
        assert!(matches!(
            secret_env(&[github()], &vault),
            Err(McpConfigError::MissingSecret(_))
        ));
    }
}
