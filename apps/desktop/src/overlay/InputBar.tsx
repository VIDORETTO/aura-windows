import { ArrowUp, AtSign, Clock, Mic, Paperclip, Square, X } from "lucide-react";
import { RecentClipDialog } from "./RecentClipDialog";
import { useCallback, useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from "react";
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
import { filterMenu, findTrigger, quickHint, replaceTrigger, templatePreview } from "./composer";

export interface MenuItem {
  id: string;
  label: string;
  hint: string;
  /** Text that replaces the trigger word (commands); empty for actions. */
  insert: string;
  run?: () => void;
  aliases?: string[];
  section: "commands" | "skills" | "context";
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

export { filterMenu };

const SECTION_KEYS = { commands: "menu.commands", skills: "menu.skills", context: "menu.context" } as const;
const RESERVED = new Set(["plano", "plan", "compactar", "compact", "tela", "screen"]);

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
  const attachSkill = useSession((s) => s.attachSkill);
  const queued = useSession((s) => s.queued);
  const compose = useSession((s) => s.compose);
  const hasThread = useSession((s) => s.threadId !== null);
  const voice = useApp((s) => s.voice);
  const voiceResultRevision = useApp((s) => s.voiceResultRevision);
  const settings = useApp((s) => s.settings);

  const [value, setValue] = useState("");
  const [caret, setCaret] = useState(0);
  const [quick, setQuick] = useState<QuickCommand[]>([]);
  const [skills, setSkills] = useState<SkillEntry[]>([]);
  const [menuIndex, setMenuIndex] = useState(0);
  /** The user moved through the menu with the arrows (Enter then picks). */
  const [navigated, setNavigated] = useState(false);
  /** Context menu opened by the @ button, without typing. */
  const [atForced, setAtForced] = useState(false);
  /** Esc closed the menu for this exact text and caret. */
  const [dismissed, setDismissed] = useState<string | null>(null);
  const [recentOpen, setRecentOpen] = useState(false);
  const ref = useRef<HTMLTextAreaElement>(null);
  const bar = useRef<HTMLDivElement>(null);
  const lastVoice = useRef(0);
  const voicePending = useRef(false);
  const [voiceBusy, setVoiceBusy] = useState(false);
  const listId = useId();

  useEffect(() => {
    ref.current?.focus();
  }, [autoFocusKey]);

  const update = (next: string, at: number) => {
    setValue(next);
    setCaret(at);
    requestAnimationFrame(() => ref.current?.setSelectionRange(at, at));
  };

  // "Edit" on a message puts its text here (015).
  useEffect(() => {
    if (!compose) return;
    update(compose.text, compose.text.length);
    ref.current?.focus();
  }, [compose]);

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
    const [next, at] = insertAtCaret(value, el?.selectionStart ?? value.length, voice.text);
    if (settings?.sendAfterDictation) {
      setValue("");
      void send(next);
    } else {
      update(next, at);
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

  const pickFiles = useCallback(async () => {
    if (!inTauri()) return;
    const { open } = await import("@tauri-apps/plugin-dialog");
    const picked = await open({ multiple: true, directory: false });
    const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
    if (paths.length) await attach(paths.map(String));
  }, [attach]);

  // `/` does something; `@` adds context (015).
  const commandItems = useMemo<MenuItem[]>(() => {
    const plan = t("command.plan.name");
    const compactName = t("command.compact.name");
    const builtins: MenuItem[] = [
      { id: "plano", label: `/${plan}`, aliases: ["plano", "plan"], hint: t("mode.plan.desc"), insert: `/${plan} `, section: "commands" },
      { id: "compactar", label: `/${compactName}`, aliases: ["compactar", "compact"], hint: t("command.compact.hint"), insert: `/${compactName}`, section: "commands" },
    ];
    const cmds: MenuItem[] = quick
      .filter((q) => q.enabled && !RESERVED.has(q.name))
      .map((q) => ({
        id: `q:${q.name}`,
        label: `/${q.name}`,
        hint: templatePreview(q.template),
        insert: `/${q.name} `,
        section: "commands" as const,
      }));
    const sk: MenuItem[] = skills.map((s) => ({ id: `s:${s.name}`, label: `/${s.name}`, hint: s.description, insert: "", run: () => void attachSkill(s.name), section: "skills" }));
    return [...builtins, ...cmds, ...sk];
  }, [quick, skills, attachSkill, t]);

  // One typable word per item; the full description goes in the hint (`@janela`, not `@janela ativa`).
  const contextItems: MenuItem[] = [
    { id: "tela", label: `@${t("context.at.screen")}`, aliases: ["tela", "screen"], hint: "Ctrl+Shift+S", insert: "", run: () => void captureScreen(false), section: "context" },
    { id: "regiao", label: `@${t("context.at.region")}`, aliases: ["regiao", "region"], hint: t("context.region.hint"), insert: "", run: () => void api.regionOpen(), section: "context" },
    { id: "janela", label: `@${t("context.at.window")}`, aliases: ["janela", "window"], hint: t("context.window"), insert: "", run: () => void captureScreen(true), section: "context" },
    { id: "selecao", label: `@${t("context.at.selection")}`, aliases: ["selecao", "selection"], hint: t("context.selection.hint"), insert: "", run: () => void captureSelection(true), section: "context" },
    { id: "arquivo", label: `@${t("context.at.file")}`, aliases: ["arquivo", "file"], hint: t("context.file.hint"), insert: "", run: () => void pickFiles(), section: "context" },
    { id: "recente", label: `@${t("context.at.recent")}`, aliases: ["recente", "recent", "ultimos"], hint: t("context.recent.hint"), insert: "", run: () => setRecentOpen(true), section: "context" },
  ];

  const trigger = findTrigger(value, caret);
  const menuKey = `${value}\u0000${caret}`;
  const menu: MenuItem[] =
    dismissed === menuKey
      ? []
      : trigger
        ? filterMenu(trigger.kind === "/" ? commandItems : contextItems, trigger.query, trigger.kind === "/" ? 10 : 8)
        : atForced
          ? contextItems
          : [];
  const active = Math.min(menuIndex, Math.max(menu.length - 1, 0));
  const optionId = (i: number) => `${listId}-opt-${i}`;

  // Hint for a typed quick command: argument and text source (015 AC-005).
  const typedQuick = /^\/(\S+)\s/.exec(value);
  const hintCmd = typedQuick ? quick.find((q) => q.enabled && q.name === typedQuick[1]) : undefined;
  const hint = hintCmd ? quickHint(hintCmd.template) : null;

  const closeMenu = () => {
    setAtForced(false);
    setNavigated(false);
    setMenuIndex(0);
  };

  const choose = (item: MenuItem) => {
    if (item.run) {
      item.run();
      if (trigger) update(...replaceTrigger(value, trigger, ""));
    } else if (trigger) {
      update(...replaceTrigger(value, trigger, item.insert));
    } else {
      update(item.insert, item.insert.length);
    }
    closeMenu();
    ref.current?.focus();
  };

  const submit = async (steerMode: boolean) => {
    const text = value;
    if (running) {
      if (!text.trim()) return;
      setValue("");
      setCaret(0);
      // Ctrl+Enter steers the running turn; Enter queues for after it (015 AC-006).
      if (steerMode) await steer(text);
      else useSession.getState().queue(text);
      return;
    }
    if (await send(text)) {
      setValue("");
      setCaret(0);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (menu.length > 0) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setNavigated(true);
        setMenuIndex((active + 1) % menu.length);
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        setNavigated(true);
        setMenuIndex((active - 1 + menu.length) % menu.length);
        return;
      }
      if (e.key === "Escape") {
        // Closes the menu and keeps the text (015 AC-002).
        e.preventDefault();
        e.stopPropagation();
        setDismissed(menuKey);
        closeMenu();
        return;
      }
      const item = menu[active];
      if (e.key === "Tab") {
        e.preventDefault();
        choose(item);
        return;
      }
      if (e.key === "Enter" && !e.shiftKey && !e.ctrlKey) {
        // Enter on a complete command (e.g. "/compactar") runs it instead of re-inserting it.
        // Aliases count too: "/compactar" with the interface in English.
        const typed = value.trim();
        const complete = !item.run && !navigated && (item.insert.trim() === typed || (item.aliases ?? []).some((a) => `/${a}` === typed));
        if (!complete) {
          e.preventDefault();
          // A bare trigger picks only after the arrows; "/" or "@" alone is never sent.
          if (trigger && trigger.query === "" && !navigated) return;
          choose(item);
          return;
        }
      }
    }
    // Ctrl+Shift+Enter inserts the answer into the previous app (Overlay).
    if (e.key === "Enter" && e.ctrlKey && e.shiftKey) {
      e.preventDefault();
      return;
    }
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (!running && /^\/\s*$/.test(value)) return;
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

  const onChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setValue(e.target.value);
    setCaret(e.target.selectionStart ?? e.target.value.length);
    setMenuIndex(0);
    setNavigated(false);
    setAtForced(false);
    setDismissed(null);
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

  const sections = (["commands", "skills", "context"] as const)
    .map((section) => ({ section, items: menu.map((m, i) => ({ m, i })).filter(({ m }) => m.section === section) }))
    .filter((g) => g.items.length > 0);

  return (
    <div ref={bar} className="relative flex flex-col gap-1.5 px-3 py-2">
      {voice.state === "failed" && voice.error === "model_missing" && (
        <VoiceInstallCard onReady={() => { useApp.getState().handle({ channel: "voice", event: { state: "idle" } }); ref.current?.focus(); }} />
      )}
      {menu.length > 0 && (
        <Floating anchor={bar} placement={compact ? "below" : "above"} grow={compact} matchWidth className="px-3">
          <div id={listId} role="listbox" aria-label={t("input.suggestions")} className="menu-surface max-h-80 overflow-y-auto rounded-md border border-line py-1 shadow-lg">
            {sections.map(({ section, items }) => (
              <div key={section} role="group" aria-label={t(SECTION_KEYS[section])}>
                {sections.length > 1 && (
                  <div role="presentation" className="px-3 pb-0.5 pt-1.5 text-[10px] font-medium uppercase tracking-wide text-muted">
                    {t(SECTION_KEYS[section])}
                  </div>
                )}
                {items.map(({ m, i }) => (
                  <div
                    key={m.id}
                    id={optionId(i)}
                    role="option"
                    aria-selected={i === active}
                    onMouseDown={(e) => {
                      e.preventDefault();
                      choose(m);
                    }}
                    className={cx("flex cursor-pointer items-baseline gap-3 px-3 py-1.5 text-sm", i === active && "bg-hover")}
                  >
                    <span className="shrink-0 font-medium">{m.label}</span>{" "}
                    <span className="truncate text-xs text-muted">{m.hint}</span>
                  </div>
                ))}
              </div>
            ))}
          </div>
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
      {queued && (
        <div className="flex items-center gap-1.5 self-start rounded-full border border-line bg-surface-strong py-0.5 pl-2 pr-1 text-xs">
          <Clock size={12} className="text-accent" aria-hidden />
          <span className="max-w-[320px] truncate">{t("queue.label", { text: queued })}</span>
          <button type="button" aria-label={t("queue.cancel")} className="rounded p-0.5 text-muted hover:bg-hover hover:text-fg" onClick={() => useSession.getState().cancelQueue()}>
            <X size={12} />
          </button>
        </div>
      )}
      <ChipList chips={chips} onRemove={(id) => void removeChip(id)} grow={compact} />
      {hintCmd && hint && (
        <p className="pl-7 text-[11px] text-muted">
          <span className="font-medium text-fg">/{hintCmd.name}</span>
          {hint.arg !== null && <> {hint.arg ? t("quick.hint.arg", { arg: hint.arg }) : t("quick.hint.argAny")}</>}
          {hint.source && <> · {t(hint.source === "selection" ? "quick.hint.selection" : "quick.hint.typed")}</>}
          {hint.screen && <> · {t("quick.hint.screen")}</>}
        </p>
      )}
      <div className="flex items-end gap-1.5">
        <div className="mb-1.5 flex h-5 w-5 shrink-0 items-center justify-center" aria-hidden>
          <span className={cx("h-2.5 w-2.5 rounded-full", running ? "animate-pulse bg-accent" : "bg-accent/70")} />
        </div>
        <textarea
          ref={ref}
          value={value}
          rows={1}
          role="combobox"
          aria-autocomplete="list"
          aria-expanded={menu.length > 0}
          aria-controls={menu.length > 0 ? listId : undefined}
          aria-activedescendant={menu.length > 0 ? optionId(active) : undefined}
          aria-label={t("input.placeholder")}
          placeholder={voice.state === "partial" ? `🎙 ${voice.text}` : listening ? t("voice.listening") : transcribing ? t("voice.transcribing") : running ? t("input.running") : hasThread ? t("input.reply") : t("input.placeholder")}
          onChange={onChange}
          onSelect={(e) => setCaret(e.currentTarget.selectionStart ?? 0)}
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
        <IconButton
          label={t("input.context")}
          onClick={() => {
            setDismissed(null);
            setMenuIndex(0);
            setAtForced((o) => !o);
            ref.current?.focus();
          }}
        >
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
