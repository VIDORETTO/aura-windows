//! Tauri commands: thin, typed forwards to `aura_app::Host`. Names match the
//! TypeScript client in `apps/desktop/src/ipc/commands.ts`.

use crate::overlay;
use aura_app::HostError;
use aura_app::attachments::AttachmentInfo;
use aura_app::consent::ConsentAnswer;
use aura_app::host::{AuthStatus, Diagnostics, Host, PrivacyView, SendRequest};
use aura_app::privacy::AccessLogEntry;
use aura_app::voice::ModelView;
use aura_asr::ptt::PttState;
use aura_codex::approvals::Decision;
use aura_codex::models::ModelInfo;
use aura_codex::modes::ConversationMode;
use aura_codex::service::{
    HistoryPage, HistoryQuery, StartOptions, StartedConversation, TranscriptMessage,
};
use aura_core::context::ContextChip;
use aura_core::placement::OverlayMode;
use aura_core::settings::{Settings, SettingsPatch};
use aura_extensions::import::DetectedServer;
use aura_extensions::mcp_config::McpServerSpec;
use aura_extensions::quick::{Expansion, QuickCommand};
use aura_extensions::skills::SkillReview;
use aura_gateway::registry::{Preset, Provider, ProviderDraft};
use aura_policy::{AgentPermission, CaptureMode, ExclusionRule, Source};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};

pub struct AppState {
    pub host: Arc<Host>,
}

type R<T> = Result<T, HostError>;

// ---------------------------------------------------------------- settings

#[tauri::command]
pub fn settings_get(s: State<'_, AppState>) -> Settings {
    s.host.settings()
}

#[tauri::command]
pub fn settings_update(
    app: AppHandle,
    s: State<'_, AppState>,
    patch: SettingsPatch,
) -> R<Settings> {
    let next = s.host.update_settings(patch)?;
    crate::apply_settings(&app, &next);
    Ok(next)
}

// ----------------------------------------------------------------- privacy

#[tauri::command]
pub fn privacy_get(s: State<'_, AppState>) -> PrivacyView {
    s.host.privacy()
}

#[tauri::command]
pub fn privacy_set_source(
    s: State<'_, AppState>,
    source: Source,
    mode: CaptureMode,
    agent: AgentPermission,
) -> R<PrivacyView> {
    s.host.set_source_policy(source, mode, agent)
}

#[tauri::command]
pub fn privacy_set_retention(
    s: State<'_, AppState>,
    days: u32,
    max_gb: u32,
    apply_to_manual: bool,
) -> R<PrivacyView> {
    s.host.set_retention(days, max_gb, apply_to_manual)
}

#[tauri::command]
pub fn privacy_set_paused(s: State<'_, AppState>, paused: bool) -> R<PrivacyView> {
    s.host.set_paused(paused)
}

#[tauri::command]
pub fn privacy_upsert_exclusion(s: State<'_, AppState>, rule: ExclusionRule) -> R<PrivacyView> {
    s.host.upsert_exclusion(rule)
}

#[tauri::command]
pub fn privacy_remove_exclusion(s: State<'_, AppState>, id: String) -> R<PrivacyView> {
    s.host.remove_exclusion(&id)
}

#[tauri::command]
pub fn privacy_access_log(s: State<'_, AppState>, limit: u32) -> R<Vec<AccessLogEntry>> {
    s.host.access_log(limit)
}

#[tauri::command]
pub fn privacy_access_thumbnail(s: State<'_, AppState>, id: i64) -> R<Option<String>> {
    s.host.access_thumbnail(id)
}

/// Access-log link: shows the Overlay with that conversation.
#[tauri::command]
pub fn privacy_open_conversation(
    app: AppHandle,
    s: State<'_, AppState>,
    thread_id: String,
) -> R<()> {
    s.host.reveal_conversation(&thread_id)?;
    overlay::show(&app);
    Ok(())
}

#[tauri::command]
pub fn privacy_clear_access_log(s: State<'_, AppState>) -> R<()> {
    s.host.clear_access_log()
}

