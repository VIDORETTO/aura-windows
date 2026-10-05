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
import { SpeechConsentDialog } from "../ui/SpeechConsentDialog";
import { speakText } from "../lib/speech";
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
import { ResizeHandles } from "./ResizeHandles";
import { StatusBar } from "./StatusBar";
import { floatBottom, useFloatLayout } from "../ui/floating";

const NO_BLOCKS: never[] = [];

/**
 * In compact mode the window follows the content height — plus room for any
 * open menu or preview, so nothing is clipped. The width is the user's.
 */
function useAutoHeight(ref: React.RefObject<HTMLDivElement | null>, compact: boolean) {
  const extra = useFloatLayout(floatBottom);
  useLayoutEffect(() => {
    if (!compact || !inTauri() || !ref.current) return;
    const el = ref.current;
    let cancelled = false;
    const fit = async () => {
      const { getCurrentWindow, LogicalSize } = await import("@tauri-apps/api/window");
      // A menu or mode change may have disposed this effect during the import.
      if (cancelled) return;
      const h = Math.max(64, Math.ceil(el.getBoundingClientRect().height), Math.ceil(extra));
      // Compare the actual viewport: remembering only the content height misses
      // external/native resizes that do not change the compact shell itself.
      if (Math.abs(h - window.innerHeight) < 2) return;
      await getCurrentWindow().setSize(new LogicalSize(window.innerWidth, h));
    };
    void fit();
    const obs = new ResizeObserver(() => void fit());
    obs.observe(el);
    const onResize = () => void fit();
    window.addEventListener("resize", onResize);
    return () => {
      cancelled = true;
      obs.disconnect();
      window.removeEventListener("resize", onResize);
    };
  }, [ref, compact, extra]);
}

/** Moves and resizes are remembered per monitor (001 AC-012). */
function usePersistPlacement() {
  useEffect(() => {
    if (!inTauri()) return;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const offs: (() => void)[] = [];
    const save = () => {
      clearTimeout(timer);
      timer = setTimeout(() => void api.overlayMoved().catch(() => undefined), 400);
    };
    void (async () => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const w = getCurrentWindow();
      offs.push(await w.onMoved(save), await w.onResized(save));
    })();
    return () => {
      clearTimeout(timer);
      offs.forEach((off) => off());
    };
  }, []);
}

