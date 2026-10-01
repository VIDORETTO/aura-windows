import { FileText, Image as ImageIcon, Lock, Monitor, Quote, Sparkles, X } from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { inTauri } from "../ipc/bridge";
import type { ContextChip } from "../ipc/types";
import { useT } from "../i18n";
import { cx } from "../ui/primitives";
import { Floating } from "../ui/floating";
import { useRef, useState } from "react";

const ICONS = { screen: Monitor, region: Monitor, selection: Quote, audio: FileText, clip: Monitor, file: FileText, image: ImageIcon, skill: Sparkles };

export function previewSrc(path: string): string {
  if (path.startsWith("data:")) return path;
  return inTauri() ? convertFileSrc(path) : path;
}

export function ChipList({ chips, onRemove, grow = false }: { chips: ContextChip[]; onRemove?: (id: string) => void; grow?: boolean }) {
  if (chips.length === 0) return null;
  return (
    <ul className="flex flex-wrap gap-1.5" aria-label="Contexto">
      {chips.map((c) => (
        <Chip key={c.id} chip={c} onRemove={onRemove} grow={grow} />
      ))}
    </ul>
  );
}

function Chip({ chip: c, onRemove, grow }: { chip: ContextChip; onRemove?: (id: string) => void; grow: boolean }) {
  const t = useT();
  const ref = useRef<HTMLLIElement>(null);
  const [hover, setHover] = useState(false);
  const Icon = c.blockedReason ? Lock : ICONS[c.kind];
  const blocked = c.blockedReason ? (t(`context.blocked.${c.blockedReason}` as "context.blocked.paused") ?? c.blockedReason) : null;
  const preview = c.previewPath && !c.blockedReason ? previewSrc(c.previewPath) : null;
  return (
    <li
      ref={ref}
      onPointerEnter={() => setHover(true)}
      onPointerLeave={() => setHover(false)}
      className={cx(
        "fade-in flex max-w-[260px] items-center gap-1.5 rounded-full border border-line bg-surface-strong py-0.5 pl-2 pr-1 text-xs",
        c.blockedReason && "text-muted",
      )}
      title={blocked ?? `${c.label} · ${t("context.tokens", { n: c.tokenEstimate })}`}
    >
      <Icon size={13} className={c.blockedReason ? "text-warning" : "text-accent"} aria-hidden />
      <span className="truncate">{blocked ? `${c.label} — ${blocked}` : c.label}</span>
      {preview && hover && (
        <Floating anchor={ref} placement="above" grow={grow} className="pointer-events-none">
          <img data-chip-preview src={preview} alt="" className="max-h-40 max-w-[240px] rounded-md border border-line shadow-lg" />
        </Floating>
      )}
      {onRemove && (
        <button
          type="button"
          className="rounded-full p-0.5 text-muted hover:bg-hover hover:text-fg"
          aria-label={t("context.remove", { label: c.label })}
          onClick={() => onRemove(c.id)}
        >
          <X size={12} />
        </button>
      )}
    </li>
  );
}