#[tauri::command]
pub fn consent_answer(s: State<'_, AppState>, id: String, answer: ConsentAnswer) -> R<()> {
    s.host.answer_consent(&id, answer)
}

// -------------------------------------------------------------------- auth

#[tauri::command]
pub fn auth_status(s: State<'_, AppState>) -> R<AuthStatus> {
    s.host.auth_status()
}

#[tauri::command]
pub async fn auth_login(
    app: AppHandle,
    s: State<'_, AppState>,
    reauthorize: Option<String>,
    force_consent: bool,
) -> R<()> {
    let url = s.host.login(reauthorize, force_consent).await?;
    crate::open_url(&app, &url);
    Ok(())
}

#[tauri::command]
pub async fn auth_cancel(s: State<'_, AppState>) -> R<()> {
    s.host.cancel_login().await;
    Ok(())
}

#[tauri::command]
pub async fn auth_logout(s: State<'_, AppState>, client_id: String) -> R<bool> {
    s.host.logout(&client_id).await
}

#[tauri::command]
pub fn auth_switch(s: State<'_, AppState>, client_id: String) -> R<()> {
    s.host.switch_account(&client_id)
}

#[tauri::command]
pub fn auth_mark_welcomed(s: State<'_, AppState>, client_id: String) -> R<()> {
    s.host.mark_welcomed(&client_id)
}

// --------------------------------------------------------------- providers

#[tauri::command]
pub fn providers_presets(s: State<'_, AppState>) -> Vec<Preset> {
    s.host.provider_presets()
}

#[tauri::command]
pub fn providers_list(s: State<'_, AppState>) -> R<Vec<Provider>> {
    s.host.providers()
}

#[tauri::command]
pub fn providers_save(
    s: State<'_, AppState>,
    draft: ProviderDraft,
    credential: Option<String>,
) -> R<Provider> {
    s.host.save_provider(draft, credential)
}

#[tauri::command]
pub fn providers_remove(app: AppHandle, s: State<'_, AppState>, id: String) -> R<()> {
    s.host.remove_provider(&id)?;
    // Voice/transcription choices that pointed at it were cleared.
    crate::apply_settings(&app, &s.host.settings());
    Ok(())
}

#[tauri::command]
pub fn providers_model_save(
    s: State<'_, AppState>,
    id: String,
    model: aura_gateway::registry::ModelSpec,
) -> R<aura_gateway::registry::Provider> {
    s.host.save_provider_model(&id, model)
}

#[tauri::command]
pub fn providers_model_remove(
    s: State<'_, AppState>,
    id: String,
    model_id: String,
) -> R<aura_gateway::registry::Provider> {
    s.host.remove_provider_model(&id, &model_id)
}
#[tauri::command]
pub async fn providers_test(s: State<'_, AppState>, id: String) -> R<Provider> {
    s.host.test_provider(&id).await
}

// ----------------------------------------------------------- conversations

#[tauri::command]
pub async fn conversation_start(
    s: State<'_, AppState>,
    options: StartOptions,
) -> R<StartedConversation> {
    s.host.start_conversation(options).await
}

#[tauri::command]
pub async fn conversation_send(s: State<'_, AppState>, request: SendRequest) -> R<String> {
    s.host.send(request).await
}

#[tauri::command]
pub async fn conversation_steer(
    s: State<'_, AppState>,
    thread_id: String,
    text: String,
    tray: String,
) -> R<String> {
    s.host.steer(&thread_id, &text, &tray).await
}

#[tauri::command]
pub async fn conversation_interrupt(s: State<'_, AppState>, thread_id: String) -> R<()> {
    s.host.interrupt(&thread_id).await
}

#[tauri::command]
pub async fn conversation_compact(s: State<'_, AppState>, thread_id: String) -> R<()> {
    s.host.compact(&thread_id).await
}

#[tauri::command]
pub async fn conversation_set_mode(
    s: State<'_, AppState>,
    thread_id: String,
    mode: ConversationMode,
) -> R<()> {
    s.host.set_mode(&thread_id, mode).await
}

#[tauri::command]
pub async fn conversation_history(s: State<'_, AppState>, query: HistoryQuery) -> R<HistoryPage> {
    s.host.history(query).await
}

