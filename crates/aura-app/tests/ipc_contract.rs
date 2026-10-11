//! IPC contract: serializes one value of every shape the UI receives or sends
//! and compares it with `apps/desktop/src/ipc/__fixtures__/contract.json`,
//! which the TypeScript tests also load. Regenerate after an intentional
//! change with `UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract`.

use aura_app::events::{ConsentRequest, HostEvent, PrivacyState};
use aura_asr::ptt::PttState;
use aura_auth::{ChatGptAccount, LoginProgress};
use aura_codex::approvals::Decision;
use aura_codex::events::{
    AppServerState, ApprovalKind, ConversationEvent as E, FileChangeSummary, ItemStatus, PlanStep,
    ToolKind, TurnError, TurnStatus,
};
use aura_codex::modes::ConversationMode;
use aura_core::context::{ChipKind, ChipPayload, ContextChip};
use aura_core::settings::Settings;
use aura_extensions::mcp_config::{ApprovalMode, EnvValue, McpServerSpec, Transport};
use aura_policy::{CaptureMode, Policy};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/desktop/src/ipc/__fixtures__/contract.json")
}

fn account() -> ChatGptAccount {
    serde_json::from_value(json!({
        "clientId": "app_123", "subject": "user-1", "email": "ana@example.com", "scopes": ["openid", "chatgpt.tokens.use.direct"],
        "planUsageEnabled": true, "expiresAt": 1_900_000_000, "active": true, "welcomed": false, "signedIn": true
    }))
    .expect("account shape")
}

