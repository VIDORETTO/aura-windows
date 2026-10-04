//! A fake Codex app-server speaking the subset of the protocol Aura uses.
//!
//! Used by contract tests (in memory) and by `aura-fake-codex` (stdio) so the
//! desktop app can be developed and E2E-tested without a ChatGPT account:
//! set `AURA_CODEX_BIN` to the `aura-fake-codex` executable.
//!
//! Behaviour switches (by user text in `turn/start`):
//! * contains `/aprovar` or `/approve` → asks a command approval first and
//!   replies with the decision it received;
//! * contains `/erro-limite` → fails the turn with a plan usage-limit error;
//! * contains `/lento` → streams slowly (for interrupt/steer tests);
//! * otherwise streams `"Olá"`, `" do Aura falso."`.

use aura_core::jsonrpc::{Incoming, Peer};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;

/// State shared by every connection, like the real app-server's on-disk
/// history. Optionally persisted to a JSON file (for the stdio binary).
#[derive(Clone, Default)]
pub struct FakeState {
    world: Arc<Mutex<World>>,
    file: Option<std::path::PathBuf>,
}

impl std::fmt::Debug for FakeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FakeState")
    }
}

impl FakeState {
    pub fn with_file(path: std::path::PathBuf) -> Self {
        let world = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .map(|v| World {
                threads: serde_json::from_value(v["threads"].clone()).unwrap_or_default(),
                turns: serde_json::from_value(v["turns"].clone()).unwrap_or_default(),
                next: v["next"].as_u64().unwrap_or(0),
                ..Default::default()
            })
            .unwrap_or_default();
        Self {
            world: Arc::new(Mutex::new(world)),
            file: Some(path),
        }
    }

