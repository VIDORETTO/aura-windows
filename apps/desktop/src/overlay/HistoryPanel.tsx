import { Archive, Pin, PinOff, Search, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { api } from "../ipc/commands";
import type { ConversationSummary } from "../ipc/types";
import { useT } from "../i18n";
import { relativeTime } from "../lib/format";
import { useApp } from "../state/app";
import { IconButton, TextField, cx } from "../ui/primitives";
import { useSession } from "./session";

export function HistoryPanel() {
  const t = useT();
  const lang = useApp((s) => s.settings?.language ?? "ptBr");
  const open = useSession((s) => s.openConversation);
  const active = useSession((s) => s.threadId);
  const [items, setItems] = useState<ConversationSummary[]>([]);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);

  const load = async (q: string) => {
    setLoading(true);
    try {
      setItems((await api.conversationHistory({ search: q || null, limit: 50 })).items);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    const h = setTimeout(() => void load(search), 150);
    return () => clearTimeout(h);
  }, [search]);

  const sorted = [...items].sort((a, b) => Number(b.pinned) - Number(a.pinned) || b.updatedAt - a.updatedAt);

  return (
    <aside className="flex w-64 shrink-0 flex-col border-r border-line" aria-label={t("history.title")}>
      <div className="relative p-2">
        <Search size={13} className="absolute left-4 top-1/2 -translate-y-1/2 text-muted" />
        <TextField autoFocus placeholder={t("history.search")} value={search} onChange={(e) => setSearch(e.target.value)} className="pl-7" />
      </div>
      <ul className="flex-1 overflow-y-auto px-1 pb-2">
        {!loading && sorted.length === 0 && <li className="px-3 py-6 text-center text-[13px] text-muted">{t("history.empty")}</li>}
        {sorted.map((c) => (
          <li key={c.id} className={cx("group flex items-center rounded-md", c.id === active && "bg-hover")}>
            <button type="button" className="min-w-0 flex-1 px-2 py-1.5 text-left" onClick={() => void open(c.id)}>
              <div className="flex items-center gap-1 truncate text-[13px] font-medium">
                {c.pinned && <Pin size={11} className="shrink-0 text-accent" />}
                <span className="truncate">{c.title || "…"}</span>
              </div>
              <div className="truncate text-[11px] text-muted">
                {relativeTime(c.updatedAt, lang)} · {c.preview}
              </div>
            </button>
            <div className="hidden shrink-0 group-hover:flex group-focus-within:flex">
              <IconButton
                label={c.pinned ? t("history.unpin") : t("history.pin")}
                className="h-6 w-6"
                onClick={async () => {
                  await api.conversationPin(c.id, !c.pinned);
                  void load(search);
                }}
              >
                {c.pinned ? <PinOff size={12} /> : <Pin size={12} />}
              </IconButton>
              <IconButton
                label={t("history.archive")}
                className="h-6 w-6"
                onClick={async () => {
                  await api.conversationArchive(c.id);
                  void load(search);
                }}
              >
                <Archive size={12} />
              </IconButton>
              <IconButton
                label={t("history.delete")}
                className="h-6 w-6 hover:text-danger"
                onClick={async () => {
                  if (!window.confirm(t("history.confirmDelete"))) return;
                  await api.conversationDelete(c.id);
                  void load(search);
                }}
              >
                <Trash2 size={12} />
              </IconButton>
            </div>
          </li>
        ))}
      </ul>
    </aside>
  );
}
