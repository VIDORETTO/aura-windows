import { useT } from "../i18n";
import { answerSpeechConsent, useSpeech } from "../lib/speech";
import { Button } from "./primitives";

/** One-time confirmation before answers go to a cloud voice (009 AC-006). */
export function SpeechConsentDialog() {
  const t = useT();
  const pending = useSpeech((s) => s.pendingConsent);
  if (!pending) return null;
  const title = t("speech.consent.title", { provider: pending.provider });
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 p-4">
      <div
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onKeyDown={(e) => {
          if (e.key === "Escape") void answerSpeechConsent(false);
        }}
        className="menu-surface w-full max-w-sm rounded-lg border border-line p-4 shadow-xl"
      >
        <h2 className="mb-2 text-sm font-semibold">{title}</h2>
        <p className="mb-4 text-[13px] text-muted">{t("speech.consent.body", { provider: pending.provider })}</p>
        <div className="flex justify-end gap-2">
          <Button size="sm" variant="ghost" onClick={() => void answerSpeechConsent(false)}>
            {t("common.cancel")}
          </Button>
          <Button size="sm" variant="primary" autoFocus onClick={() => void answerSpeechConsent(true)}>
            {t("speech.consent.accept")}
          </Button>
        </div>
      </div>
    </div>
  );
}
