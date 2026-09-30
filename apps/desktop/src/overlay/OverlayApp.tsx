// The floating Overlay: compact input bar that expands into a conversation.
// Keyboard map: docs/design/ui-ux.md ("Mapa de teclado").

import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { api } from "../ipc/commands";
import { inTauri } from "../ipc/bridge";
import type { PreviousApp } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { GLOBAL_THREAD, useConversation } from "../state/conversation";
import { UserInputCard } from "./UserInputCard";
import { WorkPanel } from "./WorkPanel";
import { Toasts } from "../ui/Toasts";
import { Kbd, Progress, cx } from "../ui/primitives";
import { percent } from "../lib/format";
import { Header } from "./Header";
import { HistoryPanel } from "./HistoryPanel";
import { InputBar } from "./InputBar";
import { ConsentCard, MessageList } from "./Messages";
import { LoginCard, WelcomeModal } from "./Onboarding";
import { FirstRun } from "./FirstRun";
import { blurAction, useSession } from "./session";
import { Minibar } from "./Minibar";

const COMPACT_WIDTH = 640;
const NO_BLOCKS: never[] = [];

/** In compact mode the window follows the content height. */
function useAutoHeight(ref: React.RefObject<HTMLDivElement | null>, compact: boolean) {
  useLayoutEffect(() => {
    if (!compact || !inTauri() || !ref.current) return;
    const el = ref.current;
    let last = 0;
    const obs = new ResizeObserver(async () => {
      const h = Math.ceil(el.getBoundingClientRect().height) + 2;
      if (Math.abs(h - last) < 2) return;
      last = h;
      const { getCurrentWindow, LogicalSize } = await import("@tauri-apps/api/window");
      await getCurrentWindow().setSize(new LogicalSize(COMPACT_WIDTH, h));
    });
    obs.observe(el);
    return () => obs.disconnect();
  }, [ref, compact]);
}

