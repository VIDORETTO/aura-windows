import { AppWindow, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { AppProfile } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { Badge, Button, Field, Section, Select, Switch, TextArea, TextField } from "../ui/primitives";

const EMPTY: AppProfile = { id: "", name: "", processPattern: "", titleGlob: null, instructions: "", attachScreen: false, defaultMode: null, defaultModel: null };

export function ProfilesSection() {
  const t = useT();
  const notify = useApp((s) => s.notify);
  const [list, setList] = useState<AppProfile[]>([]);
  const [draft, setDraft] = useState<AppProfile | null>(null);
  const reload = async () => setList(await api.profilesList());
  useEffect(() => void reload(), []);

  const fromCurrentApp = async () => {
    const app = await api.previousApp();
    setDraft({ ...EMPTY, name: app?.processName.replace(/\.exe$/i, "") ?? "", processPattern: app?.processName ?? "" });
  };

  const save = async () => {
    if (!draft) return;
    try {
      await api.profilesSave(draft);
      setDraft(null);
      await reload();
    } catch (e) {
      notify("error", errorMessage(e));
    }
  };

  return (
    <Section
      title={t("profiles.title")}
      description={t("profiles.hint")}
      actions={
        !draft && (
          <div className="flex gap-1.5">
            <Button size="sm" onClick={() => void fromCurrentApp()}>{t("profiles.fromApp")}</Button>
            <Button size="sm" variant="primary" onClick={() => setDraft({ ...EMPTY })}>{t("common.add")}</Button>
          </div>
        )
      }
    >
      {draft && (
        <div className="grid grid-cols-2 gap-3 py-3">
          <Field label={t("profiles.name")}>
            <TextField value={draft.name} onChange={(e) => setDraft({ ...draft, name: e.target.value })} />
          </Field>
          <Field label={t("profiles.process")} hint={t("profiles.process.hint")}>
            <TextField value={draft.processPattern} placeholder="code.exe" onChange={(e) => setDraft({ ...draft, processPattern: e.target.value })} />
          </Field>
          <Field label={t("profiles.title_")}>
            <TextField value={draft.titleGlob ?? ""} placeholder="*projeto*" onChange={(e) => setDraft({ ...draft, titleGlob: e.target.value || null })} />
          </Field>
          <Field label={t("profiles.mode")}>
            <Select value={draft.defaultMode ?? ""} onChange={(e) => setDraft({ ...draft, defaultMode: (e.target.value || null) as AppProfile["defaultMode"] })}>
              <option value="">—</option>
              <option value="chat">{t("mode.chat")}</option>
              <option value="task">{t("mode.task")}</option>
              <option value="plan">{t("mode.plan")}</option>
            </Select>
          </Field>
          <div className="col-span-2">
            <Field label={t("profiles.instructions")}>
              <TextArea rows={4} maxLength={4000} value={draft.instructions} onChange={(e) => setDraft({ ...draft, instructions: e.target.value })} />
            </Field>
          </div>
          <label className="col-span-2 flex items-center gap-2 text-sm">
            <Switch label={t("profiles.attachScreen")} checked={draft.attachScreen} onChange={(v) => setDraft({ ...draft, attachScreen: v })} /> {t("profiles.attachScreen")}
          </label>
          <div className="col-span-2 flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setDraft(null)}>{t("common.cancel")}</Button>
            <Button variant="primary" disabled={!draft.name.trim() || !draft.processPattern.trim()} onClick={() => void save()}>{t("common.save")}</Button>
          </div>
        </div>
      )}
      {list.length === 0 && !draft && <p className="py-3 text-[13px] text-muted">{t("profiles.empty")}</p>}
      {list.map((p) => (
        <div key={p.id} className="flex items-center gap-3 py-2.5">
          <AppWindow size={15} className="text-muted" />
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2 text-sm font-medium">
              {p.name} <Badge>{p.processPattern}</Badge>
              {p.attachScreen && <Badge tone="accent">{t("context.screen")}</Badge>}
            </div>
            <div className="truncate text-xs text-muted">{p.instructions || "—"}</div>
          </div>
          <Button size="sm" onClick={() => setDraft(p)}>{t("common.edit")}</Button>
          <Button size="sm" variant="danger" aria-label={t("common.delete")} onClick={async () => { await api.profilesDelete(p.id); await reload(); }}>
            <Trash2 size={13} />
          </Button>
        </div>
      ))}
    </Section>
  );
}
