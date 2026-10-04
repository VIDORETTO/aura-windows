import { Circle, Download, Paperclip, Play, Square, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import { inTauri } from "../ipc/bridge";
import type { PlaybackMedia, Recording } from "../ipc/types";
import { useT } from "../i18n";
import { clockDuration, formatBytes, relativeTime } from "../lib/format";
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
      {list.length > 0 && (
        <ul>
          {list.map((r) => (
            <RecordingItem key={r.id} rec={r} lang={lang} onExport={() => void exportTo(r.id)} onDelete={() => void run(() => api.recordingDelete(r.id))} />
          ))}
        </ul>
      )}
    </Section>
  );
}

const SOURCE_LABEL: Record<string, string> = { mic: "privacy.source.mic", system: "privacy.source.systemAudio", screen: "privacy.source.screen" };

/** File URL the webview can load (asset protocol inside Tauri). */
async function mediaUrl(path: string): Promise<string> {
  if (!inTauri()) return path;
  const { convertFileSrc } = await import("@tauri-apps/api/core");
  return convertFileSrc(path);
}

/** One recording: explicit duration, player and attach (004 AC-015, 005 AC-004). */
function RecordingItem({ rec: r, lang, onExport, onDelete }: { rec: Recording; lang: "ptBr" | "en"; onExport: () => void; onDelete: () => void }) {
  const t = useT();
  const notify = useApp((s) => s.notify);
  const [media, setMedia] = useState<(PlaybackMedia & { url: string })[] | null>(null);
  const [busy, setBusy] = useState(false);
  const done = r.endedAt !== null;
  const play = async () => {
    setBusy(true);
    try {
      const items = await api.recordingPlayback(r.id);
      setMedia(await Promise.all(items.map(async (m) => ({ ...m, url: await mediaUrl(m.path) }))));
    } catch (e) {
      notify("error", errorMessage(e));
    } finally {
      setBusy(false);
    }
  };
  const attach = async () => {
    setBusy(true);
    try {
      const res = await api.recordingAttach(r.id);
      notify("info", t("recordings.attached", { n: res.chips.length }));
      if (res.failed.length) notify("warning", t("recordings.attachFailed", { files: res.failed.join("; ") }));
    } catch (e) {
      notify("error", errorMessage(e));
    } finally {
      setBusy(false);
    }
  };
  let screenIndex = 0;
  return (
    <li aria-label={r.title} className="border-b border-line py-2 last:border-0">
      <div className="flex items-center gap-3">
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-medium">{r.title}</div>
          <div className="text-xs text-muted">
            {r.source} · {relativeTime(r.startedAt, lang)} ·{" "}
            {done ? (
              <>
                <span aria-label={t("recordings.duration")} className="tabular-nums">
                  {r.durationMs !== null ? clockDuration(r.durationMs) : "—"}
                </span>{" "}
                · {formatBytes(r.bytes)}
              </>
            ) : (
              t("recordings.running")
            )}
          </div>
        </div>
        <Button size="sm" disabled={!done || busy} onClick={() => void play()}>
          <Play size={13} /> {t("recordings.play")}
        </Button>
        <Button size="sm" disabled={!done || busy} onClick={() => void attach()}>
          <Paperclip size={13} /> {t("recordings.attach")}
        </Button>
        <Button size="sm" disabled={!done} onClick={onExport}>
          <Download size={13} /> {t("recordings.export")}
        </Button>
        <Button size="sm" variant="danger" aria-label={t("common.delete")} disabled={!done} onClick={onDelete}>
          <Trash2 size={13} />
        </Button>
      </div>
      {media && (
        <div className="mt-2 flex flex-col gap-2">
          {media.length === 0 && <p className="text-xs text-muted">{t("recordings.noMedia")}</p>}
          {media.map((m) => {
            const name = m.kind === "video" ? `${t(SOURCE_LABEL.screen as never)} ${++screenIndex}` : t((SOURCE_LABEL[m.source] ?? m.source) as never);
            const label = `${name} — ${r.title}`;
            return m.kind === "audio" ? (
              <audio key={m.path} controls preload="metadata" src={m.url} aria-label={label} className="w-full" />
            ) : (
              <video key={m.path} controls preload="metadata" src={m.url} aria-label={label} className="max-h-64 w-full rounded-md bg-black" />
            );
          })}
        </div>
      )}
    </li>
  );
}
