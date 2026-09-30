import { useState } from "react";
import type { Settings } from "../ipc/types";
import { useT, type MessageKey } from "../i18n";
import { chordFromEvent } from "../lib/format";
import { useApp } from "../state/app";
import { Kbd, Row, Section, Switch, cx } from "../ui/primitives";
import { updateSettings } from "./General";

type Key = "invokeShortcut" | "pushToTalkShortcut" | "globalVoiceShortcut" | "privacyPauseShortcut";

const ROWS: [Key, MessageKey][] = [
  ["invokeShortcut", "shortcuts.invoke"],
  ["pushToTalkShortcut", "shortcuts.pushToTalk"],
  ["globalVoiceShortcut", "shortcuts.globalVoice"],
  ["privacyPauseShortcut", "shortcuts.pause"],
];

function Recorder({ value, onChange, label }: { value: string; onChange: (v: string) => void; label: string }) {
  const t = useT();
  const [recording, setRecording] = useState(false);
  return (
    <button
      type="button"
      aria-label={label}
      className={cx("min-w-40 rounded-md border px-2 py-1 text-left text-[13px]", recording ? "border-accent" : "border-line")}
      onClick={() => setRecording(true)}
      onBlur={() => setRecording(false)}
      onKeyDown={(e) => {
        if (!recording) return;
        e.preventDefault();
        if (e.key === "Escape") return setRecording(false);
        const chord = chordFromEvent(e);
        if (chord) {
          onChange(chord);
          setRecording(false);
        }
      }}
    >
      {recording ? <span className="text-muted">{t("shortcuts.record")}</span> : <Kbd>{value}</Kbd>}
    </button>
  );
}

export function ShortcutsSection() {
  const t = useT();
  const s = useApp((x) => x.settings) as Settings | null;
  if (!s) return null;
  return (
    <Section title={t("settings.shortcuts")}>
      {ROWS.map(([key, label]) => (
        <Row key={key} label={t(label)}>
          <Recorder label={t(label)} value={s[key]} onChange={(v) => void updateSettings({ [key]: v })} />
        </Row>
      ))}
      <Row label={t("shortcuts.doubleTap")}>
        <Switch label={t("shortcuts.doubleTap")} checked={s.doubleTapCtrl} onChange={(v) => void updateSettings({ doubleTapCtrl: v })} />
      </Row>
    </Section>
  );
}
