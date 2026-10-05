// The Overlay session: which conversation is active, which tray collects the
// next turn's chips, mode/provider/model, and the actions the UI calls.

import { create } from "zustand";
import { api, errorCode, errorMessage } from "../ipc/commands";
import type { ContextChip, ConversationMode, ModelInfo, OverlayMode, Provider, StartOptions, RecentClip } from "../ipc/types";
import { decodeProfileModel, PLAN_PROVIDER } from "../lib/profileModel";
import { useApp } from "../state/app";
import { useConversation } from "../state/conversation";
import { translate } from "../i18n";

export const DRAFT = "draft";
export const CHATGPT_PLAN = PLAN_PROVIDER;

export type ModeKey = ConversationMode["mode"];

/** What losing focus does (009 TK-002): answers keep going in the Minibar. */
export function blurAction(o: { busy: boolean; keepOpen: boolean; pendingConsent: boolean }): "hide" | "minibar" | "stay" {
  if (o.keepOpen || o.pendingConsent) return "stay";
  return o.busy ? "minibar" : "hide";
}

interface Session {
  threadId: string | null;
  ephemeral: boolean;
  mode: ModeKey;
  /** Folders the agent may write to in Task mode (besides its workspace). */
  granted: string[];
  provider: string;
  model: string | null;
  /** Reasoning effort for the next turns; null = the model's default. */
  effort: string | null;
  chips: ContextChip[];
  overlayMode: OverlayMode;
  historyOpen: boolean;
  workOpen: boolean;
  /** Compact status pill while an answer runs in the background. */
  minibar: boolean;
  setMinibar: (on: boolean) => void;
  models: ModelInfo[];
  providers: Provider[];
  /** Enabled quick command names (unknown `/words` are sent as typed). */
  quickNames: string[];
  sending: boolean;
  /** Message typed with Enter while an answer runs; sent when the turn ends (015). */
  queued: string | null;
  queue: (text: string) => void;
  cancelQueue: () => void;
  /** Sends the queued message, if any (called when the turn ends). */
  flushQueue: () => Promise<void>;
  /** Resends the last request, optionally in another mode first (015). */
  retryLast: (mode?: ModeKey) => Promise<void>;
  /** Text the input bar should take (message "Edit"); `n` changes on each request. */
  compose: { text: string; n: number } | null;
  setCompose: (text: string) => void;
  attachSkill: (name: string) => Promise<void>;
  /** Profile of the app in front (badge, defaults). */
  profile: import("../ipc/types").AppProfile | null;
  applyProfile: () => Promise<void>;
  tray: () => string;
  setMode: (m: ModeKey) => Promise<void>;
  setGranted: (folders: string[]) => Promise<void>;
  compact: () => Promise<void>;
  setProvider: (provider: string, model: string | null) => void;
  setModel: (model: string | null) => void;
  setEffort: (effort: string | null) => void;
  setOverlayMode: (m: OverlayMode) => void;
  toggleHistory: (open?: boolean) => void;
  toggleWork: (open?: boolean) => void;
  refreshChips: () => Promise<void>;
  removeChip: (id: string) => Promise<void>;
  captureScreen: (windowOnly?: boolean) => Promise<void>;
  captureSelection: (explicit?: boolean) => Promise<void>;
  attachRecent: (clip: RecentClip) => Promise<void>;
  attach: (paths: string[]) => Promise<void>;
  send: (text: string) => Promise<boolean>;
  steer: (text: string) => Promise<void>;
  interrupt: () => Promise<void>;
  newConversation: (ephemeral?: boolean) => Promise<void>;
  openConversation: (threadId: string) => Promise<void>;
  loadCatalog: () => Promise<void>;
}

export function modeValue(m: ModeKey, granted: string[] = []): ConversationMode {
  return m === "task" ? { mode: "task", granted, network: false } : { mode: m };
}

function report(e: unknown) {
  useApp.getState().notify("error", errorMessage(e));
}

let catalogRequest = 0;