export function OverlayApp() {
  const t = useT();
  const ready = useApp((s) => s.ready);
  const auth = useApp((s) => s.auth);
  const welcome = useApp((s) => s.welcome);
  const consents = useApp((s) => s.consents);
  const downloads = useApp((s) => s.downloads);
  const settings = useApp((s) => s.settings);
  const appServer = useApp((s) => s.appServer);
  const session = useSession();
  const thread = useConversation((s) => (session.threadId ? s.threads[session.threadId] : undefined));
  const globalInputs = useConversation((s) => s.threads[GLOBAL_THREAD]?.blocks ?? NO_BLOCKS);
  const [focusKey, setFocusKey] = useState(0);
  const [previous, setPrevious] = useState<PreviousApp | null>(null);
  const shell = useRef<HTMLDivElement>(null);

  const expanded = session.overlayMode === "expanded" && !session.minibar;
  const running = thread?.running ?? false;
  const signedIn = !!auth?.active?.signedIn || session.providers.length > 0;
  useAutoHeight(shell, !expanded);

  useEffect(() => {
    void session.loadCatalog();
    void session.refreshChips();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.active?.clientId]);

  // Overlay shown by the host (hotkey, tray): focus, optional context.
  useEffect(() => {
    if (!inTauri()) return;
    let off: (() => void) | undefined;
    let offHidden: (() => void) | undefined;
    let offNew: (() => void) | undefined;
    let offChips: (() => void) | undefined;
    void (async () => {
      const { listen } = await import("@tauri-apps/api/event");
      off = await listen<{ previousApp: PreviousApp | null }>("aura://overlay", async (e) => {
        setPrevious(e.payload.previousApp);
        setFocusKey((k) => k + 1);
        const s = useSession.getState();
        await s.captureSelection();
        await s.applyProfile();
        if (useApp.getState().settings?.attachScreenOnOpen && !s.chips.some((c) => c.kind === "screen")) await s.captureScreen(false);
        await s.refreshChips();
      });
      offHidden = await listen("aura://overlay-hidden", () => useSession.getState().toggleHistory(false));
      offNew = await listen("aura://new-conversation", () => void useSession.getState().newConversation());
      offChips = await listen("aura://chips-changed", async () => {
        const s = useSession.getState();
        if (s.threadId) await api.trayMove("draft", s.threadId);
        await s.refreshChips();
      });
    })();
    return () => {
      off?.();
      offHidden?.();
      offNew?.();
      offChips?.();
    };
  }, []);

  // Answer finished while the user was elsewhere → system notification.
  const wasRunning = useRef(false);
  useEffect(() => {
    if (wasRunning.current && !running && thread && (session.minibar || document.hidden)) {
      const last = [...thread.blocks].reverse().find((b) => b.type === "assistant");
      const body = last && last.type === "assistant" ? last.text.replace(/[#*`>_]/g, "").slice(0, 140) : "";
      void api.notify(t("minibar.done"), body).catch(() => undefined);
    }
    wasRunning.current = running;
  }, [running, thread, session.minibar, t]);

  // Hide on blur unless configured to stay or an answer is streaming.
  useEffect(() => {
    if (!inTauri()) return;
    let off: (() => void) | undefined;
    void (async () => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      off = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        const s = useSession.getState();
        if (focused) {
          if (s.minibar) s.setMinibar(false);
          return;
        }
        const action = blurAction({
          keepOpen: useApp.getState().settings?.focusLoss === "keepOpen",
          busy: Object.values(useConversation.getState().threads).some((th) => th.running),
          pendingConsent: useApp.getState().consents.length > 0,
        });
        if (action === "hide") void api.overlayHide();
        else if (action === "minibar") s.setMinibar(true);
      });
    })();
    return () => off?.();
  }, []);

  // Files dropped on the window become attachments.
  useEffect(() => {
    if (!inTauri()) return;
    let off: (() => void) | undefined;
    void (async () => {
      const { getCurrentWebview } = await import("@tauri-apps/api/webview");
      off = await getCurrentWebview().onDragDropEvent((e) => {
        if (e.payload.type === "drop") void useSession.getState().attach(e.payload.paths);
      });
    })();
    return () => off?.();
  }, []);

  const onKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const s = useSession.getState();
      const k = e.key.toLowerCase();
      if (e.key === "Escape") {
        if (useApp.getState().voice.state === "listening") void api.pttCancel();
        else if (s.historyOpen) s.toggleHistory(false);
        else void api.overlayHide();
        e.preventDefault();
      } else if (e.ctrlKey && e.shiftKey && k === "s") {
        e.preventDefault();
        void s.captureScreen(false);
      } else if (e.ctrlKey && e.shiftKey && k === "e") {
        e.preventDefault();
        void s.newConversation(true);
      } else if (e.ctrlKey && !e.shiftKey && k === "n") {
        e.preventDefault();
        void s.newConversation();
      } else if (e.ctrlKey && k === "h") {
        e.preventDefault();
        s.toggleHistory();
      } else if (e.ctrlKey && e.key === ",") {
        e.preventDefault();
        void api.settingsOpen();
      } else if (e.ctrlKey && e.key === ".") {
        e.preventDefault();
        void s.interrupt();
      } else if (e.ctrlKey && e.key === "ArrowUp") {
        e.preventDefault();
        s.setOverlayMode("compact");
      } else if (e.ctrlKey && e.key === "ArrowDown") {
        e.preventDefault();
        s.setOverlayMode("expanded");
      }
    },
    [],
  );

  const codex = downloads["codex"];
  const status =
    codex && !codex.done && !codex.error
      ? t("download.codex", { pct: percent(codex.bytes, codex.total) })
      : appServer.state === "starting"
        ? t("state.starting")
        : appServer.state === "restarting"
          ? t("state.restarting")
          : appServer.state === "failed"
            ? t("state.failed", { reason: appServer.reason })
            : null;

  return (
    <div className="flex h-full w-full items-start justify-center p-px" onKeyDown={onKeyDown}>
      <div
        ref={shell}
        className={cx("overlay-shell fade-in flex w-full flex-col overflow-hidden", expanded ? "h-full" : "h-auto")}
        style={{ fontSize: 14 }}
      >
        {session.minibar ? <Minibar thread={thread} /> : null}
        {!session.minibar && expanded && <Header />}
        {session.minibar ? null : !ready ? null : !signedIn ? (
          <LoginCard />
        ) : (
          <>
            {welcome && <WelcomeModal />}
            {!welcome && settings && !settings.onboarded && <FirstRun />}
            {expanded && (
              <div className="flex min-h-0 flex-1">
                {session.historyOpen && <HistoryPanel />}
                <div className="flex min-w-0 flex-1 flex-col">
                  {thread && thread.blocks.length > 0 ? (
                    <MessageList thread={thread} />
                  ) : (
                    <div className="flex flex-1 flex-col items-center justify-center gap-2 px-6 text-center">
                      <h1 className="text-base font-semibold">{t("empty.title")}</h1>
                      <p className="max-w-sm text-[13px] text-muted">{t("empty.subtitle")}</p>
                      <div className="mt-1 flex flex-wrap justify-center gap-3 text-[12px] text-muted">
                        <span><Kbd>Ctrl+Shift+S</Kbd> {t("empty.tip1").replace("Ctrl+Shift+S", "").trim()}</span>
                        <span><Kbd>/</Kbd> {t("empty.tip2").replace("/", "").trim()}</span>
                        <span>{t("empty.tip3")}</span>
                      </div>
                      {consents.map((c) => (
                        <ConsentCard key={c.id} request={c} />
                      ))}
                    </div>
                  )}
                </div>
                {session.workOpen && session.threadId && <WorkPanel threadId={session.threadId} onClose={() => session.toggleWork(false)} />}
              </div>
            )}
            {!expanded && consents.length > 0 && (
              <div className="px-3 pt-2.5">
                {consents.map((c) => (
                  <ConsentCard key={c.id} request={c} />
                ))}
              </div>
            )}
            {globalInputs.map((b) =>
              b.type === "input" && !b.resolved ? (
                <div key={b.requestId} className="px-3 pt-2.5">
                  <UserInputCard requestId={b.requestId} source={b.source} prompt={b.prompt} autoResolveMs={b.autoResolveMs} />
                </div>
              ) : null,
            )}
            {status && (
              <div className="flex flex-col gap-1 border-t border-line px-3 py-1.5 text-[12px] text-muted">
                {status}
                {codex && !codex.done && <Progress value={percent(codex.bytes, codex.total)} />}
              </div>
            )}
            <div className={cx(expanded && "border-t border-line")}>
              <InputBar running={running} autoFocusKey={focusKey} />
            </div>
            {!expanded && previous && settings && (
              <div data-tauri-drag-region className="-mt-1 truncate px-4 pb-1.5 text-[11px] text-muted">
                {previous.processName} · {previous.title}
              </div>
            )}
          </>
        )}
      </div>
      <Toasts />
    </div>
  );
}
