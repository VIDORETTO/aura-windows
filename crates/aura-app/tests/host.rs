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
    cfg.siwc.authorize_url = "http://127.0.0.1/qa-authorize".into();
    cfg.siwc.preferred_port = 0;
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
async fn cancelling_login_emits_only_cancelled_and_allows_another_attempt() {
    let e = env().await;
    let mut rx = e.host.subscribe();
    for _ in 0..2 {
        let url = e.host.login(None, false).await.unwrap();
        assert!(url.starts_with("http://127.0.0.1/qa-authorize?"));
        until(&mut rx, |event| {
            matches!(
                event,
                HostEvent::Login(aura_auth::LoginProgress::WaitingBrowser { .. })
            )
        })
        .await;
        e.host.cancel_login().await;
        let mut terminals = vec![];
        while let Ok(Ok(event)) = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await
        {
            if let HostEvent::Login(progress) = event {
                terminals.push(
                    serde_json::to_value(progress).unwrap()["state"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                );
            }
        }
        assert_eq!(terminals, ["cancelled"]);
        assert!(e.host.auth_status().unwrap().active.is_none());
    }
}

#[tokio::test]
async fn genuine_login_failure_is_forwarded_once() {
    let e = env().await;
    let mut rx = e.host.subscribe();
    let authorize = reqwest::Url::parse(&e.host.login(None, false).await.unwrap()).unwrap();
    let redirect = authorize
        .query_pairs()
        .find(|(key, _)| key == "redirect_uri")
        .unwrap()
        .1
        .into_owned();
    until(&mut rx, |event| {
        matches!(
            event,
            HostEvent::Login(aura_auth::LoginProgress::WaitingBrowser { .. })
        )
    })
    .await;
    // State validation fails before token exchange; only the real loopback callback is contacted.
    reqwest::get(format!("{redirect}?state=qa-invalid-state&code=qa-unused"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let mut terminals = vec![];
    while let Ok(Ok(event)) = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
        if let HostEvent::Login(progress) = event {
            terminals.push(serde_json::to_value(progress).unwrap());
        }
    }
    assert_eq!(
        terminals,
        [json!({ "state": "failed", "reason": "state mismatch" })]
    );
    assert!(e.host.auth_status().unwrap().active.is_none());
}

#[tokio::test]
async fn ptt_uses_saved_microphone_unless_explicitly_overridden() {
    use aura_audio::{
        AudioError, AudioSource, AudioSourceKind, AudioStream, DeviceInfo, DeviceSel,
    };
    use std::sync::Mutex;
    struct AudioProbe {
        opened: Arc<Mutex<Vec<DeviceSel>>>,
    }
    impl AudioSource for AudioProbe {
        fn devices(&self, _: AudioSourceKind) -> Vec<DeviceInfo> {
            vec![
                DeviceInfo {
                    id: "mic-a".into(),
                    name: "Microphone A".into(),
                    is_default: true,
                },
                DeviceInfo {
                    id: "mic-b".into(),
                    name: "Microphone B".into(),
                    is_default: false,
                },
            ]
        }
        fn open(
            &self,
            kind: AudioSourceKind,
            device: &DeviceSel,
        ) -> Result<Box<dyn AudioStream>, AudioError> {
            self.opened.lock().unwrap().push(device.clone());
            aura_audio::hub::SyntheticAudio {
                freq: 440.0,
                amplitude: 0.0,
                total_ms: 1000,
                realtime: true,
                fail_ids: vec![],
            }
            .open(kind, device)
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(dir.path().join("Aura"));
    let opened = Arc::new(Mutex::new(vec![]));
    let (mut platform, _) = Platform::fake();
    platform.audio = Arc::new(AudioProbe {
        opened: opened.clone(),
    });
    let host = Host::start(HostConfig::demo(paths.clone(), platform.clone()))
        .await
        .unwrap();
    host.update_settings(serde_json::from_value(json!({ "microphoneDeviceId": "mic-b" })).unwrap())
        .unwrap();
    host.ptt_press(None).await.unwrap();
    assert_eq!(*opened.lock().unwrap(), [DeviceSel::Id("mic-b".into())]);
    host.ptt_cancel().await;
    host.ptt_press(Some("mic-a".into())).await.unwrap();
    assert_eq!(
        *opened.lock().unwrap(),
        [DeviceSel::Id("mic-b".into()), DeviceSel::Id("mic-a".into())]
    );
    host.ptt_cancel().await;
    host.shutdown().await;
    let restarted = Host::start(HostConfig::demo(paths, platform))
        .await
        .unwrap();
    restarted.ptt_press(None).await.unwrap();
    assert_eq!(
        opened.lock().unwrap().last(),
        Some(&DeviceSel::Id("mic-b".into()))
    );
    restarted.ptt_cancel().await;
    restarted
        .update_settings(serde_json::from_value(json!({ "microphoneDeviceId": null })).unwrap())
        .unwrap();
    restarted.ptt_press(None).await.unwrap();
    assert_eq!(opened.lock().unwrap().last(), Some(&DeviceSel::Default));
    restarted.ptt_cancel().await;
    restarted.shutdown().await;
}

mod audio_test {
    use super::*;
    use aura_audio::{
        AudioError, AudioSource, AudioSourceKind, AudioStream, DeviceInfo, DeviceSel,
    };
    use std::sync::Mutex;

    /// 1 kHz sine at amplitude 0.5 in real time; records what was opened.
    pub struct Tone {
        pub opened: Arc<Mutex<Vec<(AudioSourceKind, DeviceSel)>>>,
        pub fail_ids: Vec<String>,
    }
    impl AudioSource for Tone {
        fn devices(&self, _: AudioSourceKind) -> Vec<DeviceInfo> {
            vec![]
        }
        fn open(
            &self,
            kind: AudioSourceKind,
            device: &DeviceSel,
        ) -> Result<Box<dyn AudioStream>, AudioError> {
            self.opened.lock().unwrap().push((kind, device.clone()));
            aura_audio::hub::SyntheticAudio {
                freq: 1000.0,
                amplitude: 0.5,
                total_ms: 10_000,
                realtime: true,
                fail_ids: self.fail_ids.clone(),
            }
            .open(kind, device)
        }
    }

    pub async fn host_with(
        fail_ids: &[&str],
    ) -> (
        Arc<Host>,
        Arc<Mutex<Vec<(AudioSourceKind, DeviceSel)>>>,
        tempfile::TempDir,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let opened = Arc::new(Mutex::new(vec![]));
        let (mut platform, _) = Platform::fake();
        platform.audio = Arc::new(Tone {
            opened: opened.clone(),
            fail_ids: fail_ids.iter().map(|s| s.to_string()).collect(),
        });
        let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
        cfg.in_memory_store = true;
        (Host::start(cfg).await.unwrap(), opened, dir)
    }

    /// Level events of `source` received during `window`.
    pub async fn levels(
        rx: &mut tokio::sync::broadcast::Receiver<HostEvent>,
        window: Duration,
    ) -> Vec<serde_json::Value> {
        let end = tokio::time::Instant::now() + window;
        let mut out = vec![];
        while let Ok(Ok(e)) = tokio::time::timeout_at(end, rx.recv()).await {
            let v = serde_json::to_value(&e).unwrap();
            if v["channel"] == "audioLevel" {
                out.push(v["event"].clone());
            }
        }
        out
    }
}

#[tokio::test]
async fn microphone_test_reports_live_levels_from_the_saved_device_until_stopped() {
    use aura_audio::{AudioSourceKind, DeviceSel};
    let (host, opened, _dir) = audio_test::host_with(&[]).await;
    host.update_settings(serde_json::from_value(json!({ "microphoneDeviceId": "mic-b" })).unwrap())
        .unwrap();
    let mut rx = host.subscribe();
    host.audio_test_start(AudioSourceKind::Mic, None)
        .await
        .unwrap();
    let got = audio_test::levels(&mut rx, Duration::from_millis(1000)).await;
    assert_eq!(
        *opened.lock().unwrap(),
        [(AudioSourceKind::Mic, DeviceSel::Id("mic-b".into()))]
    );
    // ≥ 20 Hz over one second, allowing for start-up.
    assert!(got.len() >= 15, "only {} level events", got.len());
    // RMS of a 0.5 sine = 0.5/√2 → 20·log10(0.35355) = −9.03 dBFS.
    for event in &got[2..] {
        assert_eq!(event["source"], "mic");
        let dbfs = event["dbfs"].as_f64().unwrap();
        assert!((dbfs + 9.03).abs() < 1.0, "{dbfs}");
    }
    host.audio_test_stop(AudioSourceKind::Mic).await;
    let _ = audio_test::levels(&mut rx, Duration::from_millis(100)).await;
    assert!(
        audio_test::levels(&mut rx, Duration::from_millis(400))
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn system_audio_test_uses_the_saved_output_and_warns_when_it_falls_back() {
    use aura_audio::{AudioSourceKind, DeviceSel};
    let (host, opened, _dir) = audio_test::host_with(&["gone"]).await;
    host.update_settings(serde_json::from_value(json!({ "systemAudioDeviceId": "gone" })).unwrap())
        .unwrap();
    let mut rx = host.subscribe();
    host.audio_test_start(AudioSourceKind::SystemAudio, None)
        .await
        .unwrap();
    let mut notice = None;
    let mut system_levels = 0;
    let end = tokio::time::Instant::now() + Duration::from_millis(600);
    while let Ok(Ok(e)) = tokio::time::timeout_at(end, rx.recv()).await {
        let v = serde_json::to_value(&e).unwrap();
        if v["channel"] == "notice" {
            notice = Some(v["event"].clone());
        }
        if v["channel"] == "audioLevel" && v["event"]["source"] == "systemAudio" {
            system_levels += 1;
        }
    }
    assert_eq!(
        *opened.lock().unwrap(),
        [
            (AudioSourceKind::SystemAudio, DeviceSel::Id("gone".into())),
            (AudioSourceKind::SystemAudio, DeviceSel::Default)
        ]
    );
    let notice = notice.expect("fallback notice");
    assert_eq!(notice["level"], "warning");
    assert!(notice["message"].as_str().unwrap().contains("gone"));
    assert!(system_levels > 0);
    host.audio_test_stop(AudioSourceKind::SystemAudio).await;
}

#[tokio::test]
async fn dictation_with_a_missing_saved_microphone_uses_the_default_and_warns() {
    use aura_audio::{AudioSourceKind, DeviceSel};
    let (host, opened, _dir) = audio_test::host_with(&["gone-mic"]).await;
    host.update_settings(
        serde_json::from_value(json!({ "microphoneDeviceId": "gone-mic" })).unwrap(),
    )
    .unwrap();
    let mut rx = host.subscribe();
    host.ptt_press(None).await.unwrap();
    let notice = until(&mut rx, |e| matches!(e, HostEvent::Notice { .. })).await;
    let HostEvent::Notice { level, message } = notice else {
        unreachable!()
    };
    assert_eq!(level, "warning");
    assert!(message.contains("gone-mic"), "{message}");
    assert_eq!(
        *opened.lock().unwrap(),
        [
            (AudioSourceKind::Mic, DeviceSel::Id("gone-mic".into())),
            (AudioSourceKind::Mic, DeviceSel::Default)
        ]
    );
    host.ptt_cancel().await;
}

#[tokio::test]
async fn audio_test_respects_paused_privacy_without_opening_a_device() {
    use aura_audio::AudioSourceKind;
    let (host, opened, _dir) = audio_test::host_with(&[]).await;
    host.set_paused(true).unwrap();
    let err = host
        .audio_test_start(AudioSourceKind::Mic, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, "paused");
    assert!(opened.lock().unwrap().is_empty());
}

#[tokio::test]
async fn recordings_capture_the_saved_audio_devices() {
    use aura_audio::{AudioSourceKind, DeviceSel};
    let (host, opened, _dir) = audio_test::host_with(&[]).await;
    host.update_settings(
        serde_json::from_value(
            json!({ "microphoneDeviceId": "mic-b", "systemAudioDeviceId": "speakers-b" }),
        )
        .unwrap(),
    )
    .unwrap();
    host.set_source_policy(Source::Mic, CaptureMode::Manual, AgentPermission::Never)
        .unwrap();
    host.set_source_policy(
        Source::SystemAudio,
        CaptureMode::Manual,
        AgentPermission::Never,
    )
    .unwrap();
    host.recording_start("QA").await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    host.recording_stop().await;
    let opened = opened.lock().unwrap().clone();
    assert!(opened.contains(&(AudioSourceKind::Mic, DeviceSel::Id("mic-b".into()))));
    assert!(opened.contains(&(
        AudioSourceKind::SystemAudio,
        DeviceSel::Id("speakers-b".into())
    )));
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
    let vault = Arc::new(
        aura_store::Vault::open(&store, &aura_store::StaticKeyProtector::default()).unwrap(),
    );
    let privacy = aura_app::privacy::PrivacyRepo::new(store);
    let policy = privacy.load().unwrap();
    HostTools {
        extensions: Default::default(),
        vault,
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
    // QA-031: the log keeps the consent outcome and a sealed thumbnail.
    let entry = tools.privacy.access_log(1).unwrap().remove(0);
    assert_eq!(
        (entry.decision.as_str(), entry.reason.as_deref()),
        ("allow", Some("consent"))
    );
    assert!(entry.has_thumbnail);
    let sealed = tools.privacy.thumbnail(entry.id).unwrap().unwrap();
    assert_ne!(&sealed[1..4], b"PNG", "thumbnail must be sealed at rest");
    let png = tools.vault.open_bytes("access-thumb", &sealed).unwrap();
    assert_eq!(&png[1..4], b"PNG");
    let w = u32::from_be_bytes(png[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(png[20..24].try_into().unwrap());
    assert!(w.max(h) <= 240 && w > 0, "{w}x{h}");

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
    let entry = tools.privacy.access_log(1).unwrap().remove(0);
    assert_eq!(
        (entry.decision.as_str(), entry.reason.as_deref()),
        ("deny", Some("user"))
    );
    assert!(!entry.has_thumbnail);

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
async fn attachment_labels_expose_counts_without_translating_file_names() {
    let e = env().await;
    let path = e._dir.path().join("1 linhas.txt");
    std::fs::write(&path, "one line\n").unwrap();
    let (_, chip) = e.host.attach("draft", None, &path).await.unwrap();
    let wire = serde_json::to_value(&chip).unwrap();
    assert_eq!(
        wire["attachmentLabel"],
        json!({
            "fileName": "1 linhas.txt", "parts": [{"type": "count", "amount": 1, "unit": "line"}]
        })
    );
    assert_eq!(
        chip.payload,
        ChipPayload::Mixed {
            parts: vec![
                ChipPayload::Text {
                    text: "Anexo: 1 linhas.txt (1 linhas)".into()
                },
                ChipPayload::Text {
                    text: "one line\n".into()
                },
            ]
        }
    );
}

#[tokio::test]
async fn builtin_prompts_follow_language_without_rewriting_user_commands() {
    let e = env().await;
    e.host
        .save_quick_command("qa-custom", "Modelo pessoal em português: {texto}", false)
        .unwrap();
    e.host.toggle_quick_command("tldr", false).unwrap();
    e.host
        .update_settings(serde_json::from_value(json!({"language": "en"})).unwrap())
        .unwrap();
    let commands = e.host.quick_commands().unwrap();
    let tldr = commands.iter().find(|c| c.name == "tldr").unwrap();
    assert_eq!(
        tldr.template,
        "Summarize in up to three sentences, straight to the point:\n\n{selecao}"
    );
    assert!(!tldr.enabled);
    assert_eq!(
        commands
            .iter()
            .find(|c| c.name == "qa-custom")
            .unwrap()
            .template,
        "Modelo pessoal em português: {texto}"
    );
    e.host.toggle_quick_command("tldr", true).unwrap();
    let expanded = e
        .host
        .expand_quick_command("draft", "/tldr literal input", "")
        .await
        .unwrap();
    assert_eq!(
        expanded.prompt_text,
        "Summarize in up to three sentences, straight to the point:\n\nliteral input"
    );
    e.host
        .update_settings(serde_json::from_value(json!({"language": "ptBr"})).unwrap())
        .unwrap();
    assert_eq!(
        e.host
            .quick_commands()
            .unwrap()
            .iter()
            .find(|c| c.name == "tldr")
            .unwrap()
            .template,
        "Resuma em até três frases, direto ao ponto:\n\n{selecao}"
    );
}

/// 1×1 transparent PNG (literal bytes of a known-valid file).
const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

#[tokio::test]
async fn pasted_clipboard_image_becomes_an_image_chip_and_other_types_are_refused() {
    let e = env().await;
    let (info, chip) = e
        .host
        .attach_clipboard_image("draft", None, "image/png", PNG_1X1)
        .await
        .unwrap();
    assert_eq!(chip.kind, ChipKind::Image);
    assert_eq!(info.kind, "image");
    assert_eq!(std::fs::read(&info.stored).unwrap(), PNG_1X1);
    assert!(info.file_name.ends_with(".png"), "{}", info.file_name);
    assert_eq!(e.host.tray("draft").len(), 1);
    for mime in ["image/jpeg", "image/webp", "image/gif"] {
        // Accepted types reach ingestion (bytes here are not a valid image of that type).
        let err = e
            .host
            .attach_clipboard_image("draft", None, mime, b"x")
            .await
            .err();
        assert!(err.is_none_or(|e| e.code != "unsupported"), "{mime}");
    }
    let err = e
        .host
        .attach_clipboard_image("draft", None, "image/bmp", PNG_1X1)
        .await
        .unwrap_err();
    assert_eq!(err.code, "unsupported");
    let err = e
        .host
        .attach_clipboard_image("draft", None, "image/png", &vec![0u8; 25 * 1024 * 1024])
        .await
        .unwrap_err();
    assert_eq!(err.code, "too_large");
}

#[tokio::test]
async fn skills_settings_list_origins_toggle_and_edit_aura_skills() {
    let e = env().await;
    e.host
        .create_skill("revisar-contrato", "Revisa contratos", "Leia com calma.")
        .unwrap();
    let catalog = e.host.skills_catalog().await.unwrap();
    let aura = catalog
        .iter()
        .find(|s| s.name == "revisar-contrato")
        .unwrap();
    assert_eq!(serde_json::to_value(aura.origin).unwrap(), "aura");
    assert!(aura.enabled);
    assert!(
        catalog
            .iter()
            .any(|s| serde_json::to_value(s.origin).unwrap() == "system")
    );
    e.host.set_skill_enabled(&aura.path, false).await.unwrap();
    // Kept by Aura: CODEX_HOME/config.toml is regenerated on every start.
    assert_eq!(
        e.host.settings().disabled_skills,
        [aura.path.to_string_lossy().into_owned()]
    );
    let catalog = e.host.skills_catalog().await.unwrap();
    assert!(
        !catalog
            .iter()
            .find(|s| s.name == "revisar-contrato")
            .unwrap()
            .enabled
    );
    assert_eq!(
        e.host.skill_source("revisar-contrato").unwrap(),
        (
            "Revisa contratos".to_string(),
            "Leia com calma.".to_string()
        )
    );
    e.host
        .update_skill(
            "revisar-contrato",
            "Revisa contratos de aluguel",
            "Confira multas.",
        )
        .unwrap();
    let catalog = e.host.skills_catalog().await.unwrap();
    assert_eq!(
        catalog
            .iter()
            .find(|s| s.name == "revisar-contrato")
            .unwrap()
            .description,
        "Revisa contratos de aluguel"
    );
}

#[tokio::test]
async fn core_skills_reach_the_agent_but_not_the_user() {
    // 017 AC-007.
    let e = env().await;
    let core = e.host.paths.core_skills();
    for (name, _) in aura_app::core_skills::CORE_SKILLS {
        assert!(core.join(name).join("SKILL.md").is_file(), "{name}");
    }
    let catalog = e.host.skills_catalog().await.unwrap();
    assert!(
        catalog.iter().all(|s| !s.name.starts_with("aura-")),
        "{catalog:?}"
    );
    // The fake app-server lists every extra root: the core root is one of them.
    assert!(catalog.iter().any(|s| s.name == "skill-creator"));
    let path = core.join("aura-criar-skill").join("SKILL.md");
    assert_eq!(
        e.host
            .set_skill_enabled(&path, false)
            .await
            .unwrap_err()
            .code,
        "skill"
    );
    assert_eq!(
        e.host
            .create_skill("aura-minha", "x", "y")
            .unwrap_err()
            .code,
        "skill"
    );
}

#[tokio::test]
async fn agent_creates_extensions_through_aura_tools() {
    // 017 AC-006.
    let e = env().await;
    let (tx, _rx) = tokio::sync::broadcast::channel(16);
    let tools = tools_of(
        &e,
        tx,
        Arc::new(aura_app::consent::ConsentBroker::default()),
    );
    let weak: std::sync::Weak<dyn aura_app::tools::ExtensionsAccess> =
        Arc::downgrade(&e.host) as std::sync::Weak<dyn aura_app::tools::ExtensionsAccess>;
    tools.extensions.set(weak).ok().unwrap();
    let mut events = e.host.subscribe();
    let ctx = CallContext {
        conversation: "conv-ext".into(),
    };
    let call = |tool: &'static str, args: serde_json::Value| {
        let (tools, ctx) = (&tools, ctx.clone());
        async move { tools.call(tool, args, ctx).await }
    };
    let text = |o: &aura_mcp::ToolOutput| match &o.content[0] {
        aura_mcp::Content::Text(t) => t.clone(),
        _ => panic!("text"),
    };

    let out = call(
        "skill_save",
        json!({"name": "resumir-ata", "description": "Resume atas", "instructions": "Liste decisões."}),
    )
    .await;
    assert!(!out.is_error, "{}", text(&out));
    until(&mut events, |ev| {
        matches!(ev, HostEvent::ExtensionsChanged {})
    })
    .await;
    let skill = e
        .host
        .skills()
        .into_iter()
        .find(|s| s.manifest.name == "resumir-ata")
        .unwrap();
    assert_eq!(skill.manifest.description, "Resume atas");
    // Same name again without replace: validation error the agent can fix.
    let again = call(
        "skill_save",
        json!({"name": "resumir-ata", "description": "Outra", "instructions": "x"}),
    )
    .await;
    assert!(again.is_error);
    let replaced = call(
        "skill_save",
        json!({"name": "resumir-ata", "description": "Resume atas de condomínio", "instructions": "x", "replace": true}),
    )
    .await;
    assert!(!replaced.is_error, "{}", text(&replaced));
    assert_eq!(
        e.host.skill_source("resumir-ata").unwrap().0,
        "Resume atas de condomínio"
    );

    let out = call(
        "quick_command_save",
        json!({"name": "formal-email", "template": "Reescreva formal: {texto}"}),
    )
    .await;
    assert!(!out.is_error, "{}", text(&out));
    let formal = e
        .host
        .quick_commands()
        .unwrap()
        .into_iter()
        .find(|q| q.name == "formal-email")
        .unwrap();
    assert_eq!(formal.template, "Reescreva formal: {texto}");
    assert!(formal.enabled && !formal.builtin);

    let out = call(
        "mcp_server_save",
        json!({"name": "github", "transport": "stdio", "command": "npx",
               "args": ["-y", "@modelcontextprotocol/server-github"], "secret_env": ["GITHUB_TOKEN"]}),
    )
    .await;
    assert!(!out.is_error, "{}", text(&out));
    assert!(text(&out).contains("Configurações"), "{}", text(&out));
    let github = e
        .host
        .mcp_servers()
        .unwrap()
        .into_iter()
        .find(|s| s.name == "github")
        .unwrap();
    assert!(!github.enabled, "needs the secret first");
    assert_eq!(github.secret_vars()[0].0, "GITHUB_TOKEN");

    let out = call(
        "mcp_server_save",
        json!({"name": "docs", "transport": "http", "url": "https://example.com/mcp"}),
    )
    .await;
    assert!(!out.is_error, "{}", text(&out));
    assert!(
        e.host
            .mcp_servers()
            .unwrap()
            .iter()
            .any(|s| s.name == "docs" && s.enabled)
    );
    let reserved = call(
        "mcp_server_save",
        json!({"name": "aura", "transport": "http", "url": "https://example.com/mcp"}),
    )
    .await;
    assert!(reserved.is_error);

    let listed = call("extensions_list", json!({})).await;
    let v: serde_json::Value = serde_json::from_str(&text(&listed)).unwrap();
    assert!(
        v["skills"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == "resumir-ata")
    );
    assert!(
        v["quick_commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|q| q["name"] == "formal-email")
    );
    assert_eq!(
        v["mcp_servers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["name"] == "github")
            .unwrap()["secret_env"],
        json!(["GITHUB_TOKEN"])
    );
}

#[tokio::test]
async fn agent_task_asks_the_overlay_for_a_new_conversation() {
    // 017 AC-005.
    let e = env().await;
    let mut rx = e.host.subscribe();
    e.host.agent_task("crie uma skill", "task").unwrap();
    let HostEvent::AgentTask { text, mode } =
        until(&mut rx, |ev| matches!(ev, HostEvent::AgentTask { .. })).await
    else {
        unreachable!()
    };
    assert_eq!((text.as_str(), mode.as_str()), ("crie uma skill", "task"));
    assert_eq!(e.host.agent_task("  ", "task").unwrap_err().code, "invalid");
    assert_eq!(e.host.agent_task("x", "turbo").unwrap_err().code, "invalid");
}

#[tokio::test]
async fn attach_skill_adds_one_skill_chip_and_rejects_disabled() {
    // 015 AC-003: choosing a skill in the `/` menu becomes a Chip, not `$name` text.
    let e = env().await;
    e.host
        .create_skill("revisar-contrato", "Revisa contratos", "Leia com calma.")
        .unwrap();
    let chip = e
        .host
        .attach_skill("draft", "revisar-contrato")
        .await
        .unwrap();
    assert_eq!(serde_json::to_value(chip.kind).unwrap(), "skill");
    assert_eq!(chip.label, "revisar-contrato");
    let payload = serde_json::to_value(&chip.payload).unwrap();
    assert_eq!(payload["name"], "revisar-contrato");
    assert!(
        payload["path"].as_str().unwrap().ends_with("SKILL.md"),
        "{payload}"
    );
    e.host
        .attach_skill("draft", "revisar-contrato")
        .await
        .unwrap();
    assert_eq!(e.host.tray("draft").len(), 1, "same skill twice: one chip");

    let path = e
        .host
        .skills_catalog()
        .await
        .unwrap()
        .into_iter()
        .find(|s| s.name == "revisar-contrato")
        .unwrap()
        .path;
    e.host.set_skill_enabled(&path, false).await.unwrap();
    assert_eq!(
        e.host
            .attach_skill("t2", "revisar-contrato")
            .await
            .unwrap_err()
            .code,
        "skill"
    );
    assert_eq!(
        e.host
            .attach_skill("t2", "nao-existe")
            .await
            .unwrap_err()
            .code,
        "skill"
    );
}

#[test]
fn start_model_keeps_defaults_with_their_provider() {
    use aura_app::host::start_model;
    let plan = "aura-chatgpt-plan";
    // The Settings default lists ChatGPT-plan models: never sent to a BYOK provider.
    assert_eq!(
        start_model(plan, None, Some("gpt-6-luna"), None),
        Some("gpt-6-luna".into())
    );
    assert_eq!(
        start_model("aura-groq", None, Some("gpt-6-luna"), None),
        None
    );
    // The picker choice always wins.
    assert_eq!(
        start_model("aura-groq", Some("llama-4"), Some("gpt-6-luna"), None),
        Some("llama-4".into())
    );
    // An app profile beats the global default; its BYOK model is `aura-<id>::<model>`.
    assert_eq!(
        start_model(plan, None, Some("gpt-6-luna"), Some("gpt-6.1-sol")),
        Some("gpt-6.1-sol".into())
    );
    assert_eq!(
        start_model("aura-groq", None, None, Some("aura-groq::llama-4")),
        Some("llama-4".into())
    );
    assert_eq!(
        start_model(plan, None, Some("gpt-6-luna"), Some("aura-groq::llama-4")),
        Some("gpt-6-luna".into())
    );
    assert_eq!(
        start_model("aura-groq", None, None, Some("gpt-6.1-sol")),
        None
    );
}

#[test]
fn plan_conversations_always_start_with_a_catalog_model() {
    // 017 AC-001: a default saved before the GPT-6 catalog is not sent; the
    // plan starts with an explicit catalog model (Luna first).
    use aura_app::host::start_model;
    let plan = "aura-chatgpt-plan";
    assert_eq!(
        start_model(plan, None, Some("gpt-5.5"), None),
        Some("gpt-6-luna".into())
    );
    assert_eq!(
        start_model(plan, None, None, None),
        Some("gpt-6-luna".into())
    );
    assert_eq!(
        start_model(plan, None, Some("gpt-5.5"), Some("gpt-6-astra")),
        Some("gpt-6-astra".into())
    );
    assert_eq!(
        start_model(plan, Some("gpt-5.6-sol"), None, None),
        Some("gpt-6-luna".into())
    );
    assert_eq!(
        start_model(plan, Some("gpt-6.1-sol"), None, None),
        Some("gpt-6.1-sol".into())
    );
}

#[test]
fn mcp_status_reports_startup_errors_from_the_app_server() {
    // Shape captured from the pinned app-server (rust-v0.159.0) with one
    // failing and one working stdio server.
    let raw = json!({"data": [
        {"name": "qafail", "serverInfo": null, "tools": {}, "toolsError": "MCP startup failed: handshaking with MCP server failed: connection closed: initialize response", "authStatus": "unsupported"},
        {"name": "qaok", "serverInfo": {"name": "qa-mcp", "version": "1.0.0"}, "tools": {"qa_echo": {"name": "qa_echo"}, "qa_write": {"name": "qa_write"}}, "toolsError": null, "authStatus": "unsupported"}
    ]});
    let status = aura_app::host::parse_mcp_status(&raw);
    assert_eq!(
        status[0].error.as_deref(),
        Some(
            "MCP startup failed: handshaking with MCP server failed: connection closed: initialize response"
        )
    );
    assert!(status[0].tools.is_empty());
    assert_eq!(status[1].error, None);
    assert_eq!(status[1].tools, ["qa_echo", "qa_write"]);
}

#[cfg(windows)]
#[tokio::test]
async fn diagnosing_a_failing_stdio_server_returns_its_last_log_lines() {
    let e = env().await;
    e.host
        .save_mcp_server(
            serde_json::from_value(json!({"name": "qa-fail", "transport": {"type": "stdio", "command": "cmd", "args": ["/C", "echo token missing 1>&2 & exit 2"], "env": {"QA_MODE": {"kind": "plain", "value": "x"}}, "cwd": null}, "enabled": true, "disabledTools": [], "approvalMode": "askForWrites", "startupTimeoutSec": null, "toolTimeoutSec": null})).unwrap(),
            vec![],
            None,
        )
        .unwrap();
    let d = e.host.mcp_diagnose("qa-fail").await.unwrap();
    assert!(!d.connected);
    assert_eq!(d.exit_code, Some(2));
    assert_eq!(d.log, ["token missing"]);
    assert_eq!(
        e.host.mcp_diagnose("nope").await.unwrap_err().code,
        "not_found"
    );
}

#[tokio::test]
async fn memories_can_be_reviewed_edited_and_forgotten() {
    let e = env().await;
    assert_eq!(
        e.host.memories().unwrap(),
        aura_app::host::MemoryView::default()
    );
    let dir = e.host.paths.codex_home().join("memories");
    std::fs::create_dir_all(dir.join("rollout_summaries")).unwrap();
    std::fs::write(
        dir.join("memory_summary.md"),
        "# Summary\n- Prefers metric units\n- Lives in Lisbon\n",
    )
    .unwrap();
    std::fs::write(dir.join("MEMORY.md"), "# Registry\n- Lives in Lisbon\n").unwrap();
    let view = e.host.memories().unwrap();
    assert_eq!(view.facts, ["Prefers metric units", "Lives in Lisbon"]);
    e.host.forget_memory_fact("Lives in Lisbon").unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("memory_summary.md")).unwrap(),
        "# Summary\n- Prefers metric units\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("MEMORY.md")).unwrap(),
        "# Registry\n"
    );
    // Consolidation reads user notes, so the removal is not undone later.
    let notes: Vec<String> = std::fs::read_dir(dir.join("extensions/ad_hoc/notes"))
        .unwrap()
        .map(|f| std::fs::read_to_string(f.unwrap().path()).unwrap())
        .collect();
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("Forget: Lives in Lisbon"), "{}", notes[0]);
    e.host
        .save_memories("# Summary\n- Prefers SI units\n", "# Registry\n")
        .unwrap();
    assert_eq!(e.host.memories().unwrap().facts, ["Prefers SI units"]);
    e.host.forget_all_memories().unwrap();
    assert!(!dir.exists());
    assert!(e.host.memories().unwrap().facts.is_empty());
}

/// Minimal one-font PDF with one text line per page (offsets computed).
fn pdf_with_pages(pages: &[&str]) -> Vec<u8> {
    let n = pages.len();
    let mut objects: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".into(),
        format!(
            "<< /Type /Pages /Kids [{}] /Count {n} >>",
            (0..n)
                .map(|i| format!("{} 0 R", 4 + 2 * i))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
    ];
    for (i, text) in pages.iter().enumerate() {
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 144] /Contents {} 0 R /Resources << /Font << /F1 3 0 R >> >> >>",
            5 + 2 * i
        ));
        let stream = format!("BT /F1 12 Tf 20 100 Td ({text}) Tj ET");
        objects.push(format!(
            "<< /Length {} >>\nstream\n{stream}\nendstream",
            stream.len()
        ));
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![];
    for (i, o) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for off in offsets {
        out.extend(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

#[tokio::test]
async fn workspace_pdf_preview_returns_the_text_of_the_first_pages() {
    let e = env().await;
    let conv = e
        .host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    let pages: Vec<String> = (1..=7).map(|i| format!("QA page {i}")).collect();
    let refs: Vec<&str> = pages.iter().map(String::as_str).collect();
    std::fs::write(conv.workspace.join("report.pdf"), pdf_with_pages(&refs)).unwrap();
    let preview = e
        .host
        .workspace_pdf_preview(&conv.thread_id, "report.pdf")
        .unwrap();
    assert_eq!(preview.total_pages, 7);
    assert_eq!(
        preview
            .pages
            .iter()
            .map(|p| (p.number, p.text.trim().to_string()))
            .collect::<Vec<_>>(),
        (1..=5)
            .map(|i| (i, format!("QA page {i}")))
            .collect::<Vec<_>>()
    );
    assert!(
        e.host
            .workspace_pdf_preview(&conv.thread_id, "../outside.pdf")
            .is_err()
    );
}

#[tokio::test]
async fn retention_limits_are_configurable_validated_and_used_by_the_recorder() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(dir.path().join("Aura"));
    let (platform, _) = Platform::fake();
    let host = Host::start(HostConfig::demo(paths.clone(), platform.clone()))
        .await
        .unwrap();
    // Defaults documented in 004: 7 days, 20 GB, manual recordings kept.
    assert_eq!(
        serde_json::to_value(host.privacy()).unwrap()["retention"],
        json!({"days": 7, "maxGb": 20, "applyToManual": false})
    );
    for (days, gb) in [(0, 20), (366, 20), (7, 0), (7, 1001)] {
        assert_eq!(
            host.set_retention(days, gb, false).unwrap_err().code,
            "out_of_range",
            "{days} {gb}"
        );
    }
    let view = host.set_retention(3, 5, true).unwrap();
    assert_eq!(
        serde_json::to_value(view).unwrap()["retention"],
        json!({"days": 3, "maxGb": 5, "applyToManual": true})
    );
    // The recorders take the policy asynchronously (reconcile_capture).
    let expected = (3 * 86_400_000, 5 * 1024 * 1024 * 1024, true);
    let mut applied = None;
    for _ in 0..50 {
        let r = host.capture().retention();
        applied = Some((r.max_age_ms, r.max_bytes, r.apply_to_manual));
        if applied == Some(expected) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(applied, Some(expected));
    host.shutdown().await;
    let again = Host::start(HostConfig::demo(paths, platform))
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(again.privacy()).unwrap()["retention"]["days"],
        3
    );
    again.shutdown().await;
}

#[tokio::test]
async fn recent_buffer_length_is_limited_to_one_to_thirty_minutes() {
    let e = env().await;
    for bad in [0, 31] {
        let err = e
            .host
            .set_source_policy(
                Source::Mic,
                CaptureMode::RecentBuffer { minutes: bad },
                AgentPermission::Ask,
            )
            .unwrap_err();
        assert_eq!(err.code, "out_of_range", "{bad}");
    }
    for ok in [1, 30] {
        let view = e
            .host
            .set_source_policy(
                Source::Screen,
                CaptureMode::RecentBuffer { minutes: ok },
                AgentPermission::Ask,
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(&view).unwrap()["screen"]["mode"],
            json!({"type": "recentBuffer", "minutes": ok})
        );
    }
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
    let sel = e.host.capture_selection("draft", true).unwrap().unwrap();
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
async fn manual_provider_model_is_saved_announced_and_removable() {
    let e = env().await;
    let p = e
        .host
        .save_provider(
            serde_json::from_value(json!({"name": "Local", "preset": "custom", "wire": "chat", "baseUrl": "http://127.0.0.1:9/v1"}))
                .unwrap(),
            None,
        )
        .unwrap();
    let mut rx = e.host.subscribe();
    let saved = e
        .host
        .save_provider_model(
            &p.id,
            serde_json::from_value(json!({"id": "qwen3:8b", "displayName": "Qwen 3 8B", "contextWindow": 32768, "maxOutput": null, "supportsImages": false, "supportsTools": true, "supportsReasoning": true, "estimated": true}))
                .unwrap(),
        )
        .unwrap();
    let m = saved.models.iter().find(|m| m.id == "qwen3:8b").unwrap();
    assert!(m.manual && m.supports_tools && m.supports_reasoning && !m.estimated);
    until(&mut rx, |e| matches!(e, HostEvent::ProvidersChanged {})).await;
    let removed = e.host.remove_provider_model(&p.id, "qwen3:8b").unwrap();
    assert!(removed.models.is_empty());
}

#[tokio::test]
async fn provider_without_models_endpoint_but_manual_models_is_not_an_error() {
    let e = env().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    // No routes: GET /v1/models answers 404.
    let server =
        tokio::spawn(async move { axum::serve(listener, axum::Router::new()).await.unwrap() });
    let p = e
        .host
        .save_provider(
            serde_json::from_value(json!({"name": "Sem lista", "preset": "custom", "wire": "chat", "baseUrl": format!("http://{addr}/v1")})).unwrap(),
            None,
        )
        .unwrap();
    // Without manual models a 404 still reports the problem.
    let bare = e.host.test_provider(&p.id).await.unwrap();
    assert_eq!(serde_json::to_value(bare.status).unwrap(), "error");
    e.host
        .save_provider_model(
            &p.id,
            serde_json::from_value(json!({"id": "manual-1", "displayName": null, "contextWindow": null, "maxOutput": null, "supportsImages": false, "supportsTools": true, "supportsReasoning": false, "estimated": false})).unwrap(),
        )
        .unwrap();
    let tested = e.host.test_provider(&p.id).await.unwrap();
    server.abort();
    assert_eq!(serde_json::to_value(tested.status).unwrap(), "unverified");
    assert_eq!(tested.last_error, None);
    assert_eq!(tested.models.len(), 1);
}

#[tokio::test]
async fn saving_provider_notifies_existing_windows_without_exposing_provider_data() {
    let e = env().await;
    let mut rx = e.host.subscribe();
    let draft = serde_json::from_value(json!({"name": "QA local", "preset": "ollama"})).unwrap();
    e.host.save_provider(draft, None).unwrap();
    let event = tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let event = rx.recv().await.unwrap();
            let value = serde_json::to_value(event).unwrap();
            if value["channel"] == "providersChanged" {
                break value;
            }
        }
    })
    .await
    .expect("provider catalog invalidation must reach existing windows");
    assert_eq!(event, json!({"channel": "providersChanged", "event": {}}));
}

#[tokio::test]
async fn removing_provider_notifies_existing_windows() {
    let e = env().await;
    let draft = serde_json::from_value(json!({"name": "QA local", "preset": "ollama"})).unwrap();
    let p = e.host.save_provider(draft, None).unwrap();
    let mut rx = e.host.subscribe();
    e.host.remove_provider(&p.id).unwrap();
    let event = tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let value = serde_json::to_value(rx.recv().await.unwrap()).unwrap();
            if value["channel"] == "providersChanged" {
                break value;
            }
        }
    })
    .await
    .expect("removing a provider must invalidate every window catalog");
    assert_eq!(event, json!({"channel": "providersChanged", "event": {}}));
}

#[tokio::test]
async fn provider_discovery_notifies_existing_windows_after_models_are_stored() {
    let e = env().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = axum::Router::new().route(
        "/v1/models",
        axum::routing::get(|| async {
            axum::Json(
                json!({"data": [{"id": "qa-literal-model", "display_name": "QA literal model"}]}),
            )
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let draft = serde_json::from_value(
        json!({"name": "QA local", "preset": "ollama", "baseUrl": format!("http://{addr}/v1")}),
    )
    .unwrap();
    let p = e.host.save_provider(draft, None).unwrap();
    let mut rx = e.host.subscribe();
    let tested = e.host.test_provider(&p.id).await.unwrap();
    assert_eq!(tested.models[0].id, "qa-literal-model");
    let event = tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let value = serde_json::to_value(rx.recv().await.unwrap()).unwrap();
            if value["channel"] == "providersChanged" {
                break value;
            }
        }
    })
    .await;
    server.abort();
    assert_eq!(
        event.expect("discovered models must invalidate every window catalog"),
        json!({"channel": "providersChanged", "event": {}})
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
async fn missing_model_is_reported_to_the_overlay_without_opening_the_microphone() {
    use aura_audio::{
        AudioError, AudioSource, AudioSourceKind, AudioStream, DeviceInfo, DeviceSel,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct NeverCapture(Arc<AtomicUsize>);
    impl AudioSource for NeverCapture {
        fn devices(&self, _: AudioSourceKind) -> Vec<DeviceInfo> {
            vec![]
        }
        fn open(
            &self,
            _: AudioSourceKind,
            _: &DeviceSel,
        ) -> Result<Box<dyn AudioStream>, AudioError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(AudioError::NoDevice)
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let opened = Arc::new(AtomicUsize::new(0));
    let (mut platform, _) = Platform::fake();
    platform.audio = Arc::new(NeverCapture(opened.clone()));
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
    cfg.asr = aura_app::voice::AsrBackend::Worker {
        program: dir.path().join("worker-not-needed.exe"),
        idle: Duration::from_secs(30),
        on_spawn: None,
    };
    let host = Host::start(cfg).await.unwrap();
    let mut events = host.subscribe();
    let error = host.ptt_press(None).await.unwrap_err();
    assert_eq!(error.code, "asr");
    assert_eq!(error.message, "model_missing");
    assert_eq!(opened.load(Ordering::SeqCst), 0);
    let mut states = vec![];
    while let Ok(Ok(event)) = tokio::time::timeout(Duration::from_millis(200), events.recv()).await
    {
        if let HostEvent::Voice(state) = event {
            states.push(state);
        }
    }
    assert_eq!(
        states,
        [aura_asr::ptt::PttState::Failed {
            error: "model_missing".into()
        }]
    );
    host.shutdown().await;
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
async fn user_attaches_the_recent_buffer_as_a_clip() {
    use aura_app::host::RecentClip;
    use aura_core::context::{ChipKind, ChipPayload};
    let dir = tempfile::tempdir().unwrap();
    let (platform, _fg) = Platform::fake();
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
    cfg.in_memory_store = true;
    cfg.capture_interval = Duration::from_millis(20);
    let host = Host::start(cfg).await.unwrap();
    for s in [Source::Screen, Source::Mic] {
        host.set_source_policy(
            s,
            CaptureMode::RecentBuffer { minutes: 2 },
            AgentPermission::Never,
        )
        .unwrap();
    }
    tokio::time::sleep(Duration::from_millis(900)).await;
    let conv = host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    let clip = |minutes: f64, screen: bool, audio: Option<&str>| RecentClip {
        minutes,
        screen,
        audio: audio.map(str::to_string),
    };

    // Screen + microphone: keyframes and transcript of the same interval,
    // files kept in the conversation workspace (004 AC-014, 005 AC-007/009).
    let chip = host
        .attach_recent(
            &conv.thread_id,
            Some(&conv.thread_id),
            clip(2.0, true, Some("mic")),
        )
        .await
        .unwrap();
    assert_eq!(chip.kind, ChipKind::Clip);
    assert!(chip.label.contains("2 min"), "{}", chip.label);
    let ChipPayload::Images { paths, caption } = &chip.payload else {
        panic!("{:?}", chip.payload)
    };
    assert!((1..=8).contains(&paths.len()), "{}", paths.len());
    assert!(
        paths
            .iter()
            .all(|p| p.starts_with(&conv.workspace) && p.exists())
    );
    assert_eq!(chip.preview_path.as_ref(), paths.first());
    let caption = caption.as_deref().unwrap_or_default();
    assert!(
        caption.contains("Você: texto ditado de exemplo"),
        "{caption}"
    );
    let wav = paths[0].parent().unwrap().join("mic.wav");
    assert!(wav.exists(), "audio file of the clip in the workspace");

    // Audio only.
    let chip = host
        .attach_recent(
            &conv.thread_id,
            Some(&conv.thread_id),
            clip(1.0, false, Some("both")),
        )
        .await
        .unwrap();
    assert_eq!(chip.kind, ChipKind::Audio);
    let ChipPayload::Text { text } = &chip.payload else {
        panic!()
    };
    assert!(text.contains("Você: texto ditado de exemplo"), "{text}");

    // Attached before the conversation existed (draft): the files move into
    // the workspace of the conversation that sends it.
    let draft = host
        .attach_recent("draft", None, clip(1.0, true, None))
        .await
        .unwrap();
    let ChipPayload::Images { paths, .. } = &draft.payload else {
        panic!()
    };
    let draft_dir = paths[0].parent().unwrap().to_path_buf();
    assert!(!draft_dir.starts_with(&conv.workspace));
    host.move_tray("draft", &conv.thread_id);
    host.send(SendRequest {
        thread_id: conv.thread_id.clone(),
        text: "o que aconteceu?".into(),
        tray: conv.thread_id.clone(),
        accepts_images: true,
        options: Default::default(),
    })
    .await
    .unwrap();
    let moved = conv
        .workspace
        .join("clips")
        .join(draft_dir.file_name().unwrap())
        .join("frame-01.png");
    assert!(moved.exists(), "{moved:?}");
    assert!(!draft_dir.exists());

    // Invalid requests and sources without a buffer.
    for bad in [
        clip(1.0, false, None),
        clip(0.0, true, None),
        clip(31.0, true, None),
    ] {
        assert_eq!(
            host.attach_recent("draft", None, bad)
                .await
                .unwrap_err()
                .code,
            "invalid"
        );
    }
    assert_eq!(
        host.attach_recent("draft", None, clip(1.0, false, Some("system")))
            .await
            .unwrap_err()
            .code,
        "not_recording"
    );
    let log = host.access_log(5).unwrap();
    assert!(
        log.iter()
            .any(|e| e.requester == "user" && e.source == "screen")
    );
    host.set_paused(true).unwrap();
    assert_eq!(
        host.attach_recent("draft", None, clip(1.0, true, None))
            .await
            .unwrap_err()
            .code,
        "paused"
    );
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
    // QA-032: explicit duration (≈ the 700 ms recorded), playable copies in
    // the cache and attachment to a conversation.
    let ms = recs[0].duration_ms.expect("duration");
    assert!((400..=1500).contains(&ms), "{ms} ms");
    let media = host.recording_playback(&id).unwrap();
    let cache = host.paths.captures_tmp();
    assert!(
        media
            .iter()
            .all(|m| m.path.starts_with(&cache) && m.path.exists())
    );
    assert!(
        media
            .iter()
            .any(|m| m.kind == "audio" && m.source == "mic" && m.mime == "audio/wav"),
        "{media:?}"
    );
    assert!(
        media
            .iter()
            .any(|m| m.kind == "video" && m.source == "screen" && m.mime == "video/mp4"),
        "{media:?}"
    );
    let conv = host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    let attached = host
        .recording_attach(&id, "draft", Some(&conv.thread_id))
        .await
        .unwrap();
    assert!(
        attached
            .chips
            .iter()
            .any(|c| c.label.starts_with("mic.wav")),
        "{attached:?}"
    );
    host.recording_delete(&id).unwrap();
    assert!(host.recordings().is_empty());
    assert!(
        media.iter().all(|m| !m.path.exists()),
        "playback copies go with the recording"
    );
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

#[tokio::test]
async fn access_log_links_the_conversation_and_opens_the_thumbnail() {
    // QA-031: file-backed store so the test can write an entry the way the
    // agent tools do (same database, same vault key).
    let dir = tempfile::tempdir().unwrap();
    let (platform, _fg) = Platform::fake();
    let paths = AppPaths::new(dir.path().join("Aura"));
    let mut cfg = HostConfig::demo(paths.clone(), platform);
    cfg.siwc.preferred_port = 0;
    let host = Host::start(cfg).await.unwrap();
    let conv = host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();

    let store = aura_store::Store::open(&paths.db()).unwrap();
    let vault =
        aura_store::Vault::open(&store, &aura_store::StaticKeyProtector::new([7u8; 32])).unwrap();
    let repo = aura_app::privacy::PrivacyRepo::new(store);
    let req = aura_policy::AccessRequest {
        source: Source::Screen,
        requester: aura_policy::Requester::Agent {
            tool: "screen_capture".into(),
            conversation: conv.conversation_uuid.clone(),
        },
        target: aura_policy::Target::Range,
        visible_windows: vec![],
        background: false,
    };
    let id = repo.log(&req, &aura_policy::Decision::Allow).unwrap();
    let png = b"\x89PNG-thumb".to_vec();
    let sealed = vault.seal_bytes("access-thumb", &png).unwrap();
    repo.complete(id, None, None, Some(&sealed)).unwrap();

    let entry = host.access_log(10).unwrap().remove(0);
    assert_eq!(entry.id, id);
    assert_eq!(entry.thread_id.as_deref(), Some(conv.thread_id.as_str()));
    assert!(entry.has_thumbnail);
    // "iVBORy10aHVtYg==" is base64 of the literal bytes above.
    assert_eq!(
        host.access_thumbnail(id).unwrap().as_deref(),
        Some("data:image/png;base64,iVBORy10aHVtYg==")
    );

    let mut rx = host.subscribe();
    host.reveal_conversation(&conv.thread_id).unwrap();
    let HostEvent::OpenConversation { thread_id } =
        until(&mut rx, |e| matches!(e, HostEvent::OpenConversation { .. })).await
    else {
        unreachable!()
    };
    assert_eq!(thread_id, conv.thread_id);
    assert_eq!(
        host.reveal_conversation("nao-existe").unwrap_err().code,
        "not_found"
    );
}

#[tokio::test]
async fn speech_voice_cloud_consent_and_auto_read() {
    use axum::routing::post;
    let e = env().await;
    // Offline voices of the platform: chosen voice, no consent needed.
    let opts = e.host.speech_options().unwrap();
    assert_eq!(
        opts.voices
            .iter()
            .map(|v| v.id.as_str())
            .collect::<Vec<_>>(),
        ["fake-pt", "fake-en"]
    );
    e.host
        .update_settings(
            serde_json::from_value(json!({"ttsVoice": "fake-en", "autoRead": true})).unwrap(),
        )
        .unwrap();
    assert!(e.host.settings().auto_read);
    assert!(e.host.speak("Olá").await.is_ok());
    e.host
        .update_settings(serde_json::from_value(json!({"ttsVoice": "nope"})).unwrap())
        .unwrap();
    assert_eq!(e.host.speak("Olá").await.unwrap_err().code, "speech");

    // Cloud voice of a BYOK provider: nothing is sent before the one-time
    // confirmation (009 AC-006).
    let bodies = Arc::new(std::sync::Mutex::new(Vec::<serde_json::Value>::new()));
    let b2 = bodies.clone();
    let app = axum::Router::new().route(
        "/v1/audio/speech",
        post(move |axum::Json(body): axum::Json<serde_json::Value>| {
            let b2 = b2.clone();
            async move {
                b2.lock().unwrap().push(body);
                ([("content-type", "audio/wav")], b"RIFFqa-voice".to_vec())
            }
        }),
    );
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/v1", l.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    let p = e
        .host
        .save_provider(
            serde_json::from_value(
                json!({"name": "Voz QA", "preset": "custom", "wire": "chat", "baseUrl": base}),
            )
            .unwrap(),
            None,
        )
        .unwrap();
    let opts = e.host.speech_options().unwrap();
    assert_eq!(
        opts.cloud.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        [p.id.as_str()]
    );
    e.host
        .update_settings(
            serde_json::from_value(
                json!({"ttsVoice": null, "ttsProvider": p.id, "ttsCloudVoice": "nova"}),
            )
            .unwrap(),
        )
        .unwrap();
    let err = e.host.speak("**Olá**, mundo").await.unwrap_err();
    assert_eq!(err.code, "consent_required");
    assert!(err.message.contains("Voz QA"), "{}", err.message);
    assert!(
        bodies.lock().unwrap().is_empty(),
        "nothing sent before consent"
    );
    e.host.speech_consent().unwrap();
    let (b64, mime) = e.host.speak("**Olá**, mundo").await.unwrap();
    assert_eq!(mime, "audio/wav");
    assert_eq!(b64, "UklGRnFhLXZvaWNl"); // base64("RIFFqa-voice")
    assert_eq!(
        bodies.lock().unwrap()[0],
        json!({"model": "tts-1", "voice": "nova", "input": "Olá, mundo", "response_format": "wav"})
    );
    assert_eq!(e.host.settings().tts_cloud_consent, vec![p.id.clone()]);
    // Removing the provider forgets its voice and the consent.
    e.host.remove_provider(&p.id).unwrap();
    let s = e.host.settings();
    assert_eq!((s.tts_provider, s.tts_cloud_consent), (None, vec![]));
}

#[tokio::test]
async fn diagnostics_show_the_whole_pipeline() {
    // 010 AC-011: app-server, Gateway, MCP, worker, active captures, account
    // (no tokens), disk space and versions.
    let e = env().await;
    e.host
        .start_conversation(StartOptions::default())
        .await
        .unwrap();
    e.host
        .set_source_policy(
            Source::Mic,
            CaptureMode::RecentBuffer { minutes: 1 },
            AgentPermission::Never,
        )
        .unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    std::fs::write(e.host.paths.root.join("qa-size.bin"), vec![0u8; 1234]).unwrap();
    let d = e.host.diagnostics().await;
    assert!(
        d.gateway.reachable,
        "gateway answers on 127.0.0.1:{}",
        d.gateway.port
    );
    assert_eq!(d.gateway.port, d.gateway_port);
    assert_eq!(
        d.mcp
            .iter()
            .map(|m| (m.name.as_str(), m.tools))
            .collect::<Vec<_>>(),
        [("aura", 2)]
    );
    assert!(!d.worker.installed, "demo build has no aura-worker");
    assert!(
        d.capture.active.contains(&"mic".to_string()),
        "{:?}",
        d.capture.active
    );
    assert!(!d.capture.paused);
    assert_eq!(d.account, None);
    assert!(d.disk.aura_bytes >= 1234, "{}", d.disk.aura_bytes);
    let json = serde_json::to_value(&d).unwrap();
    for key in ["gateway", "mcp", "worker", "capture", "account", "disk"] {
        assert!(json.get(key).is_some(), "{key}");
    }
}

#[tokio::test]
async fn selection_chip_follows_the_latest_selection() {
    // 012 AC-001: returning to the Overlay refreshes one selection chip.
    let e = env().await;
    let selections = |host: &Host| -> Vec<String> {
        host.tray("draft")
            .into_iter()
            .filter(|c| c.kind == ChipKind::Selection)
            .map(|c| match c.payload {
                ChipPayload::Text { text } => text,
                _ => String::new(),
            })
            .collect()
    };
    *e.fg.selection.lock().unwrap() = Some("Aura QA seleção".into());
    e.host.capture_selection("draft", false).unwrap();
    e.host.capture_selection("draft", false).unwrap();
    assert_eq!(
        selections(&e.host),
        ["Aura QA seleção"],
        "same text, one chip"
    );

    *e.fg.selection.lock().unwrap() = Some("outro trecho".into());
    e.host.capture_selection("draft", false).unwrap();
    assert_eq!(selections(&e.host), ["outro trecho"], "new text replaces");

    // Removed by the user: not re-added automatically, only on request.
    let id = e.host.tray("draft")[0].id.clone();
    e.host.remove_chip("draft", &id).unwrap();
    assert!(e.host.capture_selection("draft", false).unwrap().is_none());
    assert!(selections(&e.host).is_empty());
    e.host.capture_selection("draft", true).unwrap();
    assert_eq!(selections(&e.host), ["outro trecho"]);

    // Used by a quick command: the same selection can come back.
    e.host
        .expand_quick_command("draft", "/traduzir inglês", "")
        .await
        .unwrap();
    assert!(selections(&e.host).is_empty());
    e.host.capture_selection("draft", false).unwrap();
    assert_eq!(selections(&e.host), ["outro trecho"]);

    // No selection in the other app: nothing changes.
    *e.fg.selection.lock().unwrap() = None;
    assert!(e.host.capture_selection("draft", false).unwrap().is_none());
    assert_eq!(selections(&e.host), ["outro trecho"]);
}

#[tokio::test]
async fn yolo_needs_the_typed_confirmation() {
    // 018 AC-001.
    let e = env().await;
    assert!(!e.host.settings().yolo);
    assert_eq!(e.host.set_yolo(true, "talvez").unwrap_err().code, "invalid");
    assert!(!e.host.settings().yolo);
    assert!(e.host.set_yolo(true, " aceito ").unwrap().yolo);
    assert!(!e.host.set_yolo(false, "").unwrap().yolo);
    assert!(e.host.set_yolo(true, "ACCEPT").unwrap().yolo);
    // A settings patch never turns it off (or on).
    let next = e
        .host
        .update_settings(
            serde_json::from_value(json!({"yolo": false, "hideOnBlur": true})).unwrap(),
        )
        .unwrap();
    assert!(next.yolo && next.hide_on_blur);
}

#[tokio::test]
async fn agent_configures_settings_with_diff_and_undo() {
    // 022 AC-002.
    let e = env().await;
    let (tx, _rx) = tokio::sync::broadcast::channel(16);
    let tools = tools_of(
        &e,
        tx,
        Arc::new(aura_app::consent::ConsentBroker::default()),
    );
    let weak: std::sync::Weak<dyn aura_app::tools::ExtensionsAccess> =
        Arc::downgrade(&e.host) as std::sync::Weak<dyn aura_app::tools::ExtensionsAccess>;
    tools.extensions.set(weak).ok().unwrap();
    let ctx = CallContext {
        conversation: "conv-set".into(),
    };
    let call = |tool: &'static str, args: serde_json::Value| {
        let (tools, ctx) = (&tools, ctx.clone());
        async move { tools.call(tool, args, ctx).await }
    };
    let text = |o: &aura_mcp::ToolOutput| match &o.content[0] {
        aura_mcp::Content::Text(t) => t.clone(),
        _ => panic!("text"),
    };
    let before = e.host.settings();

    // Proposing shows the diff and saves nothing.
    let out = call(
        "settings_propose",
        json!({"changes": {"opacity": 0.7, "hideFromCapture": false}}),
    )
    .await;
    let v: serde_json::Value = serde_json::from_str(&text(&out)).unwrap();
    assert_eq!(v["ok"], true);
    let flagged: Vec<_> = v["changes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["widens_exposure"] == true)
        .map(|c| c["key"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(flagged, ["hideFromCapture"]);
    assert_eq!(e.host.settings(), before);

    // Applying changes them; undo restores exactly the previous values.
    let out = call(
        "settings_apply",
        json!({"changes": {"opacity": 0.7, "autoRead": true}}),
    )
    .await;
    assert!(!out.is_error, "{}", text(&out));
    assert_eq!(e.host.settings().opacity, 0.7);
    assert!(e.host.settings().auto_read);
    let out = call("settings_undo", json!({})).await;
    assert!(!out.is_error, "{}", text(&out));
    assert_eq!(e.host.settings(), before);
    assert!(call("settings_undo", json!({})).await.is_error);

    // Out of reach: YOLO and shortcuts; invalid values are explained.
    for bad in [
        json!({"yolo": true}),
        json!({"invokeShortcut": "Ctrl+Q"}),
        json!({"opacity": 0.1}),
    ] {
        assert!(
            call("settings_apply", json!({"changes": bad}))
                .await
                .is_error
        );
    }
    assert!(!e.host.settings().yolo);
    assert_eq!(e.host.settings(), before);
}

#[tokio::test]
async fn reminders_and_notes_through_agent_tools() {
    // 020 AC-002/AC-003.
    let e = env().await;
    let (tx, _rx) = tokio::sync::broadcast::channel(16);
    let tools = tools_of(
        &e,
        tx,
        Arc::new(aura_app::consent::ConsentBroker::default()),
    );
    let weak: std::sync::Weak<dyn aura_app::tools::ExtensionsAccess> =
        Arc::downgrade(&e.host) as std::sync::Weak<dyn aura_app::tools::ExtensionsAccess>;
    tools.extensions.set(weak).ok().unwrap();
    let ctx = CallContext {
        conversation: "conv-rem".into(),
    };
    let call = |tool: &'static str, args: serde_json::Value| {
        let (tools, ctx) = (&tools, ctx.clone());
        async move { tools.call(tool, args, ctx).await }
    };
    let text = |o: &aura_mcp::ToolOutput| match &o.content[0] {
        aura_mcp::Content::Text(t) => t.clone(),
        _ => panic!("text"),
    };
    let mut events = e.host.subscribe();

    // The model can read the local clock.
    assert!(text(&call("clock_now", json!({})).await).contains('T'));

    let out = call(
        "reminder_create",
        json!({"text": "ligar para o João", "delay_minutes": 5}),
    )
    .await;
    assert!(!out.is_error, "{}", text(&out));
    // Not due yet, then due: the Overlay is told once.
    assert_eq!(e.host.tick_reminders(), 0);
    let later = aura_store::now_secs() + 6 * 60;
    assert_eq!(e.host.tick_reminders_at(later, 0), 1);
    assert_eq!(e.host.tick_reminders_at(later + 60, 0), 0);
    until(
        &mut events,
        |ev| matches!(ev, HostEvent::Reminder { text, .. } if text == "ligar para o João"),
    )
    .await;

    // Bad input is explained.
    assert!(call("reminder_create", json!({"text": "x"})).await.is_error);
    assert!(
        call("reminder_create", json!({"text": "x", "at": "ontem"}))
            .await
            .is_error
    );

    let listed = call(
        "reminder_create",
        json!({"text": "pagar o aluguel", "delay_minutes": 60, "repeat": "weekly"}),
    )
    .await;
    assert!(!listed.is_error);
    let v: serde_json::Value =
        serde_json::from_str(&text(&call("reminder_list", json!({})).await)).unwrap();
    let id = v[0]["id"].as_str().unwrap().to_string();
    assert_eq!(v[0]["repeat"], "weekly");
    assert!(!call("reminder_delete", json!({"id": id})).await.is_error);
    assert_eq!(text(&call("reminder_list", json!({})).await), "[]");

    // Notes: save and find by words, saved answers are a separate list.
    assert!(
        !call("note_save", json!({"text": "renovar o seguro em março"}))
            .await
            .is_error
    );
    assert!(
        !call(
            "note_save",
            json!({"text": "receita de bolo", "kind": "saved"})
        )
        .await
        .is_error
    );
    let hits = text(&call("note_search", json!({"query": "seguro marco"})).await);
    assert!(hits.contains("renovar o seguro"));
    assert_eq!(
        text(&call("note_search", json!({"query": "bolo"})).await),
        "[]"
    );
    assert!(
        text(&call("note_search", json!({"query": "bolo", "kind": "saved"})).await)
            .contains("bolo")
    );
}
