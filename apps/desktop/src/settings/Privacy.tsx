import { Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import type { AccessLogEntry, AgentPermission, CaptureMode, PrivacyView, Source, SourcePolicy } from "../ipc/types";
import { useT } from "../i18n";
import { exactTime } from "../lib/format";
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
  const [draftMinutes, setDraftMinutes] = useState(String(minutes));
  useEffect(() => setDraftMinutes(String(minutes)), [minutes]);
  /** 1–30 minutes (004 AC-013); out-of-range values snap to the limit. */
  const commitMinutes = () => {
    const n = Math.min(30, Math.max(1, Math.round(Number(draftMinutes) || minutes)));
    setDraftMinutes(String(n));
    if (n !== minutes) void set({ type: "recentBuffer", minutes: n }, policy.agent);
  };
  const label = t(`privacy.source.${source}` as "privacy.source.screen");
  return (
    <div className="grid grid-cols-[1fr_auto_auto_auto] items-center gap-3 py-2.5">
      <div className="text-sm font-medium">{label}</div>
      <Select aria-label={t("privacy.mode")} value={policy.mode.type} onChange={(e) => void set(modeFrom(e.target.value, minutes), policy.agent)}>
        {modes.map((m) => (
          <option key={m} value={m}>
            {m === "recentBuffer" ? t("privacy.mode.recentBuffer", { n: minutes }) : t(`privacy.mode.${m}` as "privacy.mode.off")}
          </option>
        ))}
      </Select>
      {policy.mode.type === "recentBuffer" ? (
        <label className="flex items-center gap-1 text-[12px] text-muted">
          <input
            type="number"
            min={1}
            max={30}
            aria-label={t("privacy.buffer.minutes", { source: label })}
            value={draftMinutes}
            onChange={(e) => setDraftMinutes(e.target.value)}
            onBlur={commitMinutes}
            onKeyDown={(e) => e.key === "Enter" && commitMinutes()}
            className="h-8 w-14 rounded-md border border-line bg-transparent px-1.5 text-[13px] text-fg"
          />
          min
        </label>
      ) : (
        <span />
      )}
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
      <RetentionSection />
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
        {log.length > 0 && (
          <ul>
            {log.map((e) => (
              <AccessLogRow key={e.id} entry={e} lang={lang} />
            ))}
          </ul>
        )}
      </Section>
    </>
  );
}

const SOURCE_KEYS: Record<string, string> = {
  screen: "privacy.source.screen",
  mic: "privacy.source.mic",
  systemAudio: "privacy.source.systemAudio",
};

/** Localized reason of an access-log entry (stable codes from the host). */
function useReason() {
  const t = useT();
  return (reason: string | null): string | null => {
    if (!reason) return null;
    const covered = /^covered:(\d+)$/.exec(reason);
    if (covered) return t("privacy.log.reason.covered", { n: covered[1] });
    const key = `privacy.log.reason.${reason}`;
    const text = t(key as never);
    return text === key ? reason : text;
  };
}

