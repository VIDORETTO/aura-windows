import { EyeOff, FileStack, FolderPlus, History, Minimize2, Maximize2, PauseCircle, Plus, Settings2, X } from "lucide-react";
import { inTauri } from "../ipc/bridge";
import { useConversation } from "../state/conversation";
import { api } from "../ipc/commands";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { IconButton, Select } from "../ui/primitives";
import { CHATGPT_PLAN, useSession, type ModeKey } from "./session";

const MANAGE_USAGE_URL = "https://chatgpt.com/settings/usage";

export function Header() {
  const t = useT();
  const s = useSession();
  const privacy = useApp((x) => x.privacy);
  const recording = useApp((x) => x.recording);
  const usage = useConversation((c) => (s.threadId ? c.threads[s.threadId]?.tokens : null));
  const pct = usage?.window ? Math.min(100, Math.round((usage.used / usage.window) * 100)) : null;
  const tokens =
    pct !== null ? (
      <button
        type="button"
        className="flex items-center gap-1 rounded px-1 hover:bg-hover hover:text-fg"
        title={t("header.compactHint")}
        aria-label={t("header.context", { pct })}
        onClick={() => void s.compact()}
      >
        <span className="h-1.5 w-10 overflow-hidden rounded-full bg-hover">
          <span className={pct > 85 ? "block h-full bg-warning" : "block h-full bg-accent"} style={{ width: `${pct}%` }} />
        </span>
        {pct}%
      </button>
    ) : null;
  const pickFolder = async () => {
    if (!inTauri()) return;
    const { open } = await import("@tauri-apps/plugin-dialog");
    const dir = await open({ directory: true, multiple: false });
    if (dir) await s.setGranted([...new Set([...s.granted, String(dir)])]);
  };
  const provider = s.providers.find((p) => `aura-${p.id}` === s.provider);
  const modelOptions =
    s.provider === CHATGPT_PLAN
      ? s.models.map((m) => ({ id: m.id, name: m.displayName }))
      : (provider?.models ?? []).map((m) => ({ id: m.id, name: m.displayName ?? m.id }));

  return (
    <header data-tauri-drag-region className="flex h-10 shrink-0 items-center gap-1.5 border-b border-line pl-3 pr-1.5">
      <span data-tauri-drag-region className="text-[13px] font-semibold">
        Aura
      </span>
      <Select
        aria-label="Provedor"
        className="h-7 max-w-[150px] text-[12px]"
        value={s.provider}
        disabled={s.threadId !== null}
        onChange={(e) => s.setProvider(e.target.value, null)}
      >
        <option value={CHATGPT_PLAN}>ChatGPT</option>
        {s.providers.map((p) => (
          <option key={p.id} value={`aura-${p.id}`}>
            {p.name}
          </option>
        ))}
      </Select>
      {modelOptions.length > 0 && (
        <Select aria-label="Modelo" className="h-7 max-w-[160px] text-[12px]" value={s.model ?? ""} onChange={(e) => s.setModel(e.target.value || null)}>
          <option value="">{t("general.defaultModel")}</option>
          {modelOptions.map((m) => (
            <option key={m.id} value={m.id}>
              {m.name}
            </option>
          ))}
        </Select>
      )}
      <Select aria-label="Modo" className="h-7 text-[12px]" value={s.mode} onChange={(e) => void s.setMode(e.target.value as ModeKey)} title={t(`mode.${s.mode}.desc`)}>
        <option value="chat">{t("mode.chat")}</option>
        <option value="task">{t("mode.task")}</option>
        <option value="plan">{t("mode.plan")}</option>
      </Select>
      {s.mode === "task" && (
        <IconButton label={t("header.grantFolder", { n: s.granted.length })} onClick={() => void pickFolder()} title={s.granted.join("\n") || t("header.grantFolder", { n: 0 })}>
          <FolderPlus size={15} />
          {s.granted.length > 0 && <span className="text-[10px]">{s.granted.length}</span>}
        </IconButton>
      )}
      <div data-tauri-drag-region className="flex min-w-0 flex-1 items-center justify-end gap-2 text-[11px] text-muted">
        {privacy?.paused && (
          <span className="flex items-center gap-1 text-warning" title={t("header.paused")}>
            <PauseCircle size={13} /> {t("header.paused")}
          </span>
        )}
        {s.ephemeral && <EyeOff size={13} aria-label={t("header.ephemeral")} />}
        {s.profile && (
          <span className="truncate rounded-full border border-accent/40 px-1.5 text-accent" title={s.profile.instructions}>
            {t("header.profile", { name: s.profile.name })}
          </span>
        )}
        {recording.length > 0 && (
          <span className="flex items-center gap-1" title={t("header.recording", { sources: recording.join(", ") })} aria-label={t("header.recording", { sources: recording.join(", ") })}>
            <span className="h-2 w-2 animate-pulse rounded-full bg-danger" /> REC
          </span>
        )}
        {tokens}
        {s.provider === CHATGPT_PLAN ? (
          <span className="truncate">
            ● {t("header.usingPlan")} ·{" "}
            <button className="underline underline-offset-2 hover:text-fg" onClick={() => void api.openExternal(MANAGE_USAGE_URL)}>
              {t("header.manageUsage")}
            </button>
          </span>
        ) : (
          <span className="truncate">{t("header.usingProvider", { name: provider?.name ?? s.provider })}</span>
        )}
      </div>
      <IconButton label={t("header.newConversation")} onClick={() => void s.newConversation()}>
        <Plus size={16} />
      </IconButton>
      {s.threadId && (
        <IconButton label={t("work.title")} active={s.workOpen} onClick={() => s.toggleWork()}>
          <FileStack size={16} />
        </IconButton>
      )}
      <IconButton label={t("header.history")} active={s.historyOpen} onClick={() => s.toggleHistory()}>
        <History size={16} />
      </IconButton>
      <IconButton label={t("header.settings")} onClick={() => void api.settingsOpen()}>
        <Settings2 size={16} />
      </IconButton>
      <IconButton
        label={s.overlayMode === "expanded" ? t("header.compact") : t("header.expand")}
        onClick={() => s.setOverlayMode(s.overlayMode === "expanded" ? "compact" : "expanded")}
      >
        {s.overlayMode === "expanded" ? <Minimize2 size={15} /> : <Maximize2 size={15} />}
      </IconButton>
      <IconButton label={t("header.close")} onClick={() => void api.overlayHide()}>
        <X size={16} />
      </IconButton>
    </header>
  );
}
