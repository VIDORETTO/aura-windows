// In-memory backend used in a plain browser (`pnpm dev`) and by tests.
// It mimics the host closely enough to exercise every screen: streaming
// answers, approvals (type "/aprovar"), login, consent, voice and downloads.

import type { Bridge } from "./bridge";
import { HOST_EVENT } from "./bridge";
import type * as T from "./types";
import { en } from "../i18n/en";
import { ptBR, type MessageKey } from "../i18n/pt-BR";

type Handler = (payload: unknown) => void;

const DEFAULT_SETTINGS: T.Settings = {
  theme: "system",
  opacity: 0.92,
  language: "ptBr",
  invokeShortcut: "Ctrl+Shift+Space",
  doubleTapCtrl: false,
  startWithWindows: false,
  hideOnBlur: false,
  privacyPauseShortcut: "Ctrl+Shift+Alt+P",
  pushToTalkShortcut: "Ctrl+Space",
  globalVoiceShortcut: "Ctrl+Alt+Space",
  attachScreenOnOpen: false,
  sendAfterDictation: false,
  microphoneDeviceId: null,
  systemAudioDeviceId: null,
  disabledSkills: [],
  defaultModel: null,
  defaultEffort: null,
  personalInstructions: "",
  appServerIdleMinutes: 15,
  workerIdleMinutes: 2,
  memories: false,
  asrLanguage: null,
  asrVocabulary: [],
  cloudAsrProvider: null,
  onboarded: true,
  ttsVoice: null,
  ttsProvider: null,
  ttsCloudVoice: "alloy",
  ttsCloudConsent: [],
  autoRead: false,
  accentColor: null,
  effortPresets: {},
  yolo: false,
  hideFromCapture: true,
  meetingKeepAudio: false,
  broadcastMode: false,
  meetingRedactPii: false,
};

const PRESETS: T.Preset[] = [
  preset("openai", "OpenAI", "responses", "https://api.openai.com/v1"),
  preset("anthropic", "Anthropic", "anthropic", "https://api.anthropic.com/v1"),
  preset("openrouter", "OpenRouter", "chat", "https://openrouter.ai/api/v1"),
  preset("groq", "Groq", "chat", "https://api.groq.com/openai/v1"),
  preset("ollama", "Ollama (local)", "chat", "http://127.0.0.1:11434/v1", false),
  preset("custom", "Endpoint personalizado", "chat", "", false),
];

function preset(id: string, name: string, wire: T.Wire, baseUrl: string, credentialRequired = true): T.Preset {
  return {
    id,
    name,
    wire,
    baseUrl,
    auth: "bearer",
    credentialRequired,
    headers: {},
    quirks: { noParallelToolCalls: false, reasoningEffort: false, noStreamOptions: false },
    transcriptionModels: [],
    tts: false,
  };
}

// The ChatGPT plan catalog (017): only the GPT-6 trio.
const MODELS: T.ModelInfo[] = [
  { id: "gpt-6-luna", displayName: "GPT-6 Luna", efforts: ["low", "medium", "high"], defaultEffort: "medium", inputModalities: ["text", "image"], isDefault: true },
  { id: "gpt-6.1-sol", displayName: "GPT-6.1 Sol", efforts: ["low", "medium", "high", "xhigh", "max"], defaultEffort: "medium", inputModalities: ["text", "image"], isDefault: false },
  { id: "gpt-6-astra", displayName: "GPT-6 Astra", efforts: ["low", "medium", "high", "xhigh", "max"], defaultEffort: "medium", inputModalities: ["text", "image"], isDefault: false },
];

// A 1×1 PNG so image chips have a preview in the browser.
const PIXEL = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkqPtfDwAFBAHT0cNBsQAAAABJRU5ErkJggg==";

export interface MockOptions {
  /** Delay between streamed deltas (ms). Tests use 0. */
  tick?: number;
  signedIn?: boolean;
}

