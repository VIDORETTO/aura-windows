import { ExternalLink } from "lucide-react";
import { api, errorMessage } from "../ipc/commands";
import type { WebSource } from "../ipc/types";
import { useT } from "../i18n";
import { webSourceUrl } from "../lib/webSources";
import { useApp } from "../state/app";

export function WebSources({ sources }: { sources: readonly WebSource[] }) {
  const t = useT();
  const valid = sources.filter((source) => webSourceUrl(source) !== null);
  if (!valid.length) return null;
  const open = (source: WebSource) => {
    const url = webSourceUrl(source);
    if (url) void api.openExternal(url).catch((error) => useApp.getState().notify("error", errorMessage(error)));
  };
  return (
    <div role="group" aria-label={t("web.sources")} className="mt-2 rounded-md border border-line px-3 py-2 text-[12px]">
      <div className="mb-1 font-medium text-muted">{t("web.sources")}</div>
      <ol className="space-y-2">
        {valid.map((source) => (
          <li key={source.sourceId} className="min-w-0">
            <button type="button" aria-label={t("web.openSource", { id: source.sourceId, title: source.title })}
              className="flex w-full items-start gap-1.5 rounded text-left text-fg hover:text-accent focus-visible:outline focus-visible:outline-accent"
              onClick={() => open(source)}>
              <span className="shrink-0 text-muted">{source.sourceId}</span>
              <span className="min-w-0 break-words">{source.title}</span>
              <ExternalLink size={12} className="mt-0.5 shrink-0 text-muted" aria-hidden />
            </button>
            <div className="mt-0.5 flex flex-wrap gap-x-2 text-muted">
              <span>{new URL(source.url).hostname}</span>
              <span>{t(source.kind === "pageContent" ? "web.pageRead" : "web.searchSnippet")}</span>
            </div>
            <div className="break-all text-muted">{source.url}</div>
          </li>
        ))}
      </ol>
    </div>
  );
}