export const useSession = create<Session>((set, get) => ({
  threadId: null,
  ephemeral: false,
  mode: "chat",
  granted: [],
  provider: CHATGPT_PLAN,
  model: null,
  effort: null,
  chips: [],
  overlayMode: "compact",
  historyOpen: false,
  workOpen: false,
  minibar: false,
  models: [],
  providers: [],
  quickNames: [],
  sending: false,
  queued: null,
  compose: null,
  profile: null,

  tray: () => get().threadId ?? DRAFT,

  setMode: async (mode) => {
    const previous = get().mode;
    // Each mode has its own remembered effort for this model (013).
    set((s) => ({ mode, effort: presetEffort({ ...s, mode }) }));
    const id = get().threadId;
    if (id) {
      try {
        await api.conversationSetMode(id, modeValue(mode, get().granted));
        // Visible in the conversation; the agent is told on the next turn (014).
        if (mode !== previous) useConversation.getState().addModeChange(id, mode);
      } catch (e) {
        set({ mode: previous });
        report(e);
      }
    }
  },

  setGranted: async (granted) => {
    set({ granted });
    if (get().mode === "task") await get().setMode("task");
  },

  compact: async () => {
    const id = get().threadId;
    if (id) await api.conversationCompact(id).catch(report);
  },

  setProvider: (provider, model) => set((s) => ({ provider, model, effort: presetEffort({ ...s, provider, model }) ?? keepEffort({ ...s, provider, model }) })),
  setModel: (model) => set((s) => ({ model, effort: presetEffort({ ...s, model }) ?? keepEffort({ ...s, model }) })),
  setEffort: (effort) => {
    set({ effort });
    void rememberEffort(get(), effort);
  },

  setOverlayMode: (overlayMode) => {
    if (get().overlayMode === overlayMode) return;
    set({ overlayMode });
    void api.overlaySetMode(overlayMode).catch(() => undefined);
  },

  toggleHistory: (open) => {
    const next = open ?? !get().historyOpen;
    set({ historyOpen: next, ...(next ? { workOpen: false } : {}) });
    if (next) get().setOverlayMode("expanded");
  },

  applyProfile: async () => {
    const profile = await api.profileActive().catch(() => null);
    set({ profile });
    if (!profile || get().threadId) return;
    if (profile.defaultMode) set({ mode: profile.defaultMode });
    if (profile.defaultModel) {
      const { provider, model } = decodeProfileModel(profile.defaultModel);
      get().setProvider(provider, model);
    }
    if (profile.attachScreen && !get().chips.some((c) => c.kind === "screen")) await get().captureScreen(false);
  },

  setMinibar: (on) => {
    set({ minibar: on });
    if (on) void api.overlaySetMode("compact").catch(() => undefined);
    else if (get().threadId) void api.overlaySetMode(get().overlayMode).catch(() => undefined);
  },

  toggleWork: (open) => {
    const next = open ?? !get().workOpen;
    set({ workOpen: next, ...(next ? { historyOpen: false } : {}) });
    if (next) get().setOverlayMode("expanded");
  },

  refreshChips: async () => set({ chips: await api.trayList(get().tray()) }),

  removeChip: async (id) => {
    try {
      set({ chips: await api.trayRemove(get().tray(), id) });
    } catch (e) {
      report(e);
    }
  },

  captureScreen: async (windowOnly = false) => {
    try {
      await api.captureScreen(get().tray(), windowOnly);
      await get().refreshChips();
    } catch (e) {
      report(e);
    }
  },

  captureSelection: async (explicit = false) => {
    try {
      const chip = await api.captureSelection(get().tray(), explicit);
      if (chip) await get().refreshChips();
    } catch (e) {
      report(e);
    }
  },

  attachRecent: async (clip) => {
    try {
      await api.contextAttachRecent(get().tray(), get().threadId, clip);
      await get().refreshChips();
    } catch (e) {
      report(e);
    }
  },

  attach: async (paths) => {
    for (const p of paths) {
      try {
        await api.attachFile(get().tray(), get().threadId, p);
      } catch (e) {
        report(e);
      }
    }
    await get().refreshChips();
  },

  send: async (raw) => {
    const s = get();
    let text = raw.trim();
    let display: string | undefined;
    if (!text && s.chips.length === 0) return false;
    if (s.sending) return false;
    set({ sending: true });
    try {
      // Quick commands expand locally before the turn (008 AC-010).
      if (text.startsWith("/") && !text.startsWith("//")) {
        const [cmd, ...rest] = text.slice(1).split(/\s+/);
        if (cmd === "plano" || cmd === "plan") {
          await get().setMode("plan");
          text = rest.join(" ");
          if (!text) return true;
        } else if (cmd === "compactar" || cmd === "compact") {
          // Compaction is an app-server operation, never a prompt (002 AC-023).
          if (!s.threadId) {
            useApp.getState().notify("info", translate(useApp.getState().settings?.language ?? "ptBr", "command.compact.empty"));
            return true;
          }
          await get().compact();
          text = rest.join(" ");
          if (!text) return true;
        } else if (cmd === "tela" || cmd === "screen") {
          await get().captureScreen(false);
          text = rest.join(" ");
          if (!text) return true;
        } else if (s.quickNames.includes(cmd)) {
          const exp = await api.quickExpand(s.tray(), text, "");
          text = exp.promptText;
          display = exp.display;
        }
      }
      let threadId = s.threadId;
      if (!threadId) {
        const opts: StartOptions = { mode: modeValue(s.mode, s.granted), provider: s.provider, model: s.model, ephemeral: s.ephemeral };
        const started = await api.conversationStart(opts);
        threadId = started.threadId;
        await api.trayMove(DRAFT, threadId);
        set({ threadId });
        useConversation.getState().setActive(threadId);
      }
      const chips = await api.trayList(threadId);
      useConversation.getState().addUserMessage(threadId, text, chips, display);
      set({ chips: [] });
      get().setOverlayMode("expanded");
      const accepts = acceptsImages(get());
      await api.conversationSend({ threadId, text, tray: threadId, acceptsImages: accepts, options: { model: s.model, effort: s.effort } });
      return true;
    } catch (e) {
      if (errorCode(e) === "context") useApp.getState().notify("warning", errorMessage(e));
      else report(e);
      return false;
    } finally {
      set({ sending: false });
    }
  },

  queue: (text) => {
    if (text.trim()) set({ queued: text.trim() });
  },

  cancelQueue: () => set({ queued: null }),

  flushQueue: async () => {
    const text = get().queued;
    if (!text) return;
    set({ queued: null });
    if (!(await get().send(text))) set({ queued: text });
  },

  retryLast: async (mode) => {
    const id = get().threadId;
    const blocks = id ? (useConversation.getState().threads[id]?.blocks ?? []) : [];
    const last = [...blocks].reverse().find((b) => b.type === "user");
    if (!last || last.type !== "user" || !last.text.trim()) return;
    if (mode && mode !== get().mode) await get().setMode(mode);
    await get().send(last.text);
  },

  setCompose: (text) => set((s) => ({ compose: { text, n: (s.compose?.n ?? 0) + 1 } })),

  attachSkill: async (name) => {
    try {
      await api.contextAttachSkill(get().tray(), name);
      await get().refreshChips();
    } catch (e) {
      report(e);
    }
  },

  steer: async (text) => {
    const id = get().threadId;
    if (!id || !text.trim()) return;
    try {
      useConversation.getState().addUserMessage(id, text.trim(), []);
      await api.conversationSteer(id, text.trim(), id);
    } catch (e) {
      report(e);
    }
  },

  interrupt: async () => {
    const id = get().threadId;
    if (id) await api.conversationInterrupt(id).catch(report);
  },

  newConversation: async (ephemeral = false) => {
    const { threadId, ephemeral: wasEphemeral } = get();
    if (threadId && wasEphemeral) await api.conversationCloseEphemeral(threadId).catch(() => undefined);
    set({ threadId: null, ephemeral, chips: [], historyOpen: false, mode: "chat", granted: [], queued: null });
    useConversation.getState().setActive(null);
    get().setOverlayMode("compact");
    await get().refreshChips();
  },

  openConversation: async (threadId) => {
    try {
      const transcript = await api.conversationOpen(threadId);
      useConversation.getState().loadTranscript(threadId, transcript);
      useConversation.getState().setActive(threadId);
      set({ threadId, ephemeral: false, historyOpen: false });
      get().setOverlayMode("expanded");
      await get().refreshChips();
    } catch (e) {
      report(e);
    }
  },

  loadCatalog: async () => {
    const request = ++catalogRequest;
    const [providers, models, quick] = await Promise.all([
      api.providersList().catch(() => [] as Provider[]),
      api.modelsList().catch(() => [] as ModelInfo[]),
      api.quickList().catch(() => []),
    ]);
    if (request !== catalogRequest) return;
    set({ providers, models, quickNames: quick.filter((q) => q.enabled).map((q) => q.name) });
    const s = get();
    if (!s.threadId) {
      const hasAccount = !!useApp.getState().auth?.active?.signedIn;
      const current = providers.find((p) => `aura-${p.id}` === s.provider);
      if ((!current && s.provider !== CHATGPT_PLAN) || (s.provider === CHATGPT_PLAN && !hasAccount && providers.length > 0)) {
        set({ provider: hasAccount || providers.length === 0 ? CHATGPT_PLAN : `aura-${providers[0].id}`, model: null, effort: null });
      } else if (current && s.model && !current.models.some((m) => m.id === s.model)) {
        set({ model: null, effort: null });
      }
    }
  },
}));

