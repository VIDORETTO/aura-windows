import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { ModelView } from "../ipc/types";
import { useT } from "../i18n";
import { modelName } from "../i18n/models";
import { formatBytes, percent } from "../lib/format";
import { useApp } from "../state/app";
import { Button, Progress } from "../ui/primitives";

export function VoiceInstallCard({ onReady }: { onReady: () => void }) {
  const t = useT();
  const language = useApp((s) => s.settings?.language);
  const downloads = useApp((s) => s.downloads);
  const [model, setModel] = useState<ModelView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [requested, setRequested] = useState(false);
  const [selecting, setSelecting] = useState(false);
  const completed = useRef<string | null>(null);
  const activeModelId = useRef<string | null>(null);
  const [requestSnapshot, setRequestSnapshot] = useState<(typeof downloads)[string] | undefined>(undefined);
  const ready = useRef(onReady);
  ready.current = onReady;
  useEffect(() => {
    let cancelled = false;
    void api.voiceModels().then((models) => {
      if (cancelled) return;
      const recommended = models.find((m) => m.entry.id === activeModelId.current) ?? models.find((m) => m.recommended);
      setModel(recommended ?? null);
      if (!recommended) setError(t("voice.install.noModel"));
    }).catch((e) => { if (!cancelled) setError(errorMessage(e)); });
    return () => { cancelled = true; };
  }, [language, t]);
  const download = model ? downloads[model.entry.id] : undefined;
  const installed = model?.installed || download?.done;
  const awaitingProgress = requested && download === requestSnapshot;
  const cancelled = !awaitingProgress && download?.error === "cancelled";
  const downloading = !!model && !installed && (awaitingProgress || (!download?.error && (model.downloading || requested)));
  const select = async (id: string) => {
    setSelecting(true);
    try { await api.voiceSelect(id); ready.current(); }
    catch (e) { setError(errorMessage(e)); }
    finally { setSelecting(false); }
  };
  useEffect(() => {
    if (!download?.done || !model || completed.current === model.entry.id) return;
    completed.current = model.entry.id;
    void select(model.entry.id);
    // Selection is a one-time response to successful installation, not render identity.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [download?.done, model?.entry.id]);
  const install = async () => {
    if (!model || downloading) return;
    setError(null);
    activeModelId.current = model.entry.id;
    setRequestSnapshot(download);
    setRequested(true);
    try { await api.voiceInstall(model.entry.id); }
    catch (e) { setRequested(false); setError(errorMessage(e)); }
  };
  const sizeMb = model ? new Intl.NumberFormat(language === "en" ? "en-US" : "pt-BR").format(Math.ceil(model.sizeBytes / 1048576)) : "";
  return (
    <section aria-label={t("voice.install.title")} className="rounded-md border border-line bg-surface p-3 text-sm">
      <p className="font-medium">{t("voice.install.title")}</p>
      <p className="mt-1 text-xs text-muted">{t("voice.install.hint")}</p>
      {model && <p className="mt-2 break-words">{modelName(t, model.entry)} · {formatBytes(model.sizeBytes)}</p>}
      {(error || (!awaitingProgress && !cancelled && download?.error)) && <p role="alert" className="mt-2 break-words text-xs text-danger">{t("voice.install.failed", { reason: error ?? download?.error ?? "" })}</p>}
      {cancelled && <p role="status" className="mt-2 text-xs text-muted">{t("voice.install.cancelled")}</p>}
      {downloading && <div className="mt-2"><Progress value={percent(download?.bytes ?? 0, download?.total ?? model!.sizeBytes)} /></div>}
      {model && <div className="mt-2 flex flex-wrap gap-2">
        {installed ? <Button size="sm" disabled={selecting} onClick={() => void select(model.entry.id)}>{t("voice.use")}</Button>
          : <Button size="sm" variant="primary" disabled={downloading} onClick={() => void install()}>{downloading ? t("common.loading") : t("voice.install.download", { size: sizeMb })}</Button>}
        {downloading && <Button size="sm" onClick={() => void api.voiceCancelInstall(model.entry.id).catch((e) => setError(errorMessage(e)))}>{t("common.cancel")}</Button>}
      </div>}
    </section>
  );
}