#[tauri::command]
pub async fn conversation_open(
    s: State<'_, AppState>,
    thread_id: String,
) -> R<Vec<TranscriptMessage>> {
    s.host.open_conversation(&thread_id).await
}

#[tauri::command]
pub async fn conversation_rename(s: State<'_, AppState>, thread_id: String, name: String) -> R<()> {
    s.host.rename(&thread_id, &name).await
}

#[tauri::command]
pub async fn conversation_pin(s: State<'_, AppState>, thread_id: String, pinned: bool) -> R<()> {
    s.host.pin(&thread_id, pinned).await
}

#[tauri::command]
pub async fn conversation_archive(s: State<'_, AppState>, thread_id: String) -> R<()> {
    s.host.archive(&thread_id).await
}

#[tauri::command]
pub async fn conversation_unarchive(s: State<'_, AppState>, thread_id: String) -> R<()> {
    s.host.unarchive(&thread_id).await
}

#[tauri::command]
pub async fn conversation_delete(s: State<'_, AppState>, thread_id: String) -> R<()> {
    s.host.delete_conversation(&thread_id).await
}

#[tauri::command]
pub async fn conversation_close_ephemeral(s: State<'_, AppState>, thread_id: String) -> R<()> {
    s.host.close_ephemeral(&thread_id).await
}

#[tauri::command]
pub async fn conversation_respond(
    s: State<'_, AppState>,
    request_id: String,
    decision: Decision,
) -> R<()> {
    s.host.respond(&request_id, decision).await
}

#[tauri::command]
pub async fn models_list(s: State<'_, AppState>) -> R<Vec<ModelInfo>> {
    s.host.models().await
}

// ----------------------------------------------------------------- context

#[tauri::command]
pub fn tray_list(s: State<'_, AppState>, tray: String) -> Vec<ContextChip> {
    s.host.tray(&tray)
}

#[tauri::command]
pub fn tray_remove(s: State<'_, AppState>, tray: String, chip_id: String) -> R<Vec<ContextChip>> {
    s.host.remove_chip(&tray, &chip_id)
}

#[tauri::command]
pub fn tray_move(s: State<'_, AppState>, from: String, to: String) {
    s.host.move_tray(&from, &to)
}

#[tauri::command]
pub async fn capture_screen(
    s: State<'_, AppState>,
    tray: String,
    window_only: bool,
) -> R<ContextChip> {
    s.host.capture_screen(&tray, window_only).await
}

/// "@últimos minutos": the user attaches the recent buffer as a Clip.
#[tauri::command]
pub async fn context_attach_recent(
    s: State<'_, AppState>,
    tray: String,
    thread_id: Option<String>,
    clip: aura_app::host::RecentClip,
) -> R<ContextChip> {
    s.host
        .attach_recent(&tray, thread_id.as_deref(), clip)
        .await
}

/// Skill chosen in the `/` menu: a Chip, not `$name` text (015).
#[tauri::command]
pub async fn context_attach_skill(
    s: State<'_, AppState>,
    tray: String,
    name: String,
) -> R<ContextChip> {
    s.host.attach_skill(&tray, &name).await
}

#[tauri::command]
pub fn capture_selection(
    s: State<'_, AppState>,
    tray: String,
    explicit: Option<bool>,
) -> R<Option<ContextChip>> {
    s.host.capture_selection(&tray, explicit.unwrap_or(false))
}

#[tauri::command]
pub async fn attach_file(
    s: State<'_, AppState>,
    tray: String,
    thread_id: Option<String>,
    path: PathBuf,
) -> R<(AttachmentInfo, ContextChip)> {
    s.host.attach(&tray, thread_id.as_deref(), &path).await
}

/// Ctrl+V image: base64 from the webview clipboard event.
#[tauri::command]
pub async fn attach_clipboard_image(
    s: State<'_, AppState>,
    tray: String,
    thread_id: Option<String>,
    mime: String,
    data: String,
) -> R<(AttachmentInfo, ContextChip)> {
    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data.as_bytes())
        .map_err(|e| HostError::new("attachment", e.to_string()))?;
    s.host
        .attach_clipboard_image(&tray, thread_id.as_deref(), &mime, &bytes)
        .await
}

