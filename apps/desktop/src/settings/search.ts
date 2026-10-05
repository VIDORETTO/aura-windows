// Settings search (017): every label, title and hint of the Settings pages,
// found by words in any order, without accents or case.

import { normalize } from "../overlay/composer";

/** Message key prefixes shown on each Settings page. */
export const PAGE_PREFIXES: Record<string, string[]> = {
  general: ["general.", "memories.", "appearance."],
  account: ["account.", "efforts."],
  providers: ["providers."],
  privacy: ["privacy.", "recordings."],
  voice: ["voice.", "tts."],
  extensions: ["extensions."],
  profiles: ["profiles."],
  shortcuts: ["shortcuts."],
  diagnostics: ["diagnostics."],
  about: ["about.", "updates."],
};

export interface SearchEntry {
  page: string;
  /** Text as shown on the page (placeholders removed). */
  text: string;
  norm: string;
}

/** Readable text of a message: no `{placeholders}`, one line, not too long. */
function readable(value: string): string | null {
  const text = value.replace(/\{[^}]*\}/g, "").replace(/\s+/g, " ").replace(/\s([:.,)])/g, "$1").trim();
  if (text.length < 3 || text.length > 140) return null;
  return text;
}

export function buildIndex(messages: Record<string, string>, pageTitles: Record<string, string>): SearchEntry[] {
  const out: SearchEntry[] = [];
  const seen = new Set<string>();
  const add = (page: string, raw: string) => {
    const text = readable(raw);
    if (!text) return;
    const key = `${page}\u0000${text}`;
    if (seen.has(key)) return;
    seen.add(key);
    out.push({ page, text, norm: normalize(text) });
  };
  for (const [page, title] of Object.entries(pageTitles)) add(page, title);
  for (const [key, value] of Object.entries(messages)) {
    const page = Object.entries(PAGE_PREFIXES).find(([, prefixes]) => prefixes.some((p) => key.startsWith(p)))?.[0];
    if (page) add(page, value);
  }
  return out;
}

/**
 * Entries containing every word of the query; whole-word and prefix hits
 * and shorter texts first.
 */
export function searchIndex(index: SearchEntry[], query: string, limit = 40): SearchEntry[] {
  const words = normalize(query).split(/\s+/).filter(Boolean);
  if (words.length === 0) return [];
  const score = (e: SearchEntry) => {
    let s = 0;
    for (const w of words) {
      if (e.norm.startsWith(w)) s += 3;
      else if (new RegExp(`(^|[^\\p{L}\\p{N}])${w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`, "u").test(e.norm)) s += 2;
      else s += 1;
    }
    return s * 1000 - e.text.length;
  };
  return index
    .filter((e) => words.every((w) => e.norm.includes(w)))
    .map((e) => ({ e, s: score(e) }))
    .sort((a, b) => b.s - a.s)
    .slice(0, limit)
    .map(({ e }) => e);
}
