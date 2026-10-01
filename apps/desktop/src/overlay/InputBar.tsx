import { ArrowUp, AtSign, Mic, Paperclip, Square } from "lucide-react";
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { api } from "../ipc/commands";
import { inTauri } from "../ipc/bridge";
import type { QuickCommand, SkillReview } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { IconButton, cx } from "../ui/primitives";
import { Floating } from "../ui/floating";
import { ChipList } from "./ChipList";
import { useSession } from "./session";

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
  const hasThread = useSession((s) => s.threadId !== null);
  const voice = useApp((s) => s.voice);
  const settings = useApp((s) => s.settings);

  const [value, setValue] = useState("");
  const [quick, setQuick] = useState<QuickCommand[]>([]);
  const [skills, setSkills] = useState<SkillReview[]>([]);
  const [menuIndex, setMenuIndex] = useState(0);
  const [atOpen, setAtOpen] = useState(false);
  const ref = useRef<HTMLTextAreaElement>(null);
  const bar = useRef<HTMLDivElement>(null);
  const lastVoice = useRef<string | null>(null);

  useEffect(() => {
    ref.current?.focus();
  }, [autoFocusKey]);

  useEffect(() => {
    void api.quickList().then(setQuick).catch(() => undefined);
    void api.skillsList().then(setSkills).catch(() => undefined);
  }, [autoFocusKey]);

  // Dictation result → caret (or send, when configured).
  useEffect(() => {
    if (voice.state !== "done" || voice.text === lastVoice.current) return;
    lastVoice.current = voice.text;
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
  }, [voice]);

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
    ];
    const cmds = quick.filter((q) => q.enabled).map((q) => ({ id: `q:${q.name}`, label: `/${q.name}`, hint: q.template.replace(/\{[^}]+\}/g, "…").split("\n")[0], insert: `/${q.name} ` }));
    const sk = skills.map((s) => ({ id: `s:${s.manifest.name}`, label: `/${s.manifest.name}`, hint: s.manifest.description, insert: `$${s.manifest.name} ` }));
    return [...builtins, ...cmds, ...sk];
  }, [quick, skills, t]);
  const shownSlash = slashQuery !== null ? filterMenu(slashItems, slashQuery) : [];

  const atItems: MenuItem[] = [
    { id: "tela", label: `@${t("context.screen").toLowerCase()}`, hint: "Ctrl+Shift+S", insert: "", run: () => void captureScreen(false) },
    { id: "regiao", label: `@${t("context.region").toLowerCase()}`, hint: "", insert: "", run: () => void api.regionOpen() },
    { id: "janela", label: `@${t("context.window").toLowerCase()}`, hint: "", insert: "", run: () => void captureScreen(true) },
    { id: "selecao", label: `@${t("context.selection").toLowerCase()}`, hint: "", insert: "", run: () => void captureSelection() },
    { id: "arquivo", label: `@${t("context.file").toLowerCase()}`, hint: "", insert: "", run: () => void pickFiles() },
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
      if (e.key === "Tab" || (e.key === "Enter" && !e.shiftKey && (atOpen || slashQuery !== ""))) {
        e.preventDefault();
        choose(menu[Math.min(menuIndex, menu.length - 1)]);
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
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void submit(e.ctrlKey);
    }
  };

  const onChange = (v: string) => {
    setValue(v);
    setMenuIndex(0);
    setAtOpen(/(^|\s)@\S*$/.test(v));
  };

  const listening = voice.state === "listening" || voice.state === "partial";
  const transcribing = voice.state === "transcribing";

  return (
    <div ref={bar} className="relative flex flex-col gap-1.5 px-3 py-2">
      {menu.length > 0 && (
        <Floating anchor={bar} placement={compact ? "below" : "above"} grow={compact} matchWidth className="px-3">
        <ul role="listbox" aria-label="Sugestões" className="menu-surface rounded-md border border-line py-1 shadow-lg">
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
          className="max-h-[180px] min-h-[28px] flex-1 resize-none bg-transparent py-1 text-[15px] leading-[21px] outline-none placeholder:text-muted focus-visible:outline-none"
        />
        <IconButton
          label={t("input.mic")}
          active={listening}
          onPointerDown={() => void api.pttPress().catch((e) => useApp.getState().notify("warning", String(e?.message ?? e)))}
          onPointerUp={() => void api.pttRelease(settings?.language === "en" ? null : "pt")}
          onPointerLeave={() => listening && void api.pttRelease(settings?.language === "en" ? null : "pt")}
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
            className="bg-accent text-white hover:bg-accent hover:text-white hover:brightness-110"
          >
            <ArrowUp size={17} />
          </IconButton>
        )}
      </div>
    </div>
  );
}