export function createMockBridge(opts: MockOptions = {}): Bridge & { state: MockState } {
  const tick = opts.tick ?? 18;
  const listeners = new Map<string, Set<Handler>>();
  const emit = (event: string, payload: unknown) => listeners.get(event)?.forEach((h) => h(payload));
  const host = (e: T.HostEvent) => emit(HOST_EVENT, e);
  const localizedQuick = () => {
    const keys: Record<string, MessageKey> = { tldr: "quick.template.tldr", traduzir: "quick.template.translate", reescrever: "quick.template.rewrite", explicar: "quick.template.explain", corrigir: "quick.template.correct", "resumir-tela": "quick.template.screen", formal: "quick.template.formal", curto: "quick.template.short", amigavel: "quick.template.friendly", golpe: "quick.template.scam", responder: "quick.template.reply", parei: "quick.template.resume", lembrar: "quick.template.remind", anota: "quick.template.note", notas: "quick.template.notes", colar: "quick.template.pasteas", salvos: "quick.template.saved", configurar: "quick.template.configure", preparo: "quick.template.prepare" };
    const table = state.settings.language === "en" ? en : ptBR;
    return state.quick.map((q) => q.builtin && keys[q.name] ? { ...q, template: table[keys[q.name]] } : q);
  };
  const state = new MockState(opts.signedIn ?? false);
  let loginAttempt = 0;
  const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

  async function streamAnswer(threadId: string, text: string) {
    const turnId = `turn_${++state.seq}`;
    const itemId = `msg_${state.seq}`;
    host({ channel: "conversation", event: { type: "turnStarted", threadId, turnId } });
    if (text.trim() === "/pergunta") {
      const requestId = `req_${state.seq}`;
      state.pendingApproval = { threadId, turnId, requestId };
      host({
        channel: "conversation",
        event: {
          type: "userInputRequested",
          threadId,
          requestId,
          prompt: { questions: [{ id: "lang", header: "Linguagem", question: "Em qual linguagem?", isOther: true, options: [{ label: "Rust" }, { label: "TypeScript" }] }] },
          autoResolveMs: null,
          source: "agent",
        },
      });
      return;
    }
    if (text.trim() === "/aprovar") {
      const requestId = `req_${state.seq}`;
      state.pendingApproval = { threadId, turnId, requestId };
      host({
        channel: "conversation",
        event: {
          type: "approvalRequested",
          threadId,
          requestId,
          kind: "command",
          command: "pip install requests",
          cwd: "C:\\Users\\voce\\projeto",
          reason: "Instalar a biblioteca para baixar a página",
          changes: [],
          options: ["accept", "acceptForSession", "decline"],
        },
      });
      return;
    }
    host({
      channel: "conversation",
      event: { type: "toolCall", threadId, itemId: `tool_${state.seq}`, kind: "mcp", title: "aura.active_window_info", status: "inProgress", detail: null },
    });
    await sleep(tick * 4);
    host({
      channel: "conversation",
      event: { type: "toolCall", threadId, itemId: `tool_${state.seq}`, kind: "mcp", title: "aura.active_window_info", status: "completed", detail: "32 ms" },
    });
    const answer = mockAnswer(text);
    state.cancelled = false;
    for (const piece of answer.match(/.{1,6}/gs) ?? []) {
      if (state.cancelled) break;
      host({ channel: "conversation", event: { type: "messageDelta", threadId, itemId, delta: piece } });
      if (tick) await sleep(tick);
    }
    host({ channel: "conversation", event: { type: "messageCompleted", threadId, itemId, text: answer } });
    host({ channel: "conversation", event: { type: "tokenUsage", threadId, used: 1200 + text.length * 2, window: 200000 } });
    host({
      channel: "conversation",
      event: { type: "turnCompleted", threadId, turnId, status: state.cancelled ? "interrupted" : "completed", error: null },
    });
    state.history.unshift({ id: threadId, title: text.slice(0, 48) || "Nova conversa", preview: answer.slice(0, 80), updatedAt: Date.now() / 1000, pinned: false, archived: false });
  }

  const handlers: Record<string, (a: Record<string, any>) => unknown> = {
    settings_get: () => state.settings,
    settings_update: ({ patch }) => {
      if (patch.opacity !== undefined && (patch.opacity < 0.5 || patch.opacity > 1)) throw { code: "settings", message: "opacidade fora do intervalo" };
      if (patch.accentColor && !/^#[0-9a-fA-F]{6}$/.test(patch.accentColor)) throw { code: "settings", message: "cor inválida" };
      if (patch.accentColor) patch = { ...patch, accentColor: patch.accentColor.toLowerCase() };
      if (patch.effortPresets) patch = { ...patch, effortPresets: Object.fromEntries(Object.entries(patch.effortPresets as Record<string, Record<string, string | null>>).map(([k, v]) => [k, Object.fromEntries(Object.entries(v).filter(([, e]) => e))]).filter(([, v]) => Object.keys(v as object).length > 0)) };
      // YOLO only changes through yolo_set (018).
      const { yolo: _yolo, ...rest } = patch;
      state.settings = { ...state.settings, ...rest };
      host({ channel: "settings", event: state.settings });
      return state.settings;
    },
    settings_open: () => undefined,
    privacy_get: () => state.privacy,
    privacy_set_source: ({ source, mode, agent }) => {
      const key = source === "systemAudio" ? "systemAudio" : source === "mic" ? "mic" : "screen";
      state.privacy = { ...state.privacy, [key]: { mode, agent } };
      return state.privacy;
    },
    privacy_set_retention: ({ days, maxGb, applyToManual }) => {
      if (days < 1 || days > 365 || maxGb < 1 || maxGb > 1000) throw { code: "out_of_range", message: "retenção fora do intervalo" };
      state.privacy = { ...state.privacy, retention: { days, maxGb, applyToManual } };
      return state.privacy;
    },
    privacy_set_paused: ({ paused }) => {
      state.privacy = { ...state.privacy, paused };
      host({ channel: "privacy", event: { paused, screen: state.privacy.screen.mode.type, mic: state.privacy.mic.mode.type, systemAudio: state.privacy.systemAudio.mode.type, recording: [] } });
      return state.privacy;
    },
    privacy_upsert_exclusion: ({ rule }) => {
      const r = { ...rule, id: rule.id || `user-${++state.seq}` };
      state.privacy = { ...state.privacy, exclusions: [...state.privacy.exclusions.filter((x) => x.id !== r.id), r] };
      return state.privacy;
    },
    privacy_remove_exclusion: ({ id }) => {
      if (state.privacy.exclusions.find((x) => x.id === id)?.builtin) throw { code: "builtin", message: "regras embutidas só podem ser desativadas" };
      state.privacy = { ...state.privacy, exclusions: state.privacy.exclusions.filter((x) => x.id !== id) };
      return state.privacy;
    },
    privacy_access_log: () => state.accessLog,
    privacy_clear_access_log: () => {
      state.accessLog = [];
    },
    privacy_access_thumbnail: ({ id }) =>
      state.accessLog.find((e) => e.id === id)?.hasThumbnail ? "data:image/png;base64,iVBORw0KGgo=" : null,
    privacy_open_conversation: ({ threadId }) => {
      host({ channel: "openConversation", event: { threadId } });
    },
    agent_task: ({ text, mode }) => {
      if (!String(text ?? "").trim()) throw { code: "invalid", message: "descreva o que o agente deve fazer" };
      host({ channel: "agentTask", event: { text: String(text).trim(), mode } });
    },
    consent_answer: () => undefined,
    auth_status: () => ({ accounts: state.account ? [state.account] : [], active: state.account }),
    auth_login: async () => {
      const attempt = ++loginAttempt;
      host({ channel: "login", event: { state: "waitingBrowser", authorizeUrl: "https://auth.openai.com/oauth/authorize?mock=1" } });
      await sleep(tick * 20);
      if (attempt !== loginAttempt) return;
      state.account = mockAccount();
      host({ channel: "login", event: { state: "completed", account: state.account, firstTime: true } });
    },
    auth_cancel: () => {
      ++loginAttempt;
      host({ channel: "login", event: { state: "cancelled" } });
    },
    auth_logout: () => {
      state.account = null;
      return true;
    },
    auth_switch: () => undefined,
    auth_mark_welcomed: () => {
      if (state.account) state.account = { ...state.account, welcomed: true };
    },
    providers_presets: () => PRESETS,
    providers_list: () => state.providers,
    providers_save: ({ draft, credential }) => {
      // Like the registry: an update keeps discovered models and the stored key.
      const existing = draft.id ? state.providers.find((x) => x.id === draft.id) : undefined;
      const p: T.Provider = {
        id: draft.id ?? `${draft.preset}-${++state.seq}`,
        name: draft.name,
        preset: draft.preset,
        wire: draft.wire ?? PRESETS.find((x) => x.id === draft.preset)?.wire ?? "chat",
        baseUrl: draft.baseUrl ?? PRESETS.find((x) => x.id === draft.preset)?.baseUrl ?? "",
        auth: "bearer",
        extraHeaders: draft.extraHeaders ?? {},
        models: existing?.models ?? [],
        credentialHint: credential ? `••••${String(credential).slice(-4)}` : existing?.credentialHint ?? null,
        status: "unverified",
        lastError: null,
        quirks: { noParallelToolCalls: false, reasoningEffort: false, noStreamOptions: false },
      };
      state.providers = existing ? state.providers.map((x) => (x.id === p.id ? p : x)) : [...state.providers, p];
      host({ channel: "providersChanged", event: {} });
      return p;
    },
    providers_model_save: ({ id, model }) => {
      const spec = { ...model, id: String(model.id).trim(), manual: true, estimated: false } as T.ModelSpec;
      state.providers = state.providers.map((p) => (p.id === id ? { ...p, models: [...p.models.filter((m) => m.id !== spec.id), spec] } : p));
      host({ channel: "providersChanged", event: {} });
      return state.providers.find((p) => p.id === id);
    },
    providers_model_remove: ({ id, modelId }) => {
      state.providers = state.providers.map((p) => (p.id === id ? { ...p, models: p.models.filter((m) => m.id !== modelId) } : p));
      host({ channel: "providersChanged", event: {} });
      return state.providers.find((p) => p.id === id);
    },
    providers_remove: ({ id }) => {
      state.providers = state.providers.filter((p) => p.id !== id);
      host({ channel: "providersChanged", event: {} });
    },
    providers_test: async ({ id }) => {
      await sleep(tick * 10);
      const p = state.providers.find((x) => x.id === id)!;
      const tested: T.Provider = {
        ...p,
        status: "verified",
        models: [{ id: "llama-3.3-70b", displayName: "Llama 3.3 70B", contextWindow: 131072, maxOutput: 32768, supportsImages: false, supportsTools: true, supportsReasoning: false, estimated: true }],
      };
      state.providers = state.providers.map((x) => (x.id === id ? tested : x));
      host({ channel: "providersChanged", event: {} });
      return tested;
    },
    conversation_start: ({ options }) => {
      const id = `thr_${++state.seq}`;
      return { threadId: id, conversationUuid: `uuid-${state.seq}`, workspace: `C:\\Users\\voce\\AppData\\Local\\Aura\\workspaces\\${id}`, ephemeral: !!options?.ephemeral };
    },
    conversation_send: ({ request }) => {
      state.trays.delete(request.tray);
      void streamAnswer(request.threadId, request.text);
      return `turn_${state.seq + 1}`;
    },
    conversation_steer: () => "turn",
    conversation_interrupt: () => {
      state.cancelled = true;
    },
    conversation_compact: ({ threadId }) => host({ channel: "conversation", event: { type: "compacted", threadId } }),
    conversation_set_mode: () => undefined,
    conversation_history: ({ query }) => {
      // Offset cursors, newest first, like the app-server pages.
      const all = state.history
        .filter((h) => !!h.archived === !!query?.archived)
        .filter((h) => !query?.search || h.title.toLowerCase().includes(String(query.search).toLowerCase()))
        .sort((a, b) => b.updatedAt - a.updatedAt);
      const start = Number(query?.cursor ?? 0);
      const end = start + Number(query?.limit ?? 50);
      return { items: all.slice(start, end), nextCursor: end < all.length ? String(end) : null };
    },
    conversation_open: ({ threadId }) => {
      const h = state.history.find((x) => x.id === threadId);
      return h ? [{ role: "user", text: h.title }, { role: "assistant", text: h.preview }] : [];
    },
    conversation_rename: ({ threadId, name }) => {
      state.history = state.history.map((h) => (h.id === threadId ? { ...h, title: name } : h));
    },
    conversation_pin: ({ threadId, pinned }) => {
      state.history = state.history.map((h) => (h.id === threadId ? { ...h, pinned } : h));
    },
    conversation_unarchive: ({ threadId }) => {
      state.history = state.history.map((h) => (h.id === threadId ? { ...h, archived: false } : h));
    },
    conversation_archive: ({ threadId }) => {
      state.history = state.history.map((h) => (h.id === threadId ? { ...h, archived: true } : h));
    },
    conversation_delete: ({ threadId }) => {
      state.history = state.history.filter((h) => h.id !== threadId);
    },
    conversation_close_ephemeral: () => undefined,
    conversation_respond: async ({ requestId, decision }) => {
      const p = state.pendingApproval;
      if (!p || p.requestId !== requestId) return;
      state.pendingApproval = null;
      host({ channel: "conversation", event: { type: "requestResolved", requestId } });
      const accepted = decision.type === "accept" || decision.type === "acceptForSession";
      const chosen = decision.type === "answer" ? (decision.content as { answers?: Record<string, { answers: string[] }> }).answers?.lang?.answers?.[0] : null;
      const text = chosen ? `Certo, vou usar ${chosen}.` : accepted ? "Pronto: instalei `requests` e baixei a página." : "Tudo bem, não executei o comando.";
      host({ channel: "conversation", event: { type: "messageDelta", threadId: p.threadId, itemId: `msg_${requestId}`, delta: text } });
      host({ channel: "conversation", event: { type: "turnCompleted", threadId: p.threadId, turnId: p.turnId, status: "completed", error: null } });
    },
    models_list: () => MODELS,
    tray_list: ({ tray }) => state.trays.get(tray) ?? [],
    tray_remove: ({ tray, chipId }) => {
      const list = (state.trays.get(tray) ?? []).filter((c) => c.id !== chipId);
      state.trays.set(tray, list);
      return list;
    },
    tray_move: ({ from, to }) => {
      state.trays.set(to, [...(state.trays.get(to) ?? []), ...(state.trays.get(from) ?? [])]);
      state.trays.delete(from);
    },
    capture_screen: ({ tray, windowOnly }) => {
      const chip: T.ContextChip = state.privacy.paused
        ? { id: `chip_${++state.seq}`, kind: "screen", label: "Tela", previewPath: null, payload: { type: "text", text: "" }, tokenEstimate: 0, blockedReason: "paused" }
        : { id: `chip_${++state.seq}`, kind: "screen", label: windowOnly ? "Janela · Code.exe" : "Tela · main.rs — Aura", previewPath: PIXEL, payload: { type: "image", path: PIXEL }, tokenEstimate: 765, blockedReason: null };
      state.trays.set(tray, [...(state.trays.get(tray) ?? []), chip]);
      return chip;
    },
    context_attach_recent: ({ tray, clip }) => {
      const what = [clip.screen ? "tela" : null, clip.audio ? { mic: "microfone", system: "áudio do sistema", both: "microfone + sistema" }[clip.audio as string] : null].filter(Boolean).join(" + ");
      const chip: T.ContextChip = { id: `chip_${++state.seq}`, kind: clip.screen ? "clip" : "audio", label: `Últimos ${clip.minutes} min · ${what}`, previewPath: clip.screen ? PIXEL : null, payload: { type: "text", text: "…" }, tokenEstimate: 3000, blockedReason: null };
      state.trays.set(tray, [...(state.trays.get(tray) ?? []), chip]);
      return chip;
    },
    context_attach_skill: ({ tray, name }) => {
      const known = state.skills.find((s) => s.manifest.name === name) ? `aura/${name}/SKILL.md` : name === "skill-creator" ? "system/skill-creator/SKILL.md" : null;
      if (!known || state.disabledSkills.includes(known)) throw { code: "skill", message: `skill indisponível: ${name}` };
      const list = state.trays.get(tray) ?? [];
      const existing = list.find((c) => c.payload.type === "skill" && c.payload.name === name);
      if (existing) return existing;
      const chip: T.ContextChip = { id: `chip_${++state.seq}`, kind: "skill", label: String(name), previewPath: null, payload: { type: "skill", name: String(name), path: known }, tokenEstimate: 0, blockedReason: null };
      state.trays.set(tray, [...list, chip]);
      return chip;
    },
    capture_selection: ({ tray }) => {
      const chip: T.ContextChip = { id: `chip_${++state.seq}`, kind: "selection", label: "❝ texto selecionado", previewPath: null, payload: { type: "text", text: "texto selecionado" }, tokenEstimate: 5, blockedReason: null };
      state.trays.set(tray, [...(state.trays.get(tray) ?? []), chip]);
      return chip;
    },
    profiles_list: () => state.profiles,
    profiles_save: ({ profile }) => {
      if (!profile.name?.trim() || !profile.processPattern?.trim()) throw { code: "profile", message: "informe um nome e o processo do app" };
      const p = { ...profile, id: profile.id || `prof_${++state.seq}` };
      state.profiles = [...state.profiles.filter((x) => x.id !== p.id), p];
      return p;
    },
    profiles_delete: ({ id }) => {
      state.profiles = state.profiles.filter((p) => p.id !== id);
    },
    profile_active: () => state.profiles.find((p) => p.processPattern.toLowerCase() === "code.exe") ?? null,
    previous_app: () => ({ window: 1, pid: 42, processName: "Code.exe", title: "main.rs — Aura", monitorId: "DISPLAY1" }),
    region_open: () => undefined,
    region_copy_text: () => undefined,
    region_commit: ({ tray, rect }) => {
      const chip: T.ContextChip = { id: `chip_${++state.seq}`, kind: "region", label: `Região ${rect.w}×${rect.h}`, previewPath: PIXEL, payload: { type: "image", path: PIXEL }, tokenEstimate: 765, blockedReason: null };
      state.trays.set(tray, [...(state.trays.get(tray) ?? []), chip]);
      return chip;
    },
    region_cancel: () => undefined,
    attach_clipboard_image: ({ tray, mime }) => {
      const name = `clipboard.${String(mime).split("/")[1]}`;
      const info: T.AttachmentInfo = { id: `att_${++state.seq}`, conversation: "draft", fileName: name, stored: name, hash: "0".repeat(64), summary: "imagem", kind: "image", tokens: 800, warnings: [] };
      const chip: T.ContextChip = { id: `chip_${state.seq}`, kind: "image", label: `Imagem colada · ${name}`, previewPath: null, payload: { type: "image", path: name }, tokenEstimate: 800, blockedReason: null };
      state.trays.set(tray, [...(state.trays.get(tray) ?? []), chip]);
      return [info, chip];
    },
    attach_file: ({ tray, path }) => {
      const name = String(path).split(/[\\/]/).pop() ?? "arquivo";
      const info: T.AttachmentInfo = { id: `att_${++state.seq}`, conversation: "draft", fileName: name, stored: path, hash: "0".repeat(64), summary: "3 páginas", kind: "pdf", tokens: 2400, warnings: [] };
      const chip: T.ContextChip = { id: `chip_${state.seq}`, kind: "file", label: `${name} · 3 páginas`, previewPath: null, payload: { type: "text", text: "…" }, tokenEstimate: 2400, blockedReason: null };
      state.trays.set(tray, [...(state.trays.get(tray) ?? []), chip]);
      return [info, chip];
    },
    attachments_list: () => [],
    insert_into_app: () => true,
    actions_list: () => state.actionList.filter((a) => a.status === "open"),
    action_done: ({ id, done }) => {
      state.actionList = state.actionList.map((a) => (a.id === id ? { ...a, status: done ? "done" : "open" } : a));
    },
    action_delete: ({ id }) => {
      state.actionList = state.actionList.filter((a) => a.id !== id);
    },
    recipes_list: () => [
      { id: "um-a-um", name: "1:1", description: "Conversa individual", notesTemplate: "Como foi · Compromissos", helpLevel: "onDemand", builtin: true },
      { id: "decisao", name: "Reunião de decisão", description: "Escolher entre opções", notesTemplate: "Decisão · Riscos", helpLevel: "balanced", builtin: true },
    ],
    meeting_brief: () => state.meetingBrief,
    meeting_set_brief: ({ text }) => {
      state.meetingBrief = String(text || "") || null;
      host({ channel: "meeting", event: { id: "", status: "briefing" } });
    },
    meeting_active: () => state.meetingNow,
    meeting_start: ({ title, kind, briefing }) => {
      const m: T.Meeting = { id: `m${state.meetingList.length + 1}`, title: String(title), kind: String(kind), briefing: String(briefing || state.meetingBrief || ""), origin: "live", status: "active", startedAt: Date.now(), endedAt: null };
      state.meetingNow = m;
      state.meetingList = [m, ...state.meetingList];
      state.meetingBrief = null;
      host({ channel: "meeting", event: { id: m.id, status: "active" } });
      return m;
    },
    meeting_note: ({ text }) => {
      const m = state.meetingNow!;
      const u: T.Utterance = { id: (state.meetingLines[m.id]?.length ?? 0) + 1, meetingId: m.id, t0: Date.now() - m.startedAt, t1: Date.now() - m.startedAt, speaker: "note", text: String(text) };
      state.meetingLines[m.id] = [...(state.meetingLines[m.id] ?? []), u];
      host({ channel: "meeting", event: { id: m.id, status: "updated" } });
      return u;
    },
    meeting_pause: () => undefined,
    meeting_stop: () => {
      const m = { ...state.meetingNow!, status: "ended" as const, endedAt: Date.now() };
      state.meetingList = state.meetingList.map((x) => (x.id === m.id ? m : x));
      state.meetingNow = null;
      host({ channel: "meeting", event: { id: m.id, status: "ended" } });
      return m;
    },
    meeting_from_buffer: ({ title }) => {
      const m: T.Meeting = { id: `m${state.meetingList.length + 1}`, title: String(title), kind: "other", briefing: "", origin: "buffer", status: "ended", startedAt: Date.now() - 600000, endedAt: Date.now() };
      state.meetingList = [m, ...state.meetingList];
      host({ channel: "meeting", event: { id: m.id, status: "ended" } });
      return m;
    },
    meetings_list: () => state.meetingList,
    meeting_utterances: ({ id }) => state.meetingLines[String(id)] ?? [],
    meeting_delete: ({ id }) => {
      state.meetingList = state.meetingList.filter((m) => m.id !== id);
    },
    meeting_search: () => [],
    capture_hiding_check: () => [{ window: "overlay", hidden: state.settings.hideFromCapture }],
    note_add: () => undefined,
    replace_target: () => state.replaceTarget,
    replace_selection: () => state.replaceTarget ?? "",
    undo_replace: () => true,
    quick_list: localizedQuick,
    quick_save: ({ name, template }) => {
      state.quick = [...state.quick.filter((q) => q.name !== name), { name, template, builtin: false, enabled: true }];
      return localizedQuick();
    },
    quick_delete: ({ name }) => {
      state.quick = state.quick.filter((q) => q.name !== name || q.builtin);
      return localizedQuick();
    },
    quick_toggle: ({ name, enabled }) => {
      state.quick = state.quick.map((q) => (q.name === name ? { ...q, enabled } : q));
      return localizedQuick();
    },
    quick_expand: ({ input, typed }) => {
      const [cmd, ...rest] = String(input).slice(1).split(/\s+/);
      const q = localizedQuick().find((x) => x.name === cmd);
      if (!q) throw { code: "quick_command", message: `comando desconhecido: /${cmd}` };
      const arg = rest[0] ?? "inglês";
      const body = typed || rest.slice(1).join(" ") || "texto selecionado";
      const prompt = q.template.replace("{selecao}", body).replace("{texto}", typed).replace(/\{args(:[^}]*)?\}/, arg).replace("{tela}", "");
      return { promptText: prompt.trim(), display: `/${cmd}${rest[0] ? ` ${rest[0]}` : ""}`, needsScreen: q.template.includes("{tela}") };
    },
    mcp_list: () => state.mcp,
    mcp_save: ({ spec }) => {
      state.mcp = [...state.mcp.filter((m) => m.name !== spec.name), spec];
      return state.mcp;
    },
    mcp_delete: ({ name }) => {
      state.mcp = state.mcp.filter((m) => m.name !== name);
      return state.mcp;
    },
    mcp_detect: () => [],
    mcp_import: () => state.mcp,
    agent_restart: () => true,
    // A stdio server whose command is "fail" does not start (status/log tests).
    mcp_status: () => [{ name: "aura", tools: ["screen_capture", "screen_text"], auth: "unsupported", error: null }, ...state.mcp.map((m) => {
      const failed = m.transport.type === "stdio" && m.transport.command === "fail";
      return { name: m.name, tools: m.transport.type === "stdio" && m.enabled && !failed ? ["qa_echo", "qa_write"].filter((t) => !m.disabledTools.includes(t)) : [], auth: m.transport.type === "http" ? "notLoggedIn" : null, error: failed ? "MCP startup failed: connection closed: initialize response" : null };
    })],
    mcp_diagnose: ({ name }) => {
      const m = state.mcp.find((x) => x.name === name);
      const failed = m?.transport.type === "stdio" && m.transport.command === "fail";
      return failed ? { connected: false, exitCode: 3, log: ["starting", "missing QA_TOKEN"] } : { connected: true, exitCode: null, log: [] };
    },
    mcp_login: () => undefined,
    workspace_files: () => [
      { path: "relatorio.md", absolute: "C:/ws/relatorio.md", bytes: 120, modified: Date.now() / 1000 },
      { path: "pagina.html", absolute: "C:/ws/pagina.html", bytes: 80, modified: Date.now() / 1000 - 5 },
      { path: "relatorio.pdf", absolute: "C:/ws/relatorio.pdf", bytes: 5000, modified: Date.now() / 1000 - 10 },
      { path: "scan.pdf", absolute: "C:/ws/scan.pdf", bytes: 9000, modified: Date.now() / 1000 - 20 },
    ],
    workspace_read: ({ path }) => (String(path).endsWith(".html") ? "<h1>Olá</h1><script>alert(1)</script>" : "# Relatório\n\nTudo certo."),
    workspace_pdf_preview: ({ path }): T.PdfPreview => (String(path).includes("scan") ? { totalPages: 2, pages: [] } : { totalPages: 7, pages: [1, 2, 3, 4, 5].map((n) => ({ number: n, text: `Texto da página ${n}` })) }),
    open_path: () => undefined,
    skills_list: () => state.skills,
    skills_review: () => {
      throw { code: "skill", message: "indisponível no modo navegador" };
    },
    skills_install: () => "",
    skills_create: ({ name, description, body }) => {
      state.skills = [...state.skills, { manifest: { name, description, extra: {} }, skillMd: body, files: [{ path: "SKILL.md", bytes: body.length, isScript: false }], warning: "" }];
      return name;
    },
    skills_delete: ({ name }) => {
      state.skills = state.skills.filter((s) => s.manifest.name !== name);
    },
    memories_get: () => memoryView(state.memorySummary, state.memoryRegistry),
    memories_forget_fact: ({ fact }) => {
      const drop = (text: string) => text.split("\n").filter((l) => l.replace(/^\s*[-*]\s+/, "").trim() !== fact || !/^\s*[-*]\s+/.test(l)).join("\n");
      state.memorySummary = drop(state.memorySummary);
      state.memoryRegistry = drop(state.memoryRegistry);
      return memoryView(state.memorySummary, state.memoryRegistry);
    },
    memories_save: ({ summary, registry }) => {
      state.memorySummary = summary;
      state.memoryRegistry = registry;
      return memoryView(summary, registry);
    },
    memories_forget_all: () => {
      state.memorySummary = "";
      state.memoryRegistry = "";
    },
    // Aura skills plus one system skill, like the app-server catalog.
    skills_catalog: (): T.SkillEntry[] => [
      ...state.skills.map((s) => ({ name: s.manifest.name, description: s.manifest.description, path: `aura/${s.manifest.name}/SKILL.md`, origin: "aura" as const, enabled: !state.disabledSkills.includes(`aura/${s.manifest.name}/SKILL.md`) })),
      { name: "skill-creator", description: "Cria Skills", path: "system/skill-creator/SKILL.md", origin: "system" as const, enabled: !state.disabledSkills.includes("system/skill-creator/SKILL.md") },
    ],
    skills_set_enabled: ({ path, enabled }) => {
      state.disabledSkills = state.disabledSkills.filter((p) => p !== path).concat(enabled ? [] : [String(path)]);
    },
    skills_source: ({ name }) => {
      const s = state.skills.find((x) => x.manifest.name === name);
      if (!s) throw { code: "skill", message: `Skill não encontrada: ${name}` };
      return [s.manifest.description, s.skillMd];
    },
    skills_update: ({ name, description, body }) => {
      state.skills = state.skills.map((s) => (s.manifest.name === name ? { ...s, manifest: { ...s.manifest, description }, skillMd: body } : s));
    },
    recording_start: ({ title }) => {
      const modes = [state.privacy.screen, state.privacy.mic, state.privacy.systemAudio];
      if (!modes.some((m) => m.mode.type === "manual")) throw { code: "recording", message: "nenhuma fonte está no modo \"Gravação manual\"" };
      const id = `rec_${++state.seq}`;
      state.recordings = [{ id, source: "screen,mic", title, startedAt: Date.now() / 1000, endedAt: null, bytes: 0, durationMs: null }, ...state.recordings];
      state.activeRecording = id;
      host({ channel: "privacy", event: { paused: state.privacy.paused, screen: "manual", mic: "manual", systemAudio: "off", recording: ["screen", "mic"] } });
      return id;
    },
    recording_stop: () => {
      const id = state.activeRecording;
      state.activeRecording = null;
      state.recordings = state.recordings.map((r) => (r.id === id ? { ...r, endedAt: Date.now() / 1000, bytes: 2_400_000, durationMs: Math.round((Date.now() / 1000 - r.startedAt) * 1000) } : r));
      host({ channel: "privacy", event: { paused: state.privacy.paused, screen: "manual", mic: "manual", systemAudio: "off", recording: [] } });
      return id;
    },
    recordings_list: () => state.recordings,
    recording_active: () => state.activeRecording,
    recording_delete: ({ id }) => {
      state.recordings = state.recordings.filter((r) => r.id !== id);
    },
    recording_export: ({ dir }) => [`${dir}/mic.wav`, `${dir}/screen-0001.mp4`],
    recording_playback: ({ id }) => [
      { kind: "audio", source: "mic", mime: "audio/wav", path: `cache/playback/${id}/mic.wav` },
      { kind: "video", source: "screen", mime: "video/mp4", path: `cache/playback/${id}/screen-0001.mp4` },
    ],
    recording_attach: ({ id }) => ({
      chips: ["mic.wav", "screen-0001.mp4"].map((name, i) => ({ id: `chip_${id}_${i}`, kind: "file" as const, label: name, previewPath: null, payload: { type: "text" as const, text: name }, tokenEstimate: 10, blockedReason: null })),
      failed: [],
    }),
    attachment_read: () => "[Página 3]\n…",
    voice_models: () => state.voiceModels,
    voice_install: async ({ id }) => {
      state.voiceModels = state.voiceModels.map((m) => (m.entry.id === id ? { ...m, downloading: true } : m));
      const total = state.voiceModels.find((m) => m.entry.id === id)?.sizeBytes ?? 1;
      for (let i = 1; i <= 5; i++) {
        await sleep(tick * 5);
        host({ channel: "download", event: { id, bytes: (total * i) / 5, total, done: false, error: null } });
      }
      state.voiceModels = state.voiceModels.map((m) => (m.entry.id === id ? { ...m, downloading: false, installed: true, selected: true } : { ...m, selected: false }));
      host({ channel: "download", event: { id, bytes: 0, total: null, done: true, error: null } });
    },
    voice_cancel_install: () => undefined,
    voice_remove: ({ id }) => {
      state.voiceModels = state.voiceModels.map((m) => (m.entry.id === id ? { ...m, installed: false, selected: false } : m));
    },
    voice_select: ({ id }) => {
      state.voiceModels = state.voiceModels.map((m) => ({ ...m, selected: m.entry.id === id }));
    },
    ptt_press: () => host({ channel: "voice", event: { state: "listening" } }),
    ptt_release: async () => {
      host({ channel: "voice", event: { state: "transcribing" } });
      await sleep(tick * 5);
      const done: T.PttState = { state: "done", text: "texto ditado de exemplo" };
      host({ channel: "voice", event: done });
      return done;
    },
    ptt_cancel: () => {
      const s: T.PttState = { state: "cancelled" };
      host({ channel: "voice", event: s });
      return s;
    },
    audio_test_start: (args) => {
      const source = args.source as T.AudioSourceKind;
      clearInterval(state.audioTests[source]);
      // A tone in the browser preview; tests drive levels through events.
      state.audioTests[source] = setInterval(() => host({ channel: "audioLevel", event: { source, dbfs: -12 - Math.random() * 6 } }), 40);
    },
    audio_test_stop: (args) => {
      const source = args.source as T.AudioSourceKind;
      clearInterval(state.audioTests[source]);
      delete state.audioTests[source];
    },
    audio_devices: ({ system }) => [{ id: system ? "Alto-falantes" : "Microfone", name: system ? "Alto-falantes (Realtek)" : "Microfone (USB)", isDefault: true }],
    overlay_set_mode: () => undefined,
    overlay_hide: () => undefined,
    overlay_moved: () => undefined,
    open_external: ({ url }) => {
      if (typeof window !== "undefined") window.open(url, "_blank", "noopener");
    },
    diagnostics: () => ({
      version: "0.1.0",
      os: "browser",
      dataDir: "(memória)",
      gatewayPort: 0,
      appServer: { state: "ready", version: "mock" },
      appServerLaunches: 1,
      providers: state.providers.length,
      mcpServers: state.mcp.length,
      pendingConsents: 0,
      gateway: { port: 0, reachable: true },
      mcp: [],
      worker: { installed: false, running: false, spawns: 0, gpu: false, model: null },
      capture: { paused: state.privacy.paused, active: [], recording: false },
      account: null,
      disk: { freeBytes: null, auraBytes: 0 },
      ...state.diagnosticsExtra,
    }),
    diagnostics_export: ({ dest }) => dest,
    updater_configured: () => false,
    onboarding_resume: () => {
      state.settings = { ...state.settings, onboarded: false };
      host({ channel: "settings", event: state.settings });
      return state.settings;
    },
    notify: () => undefined,
    speak: () => {
      const id = state.settings.ttsProvider;
      if (id && !state.settings.ttsCloudConsent.includes(id)) throw { code: "consent_required", message: state.providers.find((p) => p.id === id)?.name ?? id };
      return ["UklGRiQAAABXQVZFZm10IBAAAAABAAEAgD4AAAB9AAACABAAZGF0YQAAAAA=", "audio/wav"];
    },
    speech_options: () => ({
      voices: [
        { id: "win-maria", name: "Microsoft Maria", language: "pt-BR" },
        { id: "win-zira", name: "Microsoft Zira", language: "en-US" },
      ],
      cloud: state.providers.filter((p) => p.preset === "custom" || p.preset === "openai").map((p) => ({ id: p.id, name: p.name })),
    }),
    yolo_set: ({ enabled, confirmation }) => {
      if (enabled && !["ACEITO", "ACCEPT"].includes(String(confirmation ?? "").trim().toUpperCase())) {
        throw { code: "invalid", message: "para ligar o modo YOLO, escreva ACEITO (ou ACCEPT)" };
      }
      state.settings = { ...state.settings, yolo: !!enabled };
      host({ channel: "settings", event: state.settings });
      return state.settings;
    },
    speech_consent: () => {
      const id = state.settings.ttsProvider;
      if (id && !state.settings.ttsCloudConsent.includes(id)) state.settings = { ...state.settings, ttsCloudConsent: [...state.settings.ttsCloudConsent, id] };
      host({ channel: "settings", event: state.settings });
      return state.settings;
    },
    erase_all_data: () => {
      Object.assign(state, new MockState(false));
    },
  };

  return {
    isTauri: false,
    state,
    emitLocal: emit,
    invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      const h = handlers[cmd];
      if (!h) throw { code: "unknown_command", message: `mock: ${cmd}` };
      return (await h(args)) as R;
    },
    listen: async <P,>(event: string, handler: (payload: P) => void) => {
      const set = listeners.get(event) ?? new Set<Handler>();
      set.add(handler as Handler);
      listeners.set(event, set);
      return () => set.delete(handler as Handler);
    },
  };
}

