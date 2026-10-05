// One pill for provider, model, reasoning effort and mode (replaces three
// native selects that did not fit the header). The compact Overlay shows a
// slimmer pill so model and effort can be chosen before the first message
// (017); the mode keeps its own badge there.

import { ChevronDown } from "lucide-react";
import { useT } from "../i18n";
import { cx } from "../ui/primitives";
import { useApp } from "../state/app";
import { MenuLabel, MenuOption, PopoverPanel, usePopover } from "../ui/Popover";
import { CHATGPT_PLAN, effectiveModel, modelCapabilities, useSession, type ModeKey, type ModelCapabilities } from "./session";

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

/** A reasoning effort as a small toggle; several fit on one row. */
function EffortChip({ selected, onSelect, children }: { selected: boolean; onSelect: () => void; children: string }) {
  return (
    <button
      type="button"
      role="menuitemradio"
      aria-checked={selected}
      onClick={onSelect}
      className={cx(
        "rounded-full border px-2 py-0.5 text-[12px] hover:bg-hover",
        selected ? "border-accent bg-accent/10 text-accent" : "border-line text-fg",
      )}
    >
      {children}
    </button>
  );
}

export function ModelPicker({ compact = false }: { compact?: boolean }) {
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
  // "Default" names the model it stands for: the Settings default (when the plan still offers it) or the plan's own default.
  const effective = effectiveModel(s, settingsDefault);
  const modelName = models.find((m) => m.id === effective)?.name ?? t("general.defaultModel");
  const locked = s.threadId !== null;
  const caps = modelCapabilities(s, effective);
  const effortName = s.effort ? effortLabel(t, s.effort) : null;
  const modeName = t(`mode.${s.mode}`);
  const yoloTask = useApp((a) => a.settings?.yolo ?? false) && s.mode === "task";
  const label = compact
    ? effortName
      ? t("picker.compactLabelEffort", { model: modelName, effort: effortName })
      : t("picker.compactLabel", { model: modelName })
    : effortName
      ? t("picker.labelEffort", { model: modelName, mode: modeName, effort: effortName })
      : t("picker.label", { model: modelName, mode: modeName });

  return (
    <>
      <button
        ref={pop.anchor}
        type="button"
        onClick={pop.toggle}
        aria-expanded={pop.open}
        aria-label={label}
        title={label}
        className={cx(
          "flex min-w-0 items-center gap-1.5 border border-line hover:bg-hover",
          compact ? "h-5 max-w-[220px] rounded-full px-1.5 text-[11px]" : "h-7 max-w-[300px] rounded-md px-2 text-[12px]",
          pop.open && "bg-hover",
        )}
      >
        <span className={cx("truncate", compact ? "text-fg" : "font-medium")}>{modelName}</span>
        {effortName && (
          <>
            <span className="shrink-0 text-muted">·</span>
            <span className="shrink-0 text-muted">{effortName}</span>
          </>
        )}
        {!compact && (
          <>
            <span className="shrink-0 text-muted">·</span>
            <span className={cx("shrink-0", yoloTask ? "font-semibold text-danger" : "text-muted")}>{yoloTask ? t("yolo.badge", { mode: modeName }) : modeName}</span>
          </>
        )}
        <ChevronDown size={compact ? 11 : 13} className="shrink-0 text-muted" />
      </button>
      <PopoverPanel pop={pop} label={t(compact ? "picker.compactTitle" : "picker.title")} grow={compact} className="w-72 max-w-[calc(100vw-16px)]">
        {(s.providers.length > 0 || locked) && (
          <>
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
          </>
        )}
        <MenuLabel>{t("picker.model")}</MenuLabel>
        <div role="menu" aria-label={t("picker.model")} className="max-h-48 overflow-y-auto">
          {s.provider !== CHATGPT_PLAN && (
            <MenuOption selected={!s.model} onSelect={() => s.setModel(null)}>
              {t("general.defaultModel")}
            </MenuOption>
          )}
          {models.map((m) => (
            <MenuOption
              key={m.id}
              selected={s.provider === CHATGPT_PLAN ? m.id === effective : m.id === s.model}
              hint={capabilityHint(t, modelCapabilities(s, m.id))}
              onSelect={() => s.setModel(m.id)}
            >
              {m.name}
            </MenuOption>
          ))}
        </div>
        <MenuLabel>{t("picker.effort")}</MenuLabel>
        {caps && caps.efforts.length > 0 ? (
          <div role="menu" aria-label={t("picker.effort")} className="flex flex-wrap gap-1 px-2 pb-1 pt-0.5">
            <EffortChip selected={!s.effort} onSelect={() => s.setEffort(null)}>
              {caps.defaultEffort ? t("picker.effort.default", { effort: effortLabel(t, caps.defaultEffort).toLowerCase() }) : t("picker.effort.modelDefault")}
            </EffortChip>
            {caps.efforts.map((e) => (
              <EffortChip key={e} selected={s.effort === e} onSelect={() => s.setEffort(e)}>
                {effortLabel(t, e)}
              </EffortChip>
            ))}
          </div>
        ) : (
          <p className="px-2 pb-1 text-[11px] text-muted">{t(caps ? "picker.effort.unsupported" : "picker.effort.unknown")}</p>
        )}
        {!compact && (
          <>
            <MenuLabel>{t("picker.mode")}</MenuLabel>
            <div role="menu" aria-label={t("picker.mode")}>
              {MODES.map((m) => (
                <MenuOption key={m} selected={m === s.mode} hint={t(`mode.${m}.desc`)} onSelect={() => m !== s.mode && void s.setMode(m)}>
                  {t(`mode.${m}`)}
                </MenuOption>
              ))}
            </div>
          </>
        )}
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
  const yolo = useApp((a) => a.settings?.yolo ?? false) && mode === "task";
  const name = yolo ? t("yolo.badge", { mode: t(`mode.${mode}`) }) : t(`mode.${mode}`);
  return (
    <>
      <button
        ref={pop.anchor}
        type="button"
        onClick={pop.toggle}
        aria-expanded={pop.open}
        aria-label={t("mode.badge", { mode: name })}
        title={yolo ? t("yolo.badgeHint") : t(`mode.${mode}.desc`)}
        className={cx(
          "flex h-5 shrink-0 items-center gap-0.5 rounded-full border px-1.5 text-[11px] hover:bg-hover",
          yolo
            ? "border-danger/70 bg-danger/10 font-semibold text-danger"
            : mode === "task"
              ? "border-warning/60 text-warning"
              : mode === "plan"
                ? "border-accent/50 text-accent"
                : "border-line text-muted",
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
