// One pill for provider, model and mode (replaces three native selects that
// did not fit the header).

import { ChevronDown } from "lucide-react";
import { useT } from "../i18n";
import { cx } from "../ui/primitives";
import { useApp } from "../state/app";
import { MenuLabel, MenuOption, PopoverPanel, usePopover } from "../ui/Popover";
import { CHATGPT_PLAN, modelCapabilities, useSession, type ModeKey, type ModelCapabilities } from "./session";

const MODES: ModeKey[] = ["chat", "task", "plan"];
const EFFORT_KEYS = ["none", "minimal", "low", "medium", "high", "xhigh", "max"] as const;

function effortLabel(t: ReturnType<typeof useT>, effort: string): string {
  return (EFFORT_KEYS as readonly string[]).includes(effort) ? t(`picker.effort.${effort as (typeof EFFORT_KEYS)[number]}`) : effort;
}

function capabilityHint(t: ReturnType<typeof useT>, c: ModelCapabilities | null): string | undefined {
  if (!c) return undefined;
  const tags = [c.images && t("picker.cap.images"), c.tools && t("picker.cap.tools"), c.reasoning && t("picker.cap.reasoning")].filter(Boolean);
  return tags.length ? tags.join(" · ") : t("picker.cap.textOnly");
}

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
  const settingsDefault = useApp((a) => a.settings?.defaultModel ?? null);
  // "Default" names the model it stands for: the Settings default (ChatGPT plan) or the plan's own default.
  const effective =
    s.model ?? (s.provider === CHATGPT_PLAN ? (settingsDefault ?? s.models.find((m) => m.isDefault)?.id ?? null) : null);
  const modelName = models.find((m) => m.id === effective)?.name ?? t("general.defaultModel");
  const locked = s.threadId !== null;
  const caps = modelCapabilities(s, s.model);

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
          {[{ id: CHATGPT_PLAN, name: "ChatGPT", error: false }, ...s.providers.map((p) => ({ id: `aura-${p.id}`, name: p.name, error: p.status === "error" }))].map((p) =>
            locked && p.id !== s.provider ? null : (
              <MenuOption key={p.id} selected={p.id === s.provider} onSelect={() => !locked && s.setProvider(p.id, null)}>
                {p.name}
                {p.error && <span className="ml-2 text-danger">{t("providers.status.error")}</span>}
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
            <MenuOption key={m.id} selected={m.id === s.model} hint={capabilityHint(t, modelCapabilities(s, m.id))} onSelect={() => s.setModel(m.id)}>
              {m.name}
            </MenuOption>
          ))}
        </div>
        <MenuLabel>{t("picker.effort")}</MenuLabel>
        {caps && caps.efforts.length > 0 ? (
          <div role="menu" aria-label={t("picker.effort")}>
            <MenuOption selected={!s.effort} onSelect={() => s.setEffort(null)}>
              {caps.defaultEffort ? t("picker.effort.default", { effort: effortLabel(t, caps.defaultEffort).toLowerCase() }) : t("picker.effort.modelDefault")}
            </MenuOption>
            {caps.efforts.map((e) => (
              <MenuOption key={e} selected={s.effort === e} onSelect={() => s.setEffort(e)}>
                {effortLabel(t, e)}
              </MenuOption>
            ))}
          </div>
        ) : (
          <p className="px-2 pb-1 text-[11px] text-muted">{t(caps ? "picker.effort.unsupported" : "picker.effort.unknown")}</p>
        )}
        <MenuLabel>{t("picker.mode")}</MenuLabel>
        <div role="menu" aria-label={t("picker.mode")}>
          {MODES.map((m) => (
            <MenuOption key={m} selected={m === s.mode} hint={t(`mode.${m}.desc`)} onSelect={() => m !== s.mode && void s.setMode(m)}>
              {t(`mode.${m}`)}
            </MenuOption>
          ))}
        </div>
        <span className="sr-only">{providerName}</span>
      </PopoverPanel>
    </>
  );
}

/** Mode always visible in the compact Overlay (015 AC-007): Task stands out. */
export function ModeBadge() {
  const t = useT();
  const mode = useSession((s) => s.mode);
  const setMode = useSession((s) => s.setMode);
  const pop = usePopover();
  const name = t(`mode.${mode}`);
  return (
    <>
      <button
        ref={pop.anchor}
        type="button"
        onClick={pop.toggle}
        aria-expanded={pop.open}
        aria-label={t("mode.badge", { mode: name })}
        title={t(`mode.${mode}.desc`)}
        className={cx(
          "flex h-5 shrink-0 items-center gap-0.5 rounded-full border px-1.5 text-[11px] hover:bg-hover",
          mode === "task" ? "border-warning/60 text-warning" : mode === "plan" ? "border-accent/50 text-accent" : "border-line text-muted",
        )}
      >
        {name}
        <ChevronDown size={11} aria-hidden />
      </button>
      <PopoverPanel pop={pop} label={t("picker.mode")} grow className="w-64 max-w-[calc(100vw-16px)]">
        <div role="menu" aria-label={t("picker.mode")}>
          {MODES.map((m) => (
            <MenuOption
              key={m}
              selected={m === mode}
              hint={t(`mode.${m}.desc`)}
              onSelect={() => {
                pop.setOpen(false);
                if (m !== mode) void setMode(m);
              }}
            >
              {t(`mode.${m}`)}
            </MenuOption>
          ))}
        </div>
      </PopoverPanel>
    </>
  );
}