export interface ModelCapabilities {
  efforts: string[];
  defaultEffort: string | null;
  images: boolean;
  tools: boolean;
  reasoning: boolean;
}

const PROVIDER_EFFORTS = ["low", "medium", "high"];

/** What a model can do (002 AC-011, 003 AC-012); id null = the default model. */
export function modelCapabilities(s: Pick<Session, "provider" | "models" | "providers">, id: string | null): ModelCapabilities | null {
  if (s.provider === CHATGPT_PLAN) {
    const m = s.models.find((x) => x.id === id) ?? (id === null ? s.models.find((x) => x.isDefault) : undefined);
    if (!m) return null;
    return {
      efforts: m.efforts,
      defaultEffort: m.defaultEffort,
      images: m.inputModalities.length === 0 || m.inputModalities.includes("image"),
      tools: true,
      reasoning: m.efforts.length > 0,
    };
  }
  const provider = s.providers.find((x) => `aura-${x.id}` === s.provider);
  const spec = provider?.models.find((m) => m.id === id);
  if (!provider || !spec) return null;
  // Chat Completions only carries `reasoning_effort` for providers that accept it.
  const carriesEffort = provider.wire !== "chat" || provider.quirks.reasoningEffort;
  return {
    efforts: carriesEffort ? (spec.efforts?.length ? spec.efforts : spec.supportsReasoning ? PROVIDER_EFFORTS : []) : [],
    defaultEffort: spec.defaultEffort ?? null,
    images: spec.supportsImages,
    tools: spec.supportsTools,
    reasoning: spec.supportsReasoning,
  };
}