    async fn save(&self) {
        if let Some(path) = &self.file {
            let w = self.world.lock().await;
            let v = json!({"threads": w.threads, "turns": w.turns, "next": w.next});
            let _ = std::fs::write(path, v.to_string());
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FakeConfig {
    /// History shared across connections (restarts keep conversations).
    pub state: FakeState,
    /// Close the connection right after answering `turn/start`.
    pub crash_after_turn_start: bool,
    /// Every received message is appended here (for payload assertions).
    pub record: Option<Arc<std::sync::Mutex<Vec<Value>>>>,
    /// Delay between streamed deltas.
    pub delta_delay: Duration,
}

#[derive(Default)]
struct World {
    threads: Vec<Value>,
    turns: HashMap<String, Vec<Value>>,
    next: u64,
    active_turn: HashMap<String, String>,
    interrupted: HashMap<String, bool>,
    skill_roots: Vec<String>,
    disabled_skills: Vec<String>,
}

impl World {
    fn id(&mut self, prefix: &str) -> String {
        self.next += 1;
        format!("{prefix}_{}", self.next)
    }
}

/// Serves one connection until it closes.
pub async fn serve<R, W>(reader: R, writer: W, cfg: FakeConfig)
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let (peer, mut incoming) = Peer::spawn(reader, writer);
    let world = cfg.state.world.clone();
    while let Some(msg) = incoming.recv().await {
        match msg {
            Incoming::Notification { method, params } => {
                record(&cfg, json!({"method": method, "params": params}));
            }
            Incoming::Request {
                method,
                params,
                responder,
            } => {
                record(&cfg, json!({"method": method, "params": params}));
                let crash = cfg.crash_after_turn_start && method == "turn/start";
                handle(&peer, &world, &cfg, &method, params, responder).await;
                cfg.state.save().await;
                if crash {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    peer.close();
                    return;
                }
            }
        }
    }
}

fn record(cfg: &FakeConfig, v: Value) {
    if let Some(r) = &cfg.record {
        r.lock().unwrap().push(v);
    }
}

fn thread_obj(id: &str, preview: &str, ephemeral: bool) -> Value {
    json!({"id": id, "sessionId": id, "preview": preview, "name": null, "ephemeral": ephemeral,
           "modelProvider": "aura-chatgpt-plan", "createdAt": 1_790_000_000, "updatedAt": 1_790_000_000,
           "isPinned": false, "status": {"type": "idle"}})
}

async fn handle(
    peer: &Peer,
    world: &Arc<Mutex<World>>,
    cfg: &FakeConfig,
    method: &str,
    params: Value,
    responder: aura_core::jsonrpc::Responder,
) {
    let str_param = |k: &str| {
        params
            .get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    match method {
        "initialize" => responder.ok(json!({"userAgent": "aura-fake-codex/0.1.0", "platformFamily": "windows", "platformOs": "windows"})),
        "thread/start" => {
            let mut w = world.lock().await;
            let id = w.id("thr");
            let ephemeral = params.get("ephemeral").and_then(Value::as_bool).unwrap_or(false);
            let mut t = thread_obj(&id, "", ephemeral);
            if let Some(p) = params.get("modelProvider").and_then(Value::as_str) {
                t["modelProvider"] = json!(p);
            }
            if !ephemeral {
                w.threads.push(t.clone());
            }
            drop(w);
            responder.ok(json!({"thread": t, "instructionSources": []}));
            let _ = peer.notify("thread/started", json!({"thread": {"id": id}}));
        }
        "thread/resume" => {
            let id = str_param("threadId");
            let w = world.lock().await;
            match w.threads.iter().find(|t| t["id"] == id) {
                Some(t) => responder.ok(json!({"thread": t})),
                None => responder.err(-32600, "thread not found"),
            }
        }
        "thread/read" => {
            let id = str_param("threadId");
            let w = world.lock().await;
            match w.threads.iter().find(|t| t["id"] == id) {
                Some(t) => {
                    let mut t = t.clone();
                    t["turns"] = json!(w.turns.get(&id).cloned().unwrap_or_default());
                    responder.ok(json!({"thread": t}))
                }
                None => responder.err(-32600, "thread not found"),
            }
        }
        "thread/list" => {
            let w = world.lock().await;
            let term = params.get("searchTerm").and_then(Value::as_str).unwrap_or("").to_lowercase();
            let archived = params.get("archived").and_then(Value::as_bool).unwrap_or(false);
            // Like the real server: no modelProviders = only the configured
            // default provider; an empty list = every provider.
            let providers: Option<Vec<String>> = params.get("modelProviders").and_then(Value::as_array).map(|a| {
                a.iter().filter_map(Value::as_str).map(str::to_string).collect()
            });
            let data: Vec<Value> = w
                .threads
                .iter()
                .filter(|t| t.get("archived").and_then(Value::as_bool).unwrap_or(false) == archived)
                .filter(|t| {
                    let p = t["modelProvider"].as_str().unwrap_or_default();
                    match &providers {
                        None => p == "aura-chatgpt-plan",
                        Some(list) => list.is_empty() || list.iter().any(|x| x == p),
                    }
                })
                .filter(|t| term.is_empty() || t["preview"].as_str().unwrap_or("").to_lowercase().contains(&term))
                .cloned()
                .collect();
            responder.ok(json!({"data": data, "nextCursor": null}))
        }
        "thread/name/set" => {
            let id = str_param("threadId");
            let mut w = world.lock().await;
            if let Some(t) = w.threads.iter_mut().find(|t| t["id"] == id) {
                t["name"] = params["name"].clone();
            }
            responder.ok(json!({}));
            let _ = peer.notify("thread/name/updated", json!({"threadId": id, "name": params["name"]}));
        }
        "thread/metadata/update" => {
            let id = str_param("threadId");
            let mut w = world.lock().await;
            if let (Some(t), Some(pinned)) = (w.threads.iter_mut().find(|t| t["id"] == id), params.get("isPinned")) {
                t["isPinned"] = pinned.clone();
            }
            responder.ok(json!({}));
        }
        "thread/archive" => {
            let id = str_param("threadId");
            let mut w = world.lock().await;
            if let Some(t) = w.threads.iter_mut().find(|t| t["id"] == id) {
                t["archived"] = json!(true);
            }
            responder.ok(json!({}));
        }
        "thread/delete" => {
            let id = str_param("threadId");
            let mut w = world.lock().await;
            w.threads.retain(|t| t["id"] != id);
            responder.ok(json!({}));
            let _ = peer.notify("thread/deleted", json!({"threadId": id}));
        }
        "thread/compact/start" => {
            let id = str_param("threadId");
            responder.ok(json!({}));
            let _ = peer.notify("item/completed", json!({"threadId": id, "turnId": "compact", "item": {"type": "contextCompaction", "id": "cmp"}}));
            let _ = peer.notify(
                "thread/tokenUsage/updated",
                json!({"threadId": id, "turnId": "compact", "tokenUsage": {"total": {"totalTokens": 30000}, "last": {"totalTokens": 8000}, "modelContextWindow": 200000}}),
            );
        }
        "mcpServerStatus/list" => responder.ok(json!({"data": [
            {"name": "aura", "authStatus": "unsupported", "tools": {
                "screen_capture": {"name": "screen_capture"}, "screen_text": {"name": "screen_text"}}}
        ], "nextCursor": null})),
        "mcpServer/oauth/login" => responder.ok(json!({"authorizationUrl": "https://example.com/oauth/authorize"})),
        "config/mcpServer/reload" => responder.ok(json!({})),
        "model/list" => responder.ok(json!({"data": [
            {"id": "gpt-fake", "model": "gpt-fake", "displayName": "GPT Falso", "hidden": false, "isDefault": true,
             "defaultReasoningEffort": "medium", "inputModalities": ["text", "image"],
             "supportedReasoningEfforts": [{"reasoningEffort": "low"}, {"reasoningEffort": "medium"}, {"reasoningEffort": "high"}]}
        ], "nextCursor": null})),
        "skills/extraRoots/set" => {
            let mut w = world.lock().await;
            w.skill_roots = params["extraRoots"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_string).collect();
            responder.ok(json!({}));
        }
        "skills/list" => {
            // Extra roots as "user" skills: <root>/<name>/SKILL.md, like the real server.
            let w = world.lock().await;
            let mut skills = vec![];
            for root in &w.skill_roots {
                for entry in std::fs::read_dir(root).into_iter().flatten().flatten() {
                    let md_path = entry.path().join("SKILL.md");
                    let Ok(md) = std::fs::read_to_string(&md_path) else { continue };
                    let field = |k: &str| md.lines().find_map(|l| l.strip_prefix(&format!("{k}: ")).map(|v| v.trim_matches('"').to_string())).unwrap_or_default();
                    let path = md_path.to_string_lossy().into_owned();
                    let enabled = !w.disabled_skills.contains(&path);
                    skills.push(json!({"name": field("name"), "description": field("description"), "path": path, "scope": "user", "enabled": enabled, "pluginId": null}));
                }
            }
            skills.push(json!({"name": "skill-creator", "description": "System skill", "path": "C:/codex/skills/.system/skill-creator/SKILL.md", "scope": "system", "enabled": true, "pluginId": null}));
            responder.ok(json!({"data": [{"cwd": params["cwds"][0], "skills": skills, "errors": []}]}));
        }
        "skills/config/write" => {
            let mut w = world.lock().await;
            let path = params["path"].as_str().unwrap_or_default().to_string();
            let enabled = params["enabled"].as_bool().unwrap_or(true);
            w.disabled_skills.retain(|p| *p != path);
            if !enabled {
                w.disabled_skills.push(path);
            }
            responder.ok(json!({"effectiveEnabled": enabled}));
        }
        "turn/interrupt" => {
            let id = str_param("threadId");
            world.lock().await.interrupted.insert(id, true);
            responder.ok(json!({}));
        }
        "turn/steer" => {
            let id = str_param("threadId");
            let expected = str_param("expectedTurnId");
            let w = world.lock().await;
            match w.active_turn.get(&id) {
                Some(active) if *active == expected => responder.ok(json!({"turnId": active})),
                _ => responder.err(-32600, "no active turn"),
            }
        }
        "turn/start" => {
            let thread_id = str_param("threadId");
            let text: String = params
                .get("input")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|i| i.get("text").and_then(Value::as_str)).collect::<Vec<_>>().join(" "))
                .unwrap_or_default();
            let turn_id = {
                let mut w = world.lock().await;
                let id = w.id("turn");
                w.active_turn.insert(thread_id.clone(), id.clone());
                w.interrupted.insert(thread_id.clone(), false);
                if let Some(t) = w.threads.iter_mut().find(|t| t["id"] == thread_id)
                    && t["preview"].as_str().unwrap_or("").is_empty() {
                        t["preview"] = json!(text.chars().take(80).collect::<String>());
                    }
                id
            };
            responder.ok(json!({"turn": {"id": turn_id, "status": "inProgress", "items": [], "error": null}}));
            if cfg.crash_after_turn_start {
                return;
            }
            let peer = peer.clone();
            let world = world.clone();
            let delay = if text.contains("/lento") { Duration::from_millis(200) } else { cfg.delta_delay };
            let state = cfg.state.clone();
            tokio::spawn(async move {
                run_turn(peer, world, thread_id, turn_id, text, delay).await;
                state.save().await;
            });
        }
        _ => responder.err(aura_core::jsonrpc::RpcError::METHOD_NOT_FOUND, format!("unknown method {method}")),
    }
}

async fn run_turn(
    peer: Peer,
    world: Arc<Mutex<World>>,
    thread_id: String,
    turn_id: String,
    text: String,
    delay: Duration,
) {
    let n = |m: &str, p: Value| {
        let _ = peer.notify(m, p);
    };
    n(
        "turn/started",
        json!({"threadId": thread_id, "turn": {"id": turn_id, "status": "inProgress", "items": []}}),
    );

    if text.contains("/erro-limite") {
        let error = json!({"message": "unexpected status 429 Too Many Requests: {\"error\":{\"code\":\"subscription_sharing_usage_limit_exceeded\"}}",
                           "codexErrorInfo": {"httpConnectionFailed": {"httpStatusCode": 429}}});
        n(
            "error",
            json!({"threadId": thread_id, "turnId": turn_id, "willRetry": false, "error": error}),
        );
        n(
            "turn/completed",
            json!({"threadId": thread_id, "turn": {"id": turn_id, "status": "failed", "items": [], "error": error}}),
        );
        finish(&world, &thread_id, &turn_id, &text, "").await;
        return;
    }

    let mut answer = String::new();
    if text.contains("/aprovar") || text.contains("/approve") {
        let item = json!({"type": "commandExecution", "id": "cmd_1", "command": "echo aura", "cwd": ".", "status": "inProgress", "commandActions": []});
        n(
            "item/started",
            json!({"threadId": thread_id, "turnId": turn_id, "item": item}),
        );
        let decision = peer
            .request(
                "item/commandExecution/requestApproval",
                json!({"threadId": thread_id, "turnId": turn_id, "itemId": "cmd_1", "startedAtMs": 0,
                       "command": "echo aura", "cwd": ".", "reason": "demonstração de aprovação"}),
            )
            .await
            .ok()
            .and_then(|v| v.get("decision").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "cancel".into());
        n(
            "serverRequest/resolved",
            json!({"threadId": thread_id, "requestId": "0"}),
        );
        let status = if decision.starts_with("accept") {
            "completed"
        } else {
            "declined"
        };
        let item = json!({"type": "commandExecution", "id": "cmd_1", "command": "echo aura", "cwd": ".", "status": status,
                          "aggregatedOutput": if status == "completed" { "aura\n" } else { "" }, "commandActions": []});
        n(
            "item/completed",
            json!({"threadId": thread_id, "turnId": turn_id, "item": item}),
        );
        answer.push_str(&format!("Decisão recebida: {decision}. "));
    }

    let msg_id = format!("msg_{turn_id}");
    n(
        "item/started",
        json!({"threadId": thread_id, "turnId": turn_id, "item": {"type": "agentMessage", "id": msg_id, "text": ""}}),
    );
    let parts = [
        "Olá",
        " do Aura falso.",
        "\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    ];
    let mut interrupted = false;
    for part in parts {
        if world
            .lock()
            .await
            .interrupted
            .get(&thread_id)
            .copied()
            .unwrap_or(false)
        {
            interrupted = true;
            break;
        }
        answer.push_str(part);
        n(
            "item/agentMessage/delta",
            json!({"threadId": thread_id, "turnId": turn_id, "itemId": msg_id, "delta": part}),
        );
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }
    if world
        .lock()
        .await
        .interrupted
        .get(&thread_id)
        .copied()
        .unwrap_or(false)
    {
        interrupted = true;
    }
    n(
        "item/completed",
        json!({"threadId": thread_id, "turnId": turn_id, "item": {"type": "agentMessage", "id": msg_id, "text": answer}}),
    );
    n(
        "thread/tokenUsage/updated",
        json!({"threadId": thread_id, "turnId": turn_id, "tokenUsage": {"total": {"totalTokens": 1200}, "last": {"totalTokens": 1200}, "modelContextWindow": 200000}}),
    );
    let status = if interrupted {
        "interrupted"
    } else {
        "completed"
    };
    n(
        "turn/completed",
        json!({"threadId": thread_id, "turn": {"id": turn_id, "status": status, "items": [], "error": null}}),
    );
    finish(&world, &thread_id, &turn_id, &text, &answer).await;
}

async fn finish(
    world: &Arc<Mutex<World>>,
    thread_id: &str,
    turn_id: &str,
    user: &str,
    answer: &str,
) {
    let mut w = world.lock().await;
    w.active_turn.remove(thread_id);
    w.turns.entry(thread_id.to_string()).or_default().push(json!({
        "id": turn_id,
        "items": [
            {"type": "userMessage", "id": format!("u_{turn_id}"), "content": [{"type": "text", "text": user}]},
            {"type": "agentMessage", "id": format!("msg_{turn_id}"), "text": answer}
        ]
    }));
}
