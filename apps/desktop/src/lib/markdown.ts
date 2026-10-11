// Markdown → sanitized HTML. Runs on every streaming frame, so it must be
// fast: `marked` is synchronous and incremental-friendly; highlighting is
// lazy (loaded on the first code block) and skipped while streaming.

import DOMPurify from "dompurify";
import { Marked } from "marked";
import type { WebSource } from "../ipc/types";
import { webSourceUrl } from "./webSources";

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

/** `[12:31]` → ms since the meeting started (citations of the agent, 025). */
export function citationMs(mm: string, ss: string): number {
  return (Number(mm) * 60 + Number(ss)) * 1000;
}

/** Turns `[mm:ss]` outside code blocks into clickable citation buttons. */
export function linkCitations(html: string): string {
  return html
    .split(/(<pre[\s\S]*?<\/pre>|<code[\s\S]*?<\/code>)/)
    .map((part, i) =>
      i % 2 === 1
        ? part
        : part.replace(/\[(\d{1,3}):([0-5]\d)\]/g, (_m, mm: string, ss: string) => `<button class="cite" data-cite="${citationMs(mm, ss)}">⏱ ${mm.padStart(2, "0")}:${ss}</button>`),
    )
    .join("");
}

interface MarkdownOptions {
  streaming?: boolean;
  sources?: readonly WebSource[];
  sourceLabel?: (source: WebSource) => string;
  unverifiedLabel?: (id: string) => string;
}

function linkWebSources(html: string, opts: MarkdownOptions): string {
  if (!html.includes("aura-source:")) return html;
  const template = document.createElement("template");
  template.innerHTML = html;
  const walker = document.createTreeWalker(template.content, NodeFilter.SHOW_TEXT);
  const nodes: Text[] = [];
  while (walker.nextNode()) nodes.push(walker.currentNode as Text);
  for (const node of nodes) {
    if (node.parentElement?.closest("pre,code,a,button,kbd,samp")) continue;
    const text = node.data;
    const matches = [...text.matchAll(/\[\[aura-source:(W[1-9]\d*)\]\]/g)];
    if (!matches.length) continue;
    const fragment = document.createDocumentFragment();
    let start = 0;
    for (const match of matches) {
      fragment.append(document.createTextNode(text.slice(start, match.index)));
      const id = match[1];
      const source = opts.sources?.find((entry) => entry.sourceId === id && webSourceUrl(entry));
      const element = document.createElement(source ? "button" : "span");
      if (source) {
        element.className = "cite";
        element.setAttribute("type", "button");
        element.setAttribute("data-web-source", id);
        element.setAttribute("aria-label", opts.sourceLabel?.(source) ?? id);
        element.textContent = id;
      } else {
        element.className = "text-muted";
        element.textContent = opts.unverifiedLabel?.(id) ?? `Unverified reference: ${id}`;
      }
      fragment.append(element);
      start = match.index + match[0].length;
    }
    fragment.append(document.createTextNode(text.slice(start)));
    node.replaceWith(fragment);
  }
  return template.innerHTML;
}

export function renderMarkdown(md: string, opts: MarkdownOptions = {}): string {
  const source = opts.streaming ? closeOpenFence(md) : md;
  const html = (opts.streaming || !hljs ? plain : highlighted).parse(source, { async: false }) as string;
  // Model HTML cannot supply interactive citation controls. Aura adds its own
  // buttons after sanitization, using trusted provenance or parsed timestamps.
  const safe = DOMPurify.sanitize(html, {
    ALLOWED_ATTR: ["class", "data-href", "data-lang", "data-cite", "title", "colspan", "rowspan", "align"],
    FORBID_TAGS: ["style", "iframe", "form", "input", "button", "img", "video", "audio", "object", "embed"],
  });
  return linkWebSources(linkCitations(safe), opts);
}

export function hasCode(md: string): boolean {
  return /^(```|~~~)/m.test(md);
}