/** Key of a model in `Settings.effortPresets` (013). */
export function effortKey(provider: string, model: string | null): string {
  return `${provider}::${model ?? "default"}`;
}

/** Effort remembered for this provider, model and mode, if the model accepts it. */
function presetEffort(s: Pick<Session, "provider" | "model" | "models" | "providers" | "mode">): string | null {
  const saved = useApp.getState().settings?.effortPresets?.[effortKey(s.provider, s.model)]?.[s.mode] ?? null;
  return saved && modelCapabilities(s, s.model)?.efforts.includes(saved) ? saved : null;
}

/** Saves the picker choice as this model's effort in the current mode. */
async function rememberEffort(s: Pick<Session, "provider" | "model" | "mode">, effort: string | null) {
  const settings = useApp.getState().settings;
  if (!settings) return;
  const key = effortKey(s.provider, s.model);
  const presets = { ...(settings.effortPresets ?? {}) };
  const entry = { ...(presets[key] ?? {}), [s.mode]: effort };
  if (!effort) delete entry[s.mode];
  if (Object.values(entry).some(Boolean)) presets[key] = entry;
  else delete presets[key];
  try {
    useApp.getState().setSettings(await api.settingsUpdate({ effortPresets: presets }));
  } catch (e) {
    report(e);
  }
}

/** The chosen effort survives a model change only when the new model supports it. */
function keepEffort(s: Pick<Session, "provider" | "model" | "models" | "providers" | "effort">): string | null {
  if (!s.effort) return null;
  return modelCapabilities(s, s.model)?.efforts.includes(s.effort) ? s.effort : null;
}

function acceptsImages(s: Session): boolean {
  if (s.provider === CHATGPT_PLAN) {
    const m = s.models.find((x) => x.id === s.model) ?? s.models.find((x) => x.isDefault);
    return !m || m.inputModalities.length === 0 || m.inputModalities.includes("image");
  }
  const p = s.providers.find((x) => `aura-${x.id}` === s.provider);
  const spec = p?.models.find((m) => m.id === s.model);
  return spec ? spec.supportsImages : true;
}