/** One access: exact time, source, tool, outcome, thumbnail and conversation (QA-031). */
function AccessLogRow({ entry: e, lang }: { entry: AccessLogEntry; lang: "ptBr" | "en" }) {
  const t = useT();
  const reasonText = useReason();
  const notify = useApp((s) => s.notify);
  const [thumb, setThumb] = useState<string | null>(null);
  const when = exactTime(e.at, lang);
  const tool = e.tool ?? e.requester;
  const source = SOURCE_KEYS[e.source] ? t(SOURCE_KEYS[e.source] as never) : e.source;
  const reason = reasonText(e.reason);
  const showThumb = async () => {
    try {
      setThumb(await api.privacyAccessThumbnail(e.id));
    } catch (err) {
      notify("error", errorMessage(err));
    }
  };
  const openConversation = async () => {
    try {
      await api.privacyOpenConversation(e.threadId!);
    } catch (err) {
      notify("error", errorMessage(err));
    }
  };
  return (
    <li aria-label={`${tool} · ${source} · ${when}`} className="border-b border-line py-2 text-[12px] last:border-0">
      <div className="flex items-center gap-3">
        <time dateTime={new Date(e.at * 1000).toISOString()} className="w-36 shrink-0 tabular-nums text-muted">
          {when}
        </time>
        <span className="w-24 shrink-0">{source}</span>
        <span className="flex-1 truncate font-mono">{tool}</span>
        <Badge tone={e.decision === "deny" ? "danger" : e.decision === "ask" ? "warning" : "success"}>
          {t(`privacy.log.decision.${e.decision}` as never)}
        </Badge>
      </div>
      {(reason || e.hasThumbnail || e.threadId) && (
        <div className="mt-1 flex flex-wrap items-center gap-2 pl-[9.75rem] text-muted">
          {reason && <span>{reason}</span>}
          <span className="flex-1" />
          {e.hasThumbnail && !thumb && (
            <Button size="sm" variant="ghost" onClick={showThumb}>
              {t("privacy.log.thumbnail")}
            </Button>
          )}
          {e.threadId && (
            <Button size="sm" variant="ghost" onClick={openConversation}>
              {t("privacy.log.openConversation")}
            </Button>
          )}
        </div>
      )}
      {thumb && (
        <img src={thumb} alt={t("privacy.log.thumbnailAlt")} className="mt-2 ml-[9.75rem] max-h-40 rounded-md border border-line" />
      )}
    </li>
  );
}

/** Limits for continuous recordings (004 AC-016, 005 AC-006). */
function RetentionSection() {
  const t = useT();
  const privacy = useApp((s) => s.privacy);
  const setPrivacy = useApp((s) => s.setPrivacy);
  const retention = privacy?.retention ?? { days: 7, maxGb: 20, applyToManual: false };
  const [days, setDays] = useState(String(retention.days));
  const [gb, setGb] = useState(String(retention.maxGb));
  const [manual, setManual] = useState(retention.applyToManual);
  useEffect(() => {
    setDays(String(retention.days));
    setGb(String(retention.maxGb));
    setManual(retention.applyToManual);
  }, [retention.days, retention.maxGb, retention.applyToManual]);
  const d = Number(days);
  const g = Number(gb);
  const valid = Number.isInteger(d) && d >= 1 && d <= 365 && Number.isInteger(g) && g >= 1 && g <= 1000;
  const dirty = d !== retention.days || g !== retention.maxGb || manual !== retention.applyToManual;
  const input = "h-8 w-20 rounded-md border border-line bg-transparent px-2 text-[13px]";
  return (
    <Section title={t("privacy.retention")} description={t("privacy.retention.when")}>
      <Row label={t("privacy.retention.days")}>
        <input type="number" min={1} max={365} aria-label={t("privacy.retention.days")} value={days} onChange={(e) => setDays(e.target.value)} className={input} />
      </Row>
      <Row label={t("privacy.retention.space")}>
        <input type="number" min={1} max={1000} aria-label={t("privacy.retention.space")} value={gb} onChange={(e) => setGb(e.target.value)} className={input} />
      </Row>
      <Row label={t("privacy.retention.manual")}>
        <Switch label={t("privacy.retention.manual")} checked={manual} onChange={setManual} />
      </Row>
      {!valid && <p role="alert" className="text-xs text-danger">{t("privacy.retention.invalid")}</p>}
      <div className="flex justify-end py-2">
        <Button
          size="sm"
          variant="primary"
          disabled={!valid || !dirty}
          onClick={async () => {
            try {
              setPrivacy(await api.privacySetRetention(d, g, manual));
            } catch (e) {
              useApp.getState().notify("error", errorMessage(e));
            }
          }}
        >
          {t("privacy.retention.save")}
        </Button>
      </div>
    </Section>
  );
}