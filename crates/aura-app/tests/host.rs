//! End-to-end host tests: fake OS platform + in-process fake app-server, real
//! store, gateway, MCP tools, policy, attachments and extensions.

use aura_app::consent::ConsentAnswer;
use aura_app::events::HostEvent;
use aura_app::host::SendRequest;
use aura_app::paths::AppPaths;
use aura_app::platform::{FakeForeground, Platform};
use aura_app::tools::{HostTools, NoRecentMedia};
use aura_app::{Host, HostConfig};
use aura_codex::events::ConversationEvent;
use aura_codex::service::StartOptions;
use aura_core::context::{ChipKind, ChipPayload};
use aura_mcp::{CallContext, Content, ToolHandler};
use aura_policy::{AgentPermission, CaptureMode, Source};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

struct Env {
    host: Arc<Host>,
    fg: Arc<FakeForeground>,
    platform: Platform,
    _dir: tempfile::TempDir,
}

async fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let (platform, fg) = Platform::fake();
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform.clone());
    cfg.in_memory_store = true;
    let host = Host::start(cfg).await.unwrap();
    Env {
        host,
        fg,
        platform,
        _dir: dir,
    }
}

async fn until<F: Fn(&HostEvent) -> bool>(
    rx: &mut tokio::sync::broadcast::Receiver<HostEvent>,
    f: F,
) -> HostEvent {
    loop {
        let e = tokio::time::timeout(Duration::from_secs(20), rx.recv())
            .await
            .expect("event in time")
            .expect("open");
        if f(&e) {
            return e;
        }
    }
}

#[tokio::test]
async fn screen_chip_and_turn_round_trip() {
    let e = env().await;
    let mut rx = e.host.subscribe();
    let conv = e
        .host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    let chip = e.host.capture_screen(&conv.thread_id, false).await.unwrap();
    assert_eq!(chip.kind, ChipKind::Screen);
    assert!(chip.blocked_reason.is_none());
    let ChipPayload::Image { path } = &chip.payload else {
        panic!("image chip")
    };
    assert!(path.exists());
    assert_eq!(e.host.tray(&conv.thread_id).len(), 1);

    e.host
        .send(SendRequest {
            thread_id: conv.thread_id.clone(),
            text: "o que tem na tela?".into(),
            tray: conv.thread_id.clone(),
            accepts_images: true,
            options: Default::default(),
        })
        .await
        .unwrap();
    until(&mut rx, |e| {
        matches!(
            e,
            HostEvent::Conversation(ConversationEvent::TurnCompleted { .. })
        )
    })
    .await;
    // The tray was consumed by the turn.
    assert!(e.host.tray(&conv.thread_id).is_empty());

    let page = e.host.history(Default::default()).await.unwrap();
    assert_eq!(page.items.len(), 1);
    e.host.delete_conversation(&conv.thread_id).await.unwrap();
    e.host.shutdown().await;
}

#[tokio::test]
async fn paused_privacy_blocks_user_capture() {
    let e = env().await;
    e.host.set_paused(true).unwrap();
    let chip = e.host.capture_screen("draft", false).await.unwrap();
    assert_eq!(chip.blocked_reason.as_deref(), Some("paused"));
    // Paused state survives through the repository.
    assert!(e.host.privacy().paused);
    e.host.toggle_pause().unwrap();
    assert!(!e.host.privacy().paused);
}

// Tauri runs sync commands (and tray/hotkey handlers) on threads without a Tokio
// context; the host must not rely on the caller's runtime there.
#[tokio::test]
async fn privacy_commands_work_outside_the_runtime() {
    let e = env().await;
    let mut rx = e.host.subscribe();
    let host = e.host.clone();
    std::thread::spawn(move || host.set_paused(true).map(|v| v.paused))
        .join()
        .expect("set_paused must not panic off the runtime")
        .unwrap();
    until(
        &mut rx,
        |ev| matches!(ev, HostEvent::Privacy(s) if s.paused),
    )
    .await;
}

struct NoSpace;
impl aura_asr::download::DiskSpace for NoSpace {
    fn free_bytes(&self, _: &std::path::Path) -> Option<u64> {
        Some(0)
    }
}