#[tauri::command]
pub fn attachments_list(s: State<'_, AppState>, thread_id: String) -> Vec<AttachmentInfo> {
    s.host.attachments(&thread_id)
}

#[tauri::command]
pub fn insert_into_app(app: AppHandle, s: State<'_, AppState>, text: String) -> bool {
    overlay::hide(&app);
    s.host.insert_into_app(&text)
}

// ---------------------------------------------------------- quick commands

#[tauri::command]
pub fn quick_list(s: State<'_, AppState>) -> R<Vec<QuickCommand>> {
    s.host.quick_commands()
}

#[tauri::command]
pub fn quick_save(
    s: State<'_, AppState>,
    name: String,
    template: String,
    replace: bool,
) -> R<Vec<QuickCommand>> {
    s.host.save_quick_command(&name, &template, replace)
}

#[tauri::command]
pub fn quick_delete(s: State<'_, AppState>, name: String) -> R<Vec<QuickCommand>> {
    s.host.delete_quick_command(&name)
}

#[tauri::command]
pub fn quick_toggle(s: State<'_, AppState>, name: String, enabled: bool) -> R<Vec<QuickCommand>> {
    s.host.toggle_quick_command(&name, enabled)
}

#[tauri::command]
pub async fn quick_expand(
    s: State<'_, AppState>,
    tray: String,
    input: String,
    typed: String,
) -> R<Expansion> {
    s.host.expand_quick_command(&tray, &input, &typed).await
}

// --------------------------------------------------------------------- MCP

#[tauri::command]
pub fn mcp_list(s: State<'_, AppState>) -> R<Vec<McpServerSpec>> {
    s.host.mcp_servers()
}

#[tauri::command]
pub fn mcp_save(
    s: State<'_, AppState>,
    spec: McpServerSpec,
    secrets: Vec<(String, String)>,
    bearer: Option<String>,
) -> R<Vec<McpServerSpec>> {
    s.host.save_mcp_server(spec, secrets, bearer)
}

#[tauri::command]
pub fn mcp_delete(s: State<'_, AppState>, name: String) -> R<Vec<McpServerSpec>> {
    s.host.delete_mcp_server(&name)
}

#[tauri::command]
pub fn mcp_detect(s: State<'_, AppState>) -> Vec<DetectedServer> {
    s.host.detect_mcp_imports()
}

#[tauri::command]
pub fn mcp_import(s: State<'_, AppState>, names: Vec<String>) -> R<Vec<McpServerSpec>> {
    // Re-detect so secrets never travel through the UI.
    let detected = s.host.detect_mcp_imports();
    s.host.import_mcp_servers(detected, &names)
}

#[tauri::command]
pub async fn mcp_diagnose(
    s: State<'_, AppState>,
    name: String,
) -> R<aura_app::mcp_diag::McpDiagnosis> {
    s.host.mcp_diagnose(&name).await
}

#[tauri::command]
pub async fn mcp_status(s: State<'_, AppState>) -> R<Vec<aura_app::host::McpStatus>> {
    s.host.mcp_status().await
}

#[tauri::command]
pub async fn mcp_login(app: AppHandle, s: State<'_, AppState>, name: String) -> R<()> {
    if let Some(url) = s.host.mcp_login(&name).await? {
        crate::open_url(&app, &url);
    }
    Ok(())
}

#[tauri::command]
pub fn workspace_files(
    s: State<'_, AppState>,
    thread_id: String,
) -> Vec<aura_app::host::WorkspaceFile> {
    s.host.workspace_files(&thread_id)
}

#[tauri::command]
pub fn workspace_read(s: State<'_, AppState>, thread_id: String, path: String) -> R<String> {
    s.host.read_workspace_file(&thread_id, &path)
}

#[tauri::command]
pub fn workspace_pdf_preview(
    s: State<'_, AppState>,
    thread_id: String,
    path: String,
) -> R<aura_app::host::PdfPreview> {
    s.host.workspace_pdf_preview(&thread_id, &path)
}

