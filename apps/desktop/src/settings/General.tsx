import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { ModelInfo, SettingsPatch } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { Row, Section, Select, Switch, TextArea } from "../ui/primitives";

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
    </>
  );
}
