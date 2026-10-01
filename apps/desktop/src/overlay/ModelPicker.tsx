// One pill for provider, model and mode (replaces three native selects that
// did not fit the header).

import { ChevronDown } from "lucide-react";
import { useT } from "../i18n";
import { cx } from "../ui/primitives";
import { MenuLabel, MenuOption, PopoverPanel, usePopover } from "../ui/Popover";
import { CHATGPT_PLAN, useSession, type ModeKey } from "./session";

const MODES: ModeKey[] = ["chat", "task", "plan"];

export function ModelPicker() {
  const t = useT();
  const s = useSession();
  const pop = usePopover();
  const provider = s.providers.find((p) => `aura-${p.id}` === s.provider);
  const models =
    s.provider === CHATGPT_PLAN
      ? s.models.map((m) => ({ id: m.id, name: m.displayName }))
      : (provider?.models ?? []).map((m) => ({ id: m.id, name: m.displayName ?? m.id }));
  const providerName = s.provider === CHATGPT_PLAN ? "ChatGPT" : (provider?.name ?? s.provider);
  const modelName = models.find((m) => m.id === s.model)?.name ?? t("general.defaultModel");
  const locked = s.threadId !== null;

  return (
    <>
      <button
        ref={pop.anchor}
        type="button"
        onClick={pop.toggle}
        aria-expanded={pop.open}
        aria-label={t("picker.label", { model: modelName, mode: t(`mode.${s.mode}`) })}
        className={cx(
          "flex h-7 min-w-0 max-w-[260px] items-center gap-1.5 rounded-md border border-line px-2 text-[12px] hover:bg-hover",
          pop.open && "bg-hover",
        )}
      >
        <span className="truncate font-medium">{modelName}</span>
        <span className="shrink-0 text-muted">·</span>
        <span className="shrink-0 text-muted">{t(`mode.${s.mode}`)}</span>
        <ChevronDown size={13} className="shrink-0 text-muted" />
      </button>
      <PopoverPanel pop={pop} label={t("picker.title")} className="w-72 max-w-[calc(100vw-16px)]">
        <MenuLabel>{t("picker.provider")}</MenuLabel>
        {locked && <p className="px-2 pb-1 text-[11px] text-muted">{t("picker.providerLocked")}</p>}
        <div role="menu" aria-label={t("picker.provider")}>
          {[{ id: CHATGPT_PLAN, name: "ChatGPT" }, ...s.providers.map((p) => ({ id: `aura-${p.id}`, name: p.name }))].map((p) =>
            locked && p.id !== s.provider ? null : (
              <MenuOption key={p.id} selected={p.id === s.provider} onSelect={() => !locked && s.setProvider(p.id, null)}>
                {p.name}
              </MenuOption>
            ),
          )}
        </div>
        <MenuLabel>{t("picker.model")}</MenuLabel>
        <div role="menu" aria-label={t("picker.model")} className="max-h-48 overflow-y-auto">
          <MenuOption selected={!s.model} onSelect={() => s.setModel(null)}>
            {t("general.defaultModel")}
          </MenuOption>
          {models.map((m) => (
            <MenuOption key={m.id} selected={m.id === s.model} onSelect={() => s.setModel(m.id)}>
              {m.name}
            </MenuOption>
          ))}
        </div>
        <MenuLabel>{t("picker.mode")}</MenuLabel>
        <div role="menu" aria-label={t("picker.mode")}>
          {MODES.map((m) => (
            <MenuOption key={m} selected={m === s.mode} hint={t(`mode.${m}.desc`)} onSelect={() => void s.setMode(m)}>
              {t(`mode.${m}`)}
            </MenuOption>
          ))}
        </div>
        <span className="sr-only">{providerName}</span>
      </PopoverPanel>
    </>
  );
}
