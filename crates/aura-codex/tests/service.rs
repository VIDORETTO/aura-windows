//! Contract tests of `CodexService` against the fake app-server (in memory).

use aura_codex::approvals::Decision;
use aura_codex::events::{AppServerState, ConversationEvent as E, TurnError, TurnStatus};
use aura_codex::fake::{FakeConfig, serve};
use aura_codex::launcher::InMemoryLauncher;
use aura_codex::modes::{ConversationMode, UiLanguage, persona};
use aura_codex::service::{CodexService, HistoryQuery, StartOptions, TurnOptions};
use aura_codex::supervisor::{AppServerSupervisor, SupervisorConfig};
use aura_core::context::TurnInput;
use aura_store::Store;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

struct Harness {
    svc: Arc<CodexService>,
    record: Arc<Mutex<Vec<Value>>>,
    _dir: tempfile::TempDir,
    root: std::path::PathBuf,
}

fn harness(cfg: FakeConfig, sup_cfg: SupervisorConfig) -> Harness {
    let record = Arc::new(Mutex::new(Vec::new()));
    let mut cfg = cfg;
    // `cfg.state` is shared by every launch, like the real on-disk history.
    cfg.record = Some(record.clone());
    let launcher = InMemoryLauncher::new(move |stream| {
        let cfg = cfg.clone();
        tokio::spawn(async move {
            let (r, w) = tokio::io::split(stream);
            serve(r, w, cfg).await;
        });
    });
    let (tx, rx) = mpsc::unbounded_channel();
    let sup = AppServerSupervisor::new(Arc::new(launcher), sup_cfg, tx);
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("workspaces");
    let svc = CodexService::new(sup, rx, Store::open_in_memory().unwrap(), root.clone());
    Harness {
        svc,
        record,
        _dir: dir,
        root,
    }
}

fn default_harness() -> Harness {
    harness(FakeConfig::default(), SupervisorConfig::default())
}

fn text(t: &str) -> Vec<TurnInput> {
    vec![TurnInput::Text { text: t.into() }]
}

async fn until_turn_completed(rx: &mut broadcast::Receiver<E>) -> Vec<E> {
    let mut out = Vec::new();
    loop {
        let e = tokio::time::timeout(Duration::from_secs(30), rx.recv())
            .await
            .expect("event")
            .expect("open");
        let done = matches!(e, E::TurnCompleted { .. });
        out.push(e);
        if done {
            return out;
        }
    }
}

fn sent(record: &Arc<Mutex<Vec<Value>>>, method: &str) -> Vec<Value> {
    record
        .lock()
        .unwrap()
        .iter()
        .filter(|m| m["method"] == method)
        .map(|m| m["params"].clone())
        .collect()
}

