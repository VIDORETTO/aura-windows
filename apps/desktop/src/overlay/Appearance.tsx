// Appearance popover in the Overlay header (001 AC-010, revision 2): opacity
// applies live while dragging and is saved once it settles.

import { Contrast } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { updateSettings } from "../settings/General";
import { IconButton, Switch, cx } from "../ui/primitives";
import { PopoverPanel, usePopover } from "../ui/Popover";

const THEMES = ["system", "light", "dark"] as const;

export function AppearanceButton({ grow }: { grow: boolean }) {
  const t = useT();
  const pop = usePopover();
  const settings = useApp((s) => s.settings);
  const [opacity, setOpacity] = useState(settings?.opacity ?? 0.92);
  const save = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(() => setOpacity(settings?.opacity ?? 0.92), [settings?.opacity]);
  if (!settings) return null;

  const onOpacity = (pct: number) => {
    const v = pct / 100;
    setOpacity(v);
    document.documentElement.style.setProperty("--overlay-opacity", String(v));
    if (save.current) clearTimeout(save.current);
    save.current = setTimeout(() => void updateSettings({ opacity: v }), 300);
  };

  return (
    <>
      <IconButton ref={pop.anchor} label={t("appearance.title")} active={pop.open} onClick={pop.toggle} aria-expanded={pop.open}>
        <Contrast size={15} />
      </IconButton>
      <PopoverPanel pop={pop} label={t("appearance.title")} align="end" grow={grow} className="w-64">
        <div className="flex flex-col gap-3 p-1.5">
          <label className="flex flex-col gap-1.5">
            <span className="flex items-center justify-between text-[12px] font-medium">
              {t("general.opacity")}
              <span className="tabular-nums text-muted">{Math.round(opacity * 100)}%</span>
            </span>
            <input
              type="range"
              min={50}
              max={100}
              step={1}
              value={Math.round(opacity * 100)}
              aria-label={t("general.opacity")}
              onChange={(e) => onOpacity(Number(e.target.value))}
              className="w-full accent-[var(--accent)]"
            />
          </label>
          <div className="flex flex-col gap-1.5">
            <span className="text-[12px] font-medium">{t("general.theme")}</span>
            <div role="radiogroup" aria-label={t("general.theme")} className="grid grid-cols-3 gap-1 rounded-md bg-hover p-0.5">
              {THEMES.map((th) => (
                <button
                  key={th}
                  type="button"
                  role="radio"
                  aria-checked={settings.theme === th}
                  onClick={() => void updateSettings({ theme: th })}
                  className={cx("rounded px-2 py-1 text-[12px]", settings.theme === th ? "bg-surface-strong font-medium shadow-sm" : "text-muted hover:text-fg")}
                >
                  {t(`general.theme.${th}`)}
                </button>
              ))}
            </div>
          </div>
          <label className="flex items-center justify-between gap-3 text-[12px]">
            <span>
              <span className="block font-medium">{t("general.hideOnBlur")}</span>
              <span className="block text-[11px] leading-snug text-muted">{t("appearance.hideOnBlur.hint")}</span>
            </span>
            <Switch label={t("general.hideOnBlur")} checked={settings.hideOnBlur} onChange={(v) => void updateSettings({ hideOnBlur: v })} />
          </label>
        </div>
      </PopoverPanel>
    </>
  );
}
