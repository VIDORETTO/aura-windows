import { Circle, Download, Square, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import { inTauri } from "../ipc/bridge";
import type { Recording } from "../ipc/types";
import { useT } from "../i18n";
import { formatBytes, relativeTime } from "../lib/format";
import { useApp } from "../state/app";
import { Button, Section } from "../ui/primitives";

export function RecordingsSection() {
  const t = useT();
  const lang = useApp((s) => s.settings?.language ?? "ptBr");
  const notify = useApp((s) => s.notify);
  const [list, setList] = useState<Recording[]>([]);
  const [active, setActive] = useState<string | null>(null);
  const reload = async () => {
    setList(await api.recordingsList());
    setActive(await api.recordingActive());
  };
  useEffect(() => void reload(), []);

  const run = async (fn: () => Promise<unknown>) => {
    try {
      await fn();
      await reload();
    } catch (e) {
      notify("error", errorMessage(e));
    }
  };

  const exportTo = async (id: string) => {
    let dir = "Aura";
    if (inTauri()) {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({ directory: true, multiple: false });
      if (!picked) return;
      dir = String(picked);
    }
    await run(async () => notify("info", t("recordings.exported", { n: (await api.recordingExport(id, dir)).length })));
  };

  return (
    <Section
      title={t("recordings.title")}
      description={t("recordings.hint")}
      actions={
        active ? (
          <Button size="sm" variant="danger" onClick={() => void run(api.recordingStop)}>
            <Square size={12} fill="currentColor" /> {t("recordings.stop")}
          </Button>
        ) : (
          <Button size="sm" variant="primary" onClick={() => void run(() => api.recordingStart(`${t("recordings.untitled")} ${new Date().toLocaleString()}`))}>
            <Circle size={12} fill="currentColor" /> {t("recordings.start")}
          </Button>
        )
      }
    >
      {list.length === 0 && <p className="py-3 text-[13px] text-muted">{t("recordings.empty")}</p>}
      {list.map((r) => (
        <div key={r.id} className="flex items-center gap-3 py-2">
          <div className="min-w-0 flex-1">
            <div className="truncate text-sm font-medium">{r.title}</div>
            <div className="text-xs text-muted">
              {r.source} · {relativeTime(r.startedAt, lang)} · {r.endedAt ? formatBytes(r.bytes) : t("recordings.running")}
            </div>
          </div>
          <Button size="sm" disabled={!r.endedAt} onClick={() => void exportTo(r.id)}>
            <Download size={13} /> {t("recordings.export")}
          </Button>
          <Button size="sm" variant="danger" aria-label={t("common.delete")} disabled={!r.endedAt} onClick={() => void run(() => api.recordingDelete(r.id))}>
            <Trash2 size={13} />
          </Button>
        </div>
      ))}
    </Section>
  );
}