#[tokio::test]
async fn voice_install_works_outside_the_runtime() {
    let dir = tempfile::tempdir().unwrap();
    let (platform, _fg) = Platform::fake();
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
    cfg.in_memory_store = true;
    // No free space: the download ends with an error before any network access.
    cfg.disk = Arc::new(NoSpace);
    let host = Host::start(cfg).await.unwrap();
    let mut rx = host.subscribe();
    let id = host.voice_models()[0].entry.id.clone();
    let h = host.clone();
    let model = id.clone();
    std::thread::spawn(move || h.install_voice_model(&model))
        .join()
        .expect("install must not panic off the runtime")
        .unwrap();
    until(
        &mut rx,
        |ev| matches!(ev, HostEvent::Download { id: got, error: Some(_), .. } if *got == id),
    )
    .await;
}

fn tools_of(
    e: &Env,
    host_events: tokio::sync::broadcast::Sender<HostEvent>,
    consent: Arc<aura_app::consent::ConsentBroker>,
) -> HostTools {
    let store = aura_store::Store::open_in_memory().unwrap();
    let privacy = aura_app::privacy::PrivacyRepo::new(store);
    let policy = privacy.load().unwrap();
    HostTools {
        platform: e.platform.clone(),
        policy: Arc::new(std::sync::RwLock::new(policy)),
        grants: Arc::new(std::sync::RwLock::new(Default::default())),
        privacy,
        consent,
        events: host_events,
        attachments: Arc::new(aura_app::attachments::AttachmentService::new(
            Arc::new(aura_ingest::NoHeavy),
            e._dir.path().join("cache"),
        )),
        recent: Arc::new(NoRecentMedia),
        ocr_language: "pt-BR".into(),
    }
}

#[tokio::test]
async fn agent_capture_asks_the_user_and_respects_the_answer() {
    let e = env().await;
    let (tx, mut rx) = tokio::sync::broadcast::channel(64);
    let consent = Arc::new(aura_app::consent::ConsentBroker::default());
    let tools = Arc::new(tools_of(&e, tx, consent.clone()));
    let ctx = CallContext {
        conversation: "conv-1".into(),
    };

    // Default policy: screen agent permission = Ask.
    let t = tools.clone();
    let c = ctx.clone();
    let call = tokio::spawn(async move {
        t.call("screen_capture", json!({"reason": "ver o erro"}), c)
            .await
    });
    let HostEvent::Consent(req) = until(&mut rx, |e| matches!(e, HostEvent::Consent(_))).await
    else {
        unreachable!()
    };
    assert_eq!(req.tool, "screen_capture");
    assert_eq!(req.reason, "ver o erro");
    assert!(consent.answer(&req.id, ConsentAnswer::Conversation));
    let out = call.await.unwrap();
    assert!(!out.is_error, "{out:?}");
    assert!(
        out.content
            .iter()
            .any(|c| matches!(c, Content::Image { mime, .. } if mime == "image/png"))
    );

    // "Nesta Conversa" → the next call does not ask again.
    let out = tools
        .call("screen_capture", json!({"reason": "de novo"}), ctx.clone())
        .await;
    assert!(!out.is_error);

    // Another conversation still asks; denying returns a structured error.
    let t = tools.clone();
    let call = tokio::spawn(async move {
        t.call(
            "screen_text",
            json!({}),
            CallContext {
                conversation: "conv-2".into(),
            },
        )
        .await
    });
    let HostEvent::Consent(req) = until(&mut rx, |e| matches!(e, HostEvent::Consent(_))).await
    else {
        unreachable!()
    };
    consent.answer(&req.id, ConsentAnswer::Deny);
    let out = call.await.unwrap();
    assert!(out.is_error);
    let Content::Text(t) = &out.content[0] else {
        panic!()
    };
    assert!(t.contains("\"denied\""), "{t}");

    // Pausing denies without asking.
    tools.policy.write().unwrap().paused = true;
    let out = tools
        .call("active_window_info", json!({}), ctx.clone())
        .await;
    assert!(out.is_error);
    tools.policy.write().unwrap().paused = false;
    let out = tools
        .call("active_window_info", json!({}), ctx.clone())
        .await;
    let Content::Text(t) = &out.content[0] else {
        panic!()
    };
    assert!(t.contains("Code.exe"), "{t}");

    // Recent buffers are reported as unavailable (not recording).
    let out = tools
        .call("screen_recent", json!({"minutes": 2, "reason": "x"}), ctx)
        .await;
    assert!(out.is_error);
}

