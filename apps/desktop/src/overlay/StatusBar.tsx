// Status line under the input (expanded): plan/provider, context meter and the
// privacy indicators. The indicators also show in the compact header.

import { EyeOff, PauseCircle } from "lucide-react";
import { api } from "../ipc/commands";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { useConversation } from "../state/conversation";
import { CHATGPT_PLAN, useSession } from "./session";
import { PopoverPanel, usePopover } from "../ui/Popover";
import { Button } from "../ui/primitives";

const MANAGE_USAGE_URL = "https://chatgpt.com/settings/usage";

/** Capture/privacy state: must stay visible in every Overlay state. */
export function StatusBadges() {
  const t = useT();
  const privacy = useApp((x) => x.privacy);
  const recording = useApp((x) => x.recording);
  const ephemeral = useSession((s) => s.ephemeral);
  return (
    <>
      {privacy?.paused && (
        <span className="flex shrink-0 items-center gap-1 text-warning" title={t("header.paused")}>
          <PauseCircle size={13} /> {t("header.paused")}
        </span>
      )}
      {ephemeral && (
        <span className="flex shrink-0 items-center gap-1" title={t("header.ephemeral")}>
          <EyeOff size={13} aria-label={t("header.ephemeral")} />
        </span>
      )}
      {recording.length > 0 && (
        <span
          className="flex shrink-0 items-center gap-1 text-danger"
          title={t("header.recording", { sources: recording.join(", ") })}
          aria-label={t("header.recording", { sources: recording.join(", ") })}
        >
          <span className="h-2 w-2 animate-pulse rounded-full bg-danger" /> REC
        </span>
      )}
    </>
  );
}

export function StatusBar() {
  const t = useT();
  const s = useSession();
  const usage = useConversation((c) => (s.threadId ? c.threads[s.threadId]?.tokens : null));
  const pct = usage?.window ? Math.min(100, Math.round((usage.used / usage.window) * 100)) : null;
  const provider = s.providers.find((p) => `aura-${p.id}` === s.provider);
  const confirm = usePopover();
  return (
    <div data-tauri-drag-region className="flex h-7 shrink-0 items-center gap-3 px-4 pb-1 text-[11px] text-muted">
      {s.provider === CHATGPT_PLAN ? (
        <span className="flex min-w-0 items-center gap-1 truncate">
          <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-success" aria-hidden />
          {t("header.usingPlan")} ·
          <button type="button" className="underline-offset-2 hover:text-fg hover:underline" onClick={() => void api.openExternal(MANAGE_USAGE_URL)}>
            {t("header.manageUsage")}
          </button>
        </span>
      ) : (
        <span className="truncate">{t("header.usingProvider", { name: provider?.name ?? s.provider })}</span>
      )}
      {s.profile && (
        <span className="truncate rounded-full border border-accent/40 px-1.5 text-accent" title={s.profile.instructions}>
          {t("header.profile", { name: s.profile.name })}
        </span>
      )}
      <span className="flex-1" />
      <StatusBadges />
      {pct !== null && (
        <button
          ref={confirm.anchor}
          type="button"
          className="flex shrink-0 items-center gap-1.5 rounded px-1 hover:bg-hover hover:text-fg"
          title={t("header.compactHint")}
          aria-label={t("header.context", { pct })}
          aria-expanded={confirm.open}
          onClick={confirm.toggle}
        >
          <span className="h-1 w-12 overflow-hidden rounded-full bg-hover">
            <span className={pct > 85 ? "block h-full bg-warning" : "block h-full bg-accent"} style={{ width: `${pct}%` }} />
          </span>
          <span className="tabular-nums">{pct}%</span>
        </button>
      )}
      {/* Compaction asks first (015 AC-009). */}
      <PopoverPanel pop={confirm} label={t("context.compactConfirm")} align="end" className="w-64">
        <p className="px-1 text-[13px] font-medium text-fg">{t("context.compactConfirm")}</p>
        <p className="px-1 pt-0.5 text-[11px] text-muted">{t("context.compactConfirm.desc")}</p>
        <div className="mt-2 flex justify-end gap-1.5">
          <Button size="sm" variant="ghost" onClick={() => confirm.setOpen(false)}>
            {t("common.cancel")}
          </Button>
          <Button
            size="sm"
            variant="primary"
            onClick={() => {
              confirm.setOpen(false);
              void s.compact();
            }}
          >
            {t("context.compactAction")}
          </Button>
        </div>
      </PopoverPanel>
    </div>
  );
}