#[tokio::test]
async fn streaming_turn_with_aura_persona_and_chat_mode() {
    let h = default_harness();
    let mut rx = h.svc.events();
    let conv = h.svc.start(StartOptions::default()).await.unwrap();
    assert!(conv.workspace.starts_with(&h.root) && conv.workspace.exists());
    h.svc
        .send(
            &conv.thread_id,
            &text("liste 3 atalhos"),
            TurnOptions::default(),
        )
        .await
        .unwrap();
    let events = until_turn_completed(&mut rx).await;

    let deltas: String = events
        .iter()
        .filter_map(|e| {
            if let E::MessageDelta { delta, .. } = e {
                Some(delta.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(deltas.starts_with("Olá do Aura falso."));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, E::MessageCompleted { text, .. } if text.contains("| a | b |")))
    );
    assert!(matches!(
        events.last(),
        Some(E::TurnCompleted {
            status: TurnStatus::Completed,
            error: None,
            ..
        })
    ));

    let start = &sent(&h.record, "thread/start")[0];
    assert_eq!(start["baseInstructions"], persona(UiLanguage::PtBr));
    assert_eq!(start["sandbox"], "read-only");
    assert_eq!(start["modelProvider"], "aura-chatgpt-plan");
    assert_eq!(start["serviceName"], "aura_desktop");
    assert_eq!(
        start["config"]["mcp_servers.aura.http_headers"]["X-Aura-Conversation"],
        conv.conversation_uuid
    );
    let turn = &sent(&h.record, "turn/start")[0];
    assert_eq!(turn["sandboxPolicy"]["type"], "readOnly");
    assert_eq!(turn["input"][0]["type"], "text");
    let init = &sent(&h.record, "initialize")[0];
    assert_eq!(init["clientInfo"]["name"], "aura_desktop");
}

#[tokio::test]
async fn approval_is_only_answered_by_the_user() {
    let h = default_harness();
    let mut rx = h.svc.events();
    let conv = h
        .svc
        .start(StartOptions {
            mode: ConversationMode::Task {
                granted: vec![],
                network: false,
            },
            ..Default::default()
        })
        .await
        .unwrap();
    h.svc
        .send(
            &conv.thread_id,
            &text("/aprovar rode echo"),
            TurnOptions::default(),
        )
        .await
        .unwrap();
    let request_id = loop {
        if let E::ApprovalRequested {
            request_id,
            command,
            ..
        } = rx.recv().await.unwrap()
        {
            assert_eq!(command.as_deref(), Some("echo aura"));
            break request_id;
        }
    };
    // Nothing answers automatically.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(h.svc.pending_request_ids(), vec![request_id.clone()]);
    h.svc
        .respond(&request_id, Decision::AcceptForSession)
        .await
        .unwrap();
    let events = until_turn_completed(&mut rx).await;
    assert!(events.iter().any(
        |e| matches!(e, E::MessageCompleted { text, .. } if text.contains("acceptForSession"))
    ));
    assert_eq!(
        sent(&h.record, "turn/start")[0]["sandboxPolicy"]["type"],
        "workspaceWrite"
    );
}

#[tokio::test]
async fn interrupt_keeps_partial_text() {
    let h = default_harness();
    let mut rx = h.svc.events();
    let conv = h.svc.start(StartOptions::default()).await.unwrap();
    h.svc
        .send(&conv.thread_id, &text("/lento"), TurnOptions::default())
        .await
        .unwrap();
    loop {
        if let E::MessageDelta { .. } = rx.recv().await.unwrap() {
            break;
        }
    }
    h.svc.interrupt(&conv.thread_id).await.unwrap();
    let events = until_turn_completed(&mut rx).await;
    let E::TurnCompleted { status, .. } = events.last().unwrap() else {
        panic!()
    };
    assert_eq!(*status, TurnStatus::Interrupted);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, E::MessageCompleted { text, .. } if text.starts_with("Olá")))
    );
}

#[tokio::test]
async fn plan_usage_limit_is_categorised() {
    let h = default_harness();
    let mut rx = h.svc.events();
    let conv = h.svc.start(StartOptions::default()).await.unwrap();
    h.svc
        .send(
            &conv.thread_id,
            &text("/erro-limite"),
            TurnOptions::default(),
        )
        .await
        .unwrap();
    let events = until_turn_completed(&mut rx).await;
    assert!(matches!(
        events.last(),
        Some(E::TurnCompleted {
            status: TurnStatus::Failed,
            error: Some(TurnError::PlanUsageLimit),
            ..
        })
    ));
}

