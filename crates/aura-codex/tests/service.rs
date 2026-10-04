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
