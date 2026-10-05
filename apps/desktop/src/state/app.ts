// App-wide state: settings, privacy, account, pending consents, downloads,
// voice and toasts. Fed by `bootstrap()` and the host event stream.

import { create } from "zustand";
import { api } from "../ipc/commands";
import { accentContrast } from "../lib/format";
import { onHostEvent } from "../ipc/bridge";
import type {
  AudioSourceKind,
  AppServerState,
  AuthStatus,
  ConsentRequest,
  HostEvent,
  LoginProgress,
  PrivacyView,
  PttState,
  Settings,
} from "../ipc/types";
import { useConversation } from "./conversation";

export interface Notice {
  id: number;
  level: "info" | "warning" | "error";
  message: string;
}

export interface Download {
  id: string;
  bytes: number;
  total: number | null;
  done: boolean;
  error: string | null;
}

interface AppStore {
  catalogRevision: number;
  /** Latest dBFS per source while a device test runs in Settings. */
  audioLevels: Partial<Record<AudioSourceKind, number>>;
  ready: boolean;
  settings: Settings | null;
  privacy: PrivacyView | null;
  auth: AuthStatus | null;
  login: LoginProgress | null;
  welcome: boolean;
  consents: ConsentRequest[];
  downloads: Record<string, Download>;
  voice: PttState;
  /** Ordered terminal results, independent of transcript content. */
  voiceResultRevision: number;
  /** Sources recording in the background (indicators). */
  recording: string[];
  appServer: AppServerState;
  notices: Notice[];
  /** Conversation another window asked the Overlay to show (access log). */
  conversationRequest: { threadId: string; seq: number } | null;
  /** "Create with AI" asked the Overlay for a new agent conversation (017). */
  agentRequest: { text: string; mode: "chat" | "task" | "plan"; seq: number } | null;
  /** Bumped when skills, quick commands or MCP servers change elsewhere. */
  extensionsRevision: number;
  setSettings: (s: Settings) => void;
  setPrivacy: (p: PrivacyView) => void;
  refreshAuth: () => Promise<void>;
  notify: (level: Notice["level"], message: string) => void;
  dismiss: (id: number) => void;
  dismissWelcome: () => void;
  handle: (e: HostEvent) => void;
}

let noticeSeq = 0;

export const useApp = create<AppStore>((set, get) => ({
  catalogRevision: 0,
  audioLevels: {},
  ready: false,
  settings: null,
  privacy: null,
  auth: null,
  login: null,
  welcome: false,
  consents: [],
  downloads: {},
  voice: { state: "idle" },
  voiceResultRevision: 0,
  recording: [],
  appServer: { state: "stopped" },
  notices: [],
  conversationRequest: null,
  agentRequest: null,
  extensionsRevision: 0,
  setSettings: (settings) => {
    set({ settings });
    applyTheme(settings);
  },
  setPrivacy: (privacy) => set({ privacy }),
  refreshAuth: async () => set({ auth: await api.authStatus() }),
  notify: (level, message) => {
    // The same message twice (e.g. a shortcut conflict) shows once.
    if (get().notices.some((n) => n.message === message)) return;
    const id = ++noticeSeq;
    set((s) => ({ notices: [...s.notices.slice(-3), { id, level, message }] }));
    setTimeout(() => get().dismiss(id), level === "error" ? 8000 : 4500);
  },
  dismiss: (id) => set((s) => ({ notices: s.notices.filter((n) => n.id !== id) })),
  dismissWelcome: () => {
    const active = get().auth?.active;
    set({ welcome: false });
    if (active) void api.authMarkWelcomed(active.clientId);
  },
  handle: (e) => {
    switch (e.channel) {
      case "audioLevel":
        set((s) => ({ audioLevels: { ...s.audioLevels, [e.event.source]: e.event.dbfs } }));
        break;
      case "providersChanged":
        set((s) => ({ catalogRevision: s.catalogRevision + 1 }));
        break;
      case "conversation":
        if (e.event.type === "appServerState") set({ appServer: e.event.state });
        else useConversation.getState().apply(e.event);
        break;
      case "login":
        set({ login: e.event.state === "completed" || e.event.state === "cancelled" ? null : e.event });
        if (e.event.state === "completed") {
          set({ welcome: e.event.firstTime || !e.event.account.welcomed });
          void get().refreshAuth();
        }
        break;
      case "consent":
        set((s) => ({ consents: [...s.consents.filter((c) => c.id !== e.event.id), e.event] }));
        break;
      case "consentResolved":
        set((s) => ({ consents: s.consents.filter((c) => c.id !== e.event.id) }));
        break;
      case "privacy":
        set((s) => ({
          recording: e.event.recording ?? [],
          privacy: s.privacy ? { ...s.privacy, paused: e.event.paused } : s.privacy,
        }));
        break;
      case "download":
        set((s) => ({ downloads: { ...s.downloads, [e.event.id]: e.event } }));
        if (e.event.error && e.event.error !== "cancelled") get().notify("error", e.event.error);
        break;
      case "voice":
        set((s) => ({
          voice: e.event,
          voiceResultRevision: s.voiceResultRevision + (e.event.state === "done" && s.voice.state !== "done" ? 1 : 0),
        }));
        break;
      case "notice":
        get().notify(e.event.level, e.event.message);
        break;
      case "openConversation":
        set((s) => ({ conversationRequest: { threadId: e.event.threadId, seq: (s.conversationRequest?.seq ?? 0) + 1 } }));
        break;
      case "agentTask":
        set((s) => ({ agentRequest: { ...e.event, seq: (s.agentRequest?.seq ?? 0) + 1 } }));
        break;
      case "reminder":
        // One toast: the Overlay window shows it, Settings/region windows do not.
        if (!/^#\/(settings|region)/.test(window.location.hash)) {
          void api.notify("Aura", e.event.text).catch(() => {});
        }
        break;
      case "extensionsChanged":
        set((s) => ({ extensionsRevision: s.extensionsRevision + 1 }));
        break;
      case "settings":
        get().setSettings(e.event);
        break;
    }
  },
}));

export function applyTheme(s: Settings) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  if (s.theme === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", s.theme);
  root.style.setProperty("--overlay-opacity", String(s.opacity));
  // Accent chosen by the user (012); high contrast keeps the system colors.
  const forced = typeof matchMedia === "function" && matchMedia("(forced-colors: active)").matches;
  if (s.accentColor && !forced) {
    root.style.setProperty("--accent", s.accentColor);
    root.style.setProperty("--accent-contrast", accentContrast(s.accentColor));
  } else {
    root.style.removeProperty("--accent");
    root.style.removeProperty("--accent-contrast");
  }
  root.lang = s.language === "en" ? "en" : "pt-BR";
}

/** Loads initial state and subscribes to host events (idempotent). */
let booted: Promise<void> | null = null;
export function bootstrap(): Promise<void> {
  if (booted) return booted;
  booted = (async () => {
    await onHostEvent((e) => useApp.getState().handle(e));
    const [settings, privacy, auth] = await Promise.all([api.settingsGet(), api.privacyGet(), api.authStatus()]);
    useApp.getState().setSettings(settings);
    useApp.setState({ privacy, auth, ready: true });
  })();
  return booted;
}

/** Tests reset the singleton between runs. */
export function resetBootstrap() {
  booted = null;
}
