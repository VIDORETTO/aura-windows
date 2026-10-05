import { normalize, quickHint, templatePreview } from "../overlay/composer";
import { Pencil, Search, Server, Sparkles, Trash2, Wand2, X, Zap } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import { inTauri } from "../ipc/bridge";
import { formatArgs, parseArgs } from "../lib/args";
import type { DetectedServer, EnvValue, McpApprovalMode, McpServerSpec, McpStatus, QuickCommand, SkillEntry, SkillReview, McpDiagnosis } from "../ipc/types";
import { useT, type MessageKey } from "../i18n";
import { useApp } from "../state/app";
import { Badge, Button, Field, Section, Select, Switch, TextArea, TextField } from "../ui/primitives";

function useNotifyError() {
  const notify = useApp((s) => s.notify);
  return (e: unknown) => notify("error", errorMessage(e));
}

/** Every word of the query appears in one of the fields (no accents, any case). */
export function matchesQuery(query: string, ...fields: (string | null | undefined)[]): boolean {
  const words = normalize(query).split(/\s+/).filter(Boolean);
  if (words.length === 0) return true;
  const hay = normalize(fields.filter(Boolean).join(" "));
  return words.every((w) => hay.includes(w));
}

/** "x of y active" and the bulk switches, acting only on the listed items. */
function BulkBar({ section, items, onAll }: { section: string; items: { enabled: boolean }[]; onAll: (enabled: boolean) => Promise<void> }) {
  const t = useT();
  const [busy, setBusy] = useState(false);
  if (items.length === 0) return null;
  const active = items.filter((i) => i.enabled).length;
  const run = async (enabled: boolean) => {
    setBusy(true);
    try {
      await onAll(enabled);
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="flex flex-wrap items-center gap-2 border-b border-line pb-2 pt-1 text-xs text-muted">
      <span role="status" aria-label={t("extensions.bulk.countOf", { section })}>
        {t("extensions.bulk.count", { active, total: items.length })}
      </span>
      <span className="flex-1" />
      <Button size="sm" variant="ghost" disabled={busy || active === items.length} aria-label={t("extensions.bulk.enableOf", { section })} onClick={() => void run(true)}>
        {t("extensions.bulk.enable")}
      </Button>
      <Button size="sm" variant="ghost" disabled={busy || active === 0} aria-label={t("extensions.bulk.disableOf", { section })} onClick={() => void run(false)}>
        {t("extensions.bulk.disable")}
      </Button>
    </div>
  );
}

type AiKind = "skill" | "quick" | "mcp";

const AI_KEYS: Record<AiKind, { button: MessageKey; placeholder: MessageKey; prompt: MessageKey }> = {
  skill: { button: "extensions.ai.skill", placeholder: "extensions.ai.skill.placeholder", prompt: "extensions.ai.skill.prompt" },
  quick: { button: "extensions.ai.quick", placeholder: "extensions.ai.quick.placeholder", prompt: "extensions.ai.quick.prompt" },
  mcp: { button: "extensions.ai.mcp", placeholder: "extensions.ai.mcp.placeholder", prompt: "extensions.ai.mcp.prompt" },
};

/** "Create with AI" (017): the request goes to the agent in Task mode. */
function AiCreate({ kind, open, onClose }: { kind: AiKind; open: boolean; onClose: () => void }) {
  const t = useT();
  const fail = useNotifyError();
  const notify = useApp((s) => s.notify);
  const [text, setText] = useState("");
  const [sending, setSending] = useState(false);
  if (!open) return null;
  const keys = AI_KEYS[kind];
  const submit = async () => {
    if (!text.trim()) return;
    setSending(true);
    try {
      await api.agentTask(t(keys.prompt, { request: text.trim() }), "task");
      notify("info", t("extensions.ai.sent"));
      setText("");
      onClose();
    } catch (e) {
      fail(e);
    } finally {
      setSending(false);
    }
  };
  return (
    <div role="group" aria-label={t(keys.button)} className="my-2 flex flex-col gap-2 rounded-md border border-accent/40 bg-accent/5 p-3">
      <div className="flex items-center gap-2 text-[13px] font-medium">
        <Wand2 size={14} className="text-accent" /> {t(keys.button)}
        <span className="flex-1" />
        <button type="button" aria-label={t("common.cancel")} className="rounded p-0.5 text-muted hover:bg-hover hover:text-fg" onClick={onClose}>
          <X size={13} />
        </button>
      </div>
      <TextArea
        rows={3}
        autoFocus
        aria-label={t("extensions.ai.describe")}
        placeholder={t(keys.placeholder)}
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            void submit();
          }
        }}
      />
      <p className="text-[11px] text-muted">{t("extensions.ai.hint")}</p>
      <div className="flex justify-end gap-2">
        <Button variant="ghost" onClick={onClose}>{t("common.cancel")}</Button>
        <Button variant="primary" disabled={!text.trim() || sending} onClick={() => void submit()}>
          {t("extensions.ai.send")}
        </Button>
      </div>
    </div>
  );
}

