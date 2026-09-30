import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import { inTauri } from "../ipc/bridge";
import { useApp } from "../state/app";
import { checkForUpdate, installUpdate, type UpdateCheck } from "../lib/updates";
import type { Diagnostics } from "../ipc/types";
import { useT } from "../i18n";
import { Button, Row, Section } from "../ui/primitives";

function stateLabel(d: Diagnostics): string {
  const s = d.appServer;
  switch (s.state) {
    case "ready":
      return `pronto (${s.version})`;
    case "failed":
      return `falhou: ${s.reason}`;
    case "downloading":
      return "baixando";
    case "restarting":
      return `reiniciando (${s.attempt})`;
    default:
      return s.state;
  }
}

export function DiagnosticsSection() {
  const t = useT();
  const [d, setD] = useState<Diagnostics | null>(null);
  useEffect(() => {
    void api.diagnostics().then(setD);
    const h = setInterval(() => void api.diagnostics().then(setD), 3000);
    return () => clearInterval(h);
  }, []);
  return (
    <>
      <Section title={t("settings.diagnostics")}>
        {d && (
          <>
            <Row label={t("diagnostics.version")}><span className="font-mono text-[13px]">{d.version} · {d.os}</span></Row>
            <Row label={t("diagnostics.appServer")}><span className="text-[13px]">{stateLabel(d)} · {d.appServerLaunches}×</span></Row>
            <Row label={t("diagnostics.gateway")}><span className="font-mono text-[13px]">127.0.0.1:{d.gatewayPort}</span></Row>
            <Row label={t("diagnostics.dataDir")}><span className="selectable font-mono text-[12px]">{d.dataDir}</span></Row>
            <Row label={t("settings.providers")}><span className="text-[13px]">{d.providers}</span></Row>
            <Row label={t("extensions.mcp")}><span className="text-[13px]">{d.mcpServers}</span></Row>
          </>
        )}
      </Section>
      <Section title={t("diagnostics.export")} description={t("diagnostics.export.hint")}>
        <Row label={t("diagnostics.export.label")}>
          <Button
            onClick={async () => {
              let dest = "aura-diagnostico.zip";
              if (inTauri()) {
                const { save } = await import("@tauri-apps/plugin-dialog");
                const picked = await save({ defaultPath: dest, filters: [{ name: "Zip", extensions: ["zip"] }] });
                if (!picked) return;
                dest = picked;
              }
              try {
                const path = await api.diagnosticsExport(dest);
                useApp.getState().notify("info", t("diagnostics.exported", { path }));
              } catch (e) {
                useApp.getState().notify("error", errorMessage(e));
              }
            }}
          >
            {t("diagnostics.export")}
          </Button>
        </Row>
      </Section>
      <Section title={t("diagnostics.erase")}>
        <Row label={t("diagnostics.eraseConfirm")}>
          <Button
            variant="danger"
            onClick={() => {
              if (window.confirm(t("diagnostics.eraseConfirm"))) void api.eraseAllData();
            }}
          >
            {t("diagnostics.erase")}
          </Button>
        </Row>
      </Section>
    </>
  );
}

export function AboutSection() {
  const t = useT();
  const [update, setUpdate] = useState<UpdateCheck | null>(null);
  const [pct, setPct] = useState<number | null>(null);
  const check = async () => {
    try {
      setUpdate(await checkForUpdate());
    } catch (e) {
      useApp.getState().notify("error", errorMessage(e));
    }
  };
  return (
    <Section title="Aura">
      <div className="flex items-center gap-4 py-4">
        <img src="/logo.svg" alt="" className="h-14 w-14" />
        <div>
          <p className="text-sm">{t("about.tagline")}</p>
          <p className="text-xs text-muted">{t("about.license")}</p>
        </div>
      </div>
      <Row label={t("updates.title")} hint={update?.kind === "none" ? t("updates.none") : update?.kind === "available" ? t("updates.available", { version: update.version }) : update?.kind === "unsupported" ? t("updates.unsupported") : pct !== null ? `${pct}%` : undefined}>
        {update?.kind === "available" ? (
          <Button variant="primary" disabled={pct !== null} onClick={() => void installUpdate(setPct).catch((e) => useApp.getState().notify("error", errorMessage(e)))}>
            {t("updates.install")}
          </Button>
        ) : (
          <Button onClick={() => void check()}>{t("updates.check")}</Button>
        )}
      </Row>
    </Section>
  );
}
