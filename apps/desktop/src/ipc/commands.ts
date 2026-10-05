// Typed client for the Tauri commands in `src-tauri/src/commands.rs`.
// Argument names are camelCase: Tauri maps them to the Rust snake_case params.

import { getBridge } from "./bridge";
import type * as T from "./types";

async function call<R>(cmd: string, args?: Record<string, unknown>): Promise<R> {
  return (await getBridge()).invoke<R>(cmd, args);
}

export const api = {
  // settings
  settingsGet: () => call<T.Settings>("settings_get"),
  settingsUpdate: (patch: T.SettingsPatch) => call<T.Settings>("settings_update", { patch }),
  settingsOpen: (section?: string) => call<void>("settings_open", { section: section ?? null }),

  // privacy
  privacyGet: () => call<T.PrivacyView>("privacy_get"),
  privacySetSource: (source: T.Source, mode: T.CaptureMode, agent: T.AgentPermission) =>
    call<T.PrivacyView>("privacy_set_source", { source, mode, agent }),
  privacySetPaused: (paused: boolean) => call<T.PrivacyView>("privacy_set_paused", { paused }),
  privacySetRetention: (days: number, maxGb: number, applyToManual: boolean) =>
    call<T.PrivacyView>("privacy_set_retention", { days, maxGb, applyToManual }),
  privacyUpsertExclusion: (rule: T.ExclusionRule) => call<T.PrivacyView>("privacy_upsert_exclusion", { rule }),
  privacyRemoveExclusion: (id: string) => call<T.PrivacyView>("privacy_remove_exclusion", { id }),
  privacyAccessLog: (limit = 200) => call<T.AccessLogEntry[]>("privacy_access_log", { limit }),
  privacyClearAccessLog: () => call<void>("privacy_clear_access_log"),
  privacyAccessThumbnail: (id: number) => call<string | null>("privacy_access_thumbnail", { id }),
  privacyOpenConversation: (threadId: string) => call<void>("privacy_open_conversation", { threadId }),
  consentAnswer: (id: string, answer: T.ConsentAnswer) => call<void>("consent_answer", { id, answer }),

  // auth
  authStatus: () => call<T.AuthStatus>("auth_status"),
  authLogin: (reauthorize: string | null = null, forceConsent = false) =>
    call<void>("auth_login", { reauthorize, forceConsent }),
  authCancel: () => call<void>("auth_cancel"),
  authLogout: (clientId: string) => call<boolean>("auth_logout", { clientId }),
  authSwitch: (clientId: string) => call<void>("auth_switch", { clientId }),
  authMarkWelcomed: (clientId: string) => call<void>("auth_mark_welcomed", { clientId }),

  // providers
  providersPresets: () => call<T.Preset[]>("providers_presets"),
  providersList: () => call<T.Provider[]>("providers_list"),
  providersSave: (draft: T.ProviderDraft, credential: string | null) =>
    call<T.Provider>("providers_save", { draft, credential }),
  providersRemove: (id: string) => call<void>("providers_remove", { id }),
  providersTest: (id: string) => call<T.Provider>("providers_test", { id }),
  providersModelSave: (id: string, model: T.ModelSpec) => call<T.Provider>("providers_model_save", { id, model }),
  providersModelRemove: (id: string, modelId: string) => call<T.Provider>("providers_model_remove", { id, modelId }),

  // conversations
  conversationStart: (options: T.StartOptions) => call<T.StartedConversation>("conversation_start", { options }),
  conversationSend: (request: T.SendRequest) => call<string>("conversation_send", { request }),
  conversationSteer: (threadId: string, text: string, tray: string) =>
    call<string>("conversation_steer", { threadId, text, tray }),
  conversationInterrupt: (threadId: string) => call<void>("conversation_interrupt", { threadId }),
  conversationCompact: (threadId: string) => call<void>("conversation_compact", { threadId }),
  conversationSetMode: (threadId: string, mode: T.ConversationMode) =>
    call<void>("conversation_set_mode", { threadId, mode }),
  conversationHistory: (query: T.HistoryQuery = {}) => call<T.HistoryPage>("conversation_history", { query }),
  conversationOpen: (threadId: string) => call<T.TranscriptMessage[]>("conversation_open", { threadId }),
  conversationRename: (threadId: string, name: string) => call<void>("conversation_rename", { threadId, name }),
  conversationPin: (threadId: string, pinned: boolean) => call<void>("conversation_pin", { threadId, pinned }),
  conversationArchive: (threadId: string) => call<void>("conversation_archive", { threadId }),
  conversationUnarchive: (threadId: string) => call<void>("conversation_unarchive", { threadId }),
  conversationDelete: (threadId: string) => call<void>("conversation_delete", { threadId }),
  conversationCloseEphemeral: (threadId: string) => call<void>("conversation_close_ephemeral", { threadId }),
  conversationRespond: (requestId: string, decision: T.ApprovalDecision) =>
    call<void>("conversation_respond", { requestId, decision }),
  modelsList: () => call<T.ModelInfo[]>("models_list"),

  // context
  trayList: (tray: string) => call<T.ContextChip[]>("tray_list", { tray }),
  trayRemove: (tray: string, chipId: string) => call<T.ContextChip[]>("tray_remove", { tray, chipId }),
  trayMove: (from: string, to: string) => call<void>("tray_move", { from, to }),
  captureScreen: (tray: string, windowOnly = false) => call<T.ContextChip>("capture_screen", { tray, windowOnly }),
  contextAttachRecent: (tray: string, threadId: string | null, clip: T.RecentClip) =>
    call<T.ContextChip>("context_attach_recent", { tray, threadId, clip }),
  contextAttachSkill: (tray: string, name: string) => call<T.ContextChip>("context_attach_skill", { tray, name }),
  captureSelection: (tray: string, explicit = false) => call<T.ContextChip | null>("capture_selection", { tray, explicit }),
  attachFile: (tray: string, threadId: string | null, path: string) =>
    call<[T.AttachmentInfo, T.ContextChip]>("attach_file", { tray, threadId, path }),
  attachClipboardImage: (tray: string, threadId: string | null, mime: string, data: string) =>
    call<[T.AttachmentInfo, T.ContextChip]>("attach_clipboard_image", { tray, threadId, mime, data }),
  attachmentsList: (threadId: string) => call<T.AttachmentInfo[]>("attachments_list", { threadId }),
  insertIntoApp: (text: string) => call<boolean>("insert_into_app", { text }),

  // quick commands
  quickList: () => call<T.QuickCommand[]>("quick_list"),
  quickSave: (name: string, template: string, replace: boolean) =>
    call<T.QuickCommand[]>("quick_save", { name, template, replace }),
  quickDelete: (name: string) => call<T.QuickCommand[]>("quick_delete", { name }),
  quickToggle: (name: string, enabled: boolean) => call<T.QuickCommand[]>("quick_toggle", { name, enabled }),
  quickExpand: (tray: string, input: string, typed: string) => call<T.Expansion>("quick_expand", { tray, input, typed }),

  // MCP + skills
  mcpList: () => call<T.McpServerSpec[]>("mcp_list"),
  mcpSave: (spec: T.McpServerSpec, secrets: [string, string][], bearer: string | null) =>
    call<T.McpServerSpec[]>("mcp_save", { spec, secrets, bearer }),
  mcpDelete: (name: string) => call<T.McpServerSpec[]>("mcp_delete", { name }),
  mcpDetect: () => call<T.DetectedServer[]>("mcp_detect"),
  mcpImport: (names: string[]) => call<T.McpServerSpec[]>("mcp_import", { names }),
  agentRestart: () => call<boolean>("agent_restart"),
  /** "Create with AI" (017): the Overlay starts a new conversation and sends `text`. */
  agentTask: (text: string, mode: T.ConversationMode["mode"]) => call<void>("agent_task", { text, mode }),
  mcpStatus: () => call<T.McpStatus[]>("mcp_status"),
  mcpDiagnose: (name: string) => call<T.McpDiagnosis>("mcp_diagnose", { name }),
  mcpLogin: (name: string) => call<void>("mcp_login", { name }),
  workspaceFiles: (threadId: string) => call<T.WorkspaceFile[]>("workspace_files", { threadId }),
  workspaceRead: (threadId: string, path: string) => call<string>("workspace_read", { threadId, path }),
  workspacePdfPreview: (threadId: string, path: string) => call<T.PdfPreview>("workspace_pdf_preview", { threadId, path }),
  openPath: (path: string, reveal = false) => call<void>("open_path", { path, reveal }),
  skillsList: () => call<T.SkillReview[]>("skills_list"),
  skillsReview: (path: string) => call<T.SkillReview>("skills_review", { path }),
  skillsInstall: (path: string, overwrite = false) => call<string>("skills_install", { path, overwrite }),
  skillsCreate: (name: string, description: string, body: string) =>
    call<string>("skills_create", { name, description, body }),
  skillsDelete: (name: string) => call<void>("skills_delete", { name }),
  skillsCatalog: () => call<T.SkillEntry[]>("skills_catalog"),
  memoriesGet: () => call<T.MemoryView>("memories_get"),
  memoriesForgetFact: (fact: string) => call<T.MemoryView>("memories_forget_fact", { fact }),
  memoriesSave: (summary: string, registry: string) => call<T.MemoryView>("memories_save", { summary, registry }),
  memoriesForgetAll: () => call<void>("memories_forget_all"),
  skillsSetEnabled: (path: string, enabled: boolean) => call<void>("skills_set_enabled", { path, enabled }),
  skillsSource: (name: string) => call<[string, string]>("skills_source", { name }),
  skillsUpdate: (name: string, description: string, body: string) => call<void>("skills_update", { name, description, body }),

  // recordings
  recordingStart: (title: string) => call<string>("recording_start", { title }),
  recordingStop: () => call<string | null>("recording_stop"),
  recordingsList: () => call<T.Recording[]>("recordings_list"),
  recordingActive: () => call<string | null>("recording_active"),
  recordingDelete: (id: string) => call<void>("recording_delete", { id }),
  recordingExport: (id: string, dir: string) => call<string[]>("recording_export", { id, dir }),
  recordingPlayback: (id: string) => call<T.PlaybackMedia[]>("recording_playback", { id }),
  recordingAttach: (id: string) => call<T.RecordingAttach>("recording_attach", { id }),
  attachmentRead: (threadId: string, id: string, selector: Record<string, string>) =>
    call<string>("attachment_read", { threadId, id, selector }),

  // voice
  voiceModels: () => call<T.ModelView[]>("voice_models"),
  voiceInstall: (id: string) => call<void>("voice_install", { id }),
  voiceCancelInstall: (id: string) => call<void>("voice_cancel_install", { id }),
  voiceRemove: (id: string) => call<void>("voice_remove", { id }),
  voiceSelect: (id: string) => call<void>("voice_select", { id }),
  pttPress: (device: string | null = null) => call<void>("ptt_press", { device }),
  pttRelease: (language: string | null, vocabulary: string[] = []) =>
    call<T.PttState>("ptt_release", { language, vocabulary }),
  pttCancel: () => call<T.PttState>("ptt_cancel"),
  audioDevices: (system: boolean) => call<T.AudioDevice[]>("audio_devices", { system }),
  audioTestStart: (source: T.AudioSourceKind, device: string | null = null) => call<void>("audio_test_start", { source, device }),
  audioTestStop: (source: T.AudioSourceKind) => call<void>("audio_test_stop", { source }),

  // app profiles
  profilesList: () => call<T.AppProfile[]>("profiles_list"),
  profilesSave: (profile: T.AppProfile) => call<T.AppProfile>("profiles_save", { profile }),
  profilesDelete: (id: string) => call<void>("profiles_delete", { id }),
  profileActive: () => call<T.AppProfile | null>("profile_active"),
  previousApp: () => call<T.PreviousApp | null>("previous_app"),

  // region selection
  regionOpen: () => call<void>("region_open"),
  regionCommit: (token: string, rect: { x: number; y: number; w: number; h: number }, tray: string) =>
    call<T.ContextChip>("region_commit", { token, rect, tray }),
  regionCancel: (token: string) => call<void>("region_cancel", { token }),

  // windows
  overlaySetMode: (mode: T.OverlayMode) => call<void>("overlay_set_mode", { mode }),
  overlayHide: () => call<void>("overlay_hide"),
  overlayMoved: () => call<void>("overlay_moved"),
  openExternal: (url: string) => call<void>("open_external", { url }),

  // diagnostics
  diagnostics: () => call<T.Diagnostics>("diagnostics"),
  notify: (title: string, body: string) => call<void>("notify", { title, body }),
  speak: (text: string) => call<[string, string]>("speak", { text }),
  speechOptions: () => call<T.SpeechOptions>("speech_options"),
  updaterConfigured: () => call<boolean>("updater_configured"),
  onboardingResume: () => call<T.Settings>("onboarding_resume"),
  speechConsent: () => call<T.Settings>("speech_consent"),
  /** YOLO (018): turning it on needs "ACEITO" or "ACCEPT". */
  yoloSet: (enabled: boolean, confirmation: string) => call<T.Settings>("yolo_set", { enabled, confirmation }),
  diagnosticsExport: (dest: string) => call<string>("diagnostics_export", { dest }),
  eraseAllData: () => call<void>("erase_all_data"),
};

export type Api = typeof api;

/** Normalizes errors from Tauri (`{code, message}`) or JS exceptions. */
export function errorMessage(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) return String((e as { message: unknown }).message);
  return String(e);
}

export function errorCode(e: unknown): string {
  if (e && typeof e === "object" && "code" in e) return String((e as { code: unknown }).code);
  return "unknown";
}
