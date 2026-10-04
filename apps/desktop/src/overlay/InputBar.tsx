import { ArrowUp, AtSign, Mic, Paperclip, Square } from "lucide-react";
import { RecentClipDialog } from "./RecentClipDialog";
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { api, errorCode, errorMessage } from "../ipc/commands";
import { inTauri } from "../ipc/bridge";
import type { QuickCommand, SkillEntry } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { IconButton, cx } from "../ui/primitives";
import { Floating } from "../ui/floating";
import { ChipList } from "./ChipList";
import { useSession } from "./session";
import { VoiceInstallCard } from "./VoiceInstallCard";

export interface MenuItem {
  id: string;
  label: string;
  hint: string;
  insert: string;
  run?: () => void;
}

const MAX_LINES = 8;

/** Inserts `text` at the caret (adds a space when needed). */
export function insertAtCaret(value: string, caret: number, text: string): [string, number] {
  const before = value.slice(0, caret);
  const after = value.slice(caret);
  const sep = before && !/\s$/.test(before) ? " " : "";
  const trail = after && !/^\s/.test(after) ? " " : "";
  const next = before + sep + text + trail + after;
  return [next, before.length + sep.length + text.length];
}

/** Clipboard image types the model inputs accept. */
const PASTE_TYPES = ["image/png", "image/jpeg", "image/webp", "image/gif"];

function toBase64(file: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result).replace(/^data:[^,]*,/, ""));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

export function filterMenu(items: MenuItem[], query: string): MenuItem[] {
  const q = query.toLowerCase();
  return items.filter((i) => i.label.toLowerCase().includes(q)).slice(0, 8);
}

