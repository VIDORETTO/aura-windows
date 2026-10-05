// Meeting panel (024): Preparo in three speeds, the live transcript with
// on-demand actions, and saved meetings. Nothing here starts on its own.

import { Mic, Pause, Play, Square, Star, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { Action, Meeting, Recipe, Utterance } from "../ipc/types";
import { useT, type MessageKey } from "../i18n";
import { useApp } from "../state/app";
import { Button, IconButton, Select, TextArea, cx } from "../ui/primitives";

export const mmss = (ms: number) => {
  const s = Math.max(0, Math.floor(ms / 1000));
  return `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
};

function Transcript({ lines }: { lines: Utterance[] }) {
  const t = useT();
  if (lines.length === 0) return <p className="px-3 py-2 text-[12px] text-muted">{t("meeting.transcript.empty")}</p>;
  return (
    <ol className="flex flex-col gap-1.5 px-3 py-2 text-[12px]" aria-label={t("meeting.transcript")}>
      {lines.map((u) => (
        <li key={u.id} className="selectable">
          <span className="mr-1.5 font-mono text-[11px] text-muted">{mmss(u.t0)}</span>
          <span className={cx("mr-1 font-medium", u.speaker === "you" ? "text-accent" : u.speaker === "note" ? "text-warning" : "text-fg")}>
            {u.speaker === "you" ? t("meeting.you") : u.speaker === "note" ? t("meeting.note") : t("meeting.them")}:
          </span>
          {u.text}
        </li>
      ))}
    </ol>
  );
}

/** Opens a conversation with the agent about a meeting (it reads it with `meeting_get`). */
const ask = (key: MessageKey, t: ReturnType<typeof useT>, vars: Record<string, string>, mode: "chat" | "task" = "chat") => void api.agentTask(t(key, vars), mode);

export function MeetingPanel() {
  const t = useT();
  const revision = useApp((s) => s.meetingRevision);
  const [active, setActive] = useState<Meeting | null>(null);
  const [past, setPast] = useState<Meeting[]>([]);
  const [brief, setBrief] = useState("");
  const [sentence, setSentence] = useState("");
  const [minutes, setMinutes] = useState(15);
  const [recipes, setRecipes] = useState<Recipe[]>([]);
  const [actions, setActions] = useState<Action[]>([]);
  const [kind, setKind] = useState("other");
  const [viewing, setViewing] = useState<Meeting | null>(null);
  const [lines, setLines] = useState<Utterance[]>([]);
  const [paused, setPaused] = useState(false);
  const [note, setNote] = useState("");
  const [now, setNow] = useState(() => Date.now());
  const fail = (e: unknown) => useApp.getState().notify("error", errorMessage(e));

  useEffect(() => {
    void (async () => {
      const [a, list, b] = await Promise.all([api.meetingActive(), api.meetingsList(), api.meetingBrief()]);
      setActive(a);
      setRecipes(await api.recipesList());
      setActions(await api.actionsList("open"));
      setPast(list.filter((m) => m.status !== "active"));
      if (b) setBrief(b);
      const shown = a ?? viewing;
      setLines(shown ? await api.meetingUtterances(shown.id) : []);
    })().catch(() => undefined);
  }, [revision, viewing]);

  useEffect(() => {
    if (!active) return;
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, [active]);

  const start = (withBrief: string) =>
    void api
      .meetingStart(t("meeting.untitled"), kind, withBrief)
      .then(() => setBrief(""))
      .catch(fail);

  if (active) {
    const id = active.id;
    return (
      <aside className="absolute inset-y-0 left-0 z-10 flex w-72 shrink-0 flex-col border-r border-line bg-[var(--surface-menu)] backdrop-blur-xl min-[1024px]:static min-[1024px]:z-auto" aria-label={t("meeting.title")}>
        <div className="flex items-center gap-1.5 border-b border-line px-3 py-2">
          <span className="h-2 w-2 shrink-0 rounded-full bg-danger" aria-hidden />
          <span className="min-w-0 flex-1 truncate text-[13px] font-medium" title={active.title}>
            {active.title}
          </span>
          <span className="font-mono text-[11px] text-muted" role="timer" aria-label={t("meeting.elapsed")}>
            {mmss(now - active.startedAt)}
          </span>
          <IconButton
            label={paused ? t("meeting.resume") : t("meeting.pause")}
            onClick={() => void api.meetingPause(!paused).then(() => setPaused(!paused)).catch(fail)}
          >
            {paused ? <Play size={14} /> : <Pause size={14} />}
          </IconButton>
          <IconButton label={t("meeting.stop")} onClick={() => void api.meetingStop().then(() => setPaused(false)).catch(fail)}>
            <Square size={13} />
          </IconButton>
        </div>
        {active.briefing && <p className="line-clamp-2 border-b border-line px-3 py-1.5 text-[11px] text-muted" title={active.briefing}>{active.briefing}</p>}
        <div className="min-h-0 flex-1 overflow-y-auto">
          <Transcript lines={lines} />
        </div>
        <form
          className="flex items-center gap-1 border-t border-line p-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (!note.trim()) return;
            void api.meetingNote(note).then(() => setNote("")).catch(fail);
          }}
        >
          <input
            aria-label={t("meeting.note.placeholder")}
            placeholder={t("meeting.note.placeholder")}
            value={note}
            onChange={(e) => setNote(e.target.value)}
            className="h-7 min-w-0 flex-1 rounded-md border border-line bg-surface-strong px-2 text-[12px] outline-none focus:border-accent"
          />
          <IconButton label={t("meeting.mark")} onClick={() => void api.meetingNote("★").catch(fail)}>
            <Star size={14} />
          </IconButton>
        </form>
        <div className="flex flex-wrap gap-1 border-t border-line p-2">
          <Button size="sm" onClick={() => ask("meeting.prompt.catchup", t, { id })}>{t("meeting.action.catchup")}</Button>
          <Button size="sm" onClick={() => ask("meeting.prompt.summary", t, { id })}>{t("meeting.action.summary")}</Button>
          <Button size="sm" onClick={() => ask("meeting.prompt.reply", t, { id })}>{t("meeting.action.reply")}</Button>
        </div>
      </aside>
    );
  }

  if (viewing) {
    const id = viewing.id;
    return (
      <aside className="absolute inset-y-0 left-0 z-10 flex w-72 shrink-0 flex-col border-r border-line bg-[var(--surface-menu)] backdrop-blur-xl min-[1024px]:static min-[1024px]:z-auto" aria-label={t("meeting.title")}>
        <div className="flex items-center gap-1.5 border-b border-line px-3 py-2">
          <button type="button" className="text-[12px] text-muted hover:text-fg" onClick={() => setViewing(null)}>← {t("meeting.back")}</button>
          <span className="min-w-0 flex-1 truncate text-right text-[13px] font-medium">{viewing.title}</span>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto">
          <Transcript lines={lines} />
        </div>
        <div className="flex flex-wrap gap-1 border-t border-line p-2">
          <Button size="sm" onClick={() => ask("meeting.prompt.after", t, { id })}>{t("meeting.action.after")}</Button>
          <Button size="sm" onClick={() => ask("meeting.prompt.promises", t, { id })}>{t("meeting.action.promises")}</Button>
          <Button size="sm" onClick={() => ask("meeting.prompt.email", t, { id })}>{t("meeting.action.email")}</Button>
          <Button size="sm" onClick={() => ask("meeting.prompt.minutes", t, { id }, "task")}>{t("meeting.action.minutes")}</Button>
          <IconButton label={t("meeting.delete")} onClick={() => void api.meetingDelete(id).then(() => setViewing(null)).catch(fail)}>
            <Trash2 size={14} />
          </IconButton>
        </div>
      </aside>
    );
  }

  return (
    <aside className="absolute inset-y-0 left-0 z-10 flex w-72 shrink-0 flex-col gap-3 overflow-y-auto border-r border-line bg-[var(--surface-menu)] p-3 backdrop-blur-xl min-[1024px]:static min-[1024px]:z-auto" aria-label={t("meeting.title")}>
      <h2 className="flex items-center gap-1.5 text-[13px] font-semibold"><Mic size={14} /> {t("meeting.new")}</h2>
      <p className="text-[11px] leading-snug text-muted">{t("meeting.optin")}</p>
      <button
        type="button"
        className="self-start text-[11px] text-muted underline-offset-2 hover:text-fg hover:underline"
        title={t("meeting.notice.text")}
        onClick={() =>
          void navigator.clipboard
            ?.writeText(t("meeting.notice.text"))
            .then(() => useApp.getState().notify("info", t("meeting.notice.copied")))
            .catch(() => undefined)
        }
      >
        {t("meeting.notice.copy")}
      </button>

      {brief && (
        <section className="flex flex-col gap-1.5 rounded-md border border-line p-2" aria-label={t("meeting.brief")}>
          <label htmlFor="meeting-brief" className="text-[12px] font-medium">{t("meeting.brief")}</label>
          <TextArea id="meeting-brief" rows={5} value={brief} onChange={(e) => setBrief(e.target.value)} />
          <Button variant="primary" size="sm" onClick={() => start(brief)}>{t("meeting.start")}</Button>
        </section>
      )}

      <section className="flex flex-col gap-1.5">
        <label htmlFor="meeting-recipe" className="text-[12px] font-medium">{t("meeting.recipe")}</label>
        <Select id="meeting-recipe" value={kind} onChange={(e) => setKind(e.target.value)} title={recipes.find((r) => r.id === kind)?.description}>
          <option value="other">{t("meeting.recipe.none")}</option>
          {recipes.map((r) => <option key={r.id} value={r.id}>{r.name}</option>)}
        </Select>
        <button type="button" className="self-start text-[11px] text-muted underline-offset-2 hover:text-fg hover:underline" onClick={() => ask("meeting.prompt.recipe", t, {})}>{t("meeting.recipe.create")}</button>
      </section>

      <section className="flex flex-col gap-1.5" aria-label={t("meeting.prepare")}>
        <label htmlFor="meeting-sentence" className="text-[12px] font-medium">⚡ {t("meeting.quick")}</label>
        <TextArea id="meeting-sentence" rows={2} value={sentence} placeholder={t("meeting.quick.placeholder")} onChange={(e) => setSentence(e.target.value)} />
        <div className="flex flex-wrap gap-1.5">
          <Button size="sm" disabled={!sentence.trim()} onClick={() => { ask("meeting.prompt.quick", t, { text: sentence.trim() }); setSentence(""); }}>{t("meeting.quick.send")}</Button>
          <Button size="sm" onClick={() => ask("meeting.prompt.grill", t, {})}>🧭 {t("meeting.grill")}</Button>
          <Button size="sm" variant="ghost" onClick={() => start("")}>▶ {t("meeting.startNow")}</Button>
        </div>
      </section>

      <section className="flex flex-col gap-1.5" aria-label={t("meeting.forgot")}>
        <span className="text-[12px] font-medium">{t("meeting.forgot")}</span>
        <div className="flex items-center gap-1.5">
          <Select aria-label={t("meeting.forgot.minutes")} value={minutes} onChange={(e) => setMinutes(Number(e.target.value))}>
            {[5, 10, 15, 30].map((m) => <option key={m} value={m}>{t("meeting.minutes", { n: m })}</option>)}
          </Select>
          <Button size="sm" onClick={() => void api.meetingFromBuffer(t("meeting.untitled"), minutes).catch(fail)}>{t("meeting.forgot.save")}</Button>
        </div>
        <p className="text-[11px] leading-snug text-muted">{t("meeting.forgot.hint")}</p>
      </section>

      {actions.length > 0 && (
        <section className="flex flex-col gap-1" aria-label={t("meeting.commitments")}>
          <span className="text-[12px] font-medium">{t("meeting.commitments")}</span>
          {actions.map((a) => (
            <label key={a.id} className="flex items-start gap-1.5 text-[12px]">
              <input
                type="checkbox"
                className="mt-0.5"
                checked={false}
                onChange={() => void api.actionDone(a.id, true).then(() => setActions((l) => l.filter((x) => x.id !== a.id))).catch(fail)}
              />
              <span className="min-w-0">
                <span className="text-muted">{a.owner === "you" ? t("meeting.owner.you") : t("meeting.owner.them")}:</span> {a.text}
                {a.due && <span className={cx("ml-1 text-[11px]", a.overdue ? "text-danger" : "text-muted")}>· {a.overdue ? t("meeting.overdue", { date: a.due }) : a.due}</span>}
              </span>
            </label>
          ))}
        </section>
      )}

      {past.length > 0 && (
        <section className="flex flex-col gap-0.5" aria-label={t("meeting.saved")}>
          <span className="text-[12px] font-medium">{t("meeting.saved")}</span>
          {past.map((m) => (
            <button key={m.id} type="button" className="truncate rounded px-1.5 py-1 text-left text-[12px] hover:bg-hover" onClick={() => setViewing(m)}>
              {m.title} <span className="text-muted">· {new Date(m.startedAt).toLocaleDateString()}</span>
            </button>
          ))}
        </section>
      )}
    </aside>
  );
}