#[tokio::test]
async fn always_permission_and_excluded_window() {
    let e = env().await;
    let (tx, _rx) = tokio::sync::broadcast::channel(64);
    let tools = tools_of(&e, tx, Arc::new(Default::default()));
    {
        let mut p = tools.policy.write().unwrap();
        p.screen.agent = AgentPermission::Always;
    }
    let ctx = CallContext {
        conversation: "c".into(),
    };
    let out = tools
        .call(
            "screen_text",
            json!({"source": "uia", "max_chars": 200}),
            ctx.clone(),
        )
        .await;
    let Content::Text(t) = &out.content[0] else {
        panic!()
    };
    assert!(
        t.starts_with("[uia]") && t.contains("mismatched types"),
        "{t}"
    );

    // Exclude the fake editor → window-scoped tools are refused.
    tools
        .policy
        .write()
        .unwrap()
        .exclusions
        .push(aura_policy::ExclusionRule {
            id: "code".into(),
            process: Some("code.exe".into()),
            title_glob: None,
            class: None,
            enabled: true,
            builtin: false,
        });
    let out = tools
        .call(
            "screen_capture",
            json!({"target": "window", "reason": "x"}),
            ctx,
        )
        .await;
    assert!(out.is_error);
    let Content::Text(t) = &out.content[0] else {
        panic!()
    };
    assert!(t.contains("excluded"), "{t}");
}

#[tokio::test]
async fn attachments_quick_commands_and_selection() {
    let e = env().await;
    let csv = e._dir.path().join("vendas.csv");
    std::fs::write(&csv, "produto;valor\ncafé;10,5\npão;3\n").unwrap();
    let (info, chip) = e.host.attach("draft", None, &csv).await.unwrap();
    assert_eq!(info.kind, "spreadsheet");
    assert_eq!(chip.kind, ChipKind::File);
    assert!(chip.label.contains("vendas.csv"));

    *e.fg.selection.lock().unwrap() = Some("olá".into());
    let sel = e.host.capture_selection("draft").unwrap().unwrap();
    assert_eq!(sel.kind, ChipKind::Selection);
    let exp = e
        .host
        .expand_quick_command("draft", "/traduzir inglês", "")
        .await
        .unwrap();
    assert_eq!(exp.prompt_text, "Traduza para inglês:\n\nolá");
    // The selection chip was consumed into the prompt; the file chip stays.
    let tray = e.host.tray("draft");
    assert_eq!(tray.len(), 1);
    assert_eq!(tray[0].kind, ChipKind::File);

    let cmds = e
        .host
        .save_quick_command("email-formal", "Formal: {selecao}", false)
        .unwrap();
    assert!(cmds.iter().any(|c| c.name == "email-formal" && !c.builtin));

    assert!(e.host.insert_into_app("texto ditado"));
    assert_eq!(
        e.fg.pasted.lock().unwrap().as_slice(),
        ["texto ditado".to_string()]
    );
}