/// Opens a file with its default app, or reveals it in Explorer.
#[tauri::command]
pub fn open_path(app: AppHandle, path: PathBuf, reveal: bool) -> R<()> {
    use tauri_plugin_opener::OpenerExt;
    let r = if reveal {
        app.opener().reveal_item_in_dir(&path)
    } else {
        app.opener().open_path(path.to_string_lossy(), None::<&str>)
    };
    r.map_err(|e| HostError::new("open", e.to_string()))
}

#[tauri::command]
pub async fn agent_restart(s: State<'_, AppState>) -> R<bool> {
    Ok(s.host.restart_agent().await)
}

// ------------------------------------------------------------------ skills

#[tauri::command]
pub fn skills_list(s: State<'_, AppState>) -> Vec<SkillReview> {
    s.host.skills()
}

#[tauri::command]
pub fn skills_review(s: State<'_, AppState>, path: PathBuf) -> R<SkillReview> {
    s.host.review_skill(&path)
}

#[tauri::command]
pub fn skills_install(s: State<'_, AppState>, path: PathBuf, overwrite: bool) -> R<PathBuf> {
    s.host.install_skill(&path, overwrite)
}

#[tauri::command]
pub fn skills_create(
    s: State<'_, AppState>,
    name: String,
    description: String,
    body: String,
) -> R<PathBuf> {
    s.host.create_skill(&name, &description, &body)
}

#[tauri::command]
pub async fn skills_catalog(s: State<'_, AppState>) -> R<Vec<aura_app::host::SkillEntry>> {
    s.host.skills_catalog().await
}

#[tauri::command]
pub async fn skills_set_enabled(s: State<'_, AppState>, path: PathBuf, enabled: bool) -> R<()> {
    s.host.set_skill_enabled(&path, enabled).await
}

#[tauri::command]
pub fn memories_get(s: State<'_, AppState>) -> R<aura_app::host::MemoryView> {
    s.host.memories()
}

#[tauri::command]
pub fn memories_forget_fact(s: State<'_, AppState>, fact: String) -> R<aura_app::host::MemoryView> {
    s.host.forget_memory_fact(&fact)
}

#[tauri::command]
pub fn memories_save(
    s: State<'_, AppState>,
    summary: String,
    registry: String,
) -> R<aura_app::host::MemoryView> {
    s.host.save_memories(&summary, &registry)
}

#[tauri::command]
pub fn memories_forget_all(s: State<'_, AppState>) -> R<()> {
    s.host.forget_all_memories()
}

#[tauri::command]
pub fn skills_source(s: State<'_, AppState>, name: String) -> R<(String, String)> {
    s.host.skill_source(&name)
}

#[tauri::command]
pub fn skills_update(
    s: State<'_, AppState>,
    name: String,
    description: String,
    body: String,
) -> R<()> {
    s.host.update_skill(&name, &description, &body)
}
#[tauri::command]
pub fn skills_delete(s: State<'_, AppState>, name: String) -> R<()> {
    s.host.delete_skill(&name)
}

// -------------------------------------------------------------- recordings

#[tauri::command]
pub async fn recording_start(s: State<'_, AppState>, title: String) -> R<String> {
    s.host.recording_start(&title).await
}

#[tauri::command]
pub async fn recording_stop(s: State<'_, AppState>) -> R<Option<String>> {
    Ok(s.host.recording_stop().await)
}

#[tauri::command]
pub fn recordings_list(s: State<'_, AppState>) -> Vec<aura_app::recorder::Recording> {
    s.host.recordings()
}

#[tauri::command]
pub fn recording_active(s: State<'_, AppState>) -> Option<String> {
    s.host.recording_active()
}

#[tauri::command]
pub fn recording_delete(s: State<'_, AppState>, id: String) -> R<()> {
    s.host.recording_delete(&id)
}

#[tauri::command]
pub fn recording_playback(
    s: State<'_, AppState>,
    id: String,
) -> R<Vec<aura_app::host::PlaybackMedia>> {
    s.host.recording_playback(&id)
}