#[tokio::test]
async fn history_rename_pin_delete_and_workspace_cleanup() {
    let h = default_harness();
    let mut rx = h.svc.events();
    let a = h.svc.start(StartOptions::default()).await.unwrap();
    h.svc
        .send(
            &a.thread_id,
            &text("fatura de setembro"),
            TurnOptions::default(),
        )
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;
    let b = h.svc.start(StartOptions::default()).await.unwrap();
    h.svc
        .send(
            &b.thread_id,
            &text("receita de bolo"),
            TurnOptions::default(),
        )
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;

    let found = h
        .svc
        .list(HistoryQuery {
            search: Some("fatura".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        found.items.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
        vec![a.thread_id.clone()]
    );
    assert_eq!(sent(&h.record, "thread/list")[0]["searchTerm"], "fatura");

    h.svc.rename(&b.thread_id, "Bolo").await.unwrap();
    h.svc.pin(&b.thread_id, true).await.unwrap();
    let all = h.svc.list(HistoryQuery::default()).await.unwrap();
    assert_eq!(all.items[0].id, b.thread_id);
    assert_eq!(all.items[0].title, "Bolo");

    let transcript = h.svc.open(&a.thread_id).await.unwrap();
    assert_eq!(transcript[0].role, "user");
    assert_eq!(transcript[0].text, "fatura de setembro");

    h.svc.delete(&a.thread_id).await.unwrap();
    assert!(!a.workspace.exists());
    let all = h.svc.list(HistoryQuery::default()).await.unwrap();
    assert!(all.items.iter().all(|i| i.id != a.thread_id));
}

#[test]
fn next_page_cursor_rereads_the_boundary_second() {
    // The pinned app-server pages by `updatedAt` with a whole-second cursor and a
    // strict "older than": conversations in the last item's second were skipped.
    use aura_codex::service::page_cursor;
    assert_eq!(
        page_cursor("2026-10-04T22:22:36Z"),
        "2026-10-04T22:22:36.999Z"
    );
    // Anything else is passed through untouched (opaque cursors).
    assert_eq!(
        page_cursor("2026-10-04T22:22:36.5Z"),
        "2026-10-04T22:22:36.5Z"
    );
    assert_eq!(page_cursor("abc123"), "abc123");
}

#[tokio::test]
async fn archived_conversations_are_listed_apart_and_can_be_restored() {
    let h = default_harness();
    let a = h.svc.start(StartOptions::default()).await.unwrap();
    let ids = |q: HistoryQuery| {
        let svc = &h.svc;
        async move {
            svc.list(q)
                .await
                .unwrap()
                .items
                .into_iter()
                .map(|i| i.id)
                .collect::<Vec<_>>()
        }
    };
    let archived = || HistoryQuery {
        archived: true,
        ..Default::default()
    };
    h.svc.archive(&a.thread_id).await.unwrap();
    assert!(!ids(HistoryQuery::default()).await.contains(&a.thread_id));
    assert!(ids(archived()).await.contains(&a.thread_id));
    h.svc.unarchive(&a.thread_id).await.unwrap();
    assert_eq!(
        sent(&h.record, "thread/unarchive")[0]["threadId"],
        a.thread_id.as_str()
    );
    assert!(ids(HistoryQuery::default()).await.contains(&a.thread_id));
    assert!(!ids(archived()).await.contains(&a.thread_id));
}

#[tokio::test]
async fn history_lists_conversations_of_every_provider() {
    let h = default_harness();
    let mut rx = h.svc.events();
    let byok = h
        .svc
        .start(StartOptions {
            provider: "aura-qa-byok".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    h.svc
        .send(
            &byok.thread_id,
            &text("conversa BYOK"),
            TurnOptions::default(),
        )
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;
    let all = h.svc.list(HistoryQuery::default()).await.unwrap();
    assert_eq!(
        all.items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        [byok.thread_id.as_str()]
    );
    assert_eq!(
        sent(&h.record, "thread/list")[0]["modelProviders"],
        json!([])
    );
}

#[tokio::test]
async fn skills_include_aura_roots_with_scope_and_can_be_disabled() {
    let h = default_harness();
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("revisar-contrato")).unwrap();
    std::fs::write(
        root.path().join("revisar-contrato/SKILL.md"),
        "---\nname: revisar-contrato\ndescription: Revisa contratos\n---\nCorpo\n",
    )
    .unwrap();
    h.svc.set_skill_roots(vec![root.path().to_path_buf()]);
    let skills = h.svc.skills().await.unwrap();
    let aura = skills
        .iter()
        .find(|s| s.name == "revisar-contrato")
        .unwrap();
    assert_eq!((aura.scope.as_str(), aura.enabled), ("user", true));
    assert!(aura.path.starts_with(root.path()));
    assert!(skills.iter().any(|s| s.scope == "system"));
    assert_eq!(
        sent(&h.record, "skills/extraRoots/set")[0]["extraRoots"],
        json!([root.path()])
    );
    h.svc.set_skill_enabled(&aura.path, false).await.unwrap();
    let after = h.svc.skills().await.unwrap();
    assert!(
        !after
            .iter()
            .find(|s| s.name == "revisar-contrato")
            .unwrap()
            .enabled
    );
    // New conversations load the Aura roots first.
    h.svc.start(StartOptions::default()).await.unwrap();
    assert_eq!(sent(&h.record, "skills/extraRoots/set").len(), 3);
}

#[tokio::test]
async fn ephemeral_conversation_leaves_nothing_behind() {
    let h = default_harness();
    let mut rx = h.svc.events();
    let conv = h
        .svc
        .start(StartOptions {
            ephemeral: true,
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(conv.workspace.to_string_lossy().contains("_ephemeral"));
    h.svc
        .send(&conv.thread_id, &text("segredo"), TurnOptions::default())
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;
    assert_eq!(sent(&h.record, "thread/start")[0]["ephemeral"], true);
    assert!(
        h.svc
            .list(HistoryQuery::default())
            .await
            .unwrap()
            .items
            .is_empty()
    );
    h.svc.close_ephemeral(&conv.thread_id).await.unwrap();
    assert!(!conv.workspace.exists());
}

#[tokio::test]
async fn empty_and_too_long_messages_are_rejected() {
    let h = default_harness();
    let conv = h.svc.start(StartOptions::default()).await.unwrap();
    assert!(
        h.svc
            .send(&conv.thread_id, &text("   "), TurnOptions::default())
            .await
            .is_err()
    );
    let long = "a".repeat(100_001);
    assert!(
        h.svc
            .send(&conv.thread_id, &text(&long), TurnOptions::default())
            .await
            .is_err()
    );
}

#[tokio::test(start_paused = true)]
async fn idle_stop_and_transparent_resume() {
    let h = default_harness();
    let mut rx = h.svc.events();
    let conv = h.svc.start(StartOptions::default()).await.unwrap();
    h.svc
        .send(&conv.thread_id, &text("oi"), TurnOptions::default())
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;
    assert_eq!(h.svc.supervisor().launch_count(), 1);

    tokio::time::sleep(Duration::from_secs(14 * 60)).await;
    assert!(matches!(
        h.svc.supervisor().status().await,
        AppServerState::Ready { .. }
    ));
    tokio::time::sleep(Duration::from_secs(5 * 60)).await;
    assert_eq!(h.svc.supervisor().status().await, AppServerState::Stopped);

    h.svc
        .send(&conv.thread_id, &text("de novo"), TurnOptions::default())
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;
    assert_eq!(h.svc.supervisor().launch_count(), 2);
    assert_eq!(
        sent(&h.record, "thread/resume")[0]["threadId"],
        conv.thread_id
    );
}

#[tokio::test(start_paused = true)]
async fn free_web_stays_disabled_and_correlated_when_a_thread_resumes() {
    let h = default_harness();
    let mut events = h.svc.events();
    let conversation = h
        .svc
        .start(StartOptions {
            config_overrides: serde_json::Map::from_iter([("web_search".into(), json!("live"))]),
            ..Default::default()
        })
        .await
        .unwrap();
    h.svc
        .send(&conversation.thread_id, &text("oi"), Default::default())
        .await
        .unwrap();
    until_turn_completed(&mut events).await;
    tokio::time::sleep(Duration::from_secs(20 * 60)).await;
    assert_eq!(h.svc.supervisor().status().await, AppServerState::Stopped);
    h.svc
        .send(
            &conversation.thread_id,
            &text("pesquise novamente"),
            Default::default(),
        )
        .await
        .unwrap();
    until_turn_completed(&mut events).await;
    assert_eq!(
        sent(&h.record, "thread/start")[0]["config"]["web_search"],
        "disabled"
    );
    let resume = &sent(&h.record, "thread/resume")[0];
    assert_eq!(
        resume["config"]["features.code_mode.direct_only_tool_namespaces"],
        json!(["mcp__aura"])
    );
    assert_eq!(resume["config"]["web_search"], "disabled");
    assert_eq!(
        resume["config"]["mcp_servers.aura.http_headers"]["X-Aura-Conversation"],
        conversation.conversation_uuid
    );
    h.svc.shutdown().await;
}

#[tokio::test]
async fn aura_mcp_starts_direct_even_when_the_caller_removes_the_namespace() {
    let h = default_harness();
    h.svc
        .start(StartOptions {
            model: Some("gpt-6-luna".into()),
            provider: "aura-chatgpt-plan".into(),
            config_overrides: serde_json::Map::from_iter([
                (
                    "features.code_mode.direct_only_tool_namespaces".into(),
                    json!([]),
                ),
                ("features.code_mode.enabled".into(), json!(true)),
            ]),
            ..Default::default()
        })
        .await
        .unwrap();
    let start = &sent(&h.record, "thread/start")[0];
    assert_eq!(
        start["config"]["features.code_mode.direct_only_tool_namespaces"],
        json!(["mcp__aura"])
    );
    assert_eq!(start["config"]["features.code_mode.enabled"], true);
    assert_eq!(start["model"], "gpt-6-luna");
    assert_eq!(start["modelProvider"], "aura-chatgpt-plan");
    h.svc.shutdown().await;
}

#[tokio::test(start_paused = true)]
async fn crash_fails_the_turn_restarts_with_backoff_then_gives_up() {
    let h = harness(
        FakeConfig {
            crash_after_turn_start: true,
            ..Default::default()
        },
        SupervisorConfig::default(),
    );
    let mut rx = h.svc.events();
    let conv = h.svc.start(StartOptions::default()).await.unwrap();
    h.svc
        .send(&conv.thread_id, &text("oi"), TurnOptions::default())
        .await
        .unwrap();
    let events = until_turn_completed(&mut rx).await;
    assert!(matches!(
        events.last(),
        Some(E::TurnCompleted {
            status: TurnStatus::Failed,
            error: Some(TurnError::AppServerCrashed),
            ..
        })
    ));
    // Restart happened after the 1 s backoff.
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert_eq!(h.svc.supervisor().launch_count(), 2);
    assert!(matches!(
        h.svc.supervisor().status().await,
        AppServerState::Ready { .. }
    ));

    for _ in 0..3 {
        let _ = h
            .svc
            .send(&conv.thread_id, &text("oi"), TurnOptions::default())
            .await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    assert!(matches!(
        h.svc.supervisor().status().await,
        AppServerState::Failed { .. }
    ));
    assert!(
        h.svc
            .send(&conv.thread_id, &text("oi"), TurnOptions::default())
            .await
            .is_err()
    );
    h.svc.supervisor().reset().await;
    assert_eq!(h.svc.supervisor().status().await, AppServerState::Stopped);
}

#[tokio::test]
async fn compaction_and_token_usage() {
    let h = default_harness();
    let mut rx = h.svc.events();
    let conv = h.svc.start(StartOptions::default()).await.unwrap();
    h.svc.compact(&conv.thread_id).await.unwrap();
    let mut saw_compacted = false;
    loop {
        match rx.recv().await.unwrap() {
            E::Compacted { .. } => saw_compacted = true,
            E::TokenUsage { used, window, .. } => {
                assert_eq!((used, window), (8000, Some(200000)));
                break;
            }
            _ => {}
        }
    }
    assert!(saw_compacted);
}

#[tokio::test]
async fn codex_model_catalog() {
    let h = default_harness();
    let models = h.svc.codex_models().await.unwrap();
    assert_eq!(models[0].id, "gpt-fake");
    assert!(models[0].is_default);
}

#[tokio::test]
async fn switching_mode_mid_conversation_tells_the_agent() {
    // 014: the developer instructions are fixed at thread start; a mode change
    // is announced in the next turn (sandbox and instruction agree) and never
    // shows in the transcript.
    let h = default_harness();
    let mut rx = h.svc.events();
    let conv = h.svc.start(StartOptions::default()).await.unwrap();
    let note_of = |turn: &Value| -> Option<String> {
        turn["input"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|i| i["text"].as_str())
            .find(|t| t.starts_with("<aura-mode>"))
            .map(str::to_string)
    };
    h.svc
        .send(&conv.thread_id, &text("primeira"), TurnOptions::default())
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;
    assert_eq!(
        note_of(&sent(&h.record, "turn/start")[0]),
        None,
        "same mode as the start"
    );

    h.svc
        .set_mode(
            &conv.thread_id,
            ConversationMode::Task {
                granted: vec![],
                network: false,
            },
        )
        .await
        .unwrap();
    h.svc
        .send(&conv.thread_id, &text("segunda"), TurnOptions::default())
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;
    let turn = &sent(&h.record, "turn/start")[1];
    assert_eq!(turn["sandboxPolicy"]["type"], "workspaceWrite");
    let note = note_of(turn).expect("mode announced");
    assert!(
        note.contains("Modo Tarefa") && note.ends_with("</aura-mode>"),
        "{note}"
    );
    assert_eq!(turn["input"][0]["text"], "segunda", "user text stays first");

    h.svc
        .send(&conv.thread_id, &text("terceira"), TurnOptions::default())
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;
    assert_eq!(
        note_of(&sent(&h.record, "turn/start")[2]),
        None,
        "announced once"
    );

    h.svc.set_language(UiLanguage::En);
    h.svc
        .set_mode(&conv.thread_id, ConversationMode::Plan)
        .await
        .unwrap();
    h.svc
        .send(&conv.thread_id, &text("quarta"), TurnOptions::default())
        .await
        .unwrap();
    until_turn_completed(&mut rx).await;
    let turn = &sent(&h.record, "turn/start")[3];
    assert_eq!(turn["sandboxPolicy"]["type"], "readOnly");
    assert!(note_of(turn).unwrap().contains("Plan mode"));

    let transcript = h.svc.open(&conv.thread_id).await.unwrap();
    let users: Vec<&str> = transcript
        .iter()
        .filter(|m| m.role == "user")
        .map(|m| m.text.as_str())
        .collect();
    assert_eq!(users, ["primeira", "segunda", "terceira", "quarta"]);
}

#[tokio::test]
async fn yolo_accepts_task_permissions_without_asking() {
    // 018 AC-002, AC-003.
    let h = default_harness();
    h.svc.set_yolo(true);
    let mut rx = h.svc.events();
    let task = h
        .svc
        .start(StartOptions {
            mode: ConversationMode::Task {
                granted: vec![],
                network: false,
            },
            ..Default::default()
        })
        .await
        .unwrap();
    h.svc
        .send(
            &task.thread_id,
            &text("/aprovar rode echo"),
            TurnOptions::default(),
        )
        .await
        .unwrap();
    let events = until_turn_completed(&mut rx).await;
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, E::ApprovalRequested { .. })),
        "YOLO never shows the request"
    );
    assert!(events.iter().any(
        |e| matches!(e, E::MessageCompleted { text, .. } if text.contains("Decisão recebida: accept."))
    ));
    assert!(h.svc.pending_request_ids().is_empty());
    let start = &sent(&h.record, "thread/start")[0];
    assert_eq!(
        (start["sandbox"].as_str(), start["approvalPolicy"].as_str()),
        (Some("danger-full-access"), Some("never"))
    );
    let turn = &sent(&h.record, "turn/start")[0];
    assert_eq!(turn["sandboxPolicy"]["type"], "dangerFullAccess");
    assert_eq!(turn["approvalPolicy"], "never");

    // Chat stays read-only and the user is still asked.
    let chat = h.svc.start(StartOptions::default()).await.unwrap();
    h.svc
        .send(
            &chat.thread_id,
            &text("/aprovar rode echo"),
            TurnOptions::default(),
        )
        .await
        .unwrap();
    let request_id = loop {
        if let E::ApprovalRequested { request_id, .. } = rx.recv().await.unwrap() {
            break request_id;
        }
    };
    assert_eq!(h.svc.pending_request_ids(), vec![request_id.clone()]);
    h.svc.respond(&request_id, Decision::Decline).await.unwrap();
    until_turn_completed(&mut rx).await;
    assert_eq!(
        sent(&h.record, "turn/start")[1]["sandboxPolicy"]["type"],
        "readOnly"
    );

    // YOLO off again: the next Task turn asks.
    h.svc.set_yolo(false);
    h.svc
        .send(
            &task.thread_id,
            &text("/aprovar rode echo"),
            TurnOptions::default(),
        )
        .await
        .unwrap();
    let request_id = loop {
        if let E::ApprovalRequested { request_id, .. } = rx.recv().await.unwrap() {
            break request_id;
        }
    };
    assert_eq!(
        sent(&h.record, "turn/start")[2]["sandboxPolicy"]["type"],
        "workspaceWrite"
    );
    h.svc.respond(&request_id, Decision::Decline).await.unwrap();
    until_turn_completed(&mut rx).await;
}
