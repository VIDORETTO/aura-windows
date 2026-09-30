import { AlertTriangle, Check, ChevronRight, Copy, CornerDownLeft, Square as StopIcon, Volume2, FileDiff, ListChecks, Loader2, ShieldQuestion, Wrench, X } from "lucide-react";
import { memo, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { api } from "../ipc/commands";
import type { Approval, Block, Thread, ToolItem } from "../state/conversation";
import { useConversation } from "../state/conversation";
import type { ConsentRequest, TurnError } from "../ipc/types";
import { useT, type MessageKey } from "../i18n";
import { hasCode, loadHighlighter, renderMarkdown } from "../lib/markdown";
import { durationMs } from "../lib/format";
import { useApp } from "../state/app";
import { Button, cx } from "../ui/primitives";
import { ChipList } from "./ChipList";
import { useSession } from "./session";
import { UserInputCard } from "./UserInputCard";

const MANAGE_USAGE_URL = "https://chatgpt.com/settings/usage";

function Markdown({ text, streaming }: { text: string; streaming: boolean }) {
  const [hl, setHl] = useState(false);
  useEffect(() => {
    if (!streaming && hasCode(text)) void loadHighlighter().then(() => setHl(true));
  }, [streaming, text]);
  const html = useMemo(() => renderMarkdown(text, { streaming }), [text, streaming, hl]);
  const onClick = (e: React.MouseEvent) => {
    const a = (e.target as HTMLElement).closest("a[data-href]");
    if (a) {
      e.preventDefault();
      void api.openExternal(a.getAttribute("data-href")!);
    }
  };
  return <div className={cx("md", streaming && "caret")} onClick={onClick} dangerouslySetInnerHTML={{ __html: html }} />;
}

function CopyButton({ text }: { text: string }) {
  const t = useT();
  const [done, setDone] = useState(false);
  return (
    <button
      type="button"
      className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-muted hover:bg-hover hover:text-fg"
      onClick={() => {
        void navigator.clipboard?.writeText(text);
        setDone(true);
        setTimeout(() => setDone(false), 1500);
      }}
    >
      {done ? <Check size={12} /> : <Copy size={12} />} {done ? t("common.copied") : t("common.copy")}
    </button>
  );
}

/** One answer plays at a time. */
let playing: HTMLAudioElement | null = null;

function SpeakButton({ text }: { text: string }) {
  const t = useT();
  const [state, setState] = useState<"idle" | "loading" | "playing">("idle");
  const stop = () => {
    playing?.pause();
    playing = null;
    setState("idle");
  };
  const play = async () => {
    if (state !== "idle") return stop();
    setState("loading");
    try {
      const [b64, mime] = await api.speak(text);
      playing?.pause();
      const audio = new Audio(`data:${mime};base64,${b64}`);
      playing = audio;
      audio.onended = () => setState("idle");
      setState("playing");
      await audio.play().catch(() => setState("idle"));
    } catch (e) {
      setState("idle");
      useApp.getState().notify("warning", String((e as { message?: string })?.message ?? e));
    }
  };
  return (
    <button type="button" className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-muted hover:bg-hover hover:text-fg" onClick={() => void play()}>
      {state === "playing" ? <StopIcon size={12} /> : <Volume2 size={12} />} {state === "playing" ? t("speech.stop") : t("speech.listen")}
    </button>
  );
}

const Assistant = memo(function Assistant({ text, streaming }: { text: string; streaming: boolean }) {
  const t = useT();
  return (
    <div className="group">
      <Markdown text={text} streaming={streaming} />
      {!streaming && text && (
        <div className="mt-1 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
          <CopyButton text={text} />
          <SpeakButton text={text} />
          <button
            type="button"
            className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-muted hover:bg-hover hover:text-fg"
            onClick={() => void api.insertIntoApp(text)}
          >
            <CornerDownLeft size={12} /> {t("common.insert")}
          </button>
        </div>
      )}
    </div>
  );
});

function ToolCard({ item }: { item: ToolItem }) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const icon =
    item.status === "inProgress" ? <Loader2 size={13} className="animate-spin" /> : item.status === "completed" ? <Check size={13} className="text-success" /> : <X size={13} className="text-danger" />;
  const label = { inProgress: t("tool.running"), completed: t("tool.done"), failed: t("tool.failed"), declined: t("tool.declined") }[item.status];
  const took = item.finishedAt ? durationMs(item.finishedAt - item.startedAt) : null;
  return (
    <div className="text-[13px] text-muted">
      <button type="button" className="flex w-full items-center gap-1.5 rounded px-1 py-0.5 text-left hover:bg-hover" onClick={() => setOpen((o) => !o)} aria-expanded={open}>
        <ChevronRight size={12} className={cx("transition-transform", open && "rotate-90")} />
        <Wrench size={12} />
        <span className="truncate font-mono text-[12px] text-fg">{item.title}</span>
        {icon}
        <span className="ml-auto shrink-0 text-[11px]">{item.detail ?? took ?? label}</span>
      </button>
      {open && item.detail && <pre className="selectable ml-6 mt-1 whitespace-pre-wrap font-mono text-[11px]">{item.detail}</pre>}
    </div>
  );
}

