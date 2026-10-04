// Command-line arguments typed in one field (MCP stdio servers, 008 AC-005).
// Semantics: whitespace separates arguments; "…" and '…' group text with
// spaces (quotes removed, adjacent parts join: --path="C:\x y"); "" and ''
// are empty arguments; inside double quotes \" is a literal quote. Any other
// backslash is literal, so Windows paths need no escaping.

export type ParsedArgs = { ok: true; args: string[] } | { ok: false; error: "unterminated" };

export function parseArgs(input: string): ParsedArgs {
  const args: string[] = [];
  let current = "";
  let started = false;
  let quote: '"' | "'" | null = null;
  for (let i = 0; i < input.length; i++) {
    const ch = input[i];
    if (quote) {
      if (quote === '"' && ch === "\\" && input[i + 1] === '"') {
        current += '"';
        i++;
      } else if (ch === quote) {
        quote = null;
      } else {
        current += ch;
      }
    } else if (ch === '"' || ch === "'") {
      quote = ch;
      started = true;
    } else if (/\s/.test(ch)) {
      if (started) args.push(current);
      current = "";
      started = false;
    } else {
      current += ch;
      started = true;
    }
  }
  if (quote) return { ok: false, error: "unterminated" };
  if (started) args.push(current);
  return { ok: true, args };
}

/** Inverse of `parseArgs` for display and editing. */
export function formatArgs(args: string[]): string {
  return args.map((a) => (a === "" || /[\s"']/.test(a) ? `"${a.replace(/"/g, '\\"')}"` : a)).join(" ");
}