#[tokio::test]
async fn providers_privacy_and_diagnostics() {
    let e = env().await;
    let draft: aura_gateway::registry::ProviderDraft =
        serde_json::from_value(json!({"name": "Groq", "preset": "groq"})).unwrap();
    let p = e
        .host
        .save_provider(draft, Some("gsk_test_key_1234".into()))
        .unwrap();
    assert_eq!(e.host.providers().unwrap().len(), 1);
    assert!(p.credential_hint.as_deref().unwrap_or("").ends_with("1234"));
    // The key lives in the vault, never in the provider row.
    assert!(!serde_json::to_string(&p).unwrap().contains("gsk_test_key"));
    let d = e.host.diagnostics().await;
    assert_eq!(d.providers, 1);
    assert!(d.gateway_port > 0);
    e.host.remove_provider(&p.id).unwrap();
    assert!(e.host.providers().unwrap().is_empty());

    let view = e
        .host
        .set_source_policy(Source::Mic, CaptureMode::Off, AgentPermission::Never)
        .unwrap();
    assert_eq!(view.mic.mode, CaptureMode::Off);
    let before = view.exclusions.len();
    let view = e
        .host
        .upsert_exclusion(aura_policy::ExclusionRule {
            id: String::new(),
            process: None,
            title_glob: Some("*Banco*".into()),
            class: None,
            enabled: true,
            builtin: false,
        })
        .unwrap();
    assert_eq!(view.exclusions.len(), before + 1);
    assert!(e.host.remove_exclusion("keepass").is_err());

    let s = e
        .host
        .update_settings(serde_json::from_value(json!({"opacity": 0.8})).unwrap())
        .unwrap();
    assert_eq!(s.opacity, 0.8);
    assert!(
        e.host
            .update_settings(serde_json::from_value(json!({"opacity": 0.2})).unwrap())
            .is_err()
    );
}

#[tokio::test]
async fn skills_and_mcp_servers() {
    let e = env().await;
    let path = e
        .host
        .create_skill("revisar-contrato", "Revisa contratos", "Leia com atenção.")
        .unwrap();
    assert!(path.join("SKILL.md").exists());
    assert_eq!(e.host.skills().len(), 1);
    e.host.delete_skill("revisar-contrato").unwrap();
    assert!(e.host.skills().is_empty());

    let spec: aura_extensions::mcp_config::McpServerSpec = serde_json::from_value(json!({
        "name": "github",
        "transport": {"type": "stdio", "command": "npx", "args": ["-y", "@modelcontextprotocol/server-github"],
                      "env": {"GITHUB_TOKEN": {"kind": "secret"}}}
    }))
    .unwrap();
    let list = e
        .host
        .save_mcp_server(spec, vec![("GITHUB_TOKEN".into(), "ghp_1".into())], None)
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(e.host.diagnostics().await.mcp_servers, 1);
    e.host.delete_mcp_server("github").unwrap();
    assert!(e.host.mcp_servers().unwrap().is_empty());
}

#[tokio::test]
async fn push_to_talk_dictation() {
    let e = env().await;
    let mut rx = e.host.subscribe();
    assert!(!e.host.voice_models().is_empty());
    e.host.ptt_press(None).await.unwrap();
    until(&mut rx, |e| {
        matches!(e, HostEvent::Voice(aura_asr::ptt::PttState::Listening))
    })
    .await;
    // The synthetic microphone produces a tone (speech-like energy).
    tokio::time::sleep(Duration::from_millis(900)).await;
    let state = e.host.ptt_release(Some("pt".into()), vec![]).await;
    assert_eq!(
        state,
        aura_asr::ptt::PttState::Done {
            text: "texto ditado de exemplo".into()
        }
    );

    e.host
        .set_source_policy(Source::Mic, CaptureMode::Off, AgentPermission::Never)
        .unwrap();
    assert_eq!(e.host.ptt_press(None).await.unwrap_err().code, "mic_off");
}

