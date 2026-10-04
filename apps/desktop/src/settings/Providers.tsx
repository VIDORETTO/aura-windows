import { Pencil, Plug, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { ModelSpec, Preset, Provider, Wire } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { Badge, Button, Field, Section, Select, Spinner, TextField } from "../ui/primitives";

/** RFC 7230 token, as the gateway validates it. */
const HEADER_NAME = /^[A-Za-z0-9!#$%&'*+.^_|~-]+$/;

export function ProvidersSection() {
  const t = useT();
  const notify = useApp((s) => s.notify);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [adding, setAdding] = useState(false);
  /** Provider being edited in place (QA-019): keeps its id, models and key. */
  const [editing, setEditing] = useState<Provider | null>(null);
  const [presetId, setPresetId] = useState("openai");
  const [name, setName] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [key, setKey] = useState("");
  const [wire, setWire] = useState<Wire>("responses");
  const [headers, setHeaders] = useState<{ name: string; value: string }[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [modelsOf, setModelsOf] = useState<string | null>(null);

  const reload = async () => setProviders(await api.providersList());
  useEffect(() => {
    void api.providersPresets().then(setPresets);
    void reload();
  }, []);

  const preset = presets.find((p) => p.id === presetId);
  const formOpen = adding || editing !== null;

  const closeForm = () => {
    setAdding(false);
    setEditing(null);
    setKey("");
    setName("");
    setBaseUrl("");
    setHeaders([]);
  };

  const startEdit = (p: Provider) => {
    setAdding(false);
    setEditing(p);
    setPresetId(p.preset);
    setName(p.name);
    setBaseUrl(p.baseUrl);
    setKey("");
    setWire(p.wire);
    setHeaders(Object.entries(p.extraHeaders).map(([name, value]) => ({ name, value })));
  };

  const custom = (editing?.preset ?? presetId) === "custom";
  const badHeader = headers.find((h) => h.name !== "" && !HEADER_NAME.test(h.name));
  const extraHeaders = Object.fromEntries(headers.filter((h) => h.name.trim()).map((h) => [h.name.trim(), h.value]));

  const save = async () => {
    setBusy("save");
    try {
      const p = await api.providersSave(
        editing
          ? { id: editing.id, name: name.trim() || editing.name, preset: editing.preset, wire: custom ? wire : editing.wire, baseUrl: baseUrl || null, extraHeaders }
          : { name: name || preset?.name || presetId, preset: presetId, baseUrl: baseUrl || null, ...(custom ? { wire } : {}), extraHeaders },
        key || null,
      );
      closeForm();
      await reload();
      await test(p.id);
    } catch (e) {
      notify("error", errorMessage(e));
    } finally {
      setBusy(null);
    }
  };

  const test = async (id: string) => {
    setBusy(id);
    try {
      const p = await api.providersTest(id);
      notify(p.status === "verified" ? "info" : "warning", p.status === "verified" ? `${p.name}: ${t("providers.status.verified")}` : `${p.name}: ${p.lastError ?? t("providers.status.error")}`);
      await reload();
    } catch (e) {
      notify("error", errorMessage(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <Section
      title={t("settings.providers")}
      description={t("providers.keyHint")}
      actions={
        !formOpen && (
          <Button size="sm" variant="primary" onClick={() => setAdding(true)}>
            {t("providers.add")}
          </Button>
        )
      }
    >
      {formOpen && (
        <div className="grid grid-cols-2 gap-3 py-3" aria-label={editing ? t("providers.editTitle", { name: editing.name }) : t("providers.add")} role="group">
          <Field label={t("providers.preset")}>
            <Select value={presetId} disabled={editing !== null} onChange={(e) => setPresetId(e.target.value)}>
              {presets.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </Select>
          </Field>
          <Field label={t("providers.name")}>
            <TextField value={name} placeholder={preset?.name} onChange={(e) => setName(e.target.value)} />
          </Field>
          <Field label={t("providers.baseUrl")}>
            <TextField value={baseUrl} placeholder={preset?.baseUrl || "https://…/v1"} onChange={(e) => setBaseUrl(e.target.value)} />
          </Field>
          <Field label={t("providers.key")}>
            <TextField
              type="password"
              autoComplete="off"
              value={key}
              onChange={(e) => setKey(e.target.value)}
              placeholder={editing?.credentialHint ? t("providers.keepKey", { hint: editing.credentialHint }) : preset?.credentialRequired ? "sk-…" : t("providers.keyOptional")}
            />
          </Field>
          {custom && (
            <Field label={t("providers.wire")}>
              <Select aria-label={t("providers.wire")} value={wire} onChange={(e) => setWire(e.target.value as Wire)}>
                <option value="responses">Responses (OpenAI)</option>
                <option value="chat">Chat Completions</option>
                <option value="anthropic">Anthropic Messages</option>
              </Select>
            </Field>
          )}
          <div className="col-span-2 grid gap-2">
            <div className="flex items-center justify-between">
              <span className="text-[13px] font-medium">{t("providers.headers")}</span>
              <Button size="sm" variant="ghost" onClick={() => setHeaders((h) => [...h, { name: "", value: "" }])}>
                {t("providers.headers.add")}
              </Button>
            </div>
            <p className="text-xs text-muted">{t("providers.headers.hint")}</p>
            {headers.map((h, i) => (
              <div key={i} className="flex gap-2">
                <TextField aria-label={t("providers.headers.name", { n: i + 1 })} value={h.name} placeholder="X-Header" onChange={(e) => setHeaders((all) => all.map((x, j) => (j === i ? { ...x, name: e.target.value } : x)))} />
                <TextField aria-label={t("providers.headers.value", { n: i + 1 })} value={h.value} onChange={(e) => setHeaders((all) => all.map((x, j) => (j === i ? { ...x, value: e.target.value } : x)))} />
                <Button size="sm" variant="ghost" aria-label={t("providers.headers.remove", { n: i + 1 })} onClick={() => setHeaders((all) => all.filter((_, j) => j !== i))}>
                  <Trash2 size={13} />
                </Button>
              </div>
            ))}
            {badHeader && <p role="alert" className="text-xs text-danger">{t("providers.headers.invalid", { name: badHeader.name })}</p>}
          </div>
          <div className="col-span-2 flex justify-end gap-2">
            <Button variant="ghost" onClick={closeForm}>
              {t("common.cancel")}
            </Button>
            <Button variant="primary" disabled={busy !== null || !!badHeader || (!editing && !!preset?.credentialRequired && !key) || (presetId === "custom" && !baseUrl)} onClick={() => void save()}>
              {busy === "save" && <Spinner />} {t("common.save")}
            </Button>
          </div>
        </div>
      )}
      {providers.length === 0 && !formOpen && <p className="py-3 text-[13px] text-muted">{t("providers.empty")}</p>}
      {providers.map((p) => (
        <div key={p.id} className="flex flex-wrap items-center gap-3 py-2.5">
          <Plug size={16} className="text-muted" />
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2 text-sm font-medium">
              {p.name}
              <Badge tone={p.status === "verified" ? "success" : p.status === "error" ? "danger" : "muted"}>{t(`providers.status.${p.status}`)}</Badge>
            </div>
            <div className="truncate text-xs text-muted">
              {p.baseUrl} · {p.wire} {p.credentialHint ? `· ${p.credentialHint}` : ""} · {t("providers.models", { n: p.models.length })}
            </div>
            {p.lastError && <div className="text-xs text-danger">{p.lastError}</div>}
          </div>
          <Button size="sm" variant="ghost" aria-expanded={modelsOf === p.id} aria-label={t("providers.modelsOf", { name: p.name })} onClick={() => setModelsOf((id) => (id === p.id ? null : p.id))}>
            {t("providers.models", { n: p.models.length })}
          </Button>
          <Button size="sm" variant="ghost" aria-label={t("providers.edit", { name: p.name })} disabled={busy !== null} onClick={() => startEdit(p)}>
            <Pencil size={13} />
          </Button>
          <Button size="sm" disabled={busy !== null} onClick={() => void test(p.id)}>
            {busy === p.id ? <Spinner /> : null} {t("providers.test")}
          </Button>
          <Button
            size="sm"
            variant="danger"
            aria-label={t("common.delete")}
            onClick={async () => {
              await api.providersRemove(p.id);
              await reload();
            }}
          >
            <Trash2 size={13} />
          </Button>
          {modelsOf === p.id && <ProviderModels provider={p} onChange={reload} />}
        </div>
      ))}
    </Section>
  );
}

const EMPTY_MODEL: ModelSpec = { id: "", displayName: null, contextWindow: null, maxOutput: null, supportsImages: false, supportsTools: false, supportsReasoning: false, estimated: false };

/** Models of one provider: discovered or entered by hand (003 AC-013). */
function ProviderModels({ provider, onChange }: { provider: Provider; onChange: () => Promise<void> }) {
  const t = useT();
  const notify = useApp((s) => s.notify);
  const [draft, setDraft] = useState<ModelSpec | null>(null);
  const caps = (m: ModelSpec) =>
    [m.supportsImages && t("picker.cap.images"), m.supportsTools && t("picker.cap.tools"), m.supportsReasoning && t("picker.cap.reasoning")].filter(Boolean).join(" · ") || t("picker.cap.textOnly");
  const run = async (fn: () => Promise<unknown>) => {
    try {
      await fn();
      await onChange();
    } catch (e) {
      notify("error", errorMessage(e));
    }
  };
  const save = () =>
    draft &&
    void run(async () => {
      await api.providersModelSave(provider.id, { ...draft, id: draft.id.trim(), displayName: draft.displayName?.trim() || null, estimated: false });
      setDraft(null);
    });
  const flag = (key: "supportsImages" | "supportsTools" | "supportsReasoning", label: string) => (
    <label className="flex items-center gap-1.5 text-[13px]">
      <input type="checkbox" checked={!!draft?.[key]} onChange={(e) => setDraft((d) => d && { ...d, [key]: e.target.checked })} />
      {label}
    </label>
  );
  return (
    <div className="basis-full rounded-md border border-line p-2.5">
      <ul className="grid gap-1.5">
        {provider.models.map((m) => {
          const label = m.displayName ?? m.id;
          return (
            <li key={m.id} aria-label={label} className="flex items-center gap-2 text-[13px]">
              <span className="min-w-0 flex-1 truncate">
                {label} <span className="text-xs text-muted">· {caps(m)}{m.manual ? ` · ${t("providers.model.manual")}` : m.estimated ? ` · ${t("providers.model.estimated")}` : ""}</span>
              </span>
              <Button size="sm" variant="ghost" aria-label={t("providers.model.edit", { name: label })} onClick={() => setDraft({ ...m })}>
                <Pencil size={12} />
              </Button>
              {m.manual && (
                <Button size="sm" variant="ghost" aria-label={t("providers.model.remove", { name: label })} onClick={() => void run(() => api.providersModelRemove(provider.id, m.id))}>
                  <Trash2 size={12} />
                </Button>
              )}
            </li>
          );
        })}
      </ul>
      {draft ? (
        <div className="mt-2 grid grid-cols-2 gap-2">
          <Field label={t("providers.model.id")}>
            <TextField value={draft.id} onChange={(e) => setDraft({ ...draft, id: e.target.value })} placeholder="llama3.1:8b" />
          </Field>
          <Field label={t("providers.model.name")}>
            <TextField value={draft.displayName ?? ""} onChange={(e) => setDraft({ ...draft, displayName: e.target.value || null })} />
          </Field>
          <Field label={t("providers.model.context")}>
            <TextField type="number" min={1} value={draft.contextWindow ?? ""} onChange={(e) => setDraft({ ...draft, contextWindow: e.target.value ? Number(e.target.value) : null })} />
          </Field>
          <div className="flex flex-wrap items-end gap-3 pb-1.5">
            {flag("supportsImages", t("picker.cap.images"))}
            {flag("supportsTools", t("picker.cap.tools"))}
            {flag("supportsReasoning", t("picker.cap.reasoning"))}
          </div>
          <div className="col-span-2 flex justify-end gap-2">
            <Button size="sm" variant="ghost" onClick={() => setDraft(null)}>{t("common.cancel")}</Button>
            <Button size="sm" variant="primary" disabled={!draft.id.trim()} onClick={save}>{t("providers.model.save")}</Button>
          </div>
        </div>
      ) : (
        <Button size="sm" className="mt-2" onClick={() => setDraft({ ...EMPTY_MODEL })}>{t("providers.model.add")}</Button>
      )}
    </div>
  );
}