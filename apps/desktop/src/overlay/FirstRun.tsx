// First-run guide after sign-in (010 TK-003): privacy defaults, shortcut,
// voice model. Every step can be skipped; it never shows again once done.

import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { AgentPermission, ModelView } from "../ipc/types";
import { useT } from "../i18n";
import { formatBytes } from "../lib/format";
import { useApp } from "../state/app";
import { Button, Kbd, Switch, cx } from "../ui/primitives";

const STEPS = ["privacy", "shortcut", "voice"] as const;

export function FirstRun() {
  const t = useT();
  const settings = useApp((s) => s.settings);
  const privacy = useApp((s) => s.privacy);
  const setSettings = useApp((s) => s.setSettings);
  const setPrivacy = useApp((s) => s.setPrivacy);
  const [step, setStep] = useState(0);
  const [recommended, setRecommended] = useState<ModelView | null>(null);
  useEffect(() => {
    void api.voiceModels().then((m) => setRecommended(m.find((x) => x.recommended) ?? null)).catch(() => undefined);
  }, []);
  if (!settings || !privacy) return null;

  const finish = async () => {
    try {
      setSettings(await api.settingsUpdate({ onboarded: true }));
    } catch (e) {
      useApp.getState().notify("error", errorMessage(e));
    }
  };
  const next = () => (step + 1 < STEPS.length ? setStep(step + 1) : void finish());
  const agent = privacy.screen.agent;
  const setAgent = async (a: AgentPermission) => setPrivacy(await api.privacySetSource("screen", privacy.screen.mode, a));

  return (
    <section aria-label={t("onboarding.title")} className="fade-in mx-4 my-3 flex flex-col gap-3 rounded-md border border-line bg-surface-strong px-4 py-3">
      <div className="flex items-center gap-2">
        <h2 className="text-sm font-semibold">{t(`onboarding.${STEPS[step]}.title`)}</h2>
        <span className="ml-auto text-[11px] text-muted">{t("onboarding.step", { n: step + 1, total: STEPS.length })}</span>
      </div>
      {STEPS[step] === "privacy" && (
        <>
          <p className="text-[13px] text-muted">{t("onboarding.privacy.body")}</p>
          <div className="flex flex-col gap-1.5" role="radiogroup" aria-label={t("privacy.agent")}>
            {(["ask", "always", "never"] as const).map((a) => (
              <label key={a} className={cx("flex cursor-pointer items-start gap-2 rounded border px-2 py-1.5 text-[13px]", agent === a ? "border-accent" : "border-line")}>
                <input type="radio" name="agent" checked={agent === a} onChange={() => void setAgent(a)} />
                <span>
                  <span className="font-medium">{t(`privacy.agent.${a}`)}</span> — {t(`onboarding.privacy.${a}`)}
                </span>
              </label>
            ))}
          </div>
        </>
      )}
      {STEPS[step] === "shortcut" && (
        <>
          <p className="text-[13px] text-muted">{t("onboarding.shortcut.body")}</p>
          <div className="flex items-center gap-2 text-sm">
            <Kbd>{settings.invokeShortcut}</Kbd> <span className="text-muted">·</span> <Kbd>Esc</Kbd>
          </div>
          <label className="flex items-center gap-2 text-[13px]">
            <Switch label={t("shortcuts.doubleTap")} checked={settings.doubleTapCtrl} onChange={async (v) => setSettings(await api.settingsUpdate({ doubleTapCtrl: v }))} />
            {t("shortcuts.doubleTap")}
          </label>
        </>
      )}
      {STEPS[step] === "voice" && (
        <>
          <p className="text-[13px] text-muted">{t("onboarding.voice.body")}</p>
          {recommended && (
            <div className="flex items-center gap-2 text-[13px]">
              <span className="font-medium">{recommended.entry.name}</span>
              <span className="text-muted">{formatBytes(recommended.sizeBytes)}</span>
              <Button size="sm" className="ml-auto" disabled={recommended.installed || recommended.downloading} onClick={() => void api.voiceInstall(recommended.entry.id).then(() => setRecommended({ ...recommended, downloading: true }))}>
                {recommended.installed ? t("voice.installed") : recommended.downloading ? t("common.loading") : t("voice.download")}
              </Button>
            </div>
          )}
        </>
      )}
      <div className="flex justify-end gap-2">
        <Button size="sm" variant="ghost" onClick={() => void finish()}>{t("onboarding.skip")}</Button>
        <Button size="sm" variant="primary" onClick={next}>{step + 1 < STEPS.length ? t("onboarding.next") : t("onboarding.done")}</Button>
      </div>
    </section>
  );
}
