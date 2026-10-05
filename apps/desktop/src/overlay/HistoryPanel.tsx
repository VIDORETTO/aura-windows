import { Archive, ArchiveRestore, Pencil, Pin, PinOff, Search, Trash2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { ConversationSummary } from "../ipc/types";
import { useT } from "../i18n";
import { relativeTime } from "../lib/format";
import { useApp } from "../state/app";
import { Button, IconButton, TextField, cx } from "../ui/primitives";
import { useSession } from "./session";

const PAGE = 50;

export function HistoryPanel() {
  const t = useT();
  const lang = useApp((s) => s.settings?.language ?? "ptBr");
  const open = useSession((s) => s.openConversation);
  const active = useSession((s) => s.threadId);
  const [items, setItems] = useState<ConversationSummary[]>([]);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState<{ id: string; name: string } | null>(null);
  /** Archived conversations instead of the active ones. */
  const [archived, setArchived] = useState(false);
  /** Conversation whose deletion waits for the in-app confirmation. */
  const [confirming, setConfirming] = useState<string | null>(null);

  const [cursor, setCursor] = useState<string | null>(null);
  const request = useRef(0);

  /** First page(s) for `q`; `count` keeps what was already loaded after an action. */
  const load = async (q: string, count = PAGE) => {
    const id = ++request.current;
    setLoading(true);
    try {
      const page = await api.conversationHistory({ search: q || null, archived, limit: Math.max(PAGE, count) });
      if (id !== request.current) return; // a newer search or page won
      setItems(page.items);
      setCursor(page.nextCursor);
    } finally {
      if (id === request.current) setLoading(false);
    }
  };

  const loadMore = async () => {
    if (!cursor) return;
    const id = ++request.current;
    setLoading(true);
    try {
      const page = await api.conversationHistory({ search: search || null, archived, limit: PAGE, cursor });
      if (id !== request.current) return;
      const fresh = page.items.filter((c) => !items.some((p) => p.id === c.id));
      setItems((prev) => [...prev, ...page.items.filter((c) => !prev.some((p) => p.id === c.id))]);
      // The next page re-reads the boundary second; nothing new and the same cursor = the end.
      setCursor(fresh.length === 0 && page.nextCursor === cursor ? null : page.nextCursor);
    } catch (e) {
      useApp.getState().notify("error", errorMessage(e));
    } finally {
      if (id === request.current) setLoading(false);
    }
  };

  const refresh = () => load(search, items.length);

  useEffect(() => {
    const h = setTimeout(() => void load(search), 150);
    return () => clearTimeout(h);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [search, archived]);

  const act = async (f: () => Promise<void>) => {
    try {
      await f();
      await refresh();
    } catch (e) {
      useApp.getState().notify("error", errorMessage(e));
    }
  };

  const rename = async () => {
    if (!editing) return;
    const name = editing.name.trim();
    const current = items.find((c) => c.id === editing.id);
    setEditing(null);
    if (!name || name === current?.title) return;
    try {
      await api.conversationRename(editing.id, name);
      await refresh();
    } catch (e) {
      useApp.getState().notify("error", errorMessage(e));
    }
  };

  const sorted = [...items].sort((a, b) => Number(b.pinned) - Number(a.pinned) || b.updatedAt - a.updatedAt);

  return (
    <aside className="absolute inset-y-0 left-0 z-10 flex w-64 shrink-0 flex-col border-r border-line bg-[var(--surface-menu)] backdrop-blur-xl min-[1024px]:static min-[1024px]:z-auto" aria-label={t("history.title")}>
      <div className="relative p-2">
        <Search size={13} className="absolute left-4 top-1/2 -translate-y-1/2 text-muted" />
        <TextField autoFocus placeholder={t("history.search")} value={search} onChange={(e) => setSearch(e.target.value)} className="pl-7" />
      </div>
      <div className="flex px-2 pb-1">
        <button
          type="button"
          aria-pressed={archived}
          onClick={() => setArchived((a) => !a)}
          className={cx("flex items-center gap-1 rounded px-1.5 py-0.5 text-[11px] text-muted hover:bg-hover hover:text-fg", archived && "bg-hover text-fg")}
        >
          <Archive size={11} /> {t("history.archived")}
        </button>
      </div>
      <ul className="flex-1 overflow-y-auto px-1 pb-2">
        {!loading && sorted.length === 0 && <li className="px-3 py-6 text-center text-[13px] text-muted">{t(archived ? "history.archivedEmpty" : "history.empty")}</li>}
        {sorted.map((c) => (
          <li key={c.id} className={cx("group flex flex-wrap items-center rounded-md", c.id === active && "bg-hover")}>
            {editing?.id === c.id ? (
              <TextField
                autoFocus
                aria-label={t("history.renameField")}
                value={editing.name}
                className="mx-1 my-1 h-7 flex-1"
                onChange={(e) => setEditing({ id: c.id, name: e.target.value })}
                onBlur={() => void rename()}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    void rename();
                  } else if (e.key === "Escape") {
                    // Cancels only the edit; the Overlay keeps its panels.
                    e.preventDefault();
                    e.stopPropagation();
                    setEditing(null);
                  }
                }}
              />
            ) : (
            <button type="button" className="min-w-0 flex-1 px-2 py-1.5 text-left" onClick={() => void open(c.id)}>
              <div className="flex items-center gap-1 truncate text-[13px] font-medium">
                {c.pinned && <Pin size={11} className="shrink-0 text-accent" />}
                <span className="truncate">{c.title || "…"}</span>
              </div>
              <div className="truncate text-[11px] text-muted">
                {relativeTime(c.updatedAt, lang)} · {c.preview}
              </div>
            </button>
            )}
            {editing?.id !== c.id && confirming !== c.id && <div className="hidden shrink-0 group-hover:flex group-focus-within:flex">
              {archived ? (
                <IconButton label={t("history.unarchive")} className="h-6 w-6" onClick={() => void act(() => api.conversationUnarchive(c.id))}>
                  <ArchiveRestore size={12} />
                </IconButton>
              ) : (
                <>
                  <IconButton label={t("history.renameItem", { title: c.title || "…" })} className="h-6 w-6" onClick={() => setEditing({ id: c.id, name: c.title })}>
                    <Pencil size={12} />
                  </IconButton>
                  <IconButton label={c.pinned ? t("history.unpin") : t("history.pin")} className="h-6 w-6" onClick={() => void act(() => api.conversationPin(c.id, !c.pinned))}>
                    {c.pinned ? <PinOff size={12} /> : <Pin size={12} />}
                  </IconButton>
                  <IconButton label={t("history.archive")} className="h-6 w-6" onClick={() => void act(() => api.conversationArchive(c.id))}>
                    <Archive size={12} />
                  </IconButton>
                </>
              )}
              <IconButton label={t("history.delete")} className="h-6 w-6 hover:text-danger" onClick={() => setConfirming(c.id)}>
                <Trash2 size={12} />
              </IconButton>
            </div>}
            {confirming === c.id && (
              // Asked here, not with the browser's confirm dialog.
              <div role="group" aria-label={t("history.confirmDelete")} className="flex w-full flex-col gap-1 px-2 pb-1.5">
                <span className="text-[12px]">{t("history.confirmDelete")}</span>
                <div className="flex gap-1.5">
                  <Button size="sm" variant="danger" autoFocus onClick={() => { setConfirming(null); void act(() => api.conversationDelete(c.id)); }}>
                    {t("common.confirm")}
                  </Button>
                  <Button size="sm" variant="ghost" onClick={() => setConfirming(null)}>
                    {t("common.cancel")}
                  </Button>
                </div>
              </div>
            )}
          </li>
        ))}
        {cursor && (
          <li className="px-2 py-2">
            <button type="button" disabled={loading} onClick={() => void loadMore()} className="w-full rounded-md px-2 py-1.5 text-[12px] text-muted hover:bg-hover disabled:opacity-50">
              {t("history.loadMore")}
            </button>
          </li>
        )}
      </ul>
    </aside>
  );
}
