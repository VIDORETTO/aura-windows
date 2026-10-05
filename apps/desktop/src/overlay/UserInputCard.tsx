import { MessageCircleQuestion } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { useConversation } from "../state/conversation";
import { Button, Field, Select, Switch, TextField, cx } from "../ui/primitives";
import { formAnswer, parseInput, questionsAnswer } from "./userInput";
import type { ApprovalDecision } from "../ipc/types";
import type { MessageKey } from "../i18n";

/** Friendly names of Aura's own write tools (017). */
const AURA_TOOLS: Record<string, MessageKey> = {
  skill_save: "toolApproval.skill_save",
  quick_command_save: "toolApproval.quick_command_save",
  mcp_server_save: "toolApproval.mcp_server_save",
};

export function UserInputCard({ requestId, source, prompt, autoResolveMs, resolved }: { requestId: string; source: string; prompt: unknown; autoResolveMs: number | null; resolved?: string }) {
  const t = useT();
  const mark = useConversation((s) => s.markApproval);
  const form = useMemo(() => parseInput(source, prompt), [source, prompt]);
  const [values, setValues] = useState<Record<string, string | boolean>>({});
  const [missing, setMissing] = useState<string[]>([]);
  const [left, setLeft] = useState(autoResolveMs ? Math.ceil(autoResolveMs / 1000) : null);

  useEffect(() => {
    if (left === null || resolved) return;
    if (left <= 0) return;
    const h = setTimeout(() => setLeft((l) => (l === null ? null : l - 1)), 1000);
    return () => clearTimeout(h);
  }, [left, resolved]);

  const send = async (decision: ApprovalDecision, how: string) => {
    try {
      mark(requestId, how);
      await api.conversationRespond(requestId, decision);
    } catch (e) {
      useApp.getState().notify("error", errorMessage(e));
    }
  };

  const title =
    form.kind === "tool"
      ? form.approval.server === "aura" && AURA_TOOLS[form.approval.tool]
        ? t(AURA_TOOLS[form.approval.tool])
        : t("toolApproval.title", { tool: form.approval.tool || "?", server: form.approval.server })
      : source === "permissions"
        ? t("input.permissionsTitle")
        : source === "agent"
          ? t("input.questionTitle")
          : t("input.formTitle", { source });

  return (
    <div role="group" aria-label={title} className={cx("rounded-md border px-3 py-2", resolved ? "border-line opacity-70" : "border-accent/50 bg-accent/5")}>
      <div className="flex items-center gap-2 text-sm font-medium">
        <MessageCircleQuestion size={14} className="text-accent" /> {title}
        {left !== null && !resolved && left > 0 && <span className="ml-auto text-[11px] text-muted">{t("input.autoResolve", { s: left })}</span>}
      </div>
      {resolved ? (
        <div className="mt-1 text-xs text-muted">{t(resolved === "accepted" ? "approval.accepted" : resolved === "declined" ? "approval.declined" : "approval.resolved")}</div>
      ) : form.kind === "tool" ? (
        <div className="mt-2 flex flex-col gap-2">
          {form.approval.params.length > 0 && (
            <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-[12px]">
              {form.approval.params.map((p) => (
                <div key={p.name} className="contents">
                  <dt className="text-muted">{p.name}</dt>
                  <dd className="selectable max-h-32 overflow-auto whitespace-pre-wrap break-words font-mono text-[11px]">{p.value}</dd>
                </div>
              ))}
            </dl>
          )}
          <div className="flex gap-1.5">
            <Button size="sm" variant="primary" onClick={() => void send({ type: "answer", content: {} }, "accepted")}>{t("toolApproval.allow")}</Button>
            <Button size="sm" variant="ghost" onClick={() => void send({ type: "decline" }, "declined")}>{t("approval.decline")}</Button>
          </div>
        </div>
      ) : form.kind === "questions" ? (
        <div className="mt-2 flex flex-col gap-3">
          {form.questions.map((q) => (
            <fieldset key={q.id} className="flex flex-col gap-1">
              <legend className="text-[13px] font-medium">{q.header ? `${q.header}: ` : ""}{q.question}</legend>
              {q.options.map((o) => (
                <label key={o.label} className="flex items-start gap-2 text-[13px]">
                  <input type="radio" name={`${requestId}-${q.id}`} checked={values[q.id] === o.label} onChange={() => setValues((v) => ({ ...v, [q.id]: o.label }))} />
                  <span>{o.label}{o.description && <span className="text-muted"> — {o.description}</span>}</span>
                </label>
              ))}
              {(q.isOther || q.options.length === 0) && (
                <TextField aria-label={q.question} placeholder={t("input.other")} value={typeof values[q.id] === "string" && !q.options.some((o) => o.label === values[q.id]) ? (values[q.id] as string) : ""} onChange={(e) => setValues((v) => ({ ...v, [q.id]: e.target.value }))} />
              )}
            </fieldset>
          ))}
          <div className="flex gap-1.5">
            <Button size="sm" variant="primary" onClick={() => void send(questionsAnswer(values as Record<string, string>), "answered")}>{t("input.answer")}</Button>
            <Button size="sm" variant="ghost" onClick={() => void send({ type: "cancel" }, "cancelled")}>{t("common.cancel")}</Button>
          </div>
        </div>
      ) : form.kind === "form" ? (
        <div className="mt-2 flex flex-col gap-2">
          {form.message && <p className="text-[13px]">{form.message}</p>}
          {form.fields.map((f) =>
            f.kind === "boolean" ? (
              <label key={f.key} className="flex items-center gap-2 text-[13px]">
                <Switch label={f.title} checked={values[f.key] === true} onChange={(v) => setValues((x) => ({ ...x, [f.key]: v }))} /> {f.title}
              </label>
            ) : (
              <Field key={f.key} label={`${f.title}${f.required ? " *" : ""}`} hint={f.description}>
                {f.kind === "enum" ? (
                  <Select value={(values[f.key] as string) ?? ""} onChange={(e) => setValues((x) => ({ ...x, [f.key]: e.target.value }))}>
                    <option value="">—</option>
                    {f.options!.map((o) => <option key={o} value={o}>{o}</option>)}
                  </Select>
                ) : (
                  <TextField inputMode={f.kind === "string" ? undefined : "decimal"} aria-invalid={missing.includes(f.key)} value={(values[f.key] as string) ?? ""} onChange={(e) => setValues((x) => ({ ...x, [f.key]: e.target.value }))} />
                )}
              </Field>
            ),
          )}
          <div className="flex gap-1.5">
            <Button
              size="sm"
              variant="primary"
              onClick={() => {
                const { decision, missing } = formAnswer(form.fields, values);
                setMissing(missing);
                if (missing.length === 0) void send(decision, "answered");
              }}
            >
              {t("input.answer")}
            </Button>
            <Button size="sm" variant="ghost" onClick={() => void send({ type: "decline" }, "declined")}>{t("approval.decline")}</Button>
          </div>
          {missing.length > 0 && <p className="text-xs text-danger">{t("input.missing", { fields: missing.join(", ") })}</p>}
        </div>
      ) : form.kind === "permissions" ? (
        <div className="mt-2 flex flex-col gap-2">
          {form.reason && <p className="text-[13px]">{form.reason}</p>}
          <pre className="selectable overflow-x-auto rounded bg-hover px-2 py-1 font-mono text-[11px]">{JSON.stringify(form.permissions, null, 2)}</pre>
          <div className="flex gap-1.5">
            <Button size="sm" variant="primary" onClick={() => void send({ type: "grant", permissions: form.permissions, session: false }, "granted")}>{t("approval.accept")}</Button>
            <Button size="sm" onClick={() => void send({ type: "grant", permissions: form.permissions, session: true }, "granted")}>{t("approval.acceptSession")}</Button>
            <Button size="sm" variant="ghost" onClick={() => void send({ type: "decline" }, "declined")}>{t("approval.decline")}</Button>
          </div>
        </div>
      ) : (
        <div className="mt-2 flex flex-col gap-2">
          <pre className="selectable max-h-40 overflow-auto rounded bg-hover px-2 py-1 font-mono text-[11px]">{JSON.stringify(form.raw, null, 2)}</pre>
          <Button size="sm" variant="ghost" onClick={() => void send({ type: "cancel" }, "cancelled")}>{t("common.cancel")}</Button>
        </div>
      )}
    </div>
  );
}