function mockAccount(): T.ChatGptAccount {
  return {
    clientId: "app_mock",
    subject: "user-mock",
    email: "voce@exemplo.com",
    scopes: ["openid", "chatgpt.tokens.use.direct"],
    planUsageEnabled: true,
    expiresAt: null,
    active: true,
    welcomed: false,
    signedIn: true,
  };
}

function mockAnswer(q: string): string {
  if (/código|code|erro|error/i.test(q)) {
    return "O erro indica **tipos incompatíveis**. Converta o valor antes de usar:\n\n```rust\nlet n: u32 = texto.trim().parse()?;\n```\n\n- `parse()` devolve `Result`\n- use `?` para propagar o erro";
  }
  return `Entendi: *${q.slice(0, 60)}*.\n\nAqui vai um resumo em três pontos:\n\n1. O Aura vê a tela só quando você permite.\n2. Respostas chegam em tempo real.\n3. Use \`/\` para comandos rápidos.`;
}

export class MockState {
  seq = 0;
  cancelled = false;
  audioTests: Partial<Record<T.AudioSourceKind, ReturnType<typeof setInterval>>> = {};
  disabledSkills: string[] = [];
  memorySummary = "";
  memoryRegistry = "";
  settings: T.Settings = { ...DEFAULT_SETTINGS };
  account: T.ChatGptAccount | null;
  providers: T.Provider[] = [];
  history: T.ConversationSummary[] = [];
  trays = new Map<string, T.ContextChip[]>();
  pendingApproval: { threadId: string; turnId: string; requestId: string } | null = null;
  accessLog: T.AccessLogEntry[] = [
    { id: 1, at: Date.now() / 1000 - 60, source: "screen", requester: "agent", tool: "screen_capture", conversation: "uuid-1", decision: "ask", reason: null, hasThumbnail: false, threadId: null },
  ];
  quick: T.QuickCommand[] = [
    { name: "tldr", template: "Resuma em até três frases, direto ao ponto:\n\n{selecao}", builtin: true, enabled: true },
    { name: "traduzir", template: "Traduza para {args:inglês}:\n\n{selecao}", builtin: true, enabled: true },
    { name: "reescrever", template: "Reescreva o texto com mais clareza:\n\n{selecao}", builtin: true, enabled: true },
    { name: "explicar", template: "Explique de forma simples e objetiva:\n\n{selecao}", builtin: true, enabled: true },
    { name: "corrigir", template: "Corrija ortografia e gramática:\n\n{selecao}", builtin: true, enabled: true },
    { name: "resumir-tela", template: "{tela}Resuma o que está na tela.\n\n{texto}", builtin: true, enabled: true },
    { name: "formal", template: "Reescreva em tom formal e profissional, mantendo o sentido e o idioma. Responda só com o texto:\n\n{selecao}", builtin: true, enabled: true },
    { name: "curto", template: "Reescreva de forma mais curta e direta, sem perder o essencial. Responda só com o texto:\n\n{selecao}", builtin: true, enabled: true },
    { name: "amigavel", template: "Reescreva em tom amigável e natural, mantendo o sentido e o idioma. Responda só com o texto:\n\n{selecao}", builtin: true, enabled: true },
    { name: "golpe", template: "Analise se o conteúdo abaixo parece golpe, phishing ou fraude (WhatsApp, e-mail, SMS, link, boleto, Pix, falsa central). Responda em linguagem simples, nesta ordem: 1) Veredito: Parece golpe, Suspeito ou Parece seguro, com o motivo principal; 2) Sinais encontrados; 3) O que fazer agora; 4) Como confirmar com segurança pelo canal oficial. Não peça nem repita senhas, códigos ou dados pessoais. Se não der para ter certeza, diga isso.\n\n{selecao}", builtin: true, enabled: true },
    { name: "responder", template: "{tela}Leia o e-mail ou a conversa que está na tela e escreva um rascunho de resposta no mesmo idioma, pronto para colar. Tom: {args:cordial}. Responda só com o texto da resposta.\n\n{texto}", builtin: true, enabled: true },
    { name: "lembrar", template: "Crie um lembrete com as ferramentas clock_now e reminder_create. Pedido do usuário: {texto}", builtin: true, enabled: true },
    { name: "anota", template: "Guarde esta nota com a ferramenta note_save, com as palavras do usuário e sem acrescentar nada, e confirme em uma linha: {texto}", builtin: true, enabled: true },
    { name: "notas", template: "Procure nas minhas notas com a ferramenta note_search (consulta: {texto}) e liste o que achar, da mais recente para a mais antiga.", builtin: true, enabled: true },
    { name: "colar", template: "Converta o texto abaixo para este formato: {args:lista com marcadores}. Responda só com o resultado, pronto para colar.\n\n{area}", builtin: true, enabled: true },
    { name: "salvos", template: "Procure nos meus textos salvos com a ferramenta note_search (kind=saved, consulta: {texto}) e mostre o que achar, o mais recente primeiro.", builtin: true, enabled: true },
    { name: "configurar", template: "$aura-configurar Quero configurar o Aura: {texto}", builtin: true, enabled: true },
    { name: "preparo", template: "$aura-preparo Quero me preparar para: {texto}", builtin: true, enabled: true },
    { name: "parei", template: "Use a ferramenta screen_recent para ver os últimos minutos da minha tela e diga, em poucas linhas, o que eu estava fazendo, em que ponto parei e qual seria o próximo passo. Se o buffer de tela estiver desligado, explique como ligá-lo em Configurações › Privacidade.\n\n{texto}", builtin: true, enabled: true },
  ];
  mcp: T.McpServerSpec[] = [];
  actionList: T.Action[] = [];
  meetingNow: T.Meeting | null = null;
  meetingList: T.Meeting[] = [];
  meetingBrief: string | null = null;
  meetingLines: Record<string, T.Utterance[]> = {};
  /** Selected text a quick command was applied to (019). */
  replaceTarget: string | null = null;
  recordings: T.Recording[] = [];
  profiles: T.AppProfile[] = [];
  /** Test override of the pipeline part of `diagnostics`. */
  diagnosticsExtra: Partial<T.Diagnostics> = {};
  activeRecording: string | null = null;
  skills: T.SkillReview[] = [];
  privacy: T.PrivacyView = {
    screen: { mode: { type: "onDemand" }, agent: "ask" },
    mic: { mode: { type: "onDemand" }, agent: "ask" },
    systemAudio: { mode: { type: "off" }, agent: "ask" },
    retention: { days: 7, maxGb: 20, applyToManual: false },
    paused: false,
    exclusions: [
      { id: "keepass", process: "KeePass.exe", titleGlob: null, class: null, enabled: true, builtin: true },
      { id: "bitwarden", process: "Bitwarden.exe", titleGlob: null, class: null, enabled: true, builtin: true },
      { id: "inprivate", process: null, titleGlob: "*InPrivate*", class: null, enabled: true, builtin: true },
    ],
  };
  voiceModels: T.ModelView[] = [
    voice("parakeet-tdt-0.6b-v3", "Parakeet TDT 0.6B v3", "Rápido e preciso em 25 idiomas europeus, incluindo português.", 670_000_000, true),
    voice("whisper-small", "Whisper Small", "Multilíngue, leve, bom para computadores modestos.", 488_000_000, false),
    voice("whisper-turbo-q5", "Whisper Large v3 Turbo (Q5)", "Melhor qualidade multilíngue; recomendado com GPU.", 574_000_000, false),
  ];

  constructor(signedIn: boolean) {
    this.account = signedIn ? mockAccount() : null;
  }
}

function voice(id: string, name: string, description: string, size: number, recommended: boolean): T.ModelView {
  return {
    entry: {
      id,
      name,
      family: id.split("-")[0],
      engine: id.startsWith("parakeet") ? "onnx-parakeet" : "ggml-whisper",
      description,
      languages: id.startsWith("parakeet") ? ["pt", "en", "es"] : ["*"],
      speed: 0.9,
      accuracy: 0.85,
      streaming: false,
      minRamMb: 2048,
      gpuRecommended: id.includes("turbo"),
      license: id.startsWith("parakeet") ? "CC-BY-4.0" : "MIT",
      sourceUrl: "https://huggingface.co",
    },
    sizeBytes: size,
    installed: false,
    selected: false,
    recommended,
    downloading: false,
  };
}

function memoryView(summary: string, registry: string): T.MemoryView {
  const facts = summary.split("\n").map((l) => /^\s*[-*]\s+(.+)/.exec(l)?.[1]?.trim()).filter((f): f is string => !!f);
  return { summary, registry, facts };
}
