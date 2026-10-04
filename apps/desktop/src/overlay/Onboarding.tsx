import { KeyRound, Loader2 } from "lucide-react";
import { AuraLogo } from "../ui/AuraLogo";
import { api } from "../ipc/commands";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { Button } from "../ui/primitives";

/** First run: ChatGPT sign-in or bring-your-own-key (002 AC-001). */
export function LoginCard() {
  const t = useT();
  const login = useApp((s) => s.login);
  const waiting = login?.state === "waitingBrowser";
  return (
    <div className="flex flex-col items-center gap-3 px-6 py-6 text-center">
      <AuraLogo className="h-12 w-12" />
      <div>
        <h1 className="text-lg font-semibold">{t("login.title")}</h1>
        <p className="mt-1 max-w-sm text-[13px] text-muted">{t("login.subtitle")}</p>
      </div>
      {login?.state === "failed" && <p className="text-[13px] text-danger">{t("login.failed", { reason: login.reason })}</p>}
      {waiting ? (
        <div className="flex items-center gap-2 text-[13px]">
          <Loader2 size={14} className="animate-spin" /> {t("login.waiting")}
          <Button size="sm" variant="ghost" onClick={() => void api.authCancel()}>
            {t("login.cancel")}
          </Button>
        </div>
      ) : (
        <div className="flex flex-col gap-2 sm:flex-row">
          <Button variant="primary" onClick={() => void api.authLogin()} className="h-9 px-4">
            {/* Official "Continue with ChatGPT" asset goes here (brand guidelines; see HANDOFF). */}
            {t("login.chatgpt")}
          </Button>
          <Button onClick={() => void api.settingsOpen("providers")} className="h-9 px-4">
            <KeyRound size={15} /> {t("login.byok")}
          </Button>
        </div>
      )}
    </div>
  );
}

export function WelcomeModal() {
  const t = useT();
  const dismiss = useApp((s) => s.dismissWelcome);
  return (
    <div role="dialog" aria-modal="true" aria-labelledby="welcome-title" className="fade-in mx-4 my-3 rounded-md border border-accent/40 bg-accent/5 px-4 py-3">
      <h2 id="welcome-title" className="text-sm font-semibold">
        {t("welcome.title")}
      </h2>
      <p className="mt-1 text-[13px] text-muted">{t("welcome.body")}</p>
      <div className="mt-2 flex gap-2">
        <Button size="sm" variant="primary" autoFocus onClick={dismiss}>
          {t("welcome.ok")}
        </Button>
        <Button size="sm" variant="ghost" onClick={() => void api.openExternal("https://chatgpt.com/settings/usage")}>
          {t("header.manageUsage")}
        </Button>
      </div>
    </div>
  );
}
