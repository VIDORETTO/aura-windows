import { useEffect, useState } from "react";
import { api } from "../ipc/commands";
import type { CaptureMode, PrivacyView, RecentClip } from "../ipc/types";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { Button } from "../ui/primitives";

/** Minutes kept by a source, or 0 when it has no recent buffer. */
function buffered(mode: CaptureMode | undefined): number {
  if (!mode) return 0;
  if (mode.type === "recentBuffer") return mode.minutes;
  if (mode.type === "continuous") return 30;
  return 0;
}

/**
 * "@últimos minutos": the user attaches the last N minutes of the recent
 * buffer — screen keyframes, audio transcript (Microphone, System, Both) or
 * both of the same interval (004 AC-014, 005 AC-007/009).
 */
export function RecentClipDialog({ onAttach, onClose }: { onAttach: (clip: RecentClip) => Promise<void>; onClose: () => void }) {
  // Settings may have changed the policy in another window: read it fresh.
  const cached = useApp((s) => s.privacy);
  const [privacy, setPrivacy] = useState<PrivacyView | null>(null);
  useEffect(() => {
    let live = true;
    api.privacyGet().then((p) => live && setPrivacy(p), () => live && setPrivacy(cached));
    return () => {
      live = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  if (!privacy) return null;
  return <RecentClipForm privacy={privacy} onAttach={onAttach} onClose={onClose} />;
}

function RecentClipForm({ privacy, onAttach, onClose }: { privacy: PrivacyView; onAttach: (clip: RecentClip) => Promise<void>; onClose: () => void }) {
  const t = useT();
  const screenMax = buffered(privacy.screen.mode);
  const micMax = buffered(privacy.mic.mode);
  const systemMax = buffered(privacy.systemAudio.mode);
  const audioChoices = [
    micMax > 0 && "mic",
    systemMax > 0 && "system",
    micMax > 0 && systemMax > 0 && "both",
  ].filter(Boolean) as ("mic" | "system" | "both")[];
  const [screen, setScreen] = useState(screenMax > 0);
  const [audio, setAudio] = useState<"" | "mic" | "system" | "both">(audioChoices.includes("both") ? "both" : (audioChoices[0] ?? ""));
  const audioMax = audio === "mic" ? micMax : audio === "system" ? systemMax : audio === "both" ? Math.max(micMax, systemMax) : 0;
  const max = Math.max(1, Math.min(30, Math.max(screen ? screenMax : 0, audioMax)));
  const [minutes, setMinutes] = useState(String(Math.min(2, max)));
  const [busy, setBusy] = useState(false);
  const n = Number(minutes);
  const none = screenMax === 0 && audioChoices.length === 0;
  const valid = !none && (screen || audio !== "") && Number.isFinite(n) && n >= 1 && n <= max;

  const submit = async () => {
    setBusy(true);
    try {
      await onAttach({ minutes: n, screen, audio: audio || null });
      onClose();
    } finally {
      setBusy(false);
    }
  };

  return (
    <div
      role="dialog"
      aria-label={t("recent.title")}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.stopPropagation();
          onClose();
        }
      }}
      className="menu-surface flex flex-col gap-2 rounded-md border border-line p-3 text-sm shadow-lg"
    >
      <div className="font-medium">{t("recent.title")}</div>
      {none ? (
        <p className="text-xs text-muted">{t("recent.none")}</p>
      ) : (
        <>
          <label className="flex items-center justify-between gap-3">
            <span>{t("recent.minutes")}</span>
            <input
              type="number"
              min={1}
              max={max}
              value={minutes}
              aria-label={t("recent.minutes")}
              onChange={(e) => setMinutes(e.target.value)}
              className="h-7 w-16 rounded-md border border-line bg-transparent px-2 text-[13px]"
            />
          </label>
          {screenMax > 0 && (
            <label className="flex items-center gap-2">
              <input type="checkbox" checked={screen} onChange={(e) => setScreen(e.target.checked)} />
              {t("context.screen")}
            </label>
          )}
          {audioChoices.length > 0 && (
            <label className="flex items-center justify-between gap-3">
              <span>{t("recent.audio")}</span>
              <select
                aria-label={t("recent.audio")}
                value={audio}
                onChange={(e) => setAudio(e.target.value as typeof audio)}
                className="h-7 rounded-md border border-line bg-transparent px-1 text-[13px]"
              >
                <option value="">{t("recent.audio.none")}</option>
                {audioChoices.map((a) => (
                  <option key={a} value={a}>
                    {t(`recent.audio.${a}` as never)}
                  </option>
                ))}
              </select>
            </label>
          )}
          <p className="text-xs text-muted">{t("recent.hint")}</p>
        </>
      )}
      <div className="flex justify-end gap-2">
        <Button size="sm" variant="ghost" onClick={onClose}>
          {t("common.cancel")}
        </Button>
        <Button size="sm" variant="primary" disabled={!valid || busy} onClick={() => void submit()}>
          {t("recent.attach")}
        </Button>
      </div>
    </div>
  );
}
