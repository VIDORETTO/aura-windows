import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { MemoryView, ModelInfo, SettingsPatch } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { Button, Field, Row, Section, Select, Switch, TextArea, cx } from "../ui/primitives";

export async function updateSettings(patch: SettingsPatch) {
  try {
    useApp.getState().setSettings(await api.settingsUpdate(patch));
  } catch (e) {
    useApp.getState().notify("error", errorMessage(e));
  }
}

export function GeneralSection() {
  const t = useT();
  const s = useApp((x) => x.settings);
  const [instructions, setInstructions] = useState(s?.personalInstructions ?? "");
  const [models, setModels] = useState<ModelInfo[]>([]);
  useEffect(() => setInstructions(s?.personalInstructions ?? ""), [s?.personalInstructions]);
  useEffect(() => void api.modelsList().then(setModels).catch(() => undefined), []);
  if (!s) return null;
  return (
    <>
      <Section title={t("settings.general")}>
        <Row label={t("general.theme")}>
          <Select aria-label={t("general.theme")} value={s.theme} onChange={(e) => void updateSettings({ theme: e.target.value as never })}>
            <option value="system">{t("general.theme.system")}</option>
            <option value="light">{t("general.theme.light")}</option>
            <option value="dark">{t("general.theme.dark")}</option>
          </Select>
        </Row>
        <Row label={t("general.opacity")} hint={`${Math.round(s.opacity * 100)}%`}>
          <input
            type="range"
            min={50}
            max={100}
            step={1}
            aria-label={t("general.opacity")}
            value={Math.round(s.opacity * 100)}
            onChange={(e) => void updateSettings({ opacity: Number(e.target.value) / 100 })}
            className="w-44 accent-[var(--accent)]"
          />
        </Row>
        <Row label={t("general.language")}>
          <Select aria-label={t("general.language")} value={s.language} onChange={(e) => void updateSettings({ language: e.target.value as never })}>
            <option value="ptBr">Português (Brasil)</option>
            <option value="en">English</option>
          </Select>
        </Row>
        <Row label={t("general.startWithWindows")}>
          <Switch label={t("general.startWithWindows")} checked={s.startWithWindows} onChange={(v) => void updateSettings({ startWithWindows: v })} />
        </Row>
        <Row label={t("general.hideOnBlur")} hint={t("general.hideOnBlur.hint")}>
          <Switch label={t("general.hideOnBlur")} checked={s.hideOnBlur} onChange={(v) => void updateSettings({ hideOnBlur: v })} />
        </Row>
        <Row label={t("general.attachScreen")}>
          <Switch label={t("general.attachScreen")} checked={s.attachScreenOnOpen} onChange={(v) => void updateSettings({ attachScreenOnOpen: v })} />
        </Row>
        <Row label={t("general.memories")} hint={t("general.memories.hint")}>
          <Switch label={t("general.memories")} checked={s.memories} onChange={(v) => void updateSettings({ memories: v })} />
        </Row>
        <AccentRow />
        <Row label={t("general.defaultModel")}>
          <Select aria-label={t("general.defaultModel")} value={s.defaultModel ?? ""} onChange={(e) => void updateSettings({ defaultModel: e.target.value || null })}>
            <option value="">—</option>
            {models.map((m) => (
              <option key={m.id} value={m.id}>
                {m.displayName}
              </option>
            ))}
          </Select>
        </Row>
        <Row label={t("general.onboarding")} hint={t("general.onboarding.hint")}>
          <Button size="sm" onClick={() => void api.onboardingResume().catch((e) => useApp.getState().notify("error", errorMessage(e)))}>
            {t("general.onboarding.resume")}
          </Button>
        </Row>
      </Section>
      <Section title={t("general.personalInstructions")} description={t("general.personalInstructions.hint")}>
        <div className="py-2">
          <TextArea
            aria-label={t("general.personalInstructions")}
            rows={4}
            value={instructions}
            maxLength={4000}
            onChange={(e) => setInstructions(e.target.value)}
            onBlur={() => instructions !== s.personalInstructions && void updateSettings({ personalInstructions: instructions })}
          />
        </div>
      </Section>
      <MemoriesSection />
    </>
  );
}