#[tokio::test]
async fn wav_attachment_is_transcribed_and_readable_by_time() {
    let e = env().await;
    let wav = e._dir.path().join("reuniao.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&wav, spec).unwrap();
    for i in 0..32_000 {
        w.write_sample(((i as f32 * 0.05).sin() * 8000.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();

    let conv = e
        .host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    let (info, chip) = e
        .host
        .attach(&conv.thread_id, Some(&conv.thread_id), &wav)
        .await
        .unwrap();
    assert_eq!(info.kind, "audio");
    assert!(chip.label.contains("transcrito"), "{}", chip.label);

    let text = e
        .host
        .read_attachment(
            &conv.thread_id,
            &info.id,
            aura_ingest::Selector {
                time: Some("00:00-00:01".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(
        text.starts_with("[00:00] texto ditado de exemplo"),
        "{text}"
    );
    let none = e
        .host
        .read_attachment(
            &conv.thread_id,
            &info.id,
            aura_ingest::Selector {
                time: Some("10:00-11:00".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(none.is_empty());
    let listed = e.host.attachments(&conv.thread_id);
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, info.id);
}

#[tokio::test]
async fn recent_buffers_feed_the_agent_tools() {
    use aura_app::tools::RecentMedia;
    let dir = tempfile::tempdir().unwrap();
    let (platform, _fg) = Platform::fake();
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
    cfg.in_memory_store = true;
    cfg.capture_interval = Duration::from_millis(20);
    let host = Host::start(cfg).await.unwrap();

    // Screen: recent buffer → encrypted segments → sampled keyframes.
    host.set_source_policy(
        Source::Screen,
        CaptureMode::RecentBuffer { minutes: 1 },
        AgentPermission::Always,
    )
    .unwrap();
    // Microphone: recent buffer; turning it off flushes the open segment.
    host.set_source_policy(
        Source::Mic,
        CaptureMode::RecentBuffer { minutes: 1 },
        AgentPermission::Always,
    )
    .unwrap();
    tokio::time::sleep(Duration::from_millis(900)).await;
    let capture = host.capture();
    assert!(capture.active().await.contains(&"screen".to_string()));
    let frames = capture.screen_frames(1.0, 4).await.unwrap();
    assert_eq!(frames.len(), 4);
    assert!(
        frames
            .iter()
            .all(|(label, png)| label.starts_with("t−") && png.starts_with(b"\x89PNG"))
    );
    // Segments are sealed on disk: no plaintext JSON frame list.
    fn files(d: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(d).unwrap().flatten() {
            if e.path().is_dir() {
                files(&e.path(), out)
            } else {
                out.push(e.path())
            }
        }
    }
    let mut segs = Vec::new();
    files(
        &dir.path().join("Aura").join("captures").join("screen"),
        &mut segs,
    );
    assert!(!segs.is_empty());
    assert!(
        segs.iter()
            .all(|p| !std::fs::read(p).unwrap().starts_with(b"["))
    );

    host.set_source_policy(Source::Mic, CaptureMode::OnDemand, AgentPermission::Always)
        .unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let transcript = capture.audio_transcript(1.0, "mic").await.unwrap();
    assert!(
        transcript.contains("Você: texto ditado de exemplo"),
        "{transcript}"
    );

    // Pause stops every recorder.
    host.set_paused(true).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(capture.active().await.is_empty());
    host.shutdown().await;
}

#[tokio::test]
async fn manual_recording_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let (platform, _fg) = Platform::fake();
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
    cfg.in_memory_store = true;
    cfg.capture_interval = Duration::from_millis(20);
    let host = Host::start(cfg).await.unwrap();
    assert_eq!(
        host.recording_start("x").await.unwrap_err().code,
        "recording"
    );
    host.set_source_policy(Source::Mic, CaptureMode::Manual, AgentPermission::Ask)
        .unwrap();
    host.set_source_policy(Source::Screen, CaptureMode::Manual, AgentPermission::Ask)
        .unwrap();
    let id = host.recording_start("Reunião").await.unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    assert_eq!(host.recording_stop().await.as_deref(), Some(id.as_str()));
    tokio::time::sleep(Duration::from_millis(200)).await;
    let recs = host.recordings();
    assert_eq!(recs.len(), 1);
    assert!(recs[0].ended_at.is_some());
    assert!(recs[0].bytes > 0);
    let out = host
        .recording_export(&id, &dir.path().join("export"))
        .unwrap();
    assert!(
        out.iter()
            .any(|p| p.extension().is_some_and(|e| e == "wav")),
        "{out:?}"
    );
    assert!(
        out.iter()
            .any(|p| p.to_string_lossy().contains("screen-0001")),
        "{out:?}"
    );
    host.recording_delete(&id).unwrap();
    assert!(host.recordings().is_empty());
    host.shutdown().await;
}

#[tokio::test]
async fn region_selection_crops_the_frozen_screen() {
    let e = env().await;
    let frozen = e.host.region_begin().await.unwrap();
    assert!(frozen.path.exists());
    assert_eq!((frozen.width, frozen.height), (320, 200));
    let chip = e
        .host
        .region_commit(
            &frozen.token,
            aura_core::placement::Rect::new(10, 10, 100, 50),
            "draft",
        )
        .unwrap();
    assert_eq!(chip.kind, ChipKind::Region);
    assert_eq!(chip.label, "Região 100×50");
    assert!(!frozen.path.exists(), "frozen screen is deleted after use");
    // The token is single-use.
    assert!(
        e.host
            .region_commit(
                &frozen.token,
                aura_core::placement::Rect::new(0, 0, 10, 10),
                "draft"
            )
            .is_err()
    );

    e.host.set_paused(true).unwrap();
    assert_eq!(e.host.region_begin().await.unwrap_err().code, "paused");
}

#[tokio::test]
async fn mcp_status_and_workspace_files() {
    let e = env().await;
    let conv = e
        .host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    let status = e.host.mcp_status().await.unwrap();
    assert_eq!(status[0].name, "aura");
    assert_eq!(
        status[0].tools,
        vec!["screen_capture".to_string(), "screen_text".to_string()]
    );
    assert_eq!(
        e.host.mcp_login("docs").await.unwrap().as_deref(),
        Some("https://example.com/oauth/authorize")
    );

    std::fs::create_dir_all(conv.workspace.join("out")).unwrap();
    std::fs::write(conv.workspace.join("out").join("relatorio.md"), "# Oi").unwrap();
    std::fs::create_dir_all(conv.workspace.join("attachments")).unwrap();
    std::fs::write(conv.workspace.join("attachments").join("x.pdf"), "x").unwrap();
    let files = e.host.workspace_files(&conv.thread_id);
    assert_eq!(
        files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(),
        vec!["out/relatorio.md"]
    );
    assert_eq!(files[0].bytes, 4);
    assert_eq!(
        e.host
            .read_workspace_file(&conv.thread_id, "out/relatorio.md")
            .unwrap(),
        "# Oi"
    );
    assert!(
        e.host
            .read_workspace_file(&conv.thread_id, "../../../etc/passwd")
            .is_err()
    );
}

#[tokio::test]
async fn app_profile_applies_to_new_conversations() {
    let e = env().await;
    assert!(e.host.active_profile().is_none());
    let p = e
        .host
        .save_profile(aura_app::profiles::AppProfile {
            id: String::new(),
            name: "VS Code".into(),
            process_pattern: "code.exe".into(),
            title_glob: None,
            instructions: "Responda com código TypeScript".into(),
            attach_screen: true,
            default_mode: Some("task".into()),
            default_model: None,
        })
        .unwrap();
    // The fake foreground app is Code.exe.
    assert_eq!(e.host.active_profile().unwrap().id, p.id);
    let conv = e
        .host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    assert!(!conv.thread_id.is_empty());
    e.host.delete_profile(&p.id).unwrap();
    assert!(e.host.profiles().unwrap().is_empty());
}

#[tokio::test]
async fn diagnostics_package_is_redacted() {
    use std::io::Read;
    let e = env().await;
    e.host
        .update_settings(
            serde_json::from_value(json!({"personalInstructions": "meu segredo pessoal"})).unwrap(),
        )
        .unwrap();
    let draft: aura_gateway::registry::ProviderDraft =
        serde_json::from_value(json!({"name": "OpenAI", "preset": "openai"})).unwrap();
    e.host
        .save_provider(
            draft,
            Some("sk-proj-abcdefghijklmnopqrstuvwxyz123456".into()),
        )
        .unwrap();
    let logs = e.host.paths.logs();
    std::fs::create_dir_all(&logs).unwrap();
    std::fs::write(
        logs.join("aura.log"),
        "INFO request Authorization: Bearer sk-proj-abcdefghijklmnopqrstuvwxyz123456\nid_token=eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0.c2lnbmF0dXJl\n",
    )
    .unwrap();
    let zip_path = e
        .host
        .export_diagnostics(&e._dir.path().join("diag.zip"))
        .await
        .unwrap();
    let mut z = zip::ZipArchive::new(std::fs::File::open(zip_path).unwrap()).unwrap();
    let mut all = String::new();
    let mut names = Vec::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).unwrap();
        names.push(f.name().to_string());
        f.read_to_string(&mut all).unwrap();
    }
    assert!(
        names.contains(&"logs/aura.log".to_string()) && names.contains(&"summary.json".to_string()),
        "{names:?}"
    );
    assert!(!all.contains("sk-proj-abcdef"), "API key leaked");
    assert!(!all.contains("eyJhbGciOi"), "JWT leaked");
    assert!(!all.contains("meu segredo pessoal"), "personal text leaked");
    assert!(all.contains("\"hasCredential\": true"));
}

#[tokio::test]
async fn speak_returns_playable_audio() {
    let e = env().await;
    let (b64, mime) = e.host.speak("**Olá**, mundo").await.unwrap();
    assert_eq!(mime, "audio/wav");
    assert!(b64.starts_with("UklGR")); // "RIFF"
    assert!(e.host.speak("```\ncode only\n```").await.is_ok());
    assert_eq!(e.host.speak("   ").await.unwrap_err().code, "speech");
}

#[tokio::test]
async fn repeated_video_frames_count_once() {
    // Media Foundation seeks to the previous keyframe: a short clip with one
    // keyframe yields the same timestamp for every requested position.
    struct OneKeyframe;
    impl aura_capture::encoder::MediaFileDecoder for OneKeyframe {
        fn audio_16k(&self, _: &std::path::Path) -> Result<Vec<f32>, String> {
            Ok(vec![])
        }
        fn video_frames(
            &self,
            _: &std::path::Path,
            max: usize,
        ) -> Result<Vec<(i64, aura_capture::frame::Frame)>, String> {
            let f = aura_capture::frame::Frame::solid(64, 36, [10, 20, 30, 255]);
            let mut v = vec![(80, f.clone()); max - 1];
            v.push((1080, f));
            Ok(v)
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let (mut platform, _fg) = Platform::fake();
    platform.media = Arc::new(OneKeyframe);
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
    cfg.in_memory_store = true;
    let host = Host::start(cfg).await.unwrap();
    let conv = host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    let mp4 = dir.path().join("curto.mp4");
    std::fs::write(&mp4, b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom").unwrap();
    let (info, chip) = host
        .attach(&conv.thread_id, Some(&conv.thread_id), &mp4)
        .await
        .unwrap();
    assert!(info.summary.contains("2 quadros"), "{}", info.summary);
    let ChipPayload::Mixed { parts } = &chip.payload else {
        panic!("mixed payload")
    };
    let images = parts
        .iter()
        .filter(|p| matches!(p, ChipPayload::Image { .. }))
        .count();
    assert_eq!(images, 2);
}

#[tokio::test]
async fn video_and_compressed_audio_attachments() {
    let e = env().await;
    let conv = e
        .host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    let mp4 = e._dir.path().join("demo.mp4");
    std::fs::write(&mp4, b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom").unwrap();
    let (info, chip) = e
        .host
        .attach(&conv.thread_id, Some(&conv.thread_id), &mp4)
        .await
        .unwrap();
    assert_eq!(info.kind, "video");
    assert!(
        info.summary.contains("3 quadros") && info.summary.contains("transcrito"),
        "{}",
        info.summary
    );
    let ChipPayload::Mixed { parts } = &chip.payload else {
        panic!("mixed payload")
    };
    assert_eq!(
        parts
            .iter()
            .filter(|p| matches!(p, ChipPayload::Image { .. }))
            .count(),
        3
    );

    let mp3 = e._dir.path().join("nota.mp3");
    std::fs::write(&mp3, b"ID3\x03\x00\x00\x00\x00\x00\x00").unwrap();
    let (info, _) = e
        .host
        .attach(&conv.thread_id, Some(&conv.thread_id), &mp3)
        .await
        .unwrap();
    assert_eq!(info.kind, "audio");
    assert!(info.summary.starts_with("áudio 00:02"), "{}", info.summary);
}
