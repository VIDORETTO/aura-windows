// Reminders, notes and saved texts (020): see and delete what the agent or
// the ⭐ button stored. Everything is local.
import { Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { NoteItem, ReminderItem } from "../ipc/types";
import { useT } from "../i18n";
import { exactTime } from "../lib/format";
import { useApp } from "../state/app";
import { Button, Section, TextField } from "../ui/primitives";

const fail = (e: unknown) => useApp.getState().notify("error", errorMessage(e));

function Reminders() {
  const t = useT();
  const lang = useApp((s) => s.settings?.language ?? "ptBr");
  const [items, setItems] = useState<ReminderItem[]>([]);
  const load = useCallback(() => void api.remindersAll().then(setItems).catch(fail), []);
  useEffect(load, [load]);
  return (
    <Section title={t("personal.reminders")} description={t("personal.reminders.hint")}>
      {items.length === 0 && <p className="text-sm text-muted">{t("personal.reminders.empty")}</p>}
      <ul className="space-y-1">
        {items.map((r) => (
          <li key={r.id} className="flex items-start gap-2 rounded-md border border-border px-3 py-2 text-sm">
            <div className="min-w-0 flex-1">
              <div>{r.text}</div>
              <div className="text-xs text-muted">
                {exactTime(r.dueAt, lang)}
                {r.repeat !== "none" && ` · ${r.repeat}`}
                {r.prompt && ` · ${t("personal.reminders.agent")}: ${r.prompt}`}
              </div>
            </div>
            <Button variant="ghost" aria-label={`${t("personal.delete")}: ${r.text}`} onClick={() => void api.reminderRemove(r.id).then(load).catch(fail)}>
              <Trash2 size={14} />
            </Button>
          </li>
        ))}
      </ul>
    </Section>
  );
}

function Notes({ kind, title }: { kind: "note" | "saved"; title: string }) {
  const t = useT();
  const lang = useApp((s) => s.settings?.language ?? "ptBr");
  const [items, setItems] = useState<NoteItem[]>([]);
  const [query, setQuery] = useState("");
  const load = useCallback(() => void api.notesAll(kind, query).then(setItems).catch(fail), [kind, query]);
  useEffect(load, [load]);
  return (
    <Section title={title}>
      <TextField aria-label={`${t("personal.search")}: ${title}`} placeholder={t("personal.search")} value={query} onChange={(e) => setQuery(e.target.value)} />
      {items.length === 0 && <p className="mt-2 text-sm text-muted">{t("personal.empty")}</p>}
      <ul className="mt-2 space-y-1">
        {items.map((n) => (
          <li key={n.id} className="flex items-start gap-2 rounded-md border border-border px-3 py-2 text-sm">
            <div className="min-w-0 flex-1">
              <div className="whitespace-pre-wrap break-words">{n.text}</div>
              <div className="text-xs text-muted">{exactTime(n.createdAt, lang)}</div>
            </div>
            <Button variant="ghost" aria-label={`${t("personal.delete")}: ${n.text}`} onClick={() => void api.noteRemove(n.id).then(load).catch(fail)}>
              <Trash2 size={14} />
            </Button>
          </li>
        ))}
      </ul>
    </Section>
  );
}

export function PersonalSection() {
  const t = useT();
  return (
    <>
      <Reminders />
      <Notes kind="note" title={t("personal.notes")} />
      <Notes kind="saved" title={t("personal.saved")} />
    </>
  );
}
