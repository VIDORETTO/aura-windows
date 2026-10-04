import { useEffect, useState } from "react";
import { AuraLogo } from "../ui/AuraLogo";
import { api, errorMessage } from "../ipc/commands";
import { inTauri } from "../ipc/bridge";
import { useApp } from "../state/app";
import { checkForUpdate, installUpdate, type UpdateCheck } from "../lib/updates";
import type { Diagnostics } from "../ipc/types";
import { formatBytes } from "../lib/format";
import { useT } from "../i18n";
import { Button, Row, Section } from "../ui/primitives";

const SOURCE: Record<string, string> = { screen: "privacy.source.screen", mic: "privacy.source.mic", system: "privacy.source.systemAudio" };

function stateLabel(d: Diagnostics, t: ReturnType<typeof useT>): string {
  const s = d.appServer;
  switch (s.state) {
    case "ready":
      return t("diagnostics.state.ready", { version: s.version });
    case "failed":
      return t("diagnostics.state.failed", { reason: s.reason });
    case "downloading":
      return t("diagnostics.state.downloading");
    case "restarting":
      return t("diagnostics.state.restarting", { attempt: s.attempt });
    default:
      return t(s.state === "starting" ? "diagnostics.state.starting" : "diagnostics.state.stopped");
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
            <Row label={t("diagnostics.appServer")}><span className="text-[13px]">{stateLabel(d, t)} · {d.appServerLaunches}×</span></Row>
            <Row label={t("diagnostics.gateway")}>
              <span className="text-[13px]">
                127.0.0.1:{d.gateway.port} · {t(d.gateway.reachable ? "diagnostics.gateway.ok" : "diagnostics.gateway.down")}
              </span>
            </Row>
            <Row label={t("diagnostics.mcp")}>
              <span className="flex flex-col items-end text-[13px]">
                {d.mcp.length === 0 && <span className="text-muted">{t("diagnostics.mcp.none")}</span>}
                {d.mcp.map((m) => (
                  <span key={m.name} className={m.error ? "text-danger" : undefined}>
                    {m.error ? t("diagnostics.mcp.error", { name: m.name, error: m.error }) : t("diagnostics.mcp.ok", { name: m.name, n: m.tools })}
                  </span>
                ))}
              </span>
            </Row>
            <Row label={t("diagnostics.worker")}>
              <span className="text-[13px]">
                {d.worker.installed
                  ? [
                      t("diagnostics.worker.installed"),
                      t(d.worker.running ? "diagnostics.worker.running" : "diagnostics.worker.idle"),
                      d.worker.gpu ? "GPU" : "CPU",
                      d.worker.model ?? t("diagnostics.worker.noModel"),
                      t("diagnostics.worker.spawns", { n: d.worker.spawns }),
                    ].join(" · ")
                  : t("diagnostics.worker.missing")}
              </span>
            </Row>
            <Row label={t("diagnostics.capture")}>
              <span className="text-[13px]">
                {d.capture.paused
                  ? t("diagnostics.capture.paused")
                  : [
                      d.capture.active.length ? d.capture.active.map((s) => t((SOURCE[s] ?? s) as never)).join(", ") : t("diagnostics.capture.none"),
                      ...(d.capture.recording ? [t("diagnostics.capture.recording")] : []),
                    ].join(" · ")}
              </span>
            </Row>
            <Row label={t("diagnostics.account")}>
              <span className="text-[13px]">
                {d.account
                  ? `${d.account.email ?? "—"} · ${t(d.account.signedIn ? "diagnostics.account.on" : "diagnostics.account.off")}`
                  : t("diagnostics.account.none")}
              </span>
            </Row>
            <Row label={t("diagnostics.disk")}>
              <span className="text-[13px]">
                {d.disk.freeBytes !== null
                  ? t("diagnostics.disk.value", { free: formatBytes(d.disk.freeBytes), used: formatBytes(d.disk.auraBytes) })
                  : t("diagnostics.disk.unknown", { used: formatBytes(d.disk.auraBytes) })}
              </span>
            </Row>
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
        <AuraLogo className="h-14 w-14" />
        <div>
          <p className="text-sm">{t("about.tagline")}</p>
          <p className="text-xs text-muted">{t("about.license")}</p>
        </div>
      </div>
      <Row label={t("updates.title")} hint={update?.kind === "none" ? t("updates.none") : update?.kind === "available" ? t("updates.available", { version: update.version }) : update?.kind === "unsupported" ? t("updates.unsupported") : update?.kind === "notConfigured" ? t("updates.notConfigured") : pct !== null ? `${pct}%` : undefined}>
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
