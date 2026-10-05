// Pure logic of the input bar menus (015): `/` does something (commands,
// quick commands, skills), `@` adds context. Both follow the caret.

export interface Trigger {
  kind: "/" | "@";
  /** Text typed after the trigger, up to the caret. */
  query: string;
  /** Range of the trigger word (`@que`) in the value. */
  start: number;
  end: number;
}

/** The trigger word at the caret: `@` at the start of any word, `/` only at the start of the message. */
export function findTrigger(value: string, caret: number): Trigger | null {
  const before = value.slice(0, caret);
  const start = Math.max(before.lastIndexOf(" "), before.lastIndexOf("\n"), before.lastIndexOf("\t")) + 1;
  const word = before.slice(start);
  if (word.startsWith("@")) return { kind: "@", query: word.slice(1), start, end: caret };
  if (word.startsWith("/") && start === 0) return { kind: "/", query: word.slice(1), start, end: caret };
  return null;
}

/** Lowercase without accents: `Região` → `regiao`. */
export function normalize(s: string): string {
  return s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
}

export interface Filterable {
  label: string;
  aliases?: string[];
}

/** Prefix matches first, then substring matches; nothing when nothing matches. */
export function filterMenu<T extends Filterable>(items: T[], query: string, limit = 8): T[] {
  const q = normalize(query);
  const names = (i: T) => [i.label.replace(/^[/@]/, ""), ...(i.aliases ?? [])].map(normalize);
  const prefix = items.filter((i) => names(i).some((n) => n.startsWith(q)));
  const inner = items.filter((i) => !prefix.includes(i) && names(i).some((n) => n.includes(q)));
  return [...prefix, ...inner].slice(0, limit);
}

/** Replaces the trigger word; an empty replacement also drops one surrounding space. */
export function replaceTrigger(value: string, t: Trigger, text: string): [string, number] {
  let before = value.slice(0, t.start);
  let after = value.slice(t.end);
  if (!text) {
    if (after.startsWith(" ") && (before === "" || before.endsWith(" "))) after = after.slice(1);
    else if (after === "" && before.endsWith(" ")) before = before.slice(0, -1);
  }
  return [before + text + after, before.length + text.length];
}

export interface QuickHint {
  /** `{args:x}` → "x"; `{args}` → ""; no argument → null. */
  arg: string | null;
  /** Where the text comes from: `{selecao}` (selection, else typed) or `{texto}`. */
  source: "selection" | "typed" | null;
  screen: boolean;
}

export function quickHint(template: string): QuickHint {
  const m = template.match(/\{args(?::([^}]*))?\}/);
  return {
    arg: m ? (m[1] ?? "") : null,
    source: template.includes("{selecao}") ? "selection" : template.includes("{texto}") ? "typed" : null,
    screen: template.includes("{tela}"),
  };
}

/** First readable line of a quick command template: `{args:x}` → `‹x›`, context placeholders hidden. */
export function templatePreview(template: string): string {
  return (
    template
      .replace(/\{args(?::([^}]*))?\}/g, (_m, d: string | undefined) => `‹${d || "…"}›`)
      .replace(/\{(selecao|texto|tela)\}/g, "")
      .split("\n")
      .map((l) => l.trim())
      .find(Boolean) ?? ""
  );
}