/** What Codex remembers across conversations (008 AC-015). */
function MemoriesSection() {
  const t = useT();
  const notify = useApp((s) => s.notify);
  const [view, setView] = useState<MemoryView | null>(null);
  const [draft, setDraft] = useState<{ summary: string; registry: string } | null>(null);
  useEffect(() => {
    void api.memoriesGet().then(setView).catch(() => undefined);
  }, []);
  const run = async (fn: () => Promise<MemoryView | void>) => {
    try {
      const next = await fn();
      setView(next ?? (await api.memoriesGet()));
    } catch (e) {
      notify("error", errorMessage(e));
    }
  };
  if (!view) return null;
  return (
    <Section title={t("memories.title")} description={t("memories.hint")}>
      {view.facts.length === 0 && !draft && <p className="py-2 text-[13px] text-muted">{t("memories.empty")}</p>}
      {view.facts.length > 0 && !draft && (
        <ul aria-label={t("memories.facts")} className="flex flex-col gap-1 py-2">
          {view.facts.map((fact) => (
            <li key={fact} className="flex items-start gap-2 text-[13px]">
              <span className="min-w-0 flex-1 break-words">{fact}</span>
              <Button size="sm" variant="ghost" aria-label={t("memories.forgetFact", { fact })} onClick={() => void run(() => api.memoriesForgetFact(fact))}>
                {t("memories.forget")}
              </Button>
            </li>
          ))}
        </ul>
      )}
      {draft ? (
        <div className="grid gap-2 py-2">
          <Field label={t("memories.summary")}>
            <TextArea rows={6} value={draft.summary} onChange={(e) => setDraft({ ...draft, summary: e.target.value })} />
          </Field>
          <Field label={t("memories.registry")}>
            <TextArea rows={6} value={draft.registry} onChange={(e) => setDraft({ ...draft, registry: e.target.value })} />
          </Field>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setDraft(null)}>{t("common.cancel")}</Button>
            <Button variant="primary" onClick={() => void run(async () => { const v = await api.memoriesSave(draft.summary, draft.registry); setDraft(null); return v; })}>{t("memories.save")}</Button>
          </div>
        </div>
      ) : (
        <div className="flex gap-2 py-2">
          <Button size="sm" onClick={() => setDraft({ summary: view.summary, registry: view.registry })}>{t("memories.edit")}</Button>
          {(view.summary || view.registry) && (
            <Button size="sm" variant="danger" onClick={() => window.confirm(t("memories.confirmForgetAll")) && void run(() => api.memoriesForgetAll())}>
              {t("memories.forgetAll")}
            </Button>
          )}
        </div>
      )}
    </Section>
  );
}
/** Accent presets (012); the first is Aura's default (#5b5bf7). */
const ACCENTS: { key: string; color: string | null }[] = [
  { key: "default", color: null },
  { key: "blue", color: "#0a64d8" },
  { key: "teal", color: "#0f8a7e" },
  { key: "green", color: "#2e8b3e" },
  { key: "orange", color: "#d9661f" },
  { key: "red", color: "#d13438" },
  { key: "pink", color: "#c2378f" },
  { key: "graphite", color: "#5c6370" },
];
const DEFAULT_ACCENT = "#5b5bf7";

/** Accent color: presets, an RGB picker and a hex code (012 AC-002). */
function AccentRow() {
  const t = useT();
  const current = useApp((s) => s.settings?.accentColor ?? null);
  const [hex, setHex] = useState(current ?? DEFAULT_ACCENT);
  const [invalid, setInvalid] = useState(false);
  useEffect(() => setHex(current ?? DEFAULT_ACCENT), [current]);
  const choose = (color: string | null) => {
    setInvalid(false);
    void updateSettings({ accentColor: color });
  };
  const applyHex = () => {
    const v = hex.trim();
    if (!/^#[0-9a-fA-F]{6}$/.test(v)) return setInvalid(true);
    choose(v.toLowerCase() === DEFAULT_ACCENT ? null : v.toLowerCase());
  };
  const custom = current !== null && !ACCENTS.some((a) => a.color === current);
  return (
    <div className="py-2.5">
      <div className="mb-2 text-sm">{t("general.accent")}</div>
      <div role="radiogroup" aria-label={t("general.accent")} className="flex flex-wrap items-center gap-2">
        {ACCENTS.map((a) => {
          const selected = a.color === current;
          return (
            <button
              key={a.key}
              type="button"
              role="radio"
              aria-checked={selected}
              aria-label={t(`general.accent.${a.key}` as never)}
              title={t(`general.accent.${a.key}` as never)}
              onClick={() => choose(a.color)}
              className={cx(
                "h-7 w-7 rounded-full border-2 transition-transform hover:scale-110",
                selected ? "border-fg" : "border-transparent",
              )}
              style={{ background: a.color ?? DEFAULT_ACCENT }}
            />
          );
        })}
        <label className={cx("ml-2 flex items-center gap-2 rounded-md border px-2 py-1", custom ? "border-fg" : "border-line")}>
          <input
            type="color"
            aria-label={t("general.accent.custom")}
            value={(current ?? DEFAULT_ACCENT).toLowerCase()}
            onChange={(e) => {
              setHex(e.target.value);
              choose(e.target.value.toLowerCase() === DEFAULT_ACCENT ? null : e.target.value.toLowerCase());
            }}
            className="h-6 w-8 cursor-pointer border-0 bg-transparent p-0"
          />
          <input
            type="text"
            aria-label={t("general.accent.hex")}
            value={hex}
            maxLength={7}
            spellCheck={false}
            onChange={(e) => {
              setHex(e.target.value);
              setInvalid(false);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") applyHex();
            }}
            onBlur={applyHex}
            className="h-6 w-20 bg-transparent font-mono text-[13px] uppercase outline-none"
          />
        </label>
      </div>
      {invalid && <p className="mt-1 text-xs text-danger">{t("general.accent.invalid")}</p>}
    </div>
  );
}
