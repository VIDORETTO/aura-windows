import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { AudioDevice, AudioSourceKind, ModelView, Provider, SpeechOptions } from "../ipc/types";
import { speakText } from "../lib/speech";
import { useT } from "../i18n";
import { modelDescription, modelName } from "../i18n/models";
import { formatBytes, percent } from "../lib/format";
import { useApp } from "../state/app";
import { Badge, Button, Progress, Row, Section, Select, Switch, TextArea, TextField } from "../ui/primitives";
import { updateSettings } from "./General";

export function VoiceSection() {
  const t = useT();
  const downloads = useApp((s) => s.downloads);
  const settings = useApp((s) => s.settings);
  const notify = useApp((s) => s.notify);
  const [models, setModels] = useState<ModelView[]>([]);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [outputs, setOutputs] = useState<AudioDevice[]>([]);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [vocab, setVocab] = useState("");
  useEffect(() => setVocab((settings?.asrVocabulary ?? []).join("\n")), [settings?.asrVocabulary]);
  const reload = async () => setModels(await api.voiceModels());
  useEffect(() => {
    void reload();
    void api.audioDevices(false).then(setDevices);
    void api.audioDevices(true).then(setOutputs).catch(() => undefined);
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
            <div key={m.entry.id} data-model={m.entry.id} className="flex items-center gap-3 py-3">
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2 text-sm font-medium">
                  {modelName(t, m.entry)}
                  {m.recommended && <Badge tone="accent">{t("voice.recommended")}</Badge>}
                  {m.selected && <Badge tone="success">{t("voice.selected")}</Badge>}
                  {m.installed && !m.selected && <Badge>{t("voice.installed")}</Badge>}
                </div>
                <div className="text-xs text-muted">
                  {modelDescription(t, m.entry)} · {formatBytes(m.sizeBytes)} · {m.entry.license}
                </div>
                <ModelFacts model={m} />
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
          <Select aria-label={t("voice.device")} value={settings?.microphoneDeviceId ?? ""} disabled={!settings} onChange={(e) => void updateSettings({ microphoneDeviceId: e.target.value || null })}>
            <option value="">{t("voice.device.default")}</option>
            {settings?.microphoneDeviceId && !devices.some((device) => device.id === settings.microphoneDeviceId) && (
              <option value={settings.microphoneDeviceId}>{t("voice.device.unavailable", { name: settings.microphoneDeviceId })}</option>
            )}
            {devices.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name}
              </option>
            ))}
          </Select>
        </Row>
        <DeviceTest source="mic" start={t("voice.test.mic")} stop={t("voice.test.micStop")} meter={t("voice.level.mic")} />
        <Row label={t("voice.systemAudio")} hint={t("voice.systemAudio.hint")}>
          <Select aria-label={t("voice.systemAudio")} value={settings?.systemAudioDeviceId ?? ""} disabled={!settings} onChange={(e) => void updateSettings({ systemAudioDeviceId: e.target.value || null })}>
            <option value="">{t("voice.device.default")}</option>
            {settings?.systemAudioDeviceId && !outputs.some((device) => device.id === settings.systemAudioDeviceId) && (
              <option value={settings.systemAudioDeviceId}>{t("voice.device.unavailable", { name: settings.systemAudioDeviceId })}</option>
            )}
            {outputs.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name}
              </option>
            ))}
          </Select>
        </Row>
        <DeviceTest source="systemAudio" start={t("voice.test.system")} stop={t("voice.test.systemStop")} meter={t("voice.level.system")} tone={t("voice.test.tone")} />
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
      <ReadAloudSection />
    </>
  );
}

/** Comparable catalog metadata (006 AC-001): bars, languages, requirements. */
function ModelFacts({ model }: { model: ModelView }) {
  const t = useT();
  const lang = useApp((s) => s.settings?.language);
  const { entry } = model;
  const ram = new Intl.NumberFormat(lang === "en" ? "en-US" : "pt-BR", { maximumFractionDigits: 1 }).format(entry.minRamMb / 1024);
  const languages = entry.languages.includes("*") ? t("voice.multilingual") : entry.languages.join(", ");
  return (
    <div className="mt-1.5 grid gap-1 text-xs text-muted">
      <div className="flex flex-wrap gap-x-4 gap-y-1">
        <Meter label={t("voice.speed")} value={entry.speed} />
        <Meter label={t("voice.accuracy")} value={entry.accuracy} />
      </div>
      <div className="break-words">{t("voice.languages", { list: languages })}</div>
      <div>{t(entry.gpuRecommended ? "voice.requirements.gpu" : "voice.requirements.cpu", { ram })}</div>
    </div>
  );
}

function Meter({ label, value }: { label: string; value: number }) {
  const pct = Math.round(Math.min(1, Math.max(0, value)) * 100);
  return (
    <span className="flex items-center gap-1.5">
      <span>{label}</span>
      <span role="meter" aria-label={label} aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100} className="h-1.5 w-16 overflow-hidden rounded-full bg-hover">
        <span className="block h-full rounded-full bg-accent" style={{ width: `${pct}%` }} />
      </span>
    </span>
  );
}