fn build() -> Value {
    let attachment_chip: ContextChip = serde_json::from_value(json!({
        "id": "file_1", "kind": "file", "label": "1 linhas.txt · 1 linhas",
        "attachmentLabel": {"fileName": "1 linhas.txt", "parts": [{"type": "count", "amount": 1, "unit": "line"}]},
        "previewPath": null, "payload": {"type": "text", "text": "literal"}, "tokenEstimate": 2, "blockedReason": null
    })).expect("attachment label contract");
    let t = "thr_1".to_string();
    let conversation = vec![
        E::TurnStarted {
            thread_id: t.clone(),
            turn_id: "turn_1".into(),
        },
        E::MessageDelta {
            thread_id: t.clone(),
            item_id: "msg_1".into(),
            delta: "Olá".into(),
        },
        E::MessageDelta {
            thread_id: t.clone(),
            item_id: "msg_1".into(),
            delta: ", mundo".into(),
        },
        E::ReasoningDelta {
            thread_id: t.clone(),
            item_id: "rs_1".into(),
            delta: "pensando".into(),
        },
        E::ToolCall {
            thread_id: t.clone(),
            item_id: "tool_1".into(),
            kind: ToolKind::Mcp,
            title: "aura.screen_text".into(),
            status: ItemStatus::InProgress,
            detail: None,
        },
        E::ToolCall {
            thread_id: t.clone(),
            item_id: "tool_1".into(),
            kind: ToolKind::Mcp,
            title: "aura.screen_text".into(),
            status: ItemStatus::Completed,
            detail: Some("120 ms".into()),
        },
        E::ApprovalRequested {
            thread_id: t.clone(),
            request_id: "req_1".into(),
            kind: ApprovalKind::Command,
            command: Some("pip install x".into()),
            cwd: Some("C:/Users/ana".into()),
            reason: Some("instalar dependência".into()),
            changes: vec![],
            options: vec!["accept".into(), "acceptForSession".into(), "decline".into()],
        },
        E::RequestResolved {
            request_id: "req_1".into(),
        },
        E::PlanUpdated {
            thread_id: t.clone(),
            turn_id: "turn_1".into(),
            explanation: None,
            steps: vec![
                PlanStep {
                    step: "Ler o arquivo".into(),
                    status: "completed".into(),
                },
                PlanStep {
                    step: "Responder".into(),
                    status: "inProgress".into(),
                },
            ],
        },
        E::FileChanges {
            thread_id: t.clone(),
            item_id: "fc_1".into(),
            changes: vec![FileChangeSummary {
                path: "notas.md".into(),
                added: 3,
                removed: 1,
                diff: "@@ -1 +1,3 @@".into(),
            }],
            status: ItemStatus::Completed,
        },
        E::TokenUsage {
            thread_id: t.clone(),
            used: 1234,
            window: Some(200_000),
        },
        E::MessageCompleted {
            thread_id: t.clone(),
            item_id: "msg_1".into(),
            text: "Olá, mundo".into(),
        },
        E::TurnCompleted {
            thread_id: t.clone(),
            turn_id: "turn_1".into(),
            status: TurnStatus::Completed,
            error: None,
        },
        E::TurnCompleted {
            thread_id: t.clone(),
            turn_id: "turn_2".into(),
            status: TurnStatus::Failed,
            error: Some(TurnError::UsageLimit {
                retry_after_secs: Some(60),
            }),
        },
        E::AppServerState {
            state: AppServerState::Ready {
                version: "0.159.0".into(),
            },
        },
    ];
    let events: Vec<HostEvent> = conversation
        .into_iter()
        .map(HostEvent::Conversation)
        .chain([
            HostEvent::Login(LoginProgress::WaitingBrowser {
                authorize_url: "https://auth.openai.com/oauth/authorize?x=1".into(),
            }),
            HostEvent::Login(LoginProgress::Completed {
                account: account(),
                first_time: true,
            }),
            HostEvent::Consent(ConsentRequest {
                id: "c1".into(),
                conversation: "uuid-1".into(),
                tool: "screen_capture".into(),
                source: "screen".into(),
                reason: "ver o erro".into(),
                app: Some("Code.exe".into()),
            }),
            HostEvent::ConsentResolved { id: "c1".into() },
            HostEvent::Privacy(PrivacyState {
                paused: true,
                screen: "onDemand".into(),
                mic: "onDemand".into(),
                system_audio: "off".into(),
                recording: vec!["screen".into()],
            }),
            HostEvent::Download {
                id: "codex".into(),
                bytes: 10,
                total: Some(100),
                done: false,
                error: None,
            },
            HostEvent::Voice(PttState::Listening),
            HostEvent::ProvidersChanged {},
            HostEvent::WebSource {
                thread_id: "thr_1".into(),
                turn_id: "turn-1".into(),
                source: aura_web::WebSource {
                    source_id: "W1".into(),
                    title: "Relatório público".into(),
                    url: "https://news.example/report".into(),
                    snippet: "Produção: 42 unidades.".into(),
                    published_at: None,
                    retrieved_at: "2026-10-10T15:00:00Z".into(),
                    kind: "pageContent".into(),
                },
            },
            HostEvent::AudioLevel {
                source: aura_audio::AudioSourceKind::Mic,
                dbfs: -12.5,
            },
            HostEvent::Voice(PttState::Partial { text: "tex".into() }),
            HostEvent::Voice(PttState::Done {
                text: "texto".into(),
            }),
            HostEvent::Notice {
                level: "warning".into(),
                message: "atalho em uso".into(),
            },
            HostEvent::OpenConversation {
                thread_id: "t1".into(),
            },
            HostEvent::AgentTask {
                text: "Crie uma Skill".into(),
                mode: "task".into(),
            },
            HostEvent::ExtensionsChanged {},
            HostEvent::Meeting {
                id: "m1".into(),
                status: "active".into(),
            },
            HostEvent::Reminder {
                id: "r1".into(),
                text: "Ligar para o João".into(),
            },
        ])
        .collect();

    let mut chip = ContextChip::new(
        ChipKind::Screen,
        "Tela · Chrome",
        ChipPayload::Image {
            path: "C:/tmp/a.png".into(),
        },
    );
    chip.id = "chip_1".into();
    chip.preview_path = Some("C:/tmp/a.png".into());
    let mut env = BTreeMap::new();
    env.insert("GITHUB_TOKEN".to_string(), EnvValue::Secret);
    env.insert("LOG".to_string(), EnvValue::Plain { value: "1".into() });
    let mcp = McpServerSpec {
        name: "github".into(),
        transport: Transport::Stdio {
            command: "npx".into(),
            args: vec!["-y".into()],
            env,
            cwd: None,
        },
        enabled: true,
        disabled_tools: vec![],
        approval_mode: ApprovalMode::AskForWrites,
        startup_timeout_sec: None,
        tool_timeout_sec: None,
    };
    let http = McpServerSpec {
        name: "docs".into(),
        transport: Transport::Http {
            url: "https://example.com/mcp".into(),
            bearer_secret: true,
            headers: BTreeMap::new(),
        },
        ..mcp.clone()
    };
    let policy = Policy::default();

    json!({
        "events": events,
        "settings": Settings::default(),
        "privacy": {"screen": policy.screen, "mic": policy.mic, "systemAudio": policy.system_audio, "paused": false, "exclusions": &policy.exclusions[..2]},
        "captureModes": [CaptureMode::Off, CaptureMode::OnDemand, CaptureMode::RecentBuffer { minutes: 10 }, CaptureMode::Manual, CaptureMode::Continuous],
        "meeting": aura_app::meeting::Meeting {
            id: "m1".into(),
            title: "Revisão Q3".into(),
            kind: "decision".into(),
            briefing: "Fechar o orçamento".into(),
            origin: "live".into(),
            status: "ended".into(),
            started_at: 1_000_000,
            ended_at: Some(2_000_000),
            project_id: Some("p1".into()),
        },
        "utterance": aura_app::meeting::Utterance {
            id: 1,
            meeting_id: "m1".into(),
            t0: 5_000,
            t1: 6_500,
            speaker: "them".into(),
            text: "Bom dia".into(),
        },
        "meetingHit": aura_app::meeting::Hit {
            meeting_id: "m1".into(),
            title: "Revisão Q3".into(),
            started_at: 1_000_000,
            t0: 5_000,
            speaker: "you".into(),
            text: "Sim, com corte de 10%".into(),
        },
        "speechStats": aura_app::speech_stats::compute(&[]),
        "transcript": [
            aura_app::web_history::TranscriptMessage { role: "user".into(), text: "Verifique a produção".into(), sources: vec![] },
            aura_app::web_history::TranscriptMessage { role: "assistant".into(), text: "Produção: 42 [[aura-source:W1]]".into(), sources: vec![aura_web::WebSource {
                source_id: "W1".into(), title: "Relatório público".into(), url: "https://news.example/report".into(), snippet: String::new(),
                published_at: None, retrieved_at: "2026-10-10T15:00:00Z".into(), kind: "pageContent".into(),
            }] },
            aura_app::web_history::TranscriptMessage { role: "assistant".into(), text: "Resposta antiga".into(), sources: vec![] },
        ],
        "recipe": aura_app::recipes::builtins().into_iter().next(),
        "chip": chip,
        "attachmentChip": attachment_chip,
        "mcpServers": [mcp, http],
        "decisions": [Decision::Accept, Decision::AcceptForSession, Decision::Decline, Decision::Cancel, Decision::Answer { content: json!({"a": 1}) }],
        "modes": [ConversationMode::Chat, ConversationMode::Task { granted: vec!["C:/proj".into()], network: false }, ConversationMode::Plan],
        "preset": aura_gateway::registry::presets().into_iter().find(|p| p.id == "groq"),
        "turnErrors": [TurnError::PlanUsageLimit, TurnError::UnsupportedCapability { param: Some("tools".into()) }, TurnError::Other { code: "x".into(), message: "y".into() }],
    })
}

