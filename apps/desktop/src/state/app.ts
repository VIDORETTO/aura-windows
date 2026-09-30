// App-wide state: settings, privacy, account, pending consents, downloads,
// voice and toasts. Fed by `bootstrap()` and the host event stream.

import { create } from "zustand";
import { api } from "../ipc/commands";
import { onHostEvent } from "../ipc/bridge";
import type {
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
  ready: boolean;
  settings: Settings | null;
  privacy: PrivacyView | null;
  auth: AuthStatus | null;
  login: LoginProgress | null;
  welcome: boolean;
  consents: ConsentRequest[];
  downloads: Record<string, Download>;
  voice: PttState;
  /** Sources recording in the background (indicators). */
  recording: string[];
  appServer: AppServerState;
  notices: Notice[];
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
  ready: false,
  settings: null,
  privacy: null,
  auth: null,
  login: null,
  welcome: false,
  consents: [],
  downloads: {},
  voice: { state: "idle" },
  recording: [],
  appServer: { state: "stopped" },
  notices: [],
  setSettings: (settings) => {
    set({ settings });
    applyTheme(settings);
  },
  setPrivacy: (privacy) => set({ privacy }),
  refreshAuth: async () => set({ auth: await api.authStatus() }),
  notify: (level, message) => {
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
      case "conversation":
        if (e.event.type === "appServerState") set({ appServer: e.event.state });
        else useConversation.getState().apply(e.event);
        break;
      case "login":
        set({ login: e.event.state === "completed" ? null : e.event });
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
        if (e.event.error) get().notify("error", e.event.error);
        break;
      case "voice":
        set({ voice: e.event });
        break;
      case "notice":
        get().notify(e.event.level, e.event.message);
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
