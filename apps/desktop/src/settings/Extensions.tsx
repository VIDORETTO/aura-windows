import { Pencil, Server, Sparkles, Trash2, Zap } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import { inTauri } from "../ipc/bridge";
import { formatArgs, parseArgs } from "../lib/args";
import type { DetectedServer, McpApprovalMode, McpServerSpec, McpStatus, QuickCommand, SkillEntry, SkillReview, McpDiagnosis } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { Badge, Button, Field, Section, Select, Switch, TextArea, TextField } from "../ui/primitives";

function useNotifyError() {
  const notify = useApp((s) => s.notify);
  return (e: unknown) => notify("error", errorMessage(e));
}

function Skills() {
  const t = useT();
  const fail = useNotifyError();
  const [skills, setSkills] = useState<SkillEntry[]>([]);
  /** Aura skill being edited (name); the create form doubles as the editor. */
  const [editing, setEditing] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
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
  useEffect(() => void reload(), []);

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

  return (
    <Section
      title={t("extensions.skills")}
      actions={
        <div className="flex gap-1.5">
          <Button size="sm" onClick={() => void importSkill()}>{t("extensions.skills.import")}</Button>
          <Button size="sm" variant="primary" onClick={() => setCreating(true)}>{t("extensions.skills.new")}</Button>
        </div>
      }
    >
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
      {skills.length === 0 && !creating && <p className="py-3 text-[13px] text-muted">{t("extensions.skills.empty")}</p>}
      <ul>
        {skills.map((s) => (
          <li key={s.path} aria-label={s.name} className="flex items-center gap-3 py-2.5">
            <Sparkles size={15} className="text-accent" />
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2 text-sm font-medium">
                {s.name}
                <Badge tone={s.origin === "aura" ? "accent" : "muted"}>{t(`extensions.skills.origin.${s.origin}`)}</Badge>
              </div>
              <div className="truncate text-xs text-muted">{s.description}</div>
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

function McpServers() {
  const t = useT();
  const fail = useNotifyError();
  const notify = useApp((s) => s.notify);
  const [servers, setServers] = useState<McpServerSpec[]>([]);
  const [detected, setDetected] = useState<DetectedServer[] | null>(null);
  const [adding, setAdding] = useState(false);
  const [kind, setKind] = useState<"stdio" | "http">("stdio");
  const [name, setName] = useState("");
  const [command, setCommand] = useState("");
  const [args, setArgs] = useState("");
  const [url, setUrl] = useState("");
  const [secretVar, setSecretVar] = useState("");
  const [secret, setSecret] = useState("");
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
  useEffect(() => void reload(), []);
  // Servers start with the agent: keep the state fresh while this page is open.
  useEffect(() => {
    const timer = setInterval(() => void api.mcpStatus().then(setStatus).catch(() => undefined), 5000);
    return () => clearInterval(timer);
  }, []);

  const save = async () => {
    const env: Record<string, { kind: "secret" }> = secretVar ? { [secretVar]: { kind: "secret" } } : {};
    const spec: McpServerSpec = {
      name,
      transport: kind === "stdio"
        ? { type: "stdio", command, args: parsedArgs.ok ? parsedArgs.args : [], env, cwd: null }
        : { type: "http", url, bearerSecret: !!secret, headers: {} },
      enabled: true,
      disabledTools: [],
      approvalMode: approval,
      startupTimeoutSec: null,
      toolTimeoutSec: null,
    };
    try {
      await api.mcpSave(spec, kind === "stdio" && secretVar ? [[secretVar, secret]] : [], kind === "http" && secret ? secret : null);
      setAdding(false);
      setSecret("");
      await reload();
      notify("info", t("extensions.mcp.restart"));
    } catch (e) {
      fail(e);
    }
  };

  return (
    <Section
      title={t("extensions.mcp")}
      description={t("extensions.mcp.restart")}
      actions={
        <div className="flex gap-1.5">
          <Button size="sm" onClick={async () => setDetected(await api.mcpDetect())}>{t("extensions.mcp.import")}</Button>
          <Button size="sm" variant="primary" onClick={() => setAdding(true)}>{t("extensions.mcp.add")}</Button>
        </div>
      }
    >
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
      {adding && (
        <div className="grid grid-cols-2 gap-3 py-3">
          <Field label={t("extensions.mcp.name")}>
            <TextField value={name} onChange={(e) => setName(e.target.value.replace(/[^a-zA-Z0-9_-]/g, "-"))} />
          </Field>
          <Field label={t("extensions.mcp.transport")}>
            <Select value={kind} onChange={(e) => setKind(e.target.value as "stdio" | "http")}>
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
              <Field label={t("extensions.mcp.secretVar")}>
                <TextField value={secretVar} placeholder="GITHUB_TOKEN" onChange={(e) => setSecretVar(e.target.value.toUpperCase())} />
              </Field>
            </>
          ) : (
            <Field label={t("extensions.mcp.url")}>
              <TextField value={url} placeholder="https://…/mcp" onChange={(e) => setUrl(e.target.value)} />
            </Field>
          )}
          <Field label={t(kind === "http" ? "extensions.mcp.token" : "extensions.mcp.secretValue")}>
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
            <Button variant="ghost" onClick={() => setAdding(false)}>{t("common.cancel")}</Button>
            <Button variant="primary" disabled={!name || (kind === "stdio" ? !command || !parsedArgs.ok : !url)} onClick={() => void save()}>{t("common.save")}</Button>
          </div>
        </div>
      )}
      {servers.length === 0 && !adding && <p className="py-3 text-[13px] text-muted">{t("extensions.mcp.empty")}</p>}
      {servers.map((s) => (
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
          <Button size="sm" variant="danger" aria-label={t("common.delete")} onClick={async () => { await api.mcpDelete(s.name); await reload(); }}>
            <Trash2 size={13} />
          </Button>
        </div>
      ))}
    </Section>
  );
}

function QuickCommands() {
  const t = useT();
  const language = useApp((s) => s.settings?.language);
  const fail = useNotifyError();
  const [list, setList] = useState<QuickCommand[]>([]);
  const [name, setName] = useState("");
  const [template, setTemplate] = useState("");
  useEffect(() => {
    let cancelled = false;
    void api.quickList().then((commands) => { if (!cancelled) setList(commands); });
    return () => { cancelled = true; };
  }, [language]);
  return (
    <Section title={t("extensions.quick")}>
      {list.map((q) => (
        <div key={q.name} className="flex items-center gap-3 py-2">
          <Zap size={14} className="text-muted" />
          <div className="min-w-0 flex-1">
            <div className="text-sm font-medium">/{q.name} {q.builtin && <Badge>{t("extensions.quick.builtin")}</Badge>}</div>
            <div className="truncate text-xs text-muted">{q.template.replace(/\n/g, " ")}</div>
          </div>
          <Switch label={q.name} checked={q.enabled} onChange={async (v) => setList(await api.quickToggle(q.name, v))} />
          {!q.builtin && (
            <Button size="sm" variant="danger" aria-label={t("common.delete")} onClick={async () => setList(await api.quickDelete(q.name))}>
              <Trash2 size={13} />
            </Button>
          )}
        </div>
      ))}
      <div className="grid grid-cols-[160px_1fr_auto] gap-2 py-2.5">
        <TextField placeholder="email-formal" value={name} onChange={(e) => setName(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, "-"))} />
        <TextField placeholder={t("extensions.quick.template")} value={template} onChange={(e) => setTemplate(e.target.value)} />
        <Button
          disabled={!name || !template}
          onClick={async () => {
            try {
              setList(await api.quickSave(name, template, false));
              setName("");
              setTemplate("");
            } catch (e) {
              fail(e);
            }
          }}
        >
          {t("common.add")}
        </Button>
      </div>
    </Section>
  );
}

export function ExtensionsSection() {
  return (
    <>
      <Skills />
      <McpServers />
      <QuickCommands />
    </>
  );
}
