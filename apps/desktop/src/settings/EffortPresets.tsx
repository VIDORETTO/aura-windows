// Reasoning effort per model and mode (013): the defaults the Overlay picks
// when a model and mode are chosen; the picker saves its choice here too.

import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { ModelInfo, ModeEfforts, Provider } from "../ipc/types";
import { useT } from "../i18n";
import { CHATGPT_PLAN, effortKey, modelCapabilities, type ModeKey } from "../overlay/session";
import { useApp } from "../state/app";
import { Section, Select } from "../ui/primitives";

const MODES: ModeKey[] = ["chat", "task", "plan"];
const LABELED = ["none", "minimal", "low", "medium", "high", "xhigh", "max"];

export function effortLabel(t: ReturnType<typeof useT>, effort: string): string {
  return LABELED.includes(effort) ? t(`picker.effort.${effort}` as never) : effort;
}

interface Row {
  key: string;
  name: string;
  efforts: string[];
  defaultEffort: string | null;
}

export function EffortPresetsSection() {
  const t = useT();
  const settings = useApp((s) => s.settings);
  const setSettings = useApp((s) => s.setSettings);
  const notify = useApp((s) => s.notify);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [providers, setProviders] = useState<Provider[]>([]);
  useEffect(() => {
    void api.modelsList().then(setModels).catch(() => undefined);
    void api.providersList().then(setProviders).catch(() => undefined);
  }, []);
  if (!settings) return null;

  const rows: Row[] = [
    ...models
      .filter((m) => m.efforts.length > 0)
      .map((m) => ({ key: effortKey(CHATGPT_PLAN, m.id), name: m.displayName, efforts: m.efforts, defaultEffort: m.defaultEffort })),
    ...providers.flatMap((p) =>
      p.models.flatMap((m) => {
        const caps = modelCapabilities({ provider: `aura-${p.id}`, models: [], providers }, m.id);
        return caps && caps.efforts.length > 0
          ? [{ key: effortKey(`aura-${p.id}`, m.id), name: `${p.name} · ${m.displayName ?? m.id}`, efforts: caps.efforts, defaultEffort: caps.defaultEffort }]
          : [];
      }),
    ),
  ];

  const change = async (key: string, mode: ModeKey, effort: string) => {
    const presets = { ...(settings.effortPresets ?? {}) };
    const entry: ModeEfforts = { ...(presets[key] ?? {}), [mode]: effort || null };
    if (!effort) delete entry[mode];
    if (Object.values(entry).some(Boolean)) presets[key] = entry;
    else delete presets[key];
    try {
      setSettings(await api.settingsUpdate({ effortPresets: presets }));
    } catch (e) {
      notify("error", errorMessage(e));
    }
  };

  return (
    <Section title={t("efforts.title")} description={t("efforts.hint")}>
      {rows.length === 0 ? (
        <p className="py-3 text-[13px] text-muted">{t("efforts.empty")}</p>
      ) : (
        // Fixed layout: the three selects share the card width instead of overflowing it.
        <table aria-label={t("efforts.title")} className="w-full table-fixed border-collapse text-[13px]">
          <thead>
            <tr className="text-left text-xs text-muted">
              <th className="w-[24%] py-1.5 font-normal">{t("efforts.model")}</th>
              {MODES.map((m) => (
                <th key={m} className="py-1.5 font-normal">
                  {t(`mode.${m}`)}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.key} className="border-t border-line">
                <th scope="row" className="truncate py-1.5 pr-2 text-left font-medium" title={r.name}>
                  {r.name}
                </th>
                {MODES.map((mode) => (
                  <td key={mode} className="py-1.5 pr-2">
                    <Select
                      aria-label={`${r.name} — ${t(`mode.${mode}`)}`}
                      className="w-full min-w-0"
                      value={settings.effortPresets?.[r.key]?.[mode] ?? ""}
                      onChange={(e) => void change(r.key, mode, e.target.value)}
                    >
                      <option value="">
                        {r.defaultEffort
                          ? t("picker.effort.default", { effort: effortLabel(t, r.defaultEffort).toLowerCase() })
                          : t("picker.effort.modelDefault")}
                      </option>
                      {r.efforts.map((e) => (
                        <option key={e} value={e}>
                          {effortLabel(t, e)}
                        </option>
                      ))}
                    </Select>
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </Section>
  );
}