/// Attaches a recording to the Overlay's current conversation (draft tray,
/// moved into the open thread by the Overlay) and shows the Overlay.
#[tauri::command]
pub async fn recording_attach(
    app: AppHandle,
    s: State<'_, AppState>,
    id: String,
) -> R<aura_app::host::RecordingAttach> {
    let res = s.host.recording_attach(&id, "draft", None).await?;
    overlay::show(&app);
    let _ = tauri::Emitter::emit_to(&app, overlay::LABEL, "aura://chips-changed", ());
    Ok(res)
}

#[tauri::command]
pub fn recording_export(s: State<'_, AppState>, id: String, dir: PathBuf) -> R<Vec<PathBuf>> {
    s.host.recording_export(&id, &dir)
}

#[tauri::command]
pub async fn attachment_read(
    s: State<'_, AppState>,
    thread_id: String,
    id: String,
    selector: aura_ingest::Selector,
) -> R<String> {
    s.host.read_attachment(&thread_id, &id, selector).await
}

// ------------------------------------------------------------------- voice

#[tauri::command]
pub fn voice_models(s: State<'_, AppState>) -> Vec<ModelView> {
    s.host.voice_models()
}

#[tauri::command]
pub fn voice_install(s: State<'_, AppState>, id: String) -> R<()> {
    s.host.install_voice_model(&id)
}

#[tauri::command]
pub fn voice_cancel_install(s: State<'_, AppState>, id: String) {
    s.host.cancel_voice_model(&id)
}

#[tauri::command]
pub fn voice_remove(s: State<'_, AppState>, id: String) -> R<()> {
    s.host.remove_voice_model(&id)
}

#[tauri::command]
pub fn voice_select(s: State<'_, AppState>, id: String) -> R<()> {
    s.host.select_voice_model(&id)
}

#[tauri::command]
pub async fn ptt_press(s: State<'_, AppState>, device: Option<String>) -> R<()> {
    s.host.ptt_press(device).await
}

#[tauri::command]
pub async fn ptt_release(
    s: State<'_, AppState>,
    language: Option<String>,
    vocabulary: Vec<String>,
) -> R<PttState> {
    Ok(s.host.ptt_release(language, vocabulary).await)
}

#[tauri::command]
pub async fn ptt_cancel(s: State<'_, AppState>) -> R<PttState> {
    Ok(s.host.ptt_cancel().await)
}

#[tauri::command]
pub async fn audio_test_start(
    s: State<'_, AppState>,
    source: aura_audio::AudioSourceKind,
    device: Option<String>,
) -> R<()> {
    s.host.audio_test_start(source, device).await
}

#[tauri::command]
pub async fn audio_test_stop(s: State<'_, AppState>, source: aura_audio::AudioSourceKind) -> R<()> {
    s.host.audio_test_stop(source).await;
    Ok(())
}

#[tauri::command]
pub fn audio_devices(s: State<'_, AppState>, system: bool) -> Vec<aura_app::host::AudioDevice> {
    s.host.audio_devices(system)
}

// ---------------------------------------------------------------- profiles

#[tauri::command]
pub fn profiles_list(s: State<'_, AppState>) -> R<Vec<aura_app::profiles::AppProfile>> {
    s.host.profiles()
}

#[tauri::command]
pub fn profiles_save(
    s: State<'_, AppState>,
    profile: aura_app::profiles::AppProfile,
) -> R<aura_app::profiles::AppProfile> {
    s.host.save_profile(profile)
}

#[tauri::command]
pub fn profiles_delete(s: State<'_, AppState>, id: String) -> R<()> {
    s.host.delete_profile(&id)
}

#[tauri::command]
pub fn profile_active(s: State<'_, AppState>) -> Option<aura_app::profiles::AppProfile> {
    s.host.active_profile()
}

#[tauri::command]
pub fn previous_app(s: State<'_, AppState>) -> Option<aura_core::events::PreviousApp> {
    s.host.previous_app()
}

// ------------------------------------------------------------------ region

