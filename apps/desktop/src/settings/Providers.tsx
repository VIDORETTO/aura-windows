import { Plug, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { Preset, Provider } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { Badge, Button, Field, Section, Select, Spinner, TextField } from "../ui/primitives";

export function ProvidersSection() {
  const t = useT();
  const notify = useApp((s) => s.notify);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [adding, setAdding] = useState(false);
  const [presetId, setPresetId] = useState("openai");
  const [name, setName] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [key, setKey] = useState("");
  const [busy, setBusy] = useState<string | null>(null);

  const reload = async () => setProviders(await api.providersList());
  useEffect(() => {
    void api.providersPresets().then(setPresets);
    void reload();
  }, []);

  const preset = presets.find((p) => p.id === presetId);

  const save = async () => {
    setBusy("save");
    try {
      const p = await api.providersSave(
        { name: name || preset?.name || presetId, preset: presetId, baseUrl: baseUrl || null },
        key || null,
      );
      setAdding(false);
      setKey("");
      setName("");
      setBaseUrl("");
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
        !adding && (
          <Button size="sm" variant="primary" onClick={() => setAdding(true)}>
            {t("providers.add")}
          </Button>
        )
      }
    >
      {adding && (
        <div className="grid grid-cols-2 gap-3 py-3">
          <Field label={t("providers.preset")}>
            <Select value={presetId} onChange={(e) => setPresetId(e.target.value)}>
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
            <TextField type="password" autoComplete="off" value={key} onChange={(e) => setKey(e.target.value)} placeholder={preset?.credentialRequired ? "sk-…" : "(opcional)"} />
          </Field>
          <div className="col-span-2 flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setAdding(false)}>
              {t("common.cancel")}
            </Button>
            <Button variant="primary" disabled={busy !== null || (!!preset?.credentialRequired && !key) || (presetId === "custom" && !baseUrl)} onClick={() => void save()}>
              {busy === "save" && <Spinner />} {t("common.save")}
            </Button>
          </div>
        </div>
      )}
      {providers.length === 0 && !adding && <p className="py-3 text-[13px] text-muted">{t("providers.empty")}</p>}
      {providers.map((p) => (
        <div key={p.id} className="flex items-center gap-3 py-2.5">
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
        </div>
      ))}
    </Section>
  );
}
