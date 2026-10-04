// "Arquivos" and "Alterações" (008 TK-005): files the agent generated in the
// conversation workspace, with safe previews, and the turn diff.

import { ExternalLink, FileText, FolderOpen, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { api } from "../ipc/commands";
import type { PdfPreview, WorkspaceFile } from "../ipc/types";
import { useT } from "../i18n";
import { formatBytes } from "../lib/format";
import { renderMarkdown } from "../lib/markdown";
import { useConversation } from "../state/conversation";
import { IconButton, cx } from "../ui/primitives";
import { previewSrc } from "./ChipList";

const IMAGE = /\.(png|jpe?g|gif|webp|bmp|svg)$/i;
const TEXT = /\.(md|markdown|txt|csv|json|log|ya?ml|toml|py|rs|ts|tsx|js|css|sql|sh|ps1)$/i;

/** HTML preview document: no scripts, no network, no same-origin (OT-003). */
export function sandboxedHtml(html: string): string {
  const csp = "default-src 'none'; img-src data:; style-src 'unsafe-inline'";
  return `<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="${csp}"><meta charset="utf-8"></head><body>${html}</body></html>`;
}

/** Splits a unified diff into typed lines for coloring. */
export function diffLines(diff: string): { kind: "add" | "del" | "hunk" | "file" | "ctx"; text: string }[] {
  return diff.split("\n").map((text) => ({
    kind: text.startsWith("+++") || text.startsWith("---") || text.startsWith("diff ")
      ? "file"
      : text.startsWith("@@")
        ? "hunk"
        : text.startsWith("+")
          ? "add"
          : text.startsWith("-")
            ? "del"
            : "ctx",
    text,
  }));
}

/** Text of the first pages (008 AC-014); the layout opens in the PDF viewer. */
function PdfText({ threadId, file }: { threadId: string; file: WorkspaceFile }) {
  const t = useT();
  const [preview, setPreview] = useState<PdfPreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    setPreview(null);
    setError(null);
    api.workspacePdfPreview(threadId, file.path).then(setPreview, (e) => setError(String(e?.message ?? e)));
  }, [threadId, file.path]);
  if (error) return <p className="text-xs text-muted">{error}</p>;
  if (!preview) return null;
  if (preview.pages.length === 0) return <p className="text-xs text-muted">{t("work.pdf.noText")}</p>;
  return (
    <section aria-label={t("work.preview", { name: file.path })} className="max-h-72 overflow-auto rounded border border-line p-2 text-[12px]">
      <p className="mb-1.5 text-[11px] text-muted">{t("work.pdf.summary", { shown: preview.pages.length, total: preview.totalPages })}</p>
      {preview.pages.map((p) => (
        <div key={p.number} className="mb-2">
          <h3 className="text-[11px] font-medium text-muted">{t("work.pdf.page", { n: p.number })}</h3>
          <p className="selectable whitespace-pre-wrap">{p.text.trim()}</p>
        </div>
      ))}
    </section>
  );
}

function Preview({ threadId, file }: { threadId: string; file: WorkspaceFile }) {
  const t = useT();
  const [text, setText] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const isHtml = /\.html?$/i.test(file.path);
  useEffect(() => {
    setText(null);
    setError(null);
    if (IMAGE.test(file.path)) return;
    if (TEXT.test(file.path) || isHtml) {
      api.workspaceRead(threadId, file.path).then(setText, (e) => setError(String(e?.message ?? e)));
    }
  }, [threadId, file.path, isHtml]);
  if (IMAGE.test(file.path)) return <img src={previewSrc(file.absolute)} alt={file.path} className="max-h-72 rounded border border-line" />;
  if (/\.pdf$/i.test(file.path)) return <PdfText threadId={threadId} file={file} />;
  if (!TEXT.test(file.path) && !isHtml) return <p className="text-xs text-muted">{t("work.noPreview")}</p>;
  if (error) return <p className="text-xs text-muted">{error}</p>;
  if (text === null) return null;
  if (isHtml) return <iframe title={file.path} sandbox="" srcDoc={sandboxedHtml(text)} className="h-72 w-full rounded border border-line bg-white" />;
  if (/\.(md|markdown)$/i.test(file.path)) return <div className="md text-[13px]" dangerouslySetInnerHTML={{ __html: renderMarkdown(text) }} />;
  return <pre className="selectable max-h-72 overflow-auto rounded bg-hover p-2 font-mono text-[11px]">{text}</pre>;
}