/// Freezes the screen and opens the full-screen region selector over it.
#[tauri::command]
pub async fn region_open(app: AppHandle, s: State<'_, AppState>) -> R<()> {
    let frozen = s.host.region_begin().await?;
    crate::open_region_selector(&app, &frozen)
}

#[tauri::command]
pub fn region_commit(
    app: AppHandle,
    s: State<'_, AppState>,
    token: String,
    rect: aura_core::placement::Rect,
    tray: String,
) -> R<ContextChip> {
    crate::close_region_selector(&app);
    let chip = s.host.region_commit(&token, rect, &tray)?;
    overlay::show(&app);
    let _ = tauri::Emitter::emit_to(&app, overlay::LABEL, "aura://chips-changed", ());
    Ok(chip)
}

#[tauri::command]
pub fn region_cancel(app: AppHandle, s: State<'_, AppState>, token: String) {
    crate::close_region_selector(&app);
    s.host.region_cancel(&token);
    overlay::show(&app);
}

// ----------------------------------------------------------------- windows

#[tauri::command]
pub fn overlay_set_mode(app: AppHandle, mode: OverlayMode) {
    overlay::set_mode(&app, mode);
}

#[tauri::command]
pub fn overlay_hide(app: AppHandle) {
    overlay::hide(&app);
}

#[tauri::command]
pub fn overlay_moved(app: AppHandle, s: State<'_, AppState>) {
    overlay::remember_placement(&app, &s.host);
}

// Async: sync commands run on the main thread, and building a window there
// never answers the IPC call on Windows (WebView2).
#[tauri::command]
pub async fn settings_open(app: AppHandle, section: Option<String>) {
    crate::open_settings(&app, section.as_deref());
}

#[tauri::command]
pub fn open_external(app: AppHandle, url: String) -> R<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(HostError::new("invalid", "apenas links http(s)"));
    }
    crate::open_url(&app, &url);
    Ok(())
}

// ------------------------------------------------------------- diagnostics

#[tauri::command]
pub async fn diagnostics(s: State<'_, AppState>) -> R<Diagnostics> {
    Ok(s.host.diagnostics().await)
}

/// "Resume getting started" (010 AC-006, QA-039): the Overlay shows the
/// first-run guide again.
#[tauri::command]
pub fn onboarding_resume(app: AppHandle, s: State<'_, AppState>) -> R<Settings> {
    let next = s.host.update_settings(SettingsPatch {
        onboarded: Some(false),
        ..Default::default()
    })?;
    crate::apply_settings(&app, &next);
    overlay::show(&app);
    Ok(next)
}

/// The build carries a real updater public key (QA-038); with the template
/// placeholder no update can be verified, so the UI says it is not set up.
#[tauri::command]
pub fn updater_configured(app: AppHandle) -> bool {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|u| u.get("pubkey"))
        .and_then(|k| k.as_str())
        .is_some_and(aura_core::release::updater_key_configured)
}

/// Windows toast (answer finished while the user was elsewhere).
#[tauri::command]
pub fn notify(app: AppHandle, title: String, body: String) -> R<()> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| HostError::new("notify", e.to_string()))
}

#[tauri::command]
pub fn speech_options(s: State<'_, AppState>) -> R<aura_app::host::SpeechOptions> {
    s.host.speech_options()
}

/// One-time consent to send answers to the chosen cloud voice (009 AC-006).
#[tauri::command]
pub fn speech_consent(app: AppHandle, s: State<'_, AppState>) -> R<Settings> {
    let next = s.host.speech_consent()?;
    crate::apply_settings(&app, &next);
    Ok(next)
}

#[tauri::command]
pub async fn speak(s: State<'_, AppState>, text: String) -> R<(String, String)> {
    s.host.speak(&text).await
}

#[tauri::command]
pub async fn diagnostics_export(s: State<'_, AppState>, dest: PathBuf) -> R<PathBuf> {
    s.host.export_diagnostics(&dest).await
}

#[tauri::command]
pub async fn erase_all_data(app: AppHandle, s: State<'_, AppState>) -> R<()> {
    s.host.erase_all_data().await?;
    app.restart();
}