function AiButton({ kind, onClick }: { kind: AiKind; onClick: () => void }) {
  const t = useT();
  return (
    <Button size="sm" onClick={onClick}>
      <Wand2 size={13} /> {t(AI_KEYS[kind].button)}
    </Button>
  );
}

function Skills({ query, revision }: { query: string; revision: number }) {
  const t = useT();
  const fail = useNotifyError();
  const [skills, setSkills] = useState<SkillEntry[]>([]);
  /** Aura skill being edited (name); the create form doubles as the editor. */
  const [editing, setEditing] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [ai, setAi] = useState(false);
  const [review, setReview] = useState<{ path: string; review: SkillReview } | null>(null);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [body, setBody] = useState("");
  const reload = async () => setSkills(await api.skillsCatalog().catch(() => [] as SkillEntry[]));
  const startEdit = async (name: string) => {
    try {
      const [desc, instructions] = await api.skillsSource(name);
      setCreating(false);
      setEditing(name);
      setName(name);
      setDescription(desc);
      setBody(instructions);
    } catch (e) {
      fail(e);
    }
  };
  const closeEditor = () => {
    setCreating(false);
    setEditing(null);
    setName("");
    setDescription("");
    setBody("");
  };
  useEffect(() => void reload(), [revision]);

  const importSkill = async () => {
    if (!inTauri()) return;
    const { open } = await import("@tauri-apps/plugin-dialog");
    const path = await open({ multiple: false, directory: false, filters: [{ name: "Skill", extensions: ["zip"] }] });
    const dir = path ?? (await open({ multiple: false, directory: true }));
    if (!dir) return;
    try {
      setReview({ path: String(dir), review: await api.skillsReview(String(dir)) });
    } catch (e) {
      fail(e);
    }
  };

  const shown = skills.filter((s) => matchesQuery(query, s.name, s.description, t(`extensions.skills.origin.${s.origin}`)));
  const setAll = async (enabled: boolean) => {
    try {
      for (const s of shown.filter((x) => x.enabled !== enabled)) await api.skillsSetEnabled(s.path, enabled);
    } catch (e) {
      fail(e);
    }
    await reload();
  };

  return (
    <Section
      title={t("extensions.skills")}
      actions={
        <div className="flex flex-wrap justify-end gap-1.5">
          <AiButton kind="skill" onClick={() => setAi(true)} />
          <Button size="sm" onClick={() => void importSkill()}>{t("extensions.skills.import")}</Button>
          <Button size="sm" variant="primary" onClick={() => setCreating(true)}>{t("extensions.skills.new")}</Button>
        </div>
      }
    >
      <AiCreate kind="skill" open={ai} onClose={() => setAi(false)} />
      {review && (
        <div className="flex flex-col gap-2 py-3">
          <div className="text-sm font-medium">{t("extensions.skills.review")}: {review.review.manifest.name}</div>
          <p className="text-xs text-warning">{review.review.warning}</p>
          <ul className="text-xs">
            {review.review.files.map((f) => (
              <li key={f.path} className="font-mono">
                {f.path} {f.isScript && <Badge tone="warning">script</Badge>}
              </li>
            ))}
          </ul>
          <pre className="selectable max-h-48 overflow-auto rounded bg-hover p-2 text-[11px]">{review.review.skillMd}</pre>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setReview(null)}>{t("common.cancel")}</Button>
            <Button
              variant="primary"
              onClick={async () => {
                try {
                  await api.skillsInstall(review.path, true);
                  setReview(null);
                  await reload();
                } catch (e) {
                  fail(e);
                }
              }}
            >
              {t("common.add")}
            </Button>
          </div>
        </div>
      )}
      {(creating || editing) && (
        <div className="grid gap-3 py-3">
          <Field label={t("extensions.skills.name")}>
            <TextField value={name} disabled={editing !== null} onChange={(e) => setName(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, "-"))} />
          </Field>
          <Field label={t("extensions.skills.description")}>
            <TextField value={description} maxLength={1024} onChange={(e) => setDescription(e.target.value)} />
          </Field>
          <Field label={t("extensions.skills.body")}>
            <TextArea rows={6} value={body} onChange={(e) => setBody(e.target.value)} />
          </Field>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={closeEditor}>{t("common.cancel")}</Button>
            <Button
              variant="primary"
              disabled={!name || !description}
              onClick={async () => {
                try {
                  if (editing) await api.skillsUpdate(editing, description, body);
                  else await api.skillsCreate(name, description, body);
                  closeEditor();
                  await reload();
                } catch (e) {
                  fail(e);
                }
              }}
            >
              {t("common.save")}
            </Button>
          </div>
        </div>
      )}
      <BulkBar section={t("extensions.skills")} items={shown} onAll={setAll} />
      {skills.length === 0 && !creating && <p className="py-3 text-[13px] text-muted">{t("extensions.skills.empty")}</p>}
      {skills.length > 0 && shown.length === 0 && <p className="py-3 text-[13px] text-muted">{t("extensions.search.none")}</p>}
      <ul>
        {shown.map((s) => (
          <li key={s.path} aria-label={s.name} className="flex items-center gap-3 py-2.5">
            <Sparkles size={15} className="text-accent" />
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2 text-sm font-medium">
                {s.name}
                <Badge tone={s.origin === "aura" ? "accent" : "muted"}>{t(`extensions.skills.origin.${s.origin}`)}</Badge>
              </div>
              <div className="truncate text-xs text-muted" title={s.description}>{s.description}</div>
            </div>
            <Switch
              label={t("extensions.skills.enable", { name: s.name })}
              checked={s.enabled}
              onChange={async (v) => {
                try {
                  await api.skillsSetEnabled(s.path, v);
                  await reload();
                } catch (e) {
                  fail(e);
                }
              }}
            />
            {s.origin === "aura" && (
              <>
                <Button size="sm" variant="ghost" aria-label={t("extensions.skills.edit", { name: s.name })} onClick={() => void startEdit(s.name)}>
                  <Pencil size={13} />
                </Button>
                <Button size="sm" variant="danger" aria-label={t("common.delete")} onClick={async () => { await api.skillsDelete(s.name); await reload(); }}>
                  <Trash2 size={13} />
                </Button>
              </>
            )}
          </li>
        ))}
      </ul>
    </Section>
  );
}

