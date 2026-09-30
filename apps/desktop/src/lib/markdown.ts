// Markdown → sanitized HTML. Runs on every streaming frame, so it must be
// fast: `marked` is synchronous and incremental-friendly; highlighting is
// lazy (loaded on the first code block) and skipped while streaming.

import DOMPurify from "dompurify";
import { Marked } from "marked";

type Hljs = typeof import("highlight.js/lib/common").default;
let hljs: Hljs | null = null;
let loading: Promise<void> | null = null;

export function loadHighlighter(): Promise<void> {
  if (hljs) return Promise.resolve();
  loading ??= import("highlight.js/lib/common").then((m) => {
    hljs = m.default;
  });
  return loading;
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);
}

function makeMarked(highlight: boolean) {
  return new Marked({
    gfm: true,
    breaks: false,
    renderer: {
      code({ text, lang }) {
        const language = (lang ?? "").split(/\s/)[0];
        let body = escapeHtml(text);
        if (highlight && hljs) {
          try {
            body = language && hljs.getLanguage(language) ? hljs.highlight(text, { language }).value : hljs.highlightAuto(text).value;
          } catch {
            /* keep escaped text */
          }
        }
        const cls = language ? ` class="language-${escapeHtml(language)}"` : "";
        return `<pre data-lang="${escapeHtml(language)}"><code${cls}>${body}</code></pre>`;
      },
      link({ href, title, text }) {
        const t = title ? ` title="${escapeHtml(title)}"` : "";
        return `<a data-href="${escapeHtml(href)}"${t}>${text}</a>`;
      },
      image({ href, text }) {
        // Remote images never load (no network from the WebView, OT-003 of 008).
        return `<span class="text-muted">[imagem: ${escapeHtml(text || href)}]</span>`;
      },
    },
  });
}

const plain = makeMarked(false);
const highlighted = makeMarked(true);

/** Closes an unterminated code fence so partial answers render sanely. */
export function closeOpenFence(md: string): string {
  const fences = md.match(/^(```|~~~)/gm)?.length ?? 0;
  return fences % 2 === 1 ? `${md}\n\`\`\`` : md;
}

export function renderMarkdown(md: string, opts: { streaming?: boolean } = {}): string {
  const source = opts.streaming ? closeOpenFence(md) : md;
  const html = (opts.streaming || !hljs ? plain : highlighted).parse(source, { async: false }) as string;
  return DOMPurify.sanitize(html, {
    ALLOWED_ATTR: ["class", "data-href", "data-lang", "title", "colspan", "rowspan", "align"],
    FORBID_TAGS: ["style", "iframe", "form", "input", "img", "video", "audio", "object", "embed"],
  });
}

export function hasCode(md: string): boolean {
  return /^(```|~~~)/m.test(md);
}