export function WorkPanel({ threadId, onClose }: { threadId: string; onClose: () => void }) {
  const t = useT();
  const diff = useConversation((s) => s.threads[threadId]?.diff ?? null);
  const running = useConversation((s) => s.threads[threadId]?.running ?? false);
  const [tab, setTab] = useState<"files" | "changes">("files");
  const [files, setFiles] = useState<WorkspaceFile[]>([]);
  const [selected, setSelected] = useState<WorkspaceFile | null>(null);
  useEffect(() => {
    void api.workspaceFiles(threadId).then(setFiles).catch(() => setFiles([]));
  }, [threadId, running]);
  const lines = useMemo(() => (diff ? diffLines(diff) : []), [diff]);

  return (
    <aside className="absolute inset-y-0 right-0 z-10 flex w-80 shrink-0 flex-col border-l border-line bg-[var(--surface-menu)] backdrop-blur-xl min-[1024px]:static min-[1024px]:z-auto" aria-label={t("work.title")}>
      <div className="flex items-center gap-1 border-b border-line px-2 py-1">
        {(["files", "changes"] as const).map((k) => (
          <button key={k} type="button" aria-pressed={tab === k} onClick={() => setTab(k)} className={cx("rounded px-2 py-1 text-[12px]", tab === k ? "bg-hover font-medium" : "text-muted hover:bg-hover")}>
            {t(k === "files" ? "work.files" : "work.changes")}
          </button>
        ))}
        <IconButton label={t("common.close")} className="ml-auto h-7 w-7" onClick={onClose}>
          <X size={14} />
        </IconButton>
      </div>
      <div className="flex-1 overflow-y-auto p-2">
        {tab === "files" ? (
          files.length === 0 ? (
            <p className="p-3 text-center text-[13px] text-muted">{t("work.noFiles")}</p>
          ) : (
            <ul className="flex flex-col gap-0.5">
              {files.map((f) => (
                <li key={f.path}>
                  <div className={cx("group flex items-center gap-1.5 rounded px-1.5 py-1 text-[12px]", selected?.path === f.path && "bg-hover")}>
                    <FileText size={13} className="shrink-0 text-muted" />
                    <button type="button" className="min-w-0 flex-1 truncate text-left" onClick={() => setSelected(f)}>
                      {f.path}
                    </button>
                    <span className="shrink-0 text-[10px] text-muted">{formatBytes(f.bytes)}</span>
                    <IconButton label={t("work.open")} className="h-6 w-6" onClick={() => void api.openPath(f.absolute)}>
                      <ExternalLink size={12} />
                    </IconButton>
                    <IconButton label={t("work.reveal")} className="h-6 w-6" onClick={() => void api.openPath(f.absolute, true)}>
                      <FolderOpen size={12} />
                    </IconButton>
                  </div>
                  {selected?.path === f.path && (
                    <div className="px-1.5 pb-2 pt-1">
                      <Preview threadId={threadId} file={f} />
                    </div>
                  )}
                </li>
              ))}
            </ul>
          )
        ) : lines.length === 0 ? (
          <p className="p-3 text-center text-[13px] text-muted">{t("work.noChanges")}</p>
        ) : (
          <pre className="selectable overflow-x-auto font-mono text-[11px] leading-4">
            {lines.map((l, i) => (
              <div key={i} className={cx(l.kind === "add" && "bg-success/10 text-success", l.kind === "del" && "bg-danger/10 text-danger", l.kind === "hunk" && "text-accent", l.kind === "file" && "font-semibold")}>
                {l.text || " "}
              </div>
            ))}
          </pre>
        )}
      </div>
    </aside>
  );
}