export function OverlayApp() {
  const t = useT();
  const ready = useApp((s) => s.ready);
  const catalogRevision = useApp((s) => s.catalogRevision);
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
  usePersistPlacement();

  useEffect(() => {
    void session.loadCatalog();
    void session.refreshChips();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.active?.clientId, catalogRevision, settings?.language]);

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

  // Settings' access log asked to show a conversation (QA-031).
  const conversationRequest = useApp((s) => s.conversationRequest);
  useEffect(() => {
    if (conversationRequest) void useSession.getState().openConversation(conversationRequest.threadId);
  }, [conversationRequest]);

  // Answer finished while the user was elsewhere → system notification.
  // "Read answers automatically" (009 AC-005): on every finished turn of
  // the open conversation, read its last answer.
  useEffect(
    () =>
      useConversation.subscribe((s, prev) => {
        const id = useSession.getState().threadId;
        const before = id ? prev.threads[id] : undefined;
        const after = id ? s.threads[id] : undefined;
        if (!before?.running || !after || after.running || !useApp.getState().settings?.autoRead) return;
        const last = [...after.blocks].reverse().find((b) => b.type === "assistant");
        if (last && last.type === "assistant" && last.text.trim()) void speakText(last.text);
      }),
    [],
  );

  // A message queued during the answer goes out when the turn ends (015 AC-006).
  useEffect(
    () =>
      useConversation.subscribe((s, prev) => {
        const id = useSession.getState().threadId;
        if (!id || !prev.threads[id]?.running || s.threads[id]?.running !== false) return;
        if (useSession.getState().queued) void useSession.getState().flushQueue();
      }),
    [],
  );

  const wasRunning = useRef(false);
  useEffect(() => {
    if (wasRunning.current && !running && thread && (session.minibar || document.hidden)) {
      const last = [...thread.blocks].reverse().find((b) => b.type === "assistant");
      const body = last && last.type === "assistant" ? last.text.replace(/[#*`>_]/g, "").slice(0, 140) : "";
      void api.notify(t("minibar.done"), body).catch(() => undefined);
    }
    wasRunning.current = running;
  }, [running, thread, session.minibar, t]);

  // Stays open on blur (default); "hide on blur" hides it, or keeps a Minibar
  // while an answer is streaming.
  useEffect(() => {
    if (!inTauri()) return;
    let off: (() => void) | undefined;
    void (async () => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      off = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        const s = useSession.getState();
        if (focused) {
          if (s.minibar) s.setMinibar(false);
          // Back from another app: its current selection and title (012 AC-001).
          void s.captureSelection();
          void api.previousApp().then((app) => app && setPrevious(app)).catch(() => undefined);
          return;
        }
        const action = blurAction({
          keepOpen: !(useApp.getState().settings?.hideOnBlur ?? false),
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
        // Esc never hides the Overlay (001 AC-005, revision 2): the shortcut
        // and "minimize to tray" do.
        if (["listening", "partial"].includes(useApp.getState().voice.state)) void api.pttCancel();
        else if (s.historyOpen) s.toggleHistory(false);
        e.preventDefault();
      } else if (e.ctrlKey && e.shiftKey && k === "s") {
        e.preventDefault();
        void s.captureScreen(false);
      } else if (e.ctrlKey && e.shiftKey && k === "l") {
        // Listen to the last answer; again stops (009 AC-005).
        e.preventDefault();
        const text = lastAnswer();
        if (text) void speakText(text);
      } else if (e.ctrlKey && e.shiftKey && e.key === "Enter") {
        // Insert the selected code block, or the last answer, into the app
        // the Overlay was opened over (009 AC-009).
        e.preventDefault();
        const text = selectedAnswerText() ?? lastAnswer();
        if (text) void api.insertIntoApp(text);
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
    <div className="flex h-full w-full items-start justify-center" onKeyDown={onKeyDown}>
      <ResizeHandles compact={!expanded} />
      <div
        ref={shell}
        className={cx("overlay-shell fade-in flex w-full flex-col overflow-hidden", expanded ? "h-full" : "h-auto")}
        style={{ fontSize: 14 }}
      >
        {session.minibar ? <Minibar thread={thread} /> : null}
        {!session.minibar && <Header compact={!expanded} subtitle={previous ? `${previous.processName} · ${previous.title}` : null} />}
        {session.minibar ? null : !ready ? null : !signedIn ? (
          <LoginCard />
        ) : (
          <>
            {welcome && <WelcomeModal />}
            {!welcome && settings && !settings.onboarded && <FirstRun />}
            {expanded && (
              <div className="relative flex min-h-0 flex-1">
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
                        <span><Kbd>@</Kbd> {t("empty.tip4").replace("@", "").trim()}</span>
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
              <InputBar running={running} autoFocusKey={focusKey} compact={!expanded} />
            </div>
            {expanded && <StatusBar />}
            {!expanded && <Toasts inline />}
          </>
        )}
      </div>
      {(expanded || session.minibar || !ready || !signedIn) && <Toasts />}
      <SpeechConsentDialog />
    </div>
  );
}

/** Last answer of the open conversation. */
function lastAnswer(): string | null {
  const id = useSession.getState().threadId;
  const thread = id ? useConversation.getState().threads[id] : undefined;
  const last = thread ? [...thread.blocks].reverse().find((b) => b.type === "assistant") : undefined;
  return last && last.type === "assistant" && last.text.trim() ? last.text : null;
}

/** Text selected inside an answer (e.g. a code block), if any. */
function selectedAnswerText(): string | null {
  const sel = typeof window !== "undefined" ? window.getSelection() : null;
  const text = sel?.toString() ?? "";
  if (!sel || sel.rangeCount === 0 || !text.trim()) return null;
  const node = sel.getRangeAt(0).commonAncestorContainer;
  const el = node instanceof Element ? node : node.parentElement;
  return el?.closest("[data-answer]") ? text : null;
}