function secretVarsOf(s: McpServerSpec): string[] {
  if (s.transport.type !== "stdio") return [];
  return Object.entries(s.transport.env)
    .filter(([, v]) => v.kind === "secret")
    .map(([k]) => k);
}

function McpServers({ query, revision }: { query: string; revision: number }) {
  const t = useT();
  const fail = useNotifyError();
  const notify = useApp((s) => s.notify);
  const [servers, setServers] = useState<McpServerSpec[]>([]);
  const [detected, setDetected] = useState<DetectedServer[] | null>(null);
  const [adding, setAdding] = useState(false);
  const [ai, setAi] = useState(false);
  /** Server being edited; the add form doubles as the editor. */
  const [editing, setEditing] = useState<McpServerSpec | null>(null);
  const [kind, setKind] = useState<"stdio" | "http">("stdio");
  const [name, setName] = useState("");
  const [command, setCommand] = useState("");
  const [args, setArgs] = useState("");
  const [url, setUrl] = useState("");
  const [secretVar, setSecretVar] = useState("");
  const [secret, setSecret] = useState("");
  /** Values typed for the secrets an edited server already has (blank keeps). */
  const [secretValues, setSecretValues] = useState<Record<string, string>>({});
  const [approval, setApproval] = useState<McpApprovalMode>("askForWrites");
  const [status, setStatus] = useState<McpStatus[]>([]);
  const parsedArgs = parseArgs(args);
  const [toolsOf, setToolsOf] = useState<string | null>(null);
  const [logs, setLogs] = useState<Record<string, McpDiagnosis | "loading">>({});
  const showLog = async (name: string) => {
    setLogs((l) => ({ ...l, [name]: "loading" }));
    try {
      const d = await api.mcpDiagnose(name);
      setLogs((l) => ({ ...l, [name]: d }));
    } catch (e) {
      setLogs((l) => {
        const { [name]: _, ...rest } = l;
        return rest;
      });
      fail(e);
    }
  };
  const setToolEnabled = async (s: McpServerSpec, tool: string, on: boolean) => {
    const disabledTools = on ? s.disabledTools.filter((x) => x !== tool) : [...s.disabledTools.filter((x) => x !== tool), tool];
    try {
      await api.mcpSave({ ...s, disabledTools }, [], null);
      await reload();
    } catch (e) {
      fail(e);
    }
  };
  const reload = async () => {
    setServers(await api.mcpList());
    setStatus(await api.mcpStatus().catch(() => []));
  };
  useEffect(() => void reload(), [revision]);
  // Servers start with the agent: keep the state fresh while this page is open.
  useEffect(() => {
    const timer = setInterval(() => void api.mcpStatus().then(setStatus).catch(() => undefined), 5000);
    return () => clearInterval(timer);
  }, []);

  const closeForm = () => {
    setAdding(false);
    setEditing(null);
    setName("");
    setCommand("");
    setArgs("");
    setUrl("");
    setSecretVar("");
    setSecret("");
    setSecretValues({});
    setApproval("askForWrites");
  };

  const startEdit = (s: McpServerSpec) => {
    closeForm();
    setEditing(s);
    setName(s.name);
    setKind(s.transport.type);
    if (s.transport.type === "stdio") {
      setCommand(s.transport.command);
      setArgs(formatArgs(s.transport.args));
    } else setUrl(s.transport.url);
    setApproval(s.approvalMode);
  };

  const save = async () => {
    const prev = editing;
    const typed = Object.entries(secretValues).filter(([, v]) => v.trim()) as [string, string][];
    let spec: McpServerSpec;
    let secrets: [string, string][] = [];
    let bearer: string | null = null;
    if (kind === "stdio") {
      const env: Record<string, EnvValue> = prev?.transport.type === "stdio" ? { ...prev.transport.env } : {};
      if (secretVar) env[secretVar] = { kind: "secret" };
      secrets = [...typed, ...(secretVar && secret ? [[secretVar, secret] as [string, string]] : [])];
      spec = {
        name,
        transport: { type: "stdio", command, args: parsedArgs.ok ? parsedArgs.args : [], env, cwd: prev?.transport.type === "stdio" ? prev.transport.cwd : null },
        enabled: true,
        disabledTools: prev?.disabledTools ?? [],
        approvalMode: approval,
        startupTimeoutSec: prev?.startupTimeoutSec ?? null,
        toolTimeoutSec: prev?.toolTimeoutSec ?? null,
      };
    } else {
      const hadBearer = prev?.transport.type === "http" && prev.transport.bearerSecret;
      bearer = secret || null;
      spec = {
        name,
        transport: { type: "http", url, bearerSecret: hadBearer || !!secret, headers: prev?.transport.type === "http" ? prev.transport.headers : {} },
        enabled: true,
        disabledTools: prev?.disabledTools ?? [],
        approvalMode: approval,
        startupTimeoutSec: prev?.startupTimeoutSec ?? null,
        toolTimeoutSec: prev?.toolTimeoutSec ?? null,
      };
    }
    // An edited server stays off unless it was on or its secret was just typed
    // (the agent saves servers that need a secret switched off).
    if (prev) spec.enabled = prev.enabled || secrets.length > 0 || !!bearer;
    try {
      await api.mcpSave(spec, secrets, bearer);
      closeForm();
      await reload();
      notify("info", t("extensions.mcp.restart"));
    } catch (e) {
      fail(e);
    }
  };

  const shown = servers.filter((s) =>
    matchesQuery(query, s.name, s.transport.type === "stdio" ? `${s.transport.command} ${s.transport.args.join(" ")}` : s.transport.url, status.find((x) => x.name === s.name)?.tools.join(" ")),
  );
  const setAll = async (enabled: boolean) => {
    try {
      for (const s of shown.filter((x) => x.enabled !== enabled)) await api.mcpSave({ ...s, enabled }, [], null);
    } catch (e) {
      fail(e);
    }
    await reload();
  };
  const formOpen = adding || editing !== null;
  const existingSecrets = editing ? secretVarsOf(editing) : [];

  return (
    <Section
      title={t("extensions.mcp")}
      description={t("extensions.mcp.restart")}
      actions={
        <div className="flex flex-wrap justify-end gap-1.5">
          <AiButton kind="mcp" onClick={() => setAi(true)} />
          <Button size="sm" onClick={async () => setDetected(await api.mcpDetect())}>{t("extensions.mcp.import")}</Button>
          <Button size="sm" variant="primary" onClick={() => { closeForm(); setAdding(true); }}>{t("extensions.mcp.add")}</Button>
        </div>
      }
    >
      <AiCreate kind="mcp" open={ai} onClose={() => setAi(false)} />
      {detected && (
        <div className="py-3">
          {detected.length === 0 ? (
            <p className="text-[13px] text-muted">{t("extensions.mcp.empty")}</p>
          ) : (
            <ul className="flex flex-col gap-1.5">
              {detected.map((d) => (
                <li key={`${d.app}-${d.spec.name}`} className="flex items-center gap-2 text-[13px]">
                  <Badge>{d.app}</Badge>
                  <span className="font-medium">{d.spec.name}</span>
                  {d.secretNames.length > 0 && <span className="text-xs text-muted">🔒 {d.secretNames.join(", ")}</span>}
                  <Button size="sm" className="ml-auto" onClick={async () => { await api.mcpImport([d.spec.name]); await reload(); }}>
                    {t("common.add")}
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      {formOpen && (
        <div role="group" aria-label={editing ? t("extensions.mcp.editOf", { name: editing.name }) : t("extensions.mcp.add")} className="grid grid-cols-2 gap-3 py-3">
          <Field label={t("extensions.mcp.name")}>
            <TextField value={name} disabled={editing !== null} onChange={(e) => setName(e.target.value.replace(/[^a-zA-Z0-9_-]/g, "-"))} />
          </Field>
          <Field label={t("extensions.mcp.transport")}>
            <Select value={kind} disabled={editing !== null} onChange={(e) => setKind(e.target.value as "stdio" | "http")}>
              <option value="stdio">stdio</option>
              <option value="http">HTTP</option>
            </Select>
          </Field>
          {kind === "stdio" ? (
            <>
              <Field label={t("extensions.mcp.command")}>
                <TextField value={command} placeholder="npx" onChange={(e) => setCommand(e.target.value)} />
              </Field>
              <Field label={t("extensions.mcp.args")} hint={t("extensions.mcp.args.hint")}>
                <TextField value={args} placeholder="-y @modelcontextprotocol/server-github" onChange={(e) => setArgs(e.target.value)} />
              </Field>
              <div className="col-span-2 -mt-1.5 text-xs">
                {parsedArgs.ok ? (
                  parsedArgs.args.length > 0 && (
                    <ol aria-label={t("extensions.mcp.args.parsed")} className="flex flex-wrap gap-1">
                      {parsedArgs.args.map((a, i) => (
                        <li key={i} className="rounded bg-hover px-1.5 py-0.5 font-mono">{a === "" ? t("extensions.mcp.args.empty") : a}</li>
                      ))}
                    </ol>
                  )
                ) : (
                  <p role="alert" className="text-danger">{t("extensions.mcp.args.unterminated")}</p>
                )}
              </div>
              {existingSecrets.map((v) => (
                <Field key={v} label={t("extensions.mcp.secretOf", { name: v })} hint={t("extensions.mcp.secretKeep")}>
                  <TextField type="password" autoComplete="off" value={secretValues[v] ?? ""} onChange={(e) => setSecretValues((x) => ({ ...x, [v]: e.target.value }))} />
                </Field>
              ))}
              <Field label={t("extensions.mcp.secretVar")}>
                <TextField value={secretVar} placeholder="GITHUB_TOKEN" onChange={(e) => setSecretVar(e.target.value.toUpperCase())} />
              </Field>
            </>
          ) : (
            <Field label={t("extensions.mcp.url")}>
              <TextField value={url} placeholder="https://…/mcp" onChange={(e) => setUrl(e.target.value)} />
            </Field>
          )}
          <Field label={t(kind === "http" ? "extensions.mcp.token" : "extensions.mcp.secretValue")} hint={editing && kind === "http" ? t("extensions.mcp.secretKeep") : undefined}>
            <TextField type="password" autoComplete="off" value={secret} onChange={(e) => setSecret(e.target.value)} />
          </Field>
          <Field label={t("extensions.mcp.approval")}>
            <Select value={approval} onChange={(e) => setApproval(e.target.value as McpApprovalMode)}>
              <option value="alwaysAsk">{t("extensions.mcp.approval.alwaysAsk")}</option>
              <option value="askForWrites">{t("extensions.mcp.approval.askForWrites")}</option>
              <option value="auto">{t("extensions.mcp.approval.auto")}</option>
            </Select>
          </Field>
          <div className="col-span-2 flex justify-end gap-2">
            <Button variant="ghost" onClick={closeForm}>{t("common.cancel")}</Button>
            <Button variant="primary" disabled={!name || (kind === "stdio" ? !command || !parsedArgs.ok : !url)} onClick={() => void save()}>{t("common.save")}</Button>
          </div>
        </div>
      )}
      <BulkBar section={t("extensions.mcp")} items={shown} onAll={setAll} />
      {servers.length === 0 && !formOpen && <p className="py-3 text-[13px] text-muted">{t("extensions.mcp.empty")}</p>}
      {servers.length > 0 && shown.length === 0 && <p className="py-3 text-[13px] text-muted">{t("extensions.search.none")}</p>}
      {shown.map((s) => (
        <div key={s.name} className="flex items-start gap-3 py-2.5">
          <Server size={15} className="mt-0.5 text-muted" />
          <div className="min-w-0 flex-1">
            <div className="text-sm font-medium">{s.name}</div>
            <div className="truncate font-mono text-[11px] text-muted">
              {s.transport.type === "stdio" ? `${s.transport.command} ${formatArgs(s.transport.args)}`.trim() : s.transport.url}
            </div>
            {(() => {
              const st = status.find((x) => x.name === s.name);
              const log = logs[s.name];
              return (
                <>
                  <div role="status" aria-label={t("extensions.mcp.stateOf", { name: s.name })} className={`truncate text-[11px] ${st?.error ? "text-danger" : st ? "text-success" : "text-muted"}`}>
                    {!s.enabled ? t("extensions.mcp.state.disabled") : st?.error ? t("extensions.mcp.state.error", { reason: st.error }) : st ? t("extensions.mcp.state.connected") : t("extensions.mcp.state.unknown")}
                  </div>
                  {st?.error && s.transport.type === "stdio" && (
                    <button type="button" className="text-[11px] text-muted underline-offset-2 hover:underline" aria-label={t("extensions.mcp.showLogOf", { name: s.name })} onClick={() => void showLog(s.name)}>
                      {t("extensions.mcp.showLog")}
                    </button>
                  )}
                  {log && (
                    <pre role="log" aria-label={t("extensions.mcp.logOf", { name: s.name })} className="selectable mt-1 max-h-40 overflow-auto whitespace-pre-wrap rounded bg-hover p-2 font-mono text-[11px]">
                      {log === "loading"
                        ? t("common.loading")
                        : [
                            log.connected ? t("extensions.mcp.log.connected") : log.exitCode !== null ? t("extensions.mcp.log.exited", { code: log.exitCode }) : t("extensions.mcp.log.noAnswer"),
                            ...(log.log.length ? log.log : [t("extensions.mcp.log.empty")]),
                          ].join("\n")}
                    </pre>
                  )}
                </>
              );
            })()}
            {(() => {
              const st = status.find((x) => x.name === s.name);
              // Disabled tools are not reported by the server; keep them listed (off).
              const tools = [...new Set([...(st?.tools ?? []), ...s.disabledTools])].sort();
              if (tools.length === 0) return null;
              return (
                <>
                  <button type="button" className="text-[11px] text-muted underline-offset-2 hover:underline" aria-expanded={toolsOf === s.name} aria-label={t("extensions.mcp.toolsOf", { name: s.name })} onClick={() => setToolsOf((n) => (n === s.name ? null : s.name))}>
                    {t("extensions.mcp.tools", { n: tools.length - s.disabledTools.length })}
                  </button>
                  {toolsOf === s.name && (
                    <div className="mt-1.5">
                      <ul className="flex flex-col gap-1">
                        {tools.map((tool) => (
                          <li key={tool}>
                            <label className="flex items-center gap-1.5 font-mono text-[12px]">
                              <input type="checkbox" checked={!s.disabledTools.includes(tool)} onChange={(e) => void setToolEnabled(s, tool, e.target.checked)} />
                              {tool}
                            </label>
                          </li>
                        ))}
                      </ul>
                      <p className="mt-1 text-[11px] text-muted">{t("extensions.mcp.tools.newConversations")}</p>
                    </div>
                  )}
                </>
              );
            })()}
          </div>
          {status.find((x) => x.name === s.name)?.auth === "notLoggedIn" && (
            <Button size="sm" onClick={() => void api.mcpLogin(s.name)}>{t("extensions.mcp.connect")}</Button>
          )}
          <Switch label={s.name} checked={s.enabled} onChange={async (v) => { await api.mcpSave({ ...s, enabled: v }, [], null); await reload(); }} />
          <Button size="sm" variant="ghost" aria-label={t("extensions.mcp.editOf", { name: s.name })} onClick={() => startEdit(s)}>
            <Pencil size={13} />
          </Button>
          <Button size="sm" variant="danger" aria-label={t("common.delete")} onClick={async () => { await api.mcpDelete(s.name); await reload(); }}>
            <Trash2 size={13} />
          </Button>
        </div>
      ))}
    </Section>
  );
}

function QuickCommands({ query, revision }: { query: string; revision: number }) {
  const t = useT();
  const language = useApp((s) => s.settings?.language);
  const fail = useNotifyError();
  const [list, setList] = useState<QuickCommand[]>([]);
  const [name, setName] = useState("");
  const [template, setTemplate] = useState("");
  const [ai, setAi] = useState(false);
  useEffect(() => {
    let cancelled = false;
    void api.quickList().then((commands) => { if (!cancelled) setList(commands); });
    return () => { cancelled = true; };
  }, [language, revision]);
  const shown = list.filter((q) => matchesQuery(query, q.name, `/${q.name}`, templatePreview(q.template)));
  const setAll = async (enabled: boolean) => {
    try {
      let next = list;
      for (const q of shown.filter((x) => x.enabled !== enabled)) next = await api.quickToggle(q.name, enabled);
      setList(next);
    } catch (e) {
      fail(e);
    }
  };
  // Editing one of the user's commands rewrites it instead of failing as a duplicate.
  const replacing = list.some((q) => q.name === name && !q.builtin);
  return (
    <Section title={t("extensions.quick")} actions={<AiButton kind="quick" onClick={() => setAi(true)} />}>
      <AiCreate kind="quick" open={ai} onClose={() => setAi(false)} />
      <BulkBar section={t("extensions.quick")} items={shown} onAll={setAll} />
      {list.length > 0 && shown.length === 0 && <p className="py-3 text-[13px] text-muted">{t("extensions.search.none")}</p>}
      {shown.map((q) => (
        <div key={q.name} className="flex items-center gap-3 py-2">
          <Zap size={14} className="text-muted" />
          <div className="min-w-0 flex-1">
            <div className="text-sm font-medium">/{q.name} {q.builtin && <Badge>{t("extensions.quick.builtin")}</Badge>}</div>
            <div className="truncate text-xs text-muted" title={q.template}>
              {templatePreview(q.template)}
              <QuickSources template={q.template} />
            </div>
          </div>
          <Switch label={q.name} checked={q.enabled} onChange={async (v) => setList(await api.quickToggle(q.name, v))} />
          {!q.builtin && (
            <Button size="sm" variant="ghost" aria-label={t("common.edit")} onClick={() => { setName(q.name); setTemplate(q.template); }}>
              <Pencil size={13} />
            </Button>
          )}
          {!q.builtin && (
            <Button size="sm" variant="danger" aria-label={t("common.delete")} onClick={async () => setList(await api.quickDelete(q.name))}>
              <Trash2 size={13} />
            </Button>
          )}
        </div>
      ))}
      <div className="grid grid-cols-[160px_1fr_auto] gap-2 py-2.5">
        <TextField placeholder="email-formal" aria-label={t("extensions.quick.name")} value={name} onChange={(e) => setName(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, "-"))} />
        <TextField placeholder={t("extensions.quick.template")} aria-label={t("extensions.quick.template")} value={template} onChange={(e) => setTemplate(e.target.value)} />
        <Button
          disabled={!name || !template}
          onClick={async () => {
            try {
              setList(await api.quickSave(name, template, replacing));
              setName("");
              setTemplate("");
            } catch (e) {
              fail(e);
            }
          }}
        >
          {replacing ? t("common.save") : t("common.add")}
        </Button>
      </div>
    </Section>
  );
}

export function ExtensionsSection() {
  const t = useT();
  const [query, setQuery] = useState("");
  // The agent (or another window) changed extensions: reload every list.
  const revision = useApp((s) => s.extensionsRevision);
  return (
    <>
      <div className="relative">
        <Search size={14} className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted" aria-hidden />
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape" && query) {
              e.stopPropagation();
              setQuery("");
            }
          }}
          aria-label={t("extensions.search")}
          placeholder={t("extensions.search.placeholder")}
          className="h-9 w-full rounded-md border border-line bg-surface-strong pl-8 pr-3 text-sm outline-none placeholder:text-muted focus:border-accent"
        />
      </div>
      <Skills query={query} revision={revision} />
      <McpServers query={query} revision={revision} />
      <QuickCommands query={query} revision={revision} />
    </>
  );
}

/** Where a quick command takes its text from, in words (no raw placeholders). */
function QuickSources({ template }: { template: string }) {
  const t = useT();
  const h = quickHint(template);
  const parts = [h.source && t(h.source === "selection" ? "quick.hint.selection" : "quick.hint.typed"), h.screen && t("quick.hint.screen")].filter(Boolean);
  return parts.length ? <span className="opacity-80"> · {parts.join(" · ")}</span> : null;
}
