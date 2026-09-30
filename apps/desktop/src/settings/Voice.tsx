import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { AudioDevice, ModelView, Provider } from "../ipc/types";
import { useT } from "../i18n";
import { formatBytes, percent } from "../lib/format";
import { useApp } from "../state/app";
import { Badge, Button, Progress, Row, Section, Select, Switch, TextArea } from "../ui/primitives";
import { updateSettings } from "./General";

export function VoiceSection() {
  const t = useT();
  const downloads = useApp((s) => s.downloads);
  const settings = useApp((s) => s.settings);
  const notify = useApp((s) => s.notify);
  const [models, setModels] = useState<ModelView[]>([]);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [vocab, setVocab] = useState("");
  useEffect(() => setVocab((settings?.asrVocabulary ?? []).join("\n")), [settings?.asrVocabulary]);
  const reload = async () => setModels(await api.voiceModels());
  useEffect(() => {
    void reload();
    void api.audioDevices(false).then(setDevices);
    void api.providersList().then(setProviders).catch(() => undefined);
  }, []);
  // Refresh when a download finishes.
  useEffect(() => {
    if (Object.values(downloads).some((d) => d.done)) void reload();
  }, [downloads]);

  const act = async (fn: () => Promise<unknown>) => {
    try {
      await fn();
      await reload();
    } catch (e) {
      notify("error", errorMessage(e));
    }
  };

  return (
    <>
      <Section title={t("voice.models")}>
        {models.map((m) => {
          const d = downloads[m.entry.id];
          const downloading = m.downloading || (d && !d.done && !d.error);
          return (
            <div key={m.entry.id} className="flex items-center gap-3 py-3">
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2 text-sm font-medium">
                  {m.entry.name}
                  {m.recommended && <Badge tone="accent">{t("voice.recommended")}</Badge>}
                  {m.selected && <Badge tone="success">{t("voice.selected")}</Badge>}
                  {m.installed && !m.selected && <Badge>{t("voice.installed")}</Badge>}
                </div>
                <div className="text-xs text-muted">
                  {m.entry.description} · {formatBytes(m.sizeBytes)} · {m.entry.license}
                </div>
                {downloading && d && <div className="mt-1.5"><Progress value={percent(d.bytes, d.total)} /></div>}
              </div>
              {downloading ? (
                <Button size="sm" variant="ghost" onClick={() => void act(() => api.voiceCancelInstall(m.entry.id))}>
                  {t("common.cancel")}
                </Button>
              ) : m.installed ? (
                <>
                  {!m.selected && (
                    <Button size="sm" onClick={() => void act(() => api.voiceSelect(m.entry.id))}>
                      {t("voice.use")}
                    </Button>
                  )}
                  <Button size="sm" variant="danger" onClick={() => void act(() => api.voiceRemove(m.entry.id))}>
                    {t("voice.remove")}
                  </Button>
                </>
              ) : (
                <Button size="sm" variant={m.recommended ? "primary" : "secondary"} onClick={() => void act(() => api.voiceInstall(m.entry.id))}>
                  {t("voice.download")}
                </Button>
              )}
            </div>
          );
        })}
      </Section>
      <Section title={t("settings.voice")}>
        <Row label={t("voice.device")}>
          <Select aria-label={t("voice.device")}>
            {devices.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name}
              </option>
            ))}
          </Select>
        </Row>
        {settings && (
          <Row label={t("voice.language")}>
            <Select aria-label={t("voice.language")} value={settings.asrLanguage ?? ""} onChange={(e) => void updateSettings({ asrLanguage: e.target.value || null })}>
              <option value="">{t("voice.language.auto")}</option>
              <option value="pt">Português</option>
              <option value="en">English</option>
              <option value="es">Español</option>
            </Select>
          </Row>
        )}
        {settings && (
          <Row label={t("voice.cloud")} hint={t("voice.cloud.hint")}>
            <Select aria-label={t("voice.cloud")} value={settings.cloudAsrProvider ?? ""} onChange={(e) => void updateSettings({ cloudAsrProvider: e.target.value || null })}>
              <option value="">{t("common.off")}</option>
              {providers.map((p) => (
                <option key={p.id} value={p.id}>{p.name}</option>
              ))}
            </Select>
          </Row>
        )}
        {settings && (
          <div className="py-2.5">
            <div className="mb-1 text-sm">{t("voice.vocabulary")}</div>
            <TextArea
              rows={3}
              aria-label={t("voice.vocabulary")}
              placeholder={t("voice.vocabulary.hint")}
              value={vocab}
              onChange={(e) => setVocab(e.target.value)}
              onBlur={() => void updateSettings({ asrVocabulary: vocab.split(/[\n,]/).map((w) => w.trim()).filter(Boolean) })}
            />
          </div>
        )}
        {settings && (
          <Row label={t("voice.sendAfter")}>
            <Switch label={t("voice.sendAfter")} checked={settings.sendAfterDictation} onChange={(v) => void updateSettings({ sendAfterDictation: v })} />
          </Row>
        )}
      </Section>
    </>
  );
}
