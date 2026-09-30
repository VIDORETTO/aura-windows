import { X } from "lucide-react";
import { useApp } from "../state/app";
import { cx } from "./primitives";

export function Toasts() {
  const notices = useApp((s) => s.notices);
  const dismiss = useApp((s) => s.dismiss);
  if (notices.length === 0) return null;
  return (
    <div className="pointer-events-none fixed bottom-3 right-3 z-50 flex max-w-sm flex-col gap-2" aria-live="polite">
      {notices.map((n) => (
        <div
          key={n.id}
          role={n.level === "error" ? "alert" : "status"}
          className={cx(
            "fade-in pointer-events-auto flex items-start gap-2 rounded-md border bg-surface-strong px-3 py-2 text-[13px] shadow-lg",
            n.level === "error" ? "border-danger/50" : n.level === "warning" ? "border-warning/50" : "border-line",
          )}
        >
          <span className="flex-1">{n.message}</span>
          <button aria-label="Fechar" className="text-muted hover:text-fg" onClick={() => dismiss(n.id)}>
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}