#[test]
fn ipc_shapes_match_the_golden_file() {
    let value = build();
    let path = fixture_path();
    let pretty = serde_json::to_string_pretty(&value).unwrap() + "\n";
    if std::env::var_os("UPDATE_GOLDEN").is_some() || !path.exists() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &pretty).unwrap();
        return;
    }
    let golden = std::fs::read_to_string(&path).unwrap();
    if pretty != golden {
        let diff: Vec<String> = pretty
            .lines()
            .zip(golden.lines())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .take(8)
            .map(|(i, (a, b))| {
                format!("line {}: now `{}` / golden `{}`", i + 1, a.trim(), b.trim())
            })
            .collect();
        panic!(
            "IPC shapes changed ({} vs {} lines): update src/ipc/types.ts and rerun with UPDATE_GOLDEN=1\n{}",
            pretty.lines().count(),
            golden.lines().count(),
            diff.join("\n")
        );
    }
}

#[test]
fn inputs_from_the_ui_deserialize_with_defaults() {
    // The UI sends partial objects; these must parse.
    let o: aura_codex::service::StartOptions =
        serde_json::from_value(json!({"mode": {"mode": "chat"}, "ephemeral": true})).unwrap();
    assert!(o.ephemeral);
    let q: aura_codex::service::HistoryQuery =
        serde_json::from_value(json!({"search": "x"})).unwrap();
    assert!(!q.archived);
    let r: aura_app::host::SendRequest =
        serde_json::from_value(json!({"threadId": "t", "text": "oi", "tray": "t"})).unwrap();
    assert!(r.accepts_images);
    let d: Decision = serde_json::from_value(json!({"type": "acceptForSession"})).unwrap();
    assert_eq!(d, Decision::AcceptForSession);
    let p: aura_core::settings::SettingsPatch =
        serde_json::from_value(json!({"opacity": 0.9, "defaultModel": null})).unwrap();
    assert_eq!(p.opacity, Some(0.9));
    let c: aura_app::consent::ConsentAnswer =
        serde_json::from_value(json!("conversation")).unwrap();
    assert_eq!(c, aura_app::consent::ConsentAnswer::Conversation);
}
