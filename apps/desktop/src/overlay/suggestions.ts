// Suggestions on the empty Overlay, by the app in front (028): the user finds
// the functions without searching menus. Pure so it is tested without a UI.

export type SuggestionAction =
  | { kind: "meeting" }
  /** Sends a slash command or a prompt; `screen` attaches the screen first. */
  | { kind: "send"; text: string; screen?: boolean };

export interface Suggestion {
  id: string;
  /** i18n key of the label. */
  label: string;
  action: SuggestionAction;
}

const has = (s: string, ...needles: string[]) => needles.some((n) => s.includes(n));

/** Up to four suggestions for `processName` and window `title`. */
export function suggestionsFor(processName: string | null | undefined, title: string | null | undefined): Suggestion[] {
  const p = (processName ?? "").toLowerCase();
  const ti = (title ?? "").toLowerCase();
  const meeting: Suggestion = { id: "meeting", label: "suggest.meeting", action: { kind: "meeting" } };
  const summarize: Suggestion = { id: "summarize", label: "suggest.summarizeScreen", action: { kind: "send", text: "/resumir-tela" } };
  const help: Suggestion = { id: "help", label: "suggest.helpNow", action: { kind: "send", text: "/ajuda", screen: true } };
  const reply: Suggestion = { id: "reply", label: "suggest.reply", action: { kind: "send", text: "/responder", screen: false } };

  const isMeetingApp = has(p, "teams", "zoom", "webex", "slack", "discord") || has(ti, "google meet", "meet -", "zoom meeting", "microsoft teams");
  if (isMeetingApp) return [meeting, help, summarize];

  if (has(p, "code", "devenv", "cursor", "idea", "pycharm", "rider", "studio")) {
    return [
      { id: "error", label: "suggest.explainError", action: { kind: "send", text: "Explique o erro que está na tela e como corrigir, passo a passo.", screen: true } },
      help,
      summarize,
    ];
  }
  if (has(p, "outlook", "olk", "thunderbird", "mail", "whatsapp", "telegram")) {
    return [reply, { id: "scam", label: "suggest.scam", action: { kind: "send", text: "/golpe", screen: true } }, summarize];
  }
  if (has(p, "chrome", "msedge", "firefox", "brave", "opera", "vivaldi")) {
    return [summarize, { id: "scam", label: "suggest.scam", action: { kind: "send", text: "/golpe", screen: true } }, { id: "doc", label: "suggest.document", action: { kind: "send", text: "/documento", screen: true } }, help];
  }
  if (has(p, "winword", "excel", "powerpnt", "acrobat", "onenote", "notepad")) {
    return [summarize, { id: "doc", label: "suggest.document", action: { kind: "send", text: "/documento", screen: true } }, help];
  }
  return [summarize, help, meeting];
}
