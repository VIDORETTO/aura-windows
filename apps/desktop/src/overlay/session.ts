// The Overlay session: which conversation is active, which tray collects the
// next turn's chips, mode/provider/model, and the actions the UI calls.

import { create } from "zustand";
import { api, errorCode, errorMessage } from "../ipc/commands";
import type { ContextChip, ConversationMode, ModelInfo, OverlayMode, Provider, StartOptions } from "../ipc/types";
import { useApp } from "../state/app";
import { useConversation } from "../state/conversation";

export const DRAFT = "draft";
export const CHATGPT_PLAN = "aura-chatgpt-plan";

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
  /** Profile of the app in front (badge, defaults). */
  profile: import("../ipc/types").AppProfile | null;
  applyProfile: () => Promise<void>;
  tray: () => string;
  setMode: (m: ModeKey) => Promise<void>;
  setGranted: (folders: string[]) => Promise<void>;
  compact: () => Promise<void>;
  setProvider: (provider: string, model: string | null) => void;
  setModel: (model: string | null) => void;
  setOverlayMode: (m: OverlayMode) => void;
  toggleHistory: (open?: boolean) => void;
  toggleWork: (open?: boolean) => void;
  refreshChips: () => Promise<void>;
  removeChip: (id: string) => Promise<void>;
  captureScreen: (windowOnly?: boolean) => Promise<void>;
  captureSelection: () => Promise<void>;
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

export const useSession = create<Session>((set, get) => ({
  threadId: null,
  ephemeral: false,
  mode: "chat",
  granted: [],
  provider: CHATGPT_PLAN,
  model: null,
  chips: [],
  overlayMode: "compact",
  historyOpen: false,
  workOpen: false,
  minibar: false,
  models: [],
  providers: [],
  quickNames: [],
  sending: false,
  profile: null,

  tray: () => get().threadId ?? DRAFT,

  setMode: async (mode) => {
    set({ mode });
    const id = get().threadId;
    if (id) {
      try {
        await api.conversationSetMode(id, modeValue(mode, get().granted));
      } catch (e) {
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

  setProvider: (provider, model) => set({ provider, model }),
  setModel: (model) => set({ model }),

  setOverlayMode: (overlayMode) => {
    if (get().overlayMode === overlayMode) return;
    set({ overlayMode });
    void api.overlaySetMode(overlayMode).catch(() => undefined);
  },

  toggleHistory: (open) => {
    const next = open ?? !get().historyOpen;
    set({ historyOpen: next });
    if (next) get().setOverlayMode("expanded");
  },

  applyProfile: async () => {
    const profile = await api.profileActive().catch(() => null);
    set({ profile });
    if (!profile || get().threadId) return;
    if (profile.defaultMode) set({ mode: profile.defaultMode });
    if (profile.defaultModel) set({ model: profile.defaultModel });
    if (profile.attachScreen && !get().chips.some((c) => c.kind === "screen")) await get().captureScreen(false);
  },

  setMinibar: (on) => {
    set({ minibar: on });
    if (on) void api.overlaySetMode("compact").catch(() => undefined);
    else if (get().threadId) void api.overlaySetMode(get().overlayMode).catch(() => undefined);
  },

  toggleWork: (open) => {
    const next = open ?? !get().workOpen;
    set({ workOpen: next });
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

  captureSelection: async () => {
    try {
      const chip = await api.captureSelection(get().tray());
      if (chip) await get().refreshChips();
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
      await api.conversationSend({ threadId, text, tray: threadId, acceptsImages: accepts, options: { model: s.model } });
      return true;
    } catch (e) {
      if (errorCode(e) === "context") useApp.getState().notify("warning", errorMessage(e));
      else report(e);
      return false;
    } finally {
      set({ sending: false });
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
    set({ threadId: null, ephemeral, chips: [], historyOpen: false, mode: "chat", granted: [] });
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
    const [providers, models, quick] = await Promise.all([
      api.providersList().catch(() => [] as Provider[]),
      api.modelsList().catch(() => [] as ModelInfo[]),
      api.quickList().catch(() => []),
    ]);
    set({ providers, models, quickNames: quick.filter((q) => q.enabled).map((q) => q.name) });
  },
}));

function acceptsImages(s: Session): boolean {
  if (s.provider === CHATGPT_PLAN) {
    const m = s.models.find((x) => x.id === s.model) ?? s.models.find((x) => x.isDefault);
    return !m || m.inputModalities.length === 0 || m.inputModalities.includes("image");
  }
  const p = s.providers.find((x) => `aura-${x.id}` === s.provider);
  const spec = p?.models.find((m) => m.id === s.model);
  return spec ? spec.supportsImages : true;
}
