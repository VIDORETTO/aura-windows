//! Detects MCP servers configured in other apps (AC-009) and converts them to
//! [`McpServerSpec`]s, moving secrets to the vault instead of plain text.

use crate::mcp_config::{ApprovalMode, EnvValue, McpServerSpec, Transport};
use aura_core::secret::Secret;
use serde::Serialize;
use serde_json::Value as Json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExternalApp {
    ClaudeDesktop,
    Cursor,
    VsCode,
    CodexCli,
}

impl ExternalApp {
    pub fn label(self) -> &'static str {
        match self {
            ExternalApp::ClaudeDesktop => "Claude Desktop",
            ExternalApp::Cursor => "Cursor",
            ExternalApp::VsCode => "VS Code",
            ExternalApp::CodexCli => "Codex CLI",
        }
    }
}

/// A server found on disk. `secrets` hold values that must go to the vault
/// on import; they are never shown in the UI (only their names are).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedServer {
    pub app: ExternalApp,
    pub source: PathBuf,
    pub spec: McpServerSpec,
    #[serde(skip)]
    pub secrets: Vec<(String, Secret<String>)>,
    #[serde(skip)]
    pub bearer: Option<Secret<String>>,
    pub secret_names: Vec<String>,
    /// e.g. VS Code `${input:…}` placeholders the user must fill after import.
    pub warnings: Vec<String>,
}

/// Where each app keeps its config, given the user's folders.
pub struct Roots {
    /// `%USERPROFILE%`
    pub home: PathBuf,
    /// `%APPDATA%` (Roaming)
    pub appdata: PathBuf,
}

impl Roots {
    pub fn candidates(&self) -> Vec<(ExternalApp, PathBuf)> {
        vec![
            (
                ExternalApp::ClaudeDesktop,
                self.appdata
                    .join("Claude")
                    .join("claude_desktop_config.json"),
            ),
            (
                ExternalApp::Cursor,
                self.home.join(".cursor").join("mcp.json"),
            ),
            (
                ExternalApp::VsCode,
                self.appdata.join("Code").join("User").join("mcp.json"),
            ),
            (
                ExternalApp::VsCode,
                self.appdata.join("Code").join("User").join("settings.json"),
            ),
            (
                ExternalApp::CodexCli,
                self.home.join(".codex").join("config.toml"),
            ),
        ]
    }
}

/// Heuristic for secret-looking variable names.
pub fn looks_secret(name: &str) -> bool {
    let n = name.to_ascii_uppercase();
    [
        "TOKEN",
        "KEY",
        "SECRET",
        "PASSWORD",
        "PASSWD",
        "PAT",
        "CREDENTIAL",
        "AUTH",
        "COOKIE",
    ]
    .iter()
    .any(|w| n.contains(w))
}

pub fn detect(roots: &Roots) -> Vec<DetectedServer> {
    let mut out = Vec::new();
    for (app, path) in roots.candidates() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let found = match app {
            ExternalApp::CodexCli => parse_codex(&text),
            ExternalApp::VsCode if path.file_name().is_some_and(|n| n == "settings.json") => {
                parse_json(&strip_jsonc(&text), &["mcp", "servers"])
            }
            ExternalApp::VsCode => parse_json(&strip_jsonc(&text), &["servers"]),
            _ => parse_json(&strip_jsonc(&text), &["mcpServers"]),
        };
        for mut d in found {
            d.app = app;
            d.source = path.clone();
            out.push(d);
        }
    }
    dedupe_names(&mut out);
    out
}

/// Two apps may use the same server name: suffix later ones (`github-cursor`).
fn dedupe_names(list: &mut [DetectedServer]) {
    let mut seen = std::collections::HashSet::new();
    for d in list.iter_mut() {
        if !seen.insert(d.spec.name.clone()) {
            let suffix = match d.app {
                ExternalApp::ClaudeDesktop => "claude",
                ExternalApp::Cursor => "cursor",
                ExternalApp::VsCode => "vscode",
                ExternalApp::CodexCli => "codex",
            };
            d.spec.name = format!("{}-{suffix}", d.spec.name);
            seen.insert(d.spec.name.clone());
        }
    }
}

