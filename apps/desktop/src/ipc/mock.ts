// In-memory backend used in a plain browser (`pnpm dev`) and by tests.
// It mimics the host closely enough to exercise every screen: streaming
// answers, approvals (type "/aprovar"), login, consent, voice and downloads.

import type { Bridge } from "./bridge";
import { HOST_EVENT } from "./bridge";
import type * as T from "./types";

type Handler = (payload: unknown) => void;

const DEFAULT_SETTINGS: T.Settings = {
  theme: "system",
  opacity: 0.92,
  language: "ptBr",
  invokeShortcut: "Ctrl+Shift+Space",
  doubleTapCtrl: false,
  startWithWindows: false,
  focusLoss: "hide",
  privacyPauseShortcut: "Ctrl+Shift+Alt+P",
  pushToTalkShortcut: "Ctrl+Space",
  globalVoiceShortcut: "Ctrl+Alt+Space",
  attachScreenOnOpen: false,
  sendAfterDictation: false,
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

const MODELS: T.ModelInfo[] = [
  { id: "gpt-5.5", displayName: "GPT-5.5", efforts: ["low", "medium", "high"], defaultEffort: "medium", inputModalities: ["text", "image"], isDefault: true },
  { id: "gpt-5.5-mini", displayName: "GPT-5.5 mini", efforts: ["low", "medium"], defaultEffort: "low", inputModalities: ["text", "image"], isDefault: false },
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
  const state = new MockState(opts.signedIn ?? false);
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
      if (patch.opacity !== undefined && (patch.opacity < 0.7 || patch.opacity > 1)) throw { code: "settings", message: "opacidade fora do intervalo" };
      state.settings = { ...state.settings, ...patch };
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
    consent_answer: () => undefined,
    auth_status: () => ({ accounts: state.account ? [state.account] : [], active: state.account }),
    auth_login: async () => {
      host({ channel: "login", event: { state: "waitingBrowser", authorizeUrl: "https://auth.openai.com/oauth/authorize?mock=1" } });
      await sleep(tick * 20);
      state.account = mockAccount();
      host({ channel: "login", event: { state: "completed", account: state.account, firstTime: true } });
    },
    auth_cancel: () => host({ channel: "login", event: { state: "cancelled" } }),
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
      const p: T.Provider = {
        id: draft.id ?? `${draft.preset}-${++state.seq}`,
        name: draft.name,
        preset: draft.preset,
        wire: draft.wire ?? PRESETS.find((x) => x.id === draft.preset)?.wire ?? "chat",
        baseUrl: draft.baseUrl ?? PRESETS.find((x) => x.id === draft.preset)?.baseUrl ?? "",
        auth: "bearer",
        extraHeaders: draft.extraHeaders ?? {},
        models: [],
        credentialHint: credential ? `••••${String(credential).slice(-4)}` : null,
        status: "unverified",
        lastError: null,
        quirks: { noParallelToolCalls: false, reasoningEffort: false, noStreamOptions: false },
      };
      state.providers = [...state.providers.filter((x) => x.id !== p.id), p];
      return p;
    },
    providers_remove: ({ id }) => {
      state.providers = state.providers.filter((p) => p.id !== id);
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
    conversation_history: ({ query }) => ({
      items: state.history.filter((h) => !query?.search || h.title.toLowerCase().includes(String(query.search).toLowerCase())),
      nextCursor: null,
    }),
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
    region_commit: ({ tray, rect }) => {
      const chip: T.ContextChip = { id: `chip_${++state.seq}`, kind: "region", label: `Região ${rect.w}×${rect.h}`, previewPath: PIXEL, payload: { type: "image", path: PIXEL }, tokenEstimate: 765, blockedReason: null };
      state.trays.set(tray, [...(state.trays.get(tray) ?? []), chip]);
      return chip;
    },
    region_cancel: () => undefined,
    attach_file: ({ tray, path }) => {
      const name = String(path).split(/[\\/]/).pop() ?? "arquivo";
      const info: T.AttachmentInfo = { id: `att_${++state.seq}`, conversation: "draft", fileName: name, stored: path, hash: "0".repeat(64), summary: "3 páginas", kind: "pdf", tokens: 2400, warnings: [] };
      const chip: T.ContextChip = { id: `chip_${state.seq}`, kind: "file", label: `${name} · 3 páginas`, previewPath: null, payload: { type: "text", text: "…" }, tokenEstimate: 2400, blockedReason: null };
      state.trays.set(tray, [...(state.trays.get(tray) ?? []), chip]);
      return [info, chip];
    },
    attachments_list: () => [],
    insert_into_app: () => true,
    quick_list: () => state.quick,
    quick_save: ({ name, template }) => {
      state.quick = [...state.quick.filter((q) => q.name !== name), { name, template, builtin: false, enabled: true }];
      return state.quick;
    },
    quick_delete: ({ name }) => {
      state.quick = state.quick.filter((q) => q.name !== name || q.builtin);
      return state.quick;
    },
    quick_toggle: ({ name, enabled }) => {
      state.quick = state.quick.map((q) => (q.name === name ? { ...q, enabled } : q));
      return state.quick;
    },
    quick_expand: ({ input, typed }) => {
      const [cmd, ...rest] = String(input).slice(1).split(/\s+/);
      const q = state.quick.find((x) => x.name === cmd);
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
    mcp_status: () => [{ name: "aura", tools: ["screen_capture", "screen_text"], auth: "unsupported" }, ...state.mcp.map((m) => ({ name: m.name, tools: [], auth: m.transport.type === "http" ? "notLoggedIn" : null }))],
    mcp_login: () => undefined,
    workspace_files: () => [
      { path: "relatorio.md", absolute: "C:/ws/relatorio.md", bytes: 120, modified: Date.now() / 1000 },
      { path: "pagina.html", absolute: "C:/ws/pagina.html", bytes: 80, modified: Date.now() / 1000 - 5 },
    ],
    workspace_read: ({ path }) => (String(path).endsWith(".html") ? "<h1>Olá</h1><script>alert(1)</script>" : "# Relatório\n\nTudo certo."),
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
    recording_start: ({ title }) => {
      const modes = [state.privacy.screen, state.privacy.mic, state.privacy.systemAudio];
      if (!modes.some((m) => m.mode.type === "manual")) throw { code: "recording", message: "nenhuma fonte está no modo \"Gravação manual\"" };
      const id = `rec_${++state.seq}`;
      state.recordings = [{ id, source: "screen,mic", title, startedAt: Date.now() / 1000, endedAt: null, bytes: 0 }, ...state.recordings];
      state.activeRecording = id;
      host({ channel: "privacy", event: { paused: state.privacy.paused, screen: "manual", mic: "manual", systemAudio: "off", recording: ["screen", "mic"] } });
      return id;
    },
    recording_stop: () => {
      const id = state.activeRecording;
      state.activeRecording = null;
      state.recordings = state.recordings.map((r) => (r.id === id ? { ...r, endedAt: Date.now() / 1000, bytes: 2_400_000 } : r));
      host({ channel: "privacy", event: { paused: state.privacy.paused, screen: "manual", mic: "manual", systemAudio: "off", recording: [] } });
      return id;
    },
    recordings_list: () => state.recordings,
    recording_active: () => state.activeRecording,
    recording_delete: ({ id }) => {
      state.recordings = state.recordings.filter((r) => r.id !== id);
    },
    recording_export: ({ dir }) => [`${dir}/mic.wav`, `${dir}/screen-0001.mp4`],
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
    }),
    diagnostics_export: ({ dest }) => dest,
    notify: () => undefined,
    speak: () => ["UklGRiQAAABXQVZFZm10IBAAAAABAAEAgD4AAAB9AAACABAAZGF0YQAAAAA=", "audio/wav"],
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
  settings: T.Settings = { ...DEFAULT_SETTINGS };
  account: T.ChatGptAccount | null;
  providers: T.Provider[] = [];
  history: T.ConversationSummary[] = [];
  trays = new Map<string, T.ContextChip[]>();
  pendingApproval: { threadId: string; turnId: string; requestId: string } | null = null;
  accessLog: T.AccessLogEntry[] = [
    { at: Date.now() / 1000 - 60, source: "screen", requester: "agent", tool: "screen_capture", conversation: "uuid-1", decision: "ask", reason: null },
  ];
  quick: T.QuickCommand[] = [
    { name: "tldr", template: "Resuma em até três frases, direto ao ponto:\n\n{selecao}", builtin: true, enabled: true },
    { name: "traduzir", template: "Traduza para {args:inglês}:\n\n{selecao}", builtin: true, enabled: true },
    { name: "reescrever", template: "Reescreva o texto com mais clareza:\n\n{selecao}", builtin: true, enabled: true },
    { name: "explicar", template: "Explique de forma simples e objetiva:\n\n{selecao}", builtin: true, enabled: true },
    { name: "corrigir", template: "Corrija ortografia e gramática:\n\n{selecao}", builtin: true, enabled: true },
    { name: "resumir-tela", template: "{tela}Resuma o que está na tela.\n\n{texto}", builtin: true, enabled: true },
  ];
  mcp: T.McpServerSpec[] = [];
  recordings: T.Recording[] = [];
  profiles: T.AppProfile[] = [];
  activeRecording: string | null = null;
  skills: T.SkillReview[] = [];
  privacy: T.PrivacyView = {
    screen: { mode: { type: "onDemand" }, agent: "ask" },
    mic: { mode: { type: "onDemand" }, agent: "ask" },
    systemAudio: { mode: { type: "off" }, agent: "ask" },
    paused: false,
    exclusions: [
      { id: "keepass", process: "KeePass.exe", titleGlob: null, class: null, enabled: true, builtin: true },
      { id: "bitwarden", process: "Bitwarden.exe", titleGlob: null, class: null, enabled: true, builtin: true },
      { id: "inprivate", process: null, titleGlob: "*InPrivate*", class: null, enabled: true, builtin: true },
    ],
  };
  voiceModels: T.ModelView[] = [
    voice("parakeet-tdt-0.6b-v3-int8", "Parakeet TDT 0.6B v3", "Rápido e preciso em 25 idiomas europeus, incluindo português.", 670_000_000, true),
    voice("whisper-small", "Whisper Small", "Multilíngue, leve, bom para computadores modestos.", 488_000_000, false),
    voice("whisper-large-v3-turbo-q5", "Whisper Large v3 Turbo (Q5)", "Melhor qualidade multilíngue; recomendado com GPU.", 574_000_000, false),
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
      gpuRecommended: id.includes("large"),
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
