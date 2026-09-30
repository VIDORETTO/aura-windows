import { CheckCircle2, Loader2 } from "lucide-react";
import { useT } from "../i18n";
import type { Thread } from "../state/conversation";
import { useSession } from "./session";

/** Status pill shown while an answer runs with the Overlay unfocused. */
export function Minibar({ thread }: { thread: Thread | undefined }) {
  const t = useT();
  const setMinibar = useSession((s) => s.setMinibar);
  const running = thread?.running ?? false;
  const last = thread ? [...thread.blocks].reverse().find((b) => b.type === "assistant") : undefined;
  const preview = last && last.type === "assistant" ? last.text.replace(/\s+/g, " ").slice(-90) : "";
  return (
    <button
      type="button"
      data-tauri-drag-region
      onClick={() => setMinibar(false)}
      className="flex w-full items-center gap-2 px-3 py-2 text-left text-[13px]"
      aria-label={running ? t("minibar.working") : t("minibar.done")}
    >
      {running ? <Loader2 size={14} className="shrink-0 animate-spin text-accent" /> : <CheckCircle2 size={14} className="shrink-0 text-success" />}
      <span className="shrink-0 font-medium">{running ? t("minibar.working") : t("minibar.done")}</span>
      <span className="truncate text-muted">{preview}</span>
    </button>
  );
}
