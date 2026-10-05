import { useApp } from "../state/app";
import { en } from "./en";
import { ptBR, type MessageKey } from "./pt-BR";
import type { Language } from "../ipc/types";

export type { MessageKey };

const tables: Record<Language, Record<MessageKey, string>> = { ptBr: ptBR, en };

/** Every message of a language (settings search, 017). */
export function messages(lang: Language): Record<MessageKey, string> {
  return tables[lang] ?? ptBR;
}

export function translate(lang: Language, key: MessageKey, vars?: Record<string, string | number>): string {
  let s = tables[lang]?.[key] ?? ptBR[key] ?? key;
  if (vars) for (const [k, v] of Object.entries(vars)) s = s.replaceAll(`{${k}}`, String(v));
  return s;
}

/** `const t = useT(); t("input.send")` — re-renders on language change. */
export function useT() {
  const lang = useApp((s) => s.settings?.language ?? "ptBr");
  return (key: MessageKey, vars?: Record<string, string | number>) => translate(lang, key, vars);
}
