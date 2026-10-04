export function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${n >= 100 || i === 0 ? Math.round(n) : n.toFixed(1)} ${units[i]}`;
}

export function percent(bytes: number, total: number | null): number {
  if (!total) return 0;
  return Math.max(0, Math.min(100, Math.round((bytes / total) * 100)));
}

export function relativeTime(unixSeconds: number, lang: "ptBr" | "en", now = Date.now()): string {
  const diff = Math.round(unixSeconds - now / 1000);
  const rtf = new Intl.RelativeTimeFormat(lang === "en" ? "en" : "pt-BR", { numeric: "auto" });
  const abs = Math.abs(diff);
  if (abs < 60) return rtf.format(diff, "second");
  if (abs < 3600) return rtf.format(Math.round(diff / 60), "minute");
  if (abs < 86400) return rtf.format(Math.round(diff / 3600), "hour");
  return rtf.format(Math.round(diff / 86400), "day");
}

/** Exact local date and time, with seconds (access log). */
export function exactTime(unixSeconds: number, lang: "ptBr" | "en", timeZone?: string): string {
  return new Date(unixSeconds * 1000).toLocaleString(lang === "en" ? "en" : "pt-BR", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    timeZone,
  });
}

/** Text color over an accent `#rrggbb`: white or near-black, whichever has
 * the higher WCAG contrast (012). */
export function accentContrast(hex: string): string {
  const channel = (i: number) => {
    const v = parseInt(hex.slice(i, i + 2), 16) / 255;
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  const lum = 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5);
  const dark = 0.0113; // relative luminance of #1b1b1f
  const onWhite = 1.05 / (lum + 0.05);
  const onDark = (lum + 0.05) / (dark + 0.05);
  return onWhite >= onDark ? "#ffffff" : "#1b1b1f";
}

/** Recording length as `m:ss` or `h:mm:ss`. */
export function clockDuration(ms: number): string {
  const total = Math.round(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = String(total % 60).padStart(2, "0");
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${m}:${s}`;
}

export function durationMs(ms: number): string {
  if (ms < 1000) return `${ms} ms`;
  return `${(ms / 1000).toFixed(ms < 10_000 ? 1 : 0)} s`;
}

/** Shortcut chord from a keyboard event, in the host's canonical form. */
export function chordFromEvent(e: KeyboardEvent | React.KeyboardEvent): string | null {
  const mods: string[] = [];
  if (e.ctrlKey) mods.push("Ctrl");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  if (e.metaKey) mods.push("Win");
  const code = e.code;
  let key: string | null = null;
  if (/^Key[A-Z]$/.test(code)) key = code.slice(3);
  else if (/^Digit\d$/.test(code)) key = code.slice(5);
  else if (/^F\d{1,2}$/.test(code)) key = code;
  else {
    const named: Record<string, string> = {
      Space: "Space", Enter: "Enter", Tab: "Tab", Escape: "Escape", Backspace: "Backspace", Delete: "Delete",
      Insert: "Insert", Home: "Home", End: "End", PageUp: "PageUp", PageDown: "PageDown", ArrowUp: "Up",
      ArrowDown: "Down", ArrowLeft: "Left", ArrowRight: "Right", Backquote: "Backquote", Minus: "Minus",
      Equal: "Equal", Comma: "Comma", Period: "Period", Slash: "Slash", Semicolon: "Semicolon", Quote: "Quote",
      BracketLeft: "BracketLeft", BracketRight: "BracketRight", Backslash: "Backslash",
    };
    key = named[code] ?? null;
  }
  if (!key || mods.length === 0) return null;
  return [...mods, key].join("+");
}
