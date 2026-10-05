import { FileStack, FolderPlus, History, Mic, Maximize2, Minimize2, Minus, Plus, Settings2 } from "lucide-react";
import { inTauri } from "../ipc/bridge";
import { api } from "../ipc/commands";
import { useT } from "../i18n";
import { IconButton } from "../ui/primitives";
import { AppearanceButton } from "./Appearance";
import { ModeBadge, ModelPicker } from "./ModelPicker";
import { StatusBadges } from "./StatusBar";
import { useSession } from "./session";

/**
 * Window chrome of the Overlay: drag area, conversation controls and the
 * "minimize to tray" button (the only way out besides the shortcut). The
 * compact variant is a slim strip above the input.
 */
export function Header({ compact, subtitle }: { compact: boolean; subtitle?: string | null }) {
  const t = useT();
  const s = useSession();
  const pickFolder = async () => {
    if (!inTauri()) return;
    const { open } = await import("@tauri-apps/plugin-dialog");
    const dir = await open({ directory: true, multiple: false });
    if (dir) await s.setGranted([...new Set([...s.granted, String(dir)])]);
  };
  const expanded = s.overlayMode === "expanded";

  return (
    <header
      data-tauri-drag-region
      className={compact ? "flex h-8 shrink-0 items-center gap-1 pl-3.5 pr-1" : "flex h-11 shrink-0 items-center gap-1.5 border-b border-line pl-3.5 pr-1.5"}
    >
      <span data-tauri-drag-region className="flex shrink-0 items-center gap-1.5 text-[13px] font-semibold">
        <span data-tauri-drag-region className="h-2 w-2 rounded-full bg-accent" aria-hidden />
        Aura
      </span>
      {compact ? (
        <>
          {/* Model and effort can be chosen before the first message (017). */}
          <ModelPicker compact />
          <ModeBadge />
          {subtitle && (
            <span data-tauri-drag-region className="min-w-0 truncate text-[11px] text-muted" title={subtitle}>
              · {subtitle}
            </span>
          )}
        </>
      ) : (
        <>
          <ModelPicker />
          {s.mode === "task" && (
            <IconButton label={t("header.grantFolder", { n: s.granted.length })} onClick={() => void pickFolder()} title={s.granted.join("\n") || t("header.grantFolder", { n: 0 })}>
              <FolderPlus size={15} />
              {s.granted.length > 0 && <span className="text-[10px]">{s.granted.length}</span>}
            </IconButton>
          )}
        </>
      )}
      <div data-tauri-drag-region className="h-full min-w-4 flex-1" />
      {compact && (
        <span className="flex items-center gap-2 pr-1 text-[11px]">
          <StatusBadges />
        </span>
      )}
      {!compact && (
        <>
          <IconButton label={t("header.newConversation")} onClick={() => void s.newConversation()}>
            <Plus size={16} />
          </IconButton>
          {s.threadId && (
            <IconButton label={t("work.title")} active={s.workOpen} onClick={() => s.toggleWork()}>
              <FileStack size={15} />
            </IconButton>
          )}
        </>
      )}
      <IconButton label={t("meeting.title")} active={s.meetingOpen} onClick={() => s.toggleMeeting()}>
        <Mic size={15} />
      </IconButton>
      <IconButton label={t("header.history")} active={s.historyOpen} onClick={() => s.toggleHistory()}>
        <History size={15} />
      </IconButton>
      <AppearanceButton grow={compact} />
      <IconButton label={t("header.settings")} onClick={() => void api.settingsOpen()}>
        <Settings2 size={15} />
      </IconButton>
      <span className="mx-0.5 h-4 w-px bg-[var(--border)]" aria-hidden />
      <IconButton label={expanded ? t("header.compact") : t("header.expand")} onClick={() => s.setOverlayMode(expanded ? "compact" : "expanded")}>
        {expanded ? <Minimize2 size={14} /> : <Maximize2 size={14} />}
      </IconButton>
      <IconButton label={t("header.minimize")} onClick={() => void api.overlayHide()}>
        <Minus size={16} />
      </IconButton>
    </header>
  );
}