/** Live device level (005 AC-001): −60…0 dBFS mapped to 0–100. Nothing is recorded. */
function DeviceTest({ source, start, stop, meter, tone }: { source: AudioSourceKind; start: string; stop: string; meter: string; tone?: string }) {
  const notify = useApp((s) => s.notify);
  const dbfs = useApp((s) => s.audioLevels[source]);
  const [on, setOn] = useState(false);
  const running = useRef(false);
  useEffect(() => () => {
    if (running.current) void api.audioTestStop(source).catch(() => undefined);
  }, [source]);
  const toggle = async () => {
    try {
      if (running.current) {
        running.current = false;
        setOn(false);
        await api.audioTestStop(source);
      } else {
        useApp.setState((s) => ({ audioLevels: { ...s.audioLevels, [source]: undefined } }));
        await api.audioTestStart(source, null);
        running.current = true;
        setOn(true);
      }
    } catch (e) {
      notify("error", errorMessage(e));
    }
  };
  const value = dbfs === undefined ? 0 : Math.round(Math.min(1, Math.max(0, (dbfs + 60) / 60)) * 100);
  return (
    <div className="flex flex-wrap items-center gap-3 pb-2.5">
      <Button size="sm" onClick={() => void toggle()}>{on ? stop : start}</Button>
      {tone && <Button size="sm" variant="ghost" onClick={playTestTone}>{tone}</Button>}
      {on && (
        <span role="meter" aria-label={meter} aria-valuenow={value} aria-valuemin={0} aria-valuemax={100} className="h-2 min-w-32 flex-1 overflow-hidden rounded-full bg-hover">
          <span className="block h-full rounded-full bg-accent transition-[width] duration-75" style={{ width: `${value}%` }} />
        </span>
      )}
    </div>
  );
}

/** 1 kHz for 1.5 s on the default output, to see the system meter move. */
function playTestTone() {
  const Ctx = window.AudioContext;
  if (!Ctx) return;
  const ctx = new Ctx();
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.frequency.value = 1000;
  gain.gain.value = 0.2;
  osc.connect(gain).connect(ctx.destination);
  osc.start();
  osc.stop(ctx.currentTime + 1.5);
  osc.onended = () => void ctx.close();
}
/** Reading answers aloud: offline Windows voice by default, an optional
 * cloud voice (one-time consent on first use) and auto-read (009 AC-005/006). */
export function ReadAloudSection() {
  const t = useT();
  const settings = useApp((s) => s.settings);
  const [options, setOptions] = useState<SpeechOptions>({ voices: [], cloud: [] });
  const [cloudVoice, setCloudVoice] = useState(settings?.ttsCloudVoice ?? "alloy");
  useEffect(() => setCloudVoice(settings?.ttsCloudVoice ?? "alloy"), [settings?.ttsCloudVoice]);
  useEffect(() => {
    void api.speechOptions().then(setOptions).catch(() => undefined);
  }, []);
  if (!settings) return null;
  const value = settings.ttsProvider ? `cloud:${settings.ttsProvider}` : (settings.ttsVoice ?? "");
  const cloud = options.cloud.find((c) => c.id === settings.ttsProvider);
  const choose = (v: string) =>
    void updateSettings(v.startsWith("cloud:") ? { ttsProvider: v.slice(6) } : { ttsProvider: null, ttsVoice: v || null });
  return (
    <Section title={t("tts.title")} description={t("tts.hint")}>
      <Row label={t("tts.voice")}>
        <Select aria-label={t("tts.voice")} value={value} onChange={(e) => choose(e.target.value)}>
          <option value="">{t("tts.voice.auto")}</option>
          <optgroup label={t("tts.voice.offline")}>
            {options.voices.map((v) => (
              <option key={v.id} value={v.id}>
                {`${v.name} (${v.language})`}
              </option>
            ))}
          </optgroup>
          {options.cloud.length > 0 && (
            <optgroup label={t("tts.voice.cloud")}>
              {options.cloud.map((c) => (
                <option key={c.id} value={`cloud:${c.id}`}>
                  {c.name}
                </option>
              ))}
            </optgroup>
          )}
        </Select>
      </Row>
      {settings.ttsProvider && (
        <>
          <p className="py-1 text-xs text-muted">{t("tts.cloud.notice", { provider: cloud?.name ?? settings.ttsProvider })}</p>
          <Row label={t("tts.cloudVoice")}>
            <TextField
              aria-label={t("tts.cloudVoice")}
              value={cloudVoice}
              onChange={(e) => setCloudVoice(e.target.value)}
              onBlur={() => cloudVoice.trim() && cloudVoice !== settings.ttsCloudVoice && void updateSettings({ ttsCloudVoice: cloudVoice.trim() })}
            />
          </Row>
        </>
      )}
      <Row label={t("tts.autoRead")} hint={t("tts.autoRead.hint")}>
        <Switch label={t("tts.autoRead")} checked={settings.autoRead} onChange={(v) => void updateSettings({ autoRead: v })} />
      </Row>
      <div className="py-2">
        <Button size="sm" onClick={() => void speakText(t("tts.sample"))}>
          {t("tts.test")}
        </Button>
      </div>
    </Section>
  );
}