function ApprovalCard({ approval, resolved }: { approval: Approval; resolved?: string }) {
  const t = useT();
  const mark = useConversation((s) => s.markApproval);
  const ref = useRef<HTMLDivElement>(null);
  const respond = async (type: "accept" | "acceptForSession" | "decline") => {
    mark(approval.requestId, type);
    await api.conversationRespond(approval.requestId, { type });
  };
  useEffect(() => {
    if (!resolved) ref.current?.focus();
  }, [resolved]);
  const title = { command: t("approval.command"), fileChange: t("approval.fileChange"), permissions: t("approval.permissions") }[approval.kind];
  return (
    <div
      ref={ref}
      tabIndex={-1}
      role="group"
      aria-label={title}
      onKeyDown={(e) => {
        if (resolved) return;
        if (e.key.toLowerCase() === "a") void respond(e.shiftKey ? "acceptForSession" : "accept");
        if (e.key.toLowerCase() === "r") void respond("decline");
      }}
      className={cx("rounded-md border px-3 py-2 outline-none", resolved ? "border-line opacity-70" : "border-warning/50 bg-warning/5 focus:border-warning")}
    >
      <div className="flex items-center gap-2 text-sm font-medium">
        <AlertTriangle size={14} className="text-warning" /> {title}
      </div>
      {approval.command && <pre className="selectable mt-1.5 overflow-x-auto rounded bg-hover px-2 py-1 font-mono text-[12px]">{approval.command}</pre>}
      {approval.cwd && <div className="mt-1 text-xs text-muted">{t("approval.in", { cwd: approval.cwd })}</div>}
      {approval.reason && <div className="mt-1 text-[13px]">{approval.reason}</div>}
      {approval.changes.length > 0 && (
        <ul className="mt-1 text-xs">
          {approval.changes.map((c) => (
            <li key={c.path} className="font-mono">
              {c.path} <span className="text-success">+{c.added}</span> <span className="text-danger">−{c.removed}</span>
            </li>
          ))}
        </ul>
      )}
      {resolved ? (
        <div className="mt-2 text-xs text-muted">{t("approval.resolved")}</div>
      ) : (
        <div className="mt-2 flex flex-wrap gap-1.5">
          <Button size="sm" variant="primary" onClick={() => void respond("accept")}>
            {t("approval.accept")} <kbd className="opacity-70">A</kbd>
          </Button>
          {approval.options.includes("acceptForSession") && (
            <Button size="sm" onClick={() => void respond("acceptForSession")}>
              {t("approval.acceptSession")}
            </Button>
          )}
          <Button size="sm" variant="ghost" onClick={() => void respond("decline")}>
            {t("approval.decline")} <kbd className="opacity-70">R</kbd>
          </Button>
        </div>
      )}
    </div>
  );
}

export function ConsentCard({ request }: { request: ConsentRequest }) {
  const t = useT();
  const answer = (a: "once" | "conversation" | "deny") => {
    useApp.setState((s) => ({ consents: s.consents.filter((c) => c.id !== request.id) }));
    void api.consentAnswer(request.id, a);
  };
  const source = t(`consent.${request.source}` as MessageKey);
  return (
    <div role="alertdialog" aria-label={t("consent.title", { source })} className="fade-in rounded-md border border-accent/50 bg-accent/5 px-3 py-2">
      <div className="flex items-center gap-2 text-sm font-medium">
        <ShieldQuestion size={15} className="text-accent" /> {t("consent.title", { source })}
      </div>
      <div className="mt-0.5 text-[13px] text-muted">
        {request.reason}
        {request.app ? ` · ${request.app}` : ""}
      </div>
      <div className="mt-2 flex flex-wrap gap-1.5">
        <Button size="sm" variant="primary" autoFocus onClick={() => answer("once")}>
          {t("consent.once")}
        </Button>
        <Button size="sm" onClick={() => answer("conversation")}>
          {t("consent.conversation")}
        </Button>
        <Button size="sm" variant="ghost" onClick={() => answer("deny")}>
          {t("consent.deny")}
        </Button>
      </div>
    </div>
  );
}