/** `compact`: the window follows the content, so menus open below and it grows. */
export function InputBar({ running, autoFocusKey, compact = false }: { running: boolean; autoFocusKey: number; compact?: boolean }) {
  const t = useT();
  const chips = useSession((s) => s.chips);
  const removeChip = useSession((s) => s.removeChip);
  const send = useSession((s) => s.send);
  const steer = useSession((s) => s.steer);
  const interrupt = useSession((s) => s.interrupt);
  const captureScreen = useSession((s) => s.captureScreen);
  const captureSelection = useSession((s) => s.captureSelection);
  const attach = useSession((s) => s.attach);
  const attachRecent = useSession((s) => s.attachRecent);
  const hasThread = useSession((s) => s.threadId !== null);
  const voice = useApp((s) => s.voice);
  const voiceResultRevision = useApp((s) => s.voiceResultRevision);
  const settings = useApp((s) => s.settings);

  const [value, setValue] = useState("");
  const [quick, setQuick] = useState<QuickCommand[]>([]);
  const [skills, setSkills] = useState<SkillEntry[]>([]);
  const [menuIndex, setMenuIndex] = useState(0);
  const [atOpen, setAtOpen] = useState(false);
  const [recentOpen, setRecentOpen] = useState(false);
  const ref = useRef<HTMLTextAreaElement>(null);
  const bar = useRef<HTMLDivElement>(null);
  const lastVoice = useRef(0);
  const voicePending = useRef(false);
  const [voiceBusy, setVoiceBusy] = useState(false);

  useEffect(() => {
    ref.current?.focus();
  }, [autoFocusKey]);

  useEffect(() => {
    if (voice.state === "failed" && voice.error === "worker_missing") {
      useApp.getState().notify("warning", t("voice.worker.missing"));
    }
  }, [voice, t]);

  useEffect(() => {
    let cancelled = false;
    void api.quickList().then((list) => { if (!cancelled) setQuick(list); }).catch(() => undefined);
    // Disabled skills leave the menu (008 AC-001).
    void api.skillsCatalog().then((list) => { if (!cancelled) setSkills(list.filter((s) => s.enabled)); }).catch(() => undefined);
    return () => { cancelled = true; };
  }, [autoFocusKey, settings?.language]);

  // Dictation result → caret (or send, when configured).
  useEffect(() => {
    if (voice.state !== "done" || voiceResultRevision === lastVoice.current) return;
    lastVoice.current = voiceResultRevision;
    const el = ref.current;
    const [next, caret] = insertAtCaret(value, el?.selectionStart ?? value.length, voice.text);
    if (settings?.sendAfterDictation) {
      setValue("");
      void send(next);
    } else {
      setValue(next);
      requestAnimationFrame(() => el?.setSelectionRange(caret, caret));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [voice, voiceResultRevision]);

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.style.height = "auto";
    const line = 21;
    el.style.height = `${Math.min(el.scrollHeight, line * MAX_LINES + 12)}px`;
  }, [value]);

  const slashQuery = value.startsWith("/") && !value.includes(" ") ? value.slice(1) : null;
  const slashItems = useMemo<MenuItem[]>(() => {
    const builtins: MenuItem[] = [
      { id: "plano", label: "/plano", hint: t("mode.plan.desc"), insert: "/plano " },
      { id: "tela", label: "/tela", hint: t("context.screen"), insert: "/tela " },
      { id: "compactar", label: "/compactar", hint: t("command.compact.hint"), insert: "/compactar" },
    ];
    const cmds = quick.filter((q) => q.enabled).map((q) => ({ id: `q:${q.name}`, label: `/${q.name}`, hint: q.template.replace(/\{[^}]+\}/g, "…").split("\n")[0], insert: `/${q.name} ` }));
    const sk = skills.map((s) => ({ id: `s:${s.name}`, label: `/${s.name}`, hint: s.description, insert: `$${s.name} ` }));
    return [...builtins, ...cmds, ...sk];
  }, [quick, skills, t]);
  const shownSlash = slashQuery !== null ? filterMenu(slashItems, slashQuery) : [];

  const atItems: MenuItem[] = [
    { id: "tela", label: `@${t("context.screen").toLowerCase()}`, hint: "Ctrl+Shift+S", insert: "", run: () => void captureScreen(false) },
    { id: "regiao", label: `@${t("context.region").toLowerCase()}`, hint: "", insert: "", run: () => void api.regionOpen() },
    { id: "janela", label: `@${t("context.window").toLowerCase()}`, hint: "", insert: "", run: () => void captureScreen(true) },
    { id: "selecao", label: `@${t("context.selection").toLowerCase()}`, hint: "", insert: "", run: () => void captureSelection(true) },
    { id: "arquivo", label: `@${t("context.file").toLowerCase()}`, hint: "", insert: "", run: () => void pickFiles() },
    { id: "recente", label: `@${t("context.recent").toLowerCase()}`, hint: t("context.recent.hint"), insert: "", run: () => setRecentOpen(true) },
  ];
  const menu = shownSlash.length > 0 ? shownSlash : atOpen ? atItems : [];

  const pickFiles = useCallback(async () => {
    if (!inTauri()) return;
    const { open } = await import("@tauri-apps/plugin-dialog");
    const picked = await open({ multiple: true, directory: false });
    const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
    if (paths.length) await attach(paths.map(String));
  }, [attach]);

  const choose = (item: MenuItem) => {
    if (item.run) {
      item.run();
      setAtOpen(false);
      setValue((v) => v.replace(/@\S*$/, ""));
    } else {
      setValue(item.insert);
    }
    setMenuIndex(0);
    ref.current?.focus();
  };

  const submit = async (steerMode: boolean) => {
    const text = value;
    if (steerMode && running) {
      setValue("");
      await steer(text);
      return;
    }
    if (running) return;
    if (await send(text)) setValue("");
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (menu.length > 0) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setMenuIndex((i) => (i + 1) % menu.length);
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        setMenuIndex((i) => (i - 1 + menu.length) % menu.length);
        return;
      }
      const item = menu[Math.min(menuIndex, menu.length - 1)];
      // Enter on a complete command (e.g. "/compactar") runs it instead of re-inserting it.
      const complete = e.key === "Enter" && !item.run && item.insert === value;
      if (!complete && (e.key === "Tab" || (e.key === "Enter" && !e.shiftKey && (atOpen || slashQuery !== "")))) {
        e.preventDefault();
        choose(item);
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        setAtOpen(false);
        if (slashQuery !== null) setValue("");
        return;
      }
    }
    // Ctrl+Shift+Enter inserts the answer into the previous app (Overlay).
    if (e.key === "Enter" && e.ctrlKey && e.shiftKey) {
      e.preventDefault();
      return;
    }
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void submit(e.ctrlKey);
    }
  };

  /** Ctrl+V with an image (002 AC-013): attach it; plain text pastes as usual. */
  const onPaste = (e: React.ClipboardEvent<HTMLTextAreaElement>) => {
    const images = [...(e.clipboardData?.items ?? [])].filter((i) => i.kind === "file" && i.type.startsWith("image/"));
    if (images.length === 0) return;
    e.preventDefault();
    for (const item of images) {
      if (!PASTE_TYPES.includes(item.type)) {
        useApp.getState().notify("warning", t("input.paste.unsupported", { type: item.type }));
        continue;
      }
      const file = item.getAsFile();
      if (file) void pasteImage(file);
    }
  };

  const pasteImage = async (file: File) => {
    try {
      const s = useSession.getState();
      await api.attachClipboardImage(s.tray(), s.threadId, file.type, await toBase64(file));
      await s.refreshChips();
    } catch (e) {
      useApp.getState().notify("error", errorMessage(e));
    }
  };

  const onChange = (v: string) => {
    setValue(v);
    setMenuIndex(0);
    setAtOpen(/(^|\s)@\S*$/.test(v));
  };

  const listening = voice.state === "listening" || voice.state === "partial";
  const transcribing = voice.state === "transcribing";
  const toggleVoice = async () => {
    if (voicePending.current || transcribing) return;
    voicePending.current = true;
    setVoiceBusy(true);
    try {
      if (listening) await api.pttRelease(null);
      else await api.pttPress();
    } catch (e) {
      const error = errorMessage(e);
      if (errorCode(e) === "asr" && (error === "model_missing" || error === "worker_missing")) {
        useApp.getState().handle({ channel: "voice", event: { state: "failed", error } });
      } else useApp.getState().notify("warning", error);
    } finally {
      voicePending.current = false;
      setVoiceBusy(false);
    }
  };

  return (
    <div ref={bar} className="relative flex flex-col gap-1.5 px-3 py-2">
      {voice.state === "failed" && voice.error === "model_missing" && (
        <VoiceInstallCard onReady={() => { useApp.getState().handle({ channel: "voice", event: { state: "idle" } }); ref.current?.focus(); }} />
      )}
      {menu.length > 0 && (
        <Floating anchor={bar} placement={compact ? "below" : "above"} grow={compact} matchWidth className="px-3">
        <ul role="listbox" aria-label={t("input.suggestions")} className="menu-surface rounded-md border border-line py-1 shadow-lg">
          {menu.map((m, i) => (
            <li
              key={m.id}
              role="option"
              aria-selected={i === menuIndex}
              onMouseDown={(e) => {
                e.preventDefault();
                choose(m);
              }}
              className={cx("flex cursor-pointer items-baseline gap-3 px-3 py-1.5 text-sm", i === menuIndex && "bg-hover")}
            >
              <span className="font-medium">{m.label}</span>
              <span className="truncate text-xs text-muted">{m.hint}</span>
            </li>
          ))}
        </ul>
        </Floating>
      )}
      {recentOpen && (
        <Floating anchor={bar} placement={compact ? "below" : "above"} grow={compact} matchWidth className="px-3">
          <RecentClipDialog
            onAttach={attachRecent}
            onClose={() => {
              setRecentOpen(false);
              ref.current?.focus();
            }}
          />
        </Floating>
      )}
      <ChipList chips={chips} onRemove={(id) => void removeChip(id)} grow={compact} />
      <div className="flex items-end gap-1.5">
        <div className="mb-1.5 flex h-5 w-5 shrink-0 items-center justify-center" aria-hidden>
          <span className={cx("h-2.5 w-2.5 rounded-full", running ? "animate-pulse bg-accent" : "bg-accent/70")} />
        </div>
        <textarea
          ref={ref}
          value={value}
          rows={1}
          aria-label={t("input.placeholder")}
          placeholder={voice.state === "partial" ? `🎙 ${voice.text}` : listening ? t("voice.listening") : transcribing ? t("voice.transcribing") : hasThread ? t("input.reply") : t("input.placeholder")}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={onKeyDown}
          onPaste={onPaste}
          className="max-h-[180px] min-h-[28px] flex-1 resize-none bg-transparent py-1 text-[15px] leading-[21px] outline-none placeholder:text-muted focus-visible:outline-none"
        />
        <IconButton
          label={t(listening ? "input.mic.finish" : "input.mic")}
          active={listening}
          aria-disabled={voiceBusy || transcribing}
          className={voiceBusy || transcribing ? "opacity-40" : undefined}
          onClick={() => void toggleVoice()}
          onKeyDown={(e) => { if (e.ctrlKey && e.key === " ") e.preventDefault(); }}
        >
          <Mic size={17} className={listening ? "text-danger" : undefined} />
        </IconButton>
        <IconButton label={t("input.context")} onClick={() => setAtOpen((o) => !o)}>
          <AtSign size={17} />
        </IconButton>
        <IconButton label={t("input.attach")} onClick={() => void pickFiles()}>
          <Paperclip size={17} />
        </IconButton>
        {running ? (
          <IconButton label={t("input.stop")} onClick={() => void interrupt()} className="bg-hover text-fg">
            <Square size={14} fill="currentColor" />
          </IconButton>
        ) : (
          <IconButton
            label={t("input.send")}
            disabled={!value.trim() && chips.length === 0}
            onClick={() => void submit(false)}
            className="bg-accent text-[var(--accent-contrast)] hover:bg-accent hover:text-[var(--accent-contrast)] hover:brightness-110"
          >
            <ArrowUp size={17} />
          </IconButton>
        )}
      </div>
    </div>
  );
}
