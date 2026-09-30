import { Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { AccessLogEntry, AgentPermission, CaptureMode, PrivacyView, Source, SourcePolicy } from "../ipc/types";
import { useT } from "../i18n";
import { relativeTime } from "../lib/format";
import { useApp } from "../state/app";
import { Badge, Button, Row, Section, Select, Switch, TextField } from "../ui/primitives";
import { RecordingsSection } from "./Recordings";

const MODES = ["off", "onDemand", "recentBuffer", "manual", "continuous"] as const;

function modeFrom(key: string, minutes: number): CaptureMode {
  return key === "recentBuffer" ? { type: "recentBuffer", minutes } : ({ type: key } as CaptureMode);
}

function SourceRow({ source, policy, onChange }: { source: Source; policy: SourcePolicy; onChange: (p: PrivacyView) => void }) {
  const t = useT();
  const minutes = policy.mode.type === "recentBuffer" ? policy.mode.minutes : 10;
  const set = async (mode: CaptureMode, agent: AgentPermission) => {
    try {
      onChange(await api.privacySetSource(source, mode, agent));
    } catch (e) {
      useApp.getState().notify("error", errorMessage(e));
    }
  };
  const modes = source === "screen" ? MODES : MODES.filter((m) => m !== "continuous" || source !== "systemAudio");
  return (
    <div className="grid grid-cols-[1fr_auto_auto] items-center gap-3 py-2.5">
      <div className="text-sm font-medium">{t(`privacy.source.${source}` as "privacy.source.screen")}</div>
      <Select aria-label={t("privacy.mode")} value={policy.mode.type} onChange={(e) => void set(modeFrom(e.target.value, minutes), policy.agent)}>
        {modes.map((m) => (
          <option key={m} value={m}>
            {m === "recentBuffer" ? t("privacy.mode.recentBuffer", { n: minutes }) : t(`privacy.mode.${m}` as "privacy.mode.off")}
          </option>
        ))}
      </Select>
      <Select aria-label={t("privacy.agent")} value={policy.agent} onChange={(e) => void set(policy.mode, e.target.value as AgentPermission)}>
        <option value="never">{t("privacy.agent")}: {t("privacy.agent.never")}</option>
        <option value="ask">{t("privacy.agent")}: {t("privacy.agent.ask")}</option>
        <option value="always">{t("privacy.agent")}: {t("privacy.agent.always")}</option>
      </Select>
    </div>
  );
}

export function PrivacySection() {
  const t = useT();
  const privacy = useApp((s) => s.privacy);
  const setPrivacy = useApp((s) => s.setPrivacy);
  const lang = useApp((s) => s.settings?.language ?? "ptBr");
  const [log, setLog] = useState<AccessLogEntry[]>([]);
  const [process, setProcess] = useState("");
  const [title, setTitle] = useState("");
  useEffect(() => void api.privacyAccessLog(100).then(setLog), []);
  if (!privacy) return null;

  return (
    <>
      <Section title={t("settings.privacy")}>
        <Row label={t("privacy.pause")} hint={privacy.paused ? t("privacy.paused") : undefined}>
          <Switch label={t("privacy.pause")} checked={privacy.paused} onChange={async (v) => setPrivacy(await api.privacySetPaused(v))} />
        </Row>
        <SourceRow source="screen" policy={privacy.screen} onChange={setPrivacy} />
        <SourceRow source="mic" policy={privacy.mic} onChange={setPrivacy} />
        <SourceRow source="systemAudio" policy={privacy.systemAudio} onChange={setPrivacy} />
      </Section>
      <Section title={t("privacy.exclusions")} description={t("privacy.exclusions.hint")}>
        {privacy.exclusions.map((r) => (
          <div key={r.id} className="flex items-center gap-3 py-2">
            <div className="min-w-0 flex-1 font-mono text-[12px]">
              {r.process ?? ""}
              {r.titleGlob ? ` “${r.titleGlob}”` : ""}
              {r.class ? ` [${r.class}]` : ""}
            </div>
            {r.builtin && <Badge>{t("privacy.builtin")}</Badge>}
            <Switch label={r.id} checked={r.enabled} onChange={async (v) => setPrivacy(await api.privacyUpsertExclusion({ ...r, enabled: v }))} />
            {!r.builtin && (
              <Button size="sm" variant="danger" aria-label={t("common.delete")} onClick={async () => setPrivacy(await api.privacyRemoveExclusion(r.id))}>
                <Trash2 size={13} />
              </Button>
            )}
          </div>
        ))}
        <div className="flex gap-2 py-2.5">
          <TextField placeholder={t("privacy.exclusion.process")} value={process} onChange={(e) => setProcess(e.target.value)} />
          <TextField placeholder={t("privacy.exclusion.title")} value={title} onChange={(e) => setTitle(e.target.value)} />
          <Button
            disabled={!process.trim() && !title.trim()}
            onClick={async () => {
              setPrivacy(await api.privacyUpsertExclusion({ id: "", process: process.trim() || null, titleGlob: title.trim() || null, class: null, enabled: true, builtin: false }));
              setProcess("");
              setTitle("");
            }}
          >
            {t("common.add")}
          </Button>
        </div>
      </Section>
      <RecordingsSection />
      <Section
        title={t("privacy.log")}
        actions={
          <Button size="sm" variant="ghost" onClick={async () => { await api.privacyClearAccessLog(); setLog([]); }}>
            {t("privacy.log.clear")}
          </Button>
        }
      >
        {log.length === 0 && <p className="py-3 text-[13px] text-muted">{t("privacy.log.empty")}</p>}
        {log.map((e, i) => (
          <div key={i} className="flex items-center gap-3 py-1.5 text-[12px]">
            <span className="w-24 shrink-0 text-muted">{relativeTime(e.at, lang)}</span>
            <span className="w-20 shrink-0">{e.source}</span>
            <span className="flex-1 truncate font-mono">{e.tool ?? e.requester}</span>
            <Badge tone={e.decision === "deny" ? "danger" : e.decision === "ask" ? "warning" : "success"}>{e.decision}</Badge>
          </div>
        ))}
      </Section>
    </>
  );
}
