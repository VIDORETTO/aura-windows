// Search box above the Settings menu (017): results replace the page list
// while there is a query; choosing one opens the page and flashes the item.

import { Search, X } from "lucide-react";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { messages, useT, type MessageKey } from "../i18n";
import { api } from "../ipc/commands";
import { useApp } from "../state/app";
import { cx } from "../ui/primitives";
import { normalize } from "../overlay/composer";
import { buildIndex, searchIndex, type SearchEntry } from "./search";

/** Scrolls to the element showing `text` inside `root` and flashes it. */
export function revealText(root: HTMLElement, text: string) {
  const wanted = normalize(text);
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);
  let best: HTMLElement | null = null;
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    const el = n as HTMLElement;
    const own = normalize(
      [el.getAttribute("aria-label") ?? "", el.getAttribute("placeholder") ?? "", el.textContent ?? ""].join(" "),
    );
    // The deepest element that still contains the text.
    if (own.includes(wanted)) best = el;
  }
  if (!best) return;
  const target = (best.closest("label, li, section > header, [role=group], .py-2\\.5, .py-2") as HTMLElement | null) ?? best;
  target.scrollIntoView?.({ block: "center" });
  target.classList.add("search-hit");
  setTimeout(() => target.classList.remove("search-hit"), 1800);
}

export function SettingsSearch({
  pages,
  inputRef,
  onOpen,
}: {
  pages: { id: string; label: MessageKey }[];
  inputRef: React.RefObject<HTMLInputElement | null>;
  onOpen: (page: string, text: string) => void;
}) {
  const t = useT();
  const lang = useApp((s) => s.settings?.language ?? "ptBr");
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const listId = useId();
  const index = useMemo(
    () => buildIndex(messages(lang), Object.fromEntries(pages.map((p) => [p.id, t(p.label)]))),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [lang, pages],
  );
  const results = useMemo(() => searchIndex(index, query), [index, query]);
  const pageName = (id: string) => t(pages.find((p) => p.id === id)!.label);
  const listRef = useRef<HTMLUListElement>(null);

  useEffect(() => setActive(0), [query]);
  useEffect(() => {
    listRef.current?.querySelector(`[aria-selected="true"]`)?.scrollIntoView?.({ block: "nearest" });
  }, [active]);

  // The last option hands the query to the agent ("Configurar com IA", 022).
  const total = results.length + 1;
  const askAi = () => {
    const request = query.trim();
    if (!request) return;
    void api.agentTask(t("settings.ai.prompt", { request }), "task");
    setQuery("");
  };

  const open = (r: SearchEntry) => {
    onOpen(r.page, r.text);
    setQuery("");
  };

  return (
    <div className="flex min-h-0 flex-col gap-1">
      <div className="relative">
        <Search size={13} className="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-muted" aria-hidden />
        <input
          ref={inputRef}
          type="text"
          role="combobox"
          aria-expanded={query.trim() !== ""}
          aria-controls={listId}
          aria-activedescendant={query.trim() ? `${listId}-${active}` : undefined}
          aria-label={t("settings.search")}
          placeholder={t("settings.search.placeholder")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown" && query.trim()) {
              e.preventDefault();
              setActive((a) => (a + 1) % total);
            } else if (e.key === "ArrowUp" && query.trim()) {
              e.preventDefault();
              setActive((a) => (a - 1 + total) % total);
            } else if (e.key === "Enter" && results[active]) {
              e.preventDefault();
              open(results[active]);
            } else if (e.key === "Enter" && query.trim() && active === results.length) {
              e.preventDefault();
              askAi();
            } else if (e.key === "Escape" && query) {
              e.preventDefault();
              e.stopPropagation();
              setQuery("");
            }
          }}
          className="h-8 w-full rounded-md border border-line bg-surface-strong pl-7 pr-7 text-[13px] outline-none placeholder:text-muted focus:border-accent"
        />
        {query && (
          <button type="button" aria-label={t("settings.search.clear")} className="absolute right-1.5 top-1/2 -translate-y-1/2 rounded p-0.5 text-muted hover:text-fg" onClick={() => { setQuery(""); inputRef.current?.focus(); }}>
            <X size={12} />
          </button>
        )}
      </div>
      {query.trim() !== "" && (
        <ul ref={listRef} id={listId} role="listbox" aria-label={t("settings.search.results")} className="flex max-h-[calc(100vh-120px)] flex-col gap-0.5 overflow-y-auto">
          {results.length === 0 && <li className="px-2 py-1.5 text-[12px] text-muted">{t("settings.search.none")}</li>}
          {results.map((r, i) => (
            <li
              key={`${r.page}-${r.text}`}
              id={`${listId}-${i}`}
              role="option"
              aria-selected={i === active}
              onMouseDown={(e) => {
                e.preventDefault();
                open(r);
              }}
              onMouseEnter={() => setActive(i)}
              className={cx("cursor-pointer rounded-md px-2 py-1.5", i === active && "bg-hover")}
            >
              <div className="line-clamp-2 text-[12px] leading-snug">{r.text}</div>
              <div className="text-[10px] uppercase tracking-wide text-muted">{pageName(r.page)}</div>
            </li>
          ))}
          <li
            id={`${listId}-${results.length}`}
            role="option"
            aria-selected={active === results.length}
            onMouseDown={(e) => {
              e.preventDefault();
              askAi();
            }}
            onMouseEnter={() => setActive(results.length)}
            className={cx("cursor-pointer rounded-md px-2 py-1.5", active === results.length && "bg-hover")}
          >
            <div className="text-[12px] leading-snug">{t("settings.search.askAi")}</div>
            <div className="line-clamp-1 text-[10px] text-muted">{query.trim()}</div>
          </li>
        </ul>
      )}
    </div>
  );
}
