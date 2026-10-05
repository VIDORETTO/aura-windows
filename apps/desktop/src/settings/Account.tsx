import { LogOut } from "lucide-react";
import { api } from "../ipc/commands";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { Badge, Button, Row, Section } from "../ui/primitives";
import { EffortPresetsSection } from "./EffortPresets";

export function AccountSection() {
  return (
    <>
      <AccountCard />
      <EffortPresetsSection />
    </>
  );
}

function AccountCard() {
  const t = useT();
  const auth = useApp((s) => s.auth);
  const login = useApp((s) => s.login);
  const refresh = useApp((s) => s.refreshAuth);
  const active = auth?.active;
  return (
    <Section title={t("account.chatgpt")}>
      {active?.signedIn ? (
        <>
          <Row label={t("account.signedAs", { email: active.email ?? active.subject })}>
            <Button
              variant="ghost"
              onClick={async () => {
                await api.authLogout(active.clientId);
                await refresh();
              }}
            >
              <LogOut size={14} /> {t("account.signOut")}
            </Button>
          </Row>
          <Row label={t("account.planUsage")}>
            <Badge tone={active.planUsageEnabled ? "success" : "warning"}>{active.planUsageEnabled ? t("common.on") : t("common.off")}</Badge>
          </Row>
          <Row label={t("header.manageUsage")}>
            <Button variant="ghost" onClick={() => void api.openExternal("https://chatgpt.com/settings/usage")}>
              chatgpt.com/settings/usage
            </Button>
          </Row>
        </>
      ) : (
        <Row label={t("account.notSigned")} hint={login?.state === "waitingBrowser" ? t("login.waiting") : undefined}>
          <Button variant="primary" onClick={() => void api.authLogin()}>
            {t("account.signIn")}
          </Button>
        </Row>
      )}
      {(auth?.accounts.length ?? 0) > 1 &&
        auth!.accounts
          .filter((a) => a.clientId !== active?.clientId)
          .map((a) => (
            <Row key={a.clientId} label={a.email ?? a.subject}>
              <Button
                size="sm"
                onClick={async () => {
                  await api.authSwitch(a.clientId);
                  await refresh();
                }}
              >
                {t("voice.use")}
              </Button>
            </Row>
          ))}
    </Section>
  );
}