function ErrorCard({ error }: { error: TurnError }) {
  const t = useT();
  const newConversation = useSession((s) => s.newConversation);
  const text =
    error.kind === "usageLimit" && error.retryAfterSecs
      ? t("error.usageLimitRetry", { s: error.retryAfterSecs })
      : error.kind === "other"
        ? t("error.other", { message: error.message })
        : t(`error.${error.kind}` as MessageKey);
  return (
    <div role="alert" className="rounded-md border border-danger/40 px-3 py-2 text-[13px]">
      <div className="flex items-center gap-2">
        <AlertTriangle size={14} className="text-danger" /> {text}
      </div>
      <div className="mt-2 flex flex-wrap gap-1.5">
        {error.kind === "planUsageLimit" && (
          <>
            <Button size="sm" variant="primary" onClick={() => void api.openExternal(MANAGE_USAGE_URL)}>
              {t("header.manageUsage")}
            </Button>
            <Button size="sm" onClick={() => void api.settingsOpen("providers")}>
              {t("error.useOtherProvider")}
            </Button>
          </>
        )}
        {error.kind === "sessionExpired" && (
          <Button size="sm" variant="primary" onClick={() => void api.authLogin(useApp.getState().auth?.active?.clientId ?? null)}>
            {t("error.signInAgain")}
          </Button>
        )}
        {error.kind === "contextTooLong" && (
          <Button size="sm" onClick={() => void newConversation()}>
            {t("header.newConversation")}
          </Button>
        )}
      </div>
    </div>
  );
}

function PlanPanel({ plan }: { plan: NonNullable<Thread["plan"]> }) {
  const t = useT();
  return (
    <div className="rounded-md border border-line px-3 py-2 text-[13px]">
      <div className="mb-1 flex items-center gap-1.5 font-medium">
        <ListChecks size={14} /> {t("plan.title")}
      </div>
      {plan.explanation && <p className="mb-1 text-muted">{plan.explanation}</p>}
      <ol className="space-y-0.5">
        {plan.steps.map((s, i) => (
          <li key={i} className="flex items-start gap-2">
            <span className={cx("mt-1.5 h-2 w-2 shrink-0 rounded-full", s.status === "completed" ? "bg-success" : s.status === "inProgress" ? "animate-pulse bg-accent" : "bg-line")} />
            <span className={cx(s.status === "completed" && "text-muted line-through")}>{s.step}</span>
          </li>
        ))}
      </ol>
    </div>
  );
}

function BlockView({ block }: { block: Block }) {
  switch (block.type) {
    case "user":
      return (
        <div className="flex flex-col items-end gap-1">
          {block.chips.length > 0 && <ChipList chips={block.chips} />}
          {(block.display ?? block.text) && (
            <div className="selectable max-w-[85%] whitespace-pre-wrap rounded-lg rounded-br-sm bg-accent/12 px-3 py-1.5">{block.display ?? block.text}</div>
          )}
        </div>
      );
    case "assistant":
      return <Assistant text={block.text} streaming={block.streaming} />;
    case "reasoning":
      return <details className="text-[12px] text-muted"><summary className="cursor-pointer">…</summary><p className="selectable whitespace-pre-wrap">{block.text}</p></details>;
    case "tool":
      return <ToolCard item={block.item} />;
    case "files":
      return (
        <div className="flex items-center gap-1.5 text-[13px] text-muted">
          <FileDiff size={13} />
          {block.changes.map((c) => (
            <span key={c.path} className="font-mono text-[12px] text-fg">
              {c.path} <span className="text-success">+{c.added}</span> <span className="text-danger">−{c.removed}</span>
            </span>
          ))}
        </div>
      );
    case "approval":
      return <ApprovalCard approval={block.approval} resolved={block.resolved} />;
    case "plan":
      return <Assistant text={block.text} streaming={false} />;
    case "error":
      return <ErrorCard error={block.error} />;
    case "input":
      return <UserInputCard requestId={block.requestId} source={block.source} prompt={block.prompt} autoResolveMs={block.autoResolveMs} resolved={block.resolved} />;
  }
}

/** Scrolls with the stream unless the user scrolled up to read. */
export function MessageList({ thread }: { thread: Thread }) {
  const ref = useRef<HTMLDivElement>(null);
  const stick = useRef(true);
  const consents = useApp((s) => s.consents);
  useLayoutEffect(() => {
    const el = ref.current;
    if (el && stick.current) el.scrollTop = el.scrollHeight;
  });
  return (
    <div
      ref={ref}
      className="flex-1 space-y-3 overflow-y-auto px-4 py-3"
      aria-live="polite"
      aria-busy={thread.running}
      onScroll={(e) => {
        const el = e.currentTarget;
        stick.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
      }}
    >
      {thread.blocks.map((b) => (
        <BlockView key={b.type === "tool" ? b.item.id : b.type === "approval" ? b.approval.requestId : b.type === "input" ? b.requestId : b.id} block={b} />
      ))}
      {thread.plan && thread.plan.steps.length > 0 && <PlanPanel plan={thread.plan} />}
      {consents.map((c) => (
        <ConsentCard key={c.id} request={c} />
      ))}
      {thread.running && !thread.blocks.some((b) => b.type === "assistant" && b.streaming) && (
        <div className="flex items-center gap-2 text-[13px] text-muted">
          <Loader2 size={13} className="animate-spin" />
        </div>
      )}
    </div>
  );
}