/// Removes `//` and `/* */` comments and trailing commas (VS Code JSONC).
pub fn strip_jsonc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let b: Vec<char> = s.chars().collect();
    let (mut i, mut in_str) = (0, false);
    while i < b.len() {
        let c = b[i];
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < b.len() {
                out.push(b[i + 1]);
                i += 1;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
            out.push(c);
        } else if c == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
            continue;
        } else if c == '/' && b.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        } else if c == ',' {
            let next = b[i + 1..].iter().find(|c| !c.is_whitespace());
            if !matches!(next, Some('}') | Some(']')) {
                out.push(c);
            }
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

fn sanitize_name(n: &str) -> String {
    let s: String = n
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let s = s.trim_matches('-').chars().take(64).collect::<String>();
    if s.is_empty() || s.eq_ignore_ascii_case("aura") {
        format!("importado-{s}")
    } else {
        s
    }
}

fn blank(app: ExternalApp) -> DetectedServer {
    DetectedServer {
        app,
        source: PathBuf::new(),
        spec: McpServerSpec {
            name: String::new(),
            transport: Transport::Stdio {
                command: String::new(),
                args: vec![],
                env: BTreeMap::new(),
                cwd: None,
            },
            enabled: true,
            disabled_tools: vec![],
            approval_mode: ApprovalMode::AskForWrites,
            startup_timeout_sec: None,
            tool_timeout_sec: None,
        },
        secrets: vec![],
        bearer: None,
        secret_names: vec![],
        warnings: vec![],
    }
}

fn str_map(v: Option<&Json>) -> BTreeMap<String, String> {
    v.and_then(Json::as_object)
        .map(|m| {
            m.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| v.to_string()),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Builds a detected server from `command/args/env` or `url/headers`.
fn build(
    name: &str,
    command: Option<String>,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    cwd: Option<String>,
    url: Option<String>,
    headers: BTreeMap<String, String>,
) -> Option<DetectedServer> {
    let mut d = blank(ExternalApp::ClaudeDesktop);
    d.spec.name = sanitize_name(name);
    let warn_input = |v: &str, d: &mut DetectedServer| {
        if v.contains("${input:") || v.contains("${env:") {
            d.warnings
                .push(format!("preencha o valor de {v} depois de importar"));
        }
    };
    if let Some(command) = command.filter(|c| !c.trim().is_empty()) {
        let mut spec_env = BTreeMap::new();
        for (k, v) in env {
            warn_input(&v, &mut d);
            if looks_secret(&k) && !v.is_empty() {
                spec_env.insert(k.clone(), EnvValue::Secret);
                d.secret_names.push(k.clone());
                d.secrets.push((k, Secret::new(v)));
            } else {
                spec_env.insert(k, EnvValue::Plain { value: v });
            }
        }
        d.spec.transport = Transport::Stdio {
            command,
            args,
            env: spec_env,
            cwd,
        };
        return Some(d);
    }
    let url = url?;
    let mut plain = BTreeMap::new();
    let mut bearer = None;
    for (k, v) in headers {
        warn_input(&v, &mut d);
        if k.eq_ignore_ascii_case("authorization") {
            let token = v
                .strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
                .unwrap_or(&v)
                .to_string();
            bearer = Some(Secret::new(token));
        } else if looks_secret(&k) {
            d.warnings.push(format!(
                "o cabeçalho {k} parece um segredo e não foi importado; configure-o no Aura"
            ));
        } else {
            plain.insert(k, v);
        }
    }
    if bearer.is_some() {
        d.secret_names.push("Authorization".into());
    }
    d.spec.transport = Transport::Http {
        url,
        bearer_secret: bearer.is_some(),
        headers: plain,
    };
    d.bearer = bearer;
    Some(d)
}

/// Claude Desktop / Cursor (`mcpServers`) and VS Code (`servers`).
pub fn parse_json(text: &str, path: &[&str]) -> Vec<DetectedServer> {
    let Ok(root) = serde_json::from_str::<Json>(text) else {
        return vec![];
    };
    let mut node = &root;
    for key in path {
        match node.get(key) {
            Some(n) => node = n,
            None => return vec![],
        }
    }
    let Some(map) = node.as_object() else {
        return vec![];
    };
    map.iter()
        .filter_map(|(name, v)| {
            let args = v
                .get("args")
                .and_then(Json::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            build(
                name,
                v.get("command").and_then(Json::as_str).map(str::to_string),
                args,
                str_map(v.get("env")),
                v.get("cwd").and_then(Json::as_str).map(str::to_string),
                v.get("url")
                    .or_else(|| v.get("serverUrl"))
                    .and_then(Json::as_str)
                    .map(str::to_string),
                str_map(v.get("headers")),
            )
        })
        .collect()
}

/// Codex CLI `~/.codex/config.toml` `[mcp_servers.*]`.
pub fn parse_codex(text: &str) -> Vec<DetectedServer> {
    let Ok(root) = text.parse::<toml::Table>() else {
        return vec![];
    };
    let Some(servers) = root.get("mcp_servers").and_then(|v| v.as_table()) else {
        return vec![];
    };
    servers
        .iter()
        .filter_map(|(name, v)| {
            let t = v.as_table()?;
            let s = |k: &str| t.get(k).and_then(|v| v.as_str()).map(str::to_string);
            let map = |k: &str| -> BTreeMap<String, String> {
                t.get(k)
                    .and_then(|v| v.as_table())
                    .map(|m| {
                        m.iter()
                            .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let args = t
                .get("args")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let mut d = build(
                name,
                s("command"),
                args,
                map("env"),
                s("cwd"),
                s("url"),
                map("http_headers"),
            )?;
            if let Some(var) = s("bearer_token_env_var") {
                d.warnings.push(format!(
                    "o token vinha da variável {var}; informe-o no Aura"
                ));
            }
            if let Some(dis) = t.get("disabled_tools").and_then(|v| v.as_array()) {
                d.spec.disabled_tools = dis
                    .iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect();
            }
            if t.get("enabled").and_then(|v| v.as_bool()) == Some(false) {
                d.spec.enabled = false;
            }
            Some(d)
        })
        .collect()
}

pub fn default_roots() -> Option<Roots> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)?;
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("AppData").join("Roaming"));
    Some(Roots { home, appdata })
}

pub fn is_under(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonc_comments_and_trailing_commas() {
        let s = strip_jsonc("{\n // c\n \"a\": \"http://x\", /* b */ \"b\": [1,2,],\n}");
        let v: Json = serde_json::from_str(&s).unwrap();
        assert_eq!(v["a"], "http://x");
        assert_eq!(v["b"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn secret_heuristic() {
        for s in [
            "GITHUB_TOKEN",
            "OPENAI_API_KEY",
            "db_password",
            "SLACK_BOT_TOKEN",
        ] {
            assert!(looks_secret(s), "{s}");
        }
        for s in ["LOG_LEVEL", "PORT", "NODE_ENV"] {
            assert!(!looks_secret(s), "{s}");
        }
    }
}
