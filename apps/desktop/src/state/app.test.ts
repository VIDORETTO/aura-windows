import { vi } from "vitest";
import { api } from "../ipc/commands";
import { freshApp } from "../test/harness";
import { useApp } from "./app";

describe("notices", () => {
  it.each([
    ["ptBr", "Este modelo não suporta ferramentas. A pesquisa na internet exige um modelo com esse suporte; a conversa continua disponível."],
    ["en", "This model does not support tools. Internet research requires a model with tool support; ordinary chat remains available."],
  ] as const)("explains unavailable web tools in %s", async (language, message) => {
    await freshApp({ signedIn: true });
    useApp.getState().setSettings({ ...useApp.getState().settings!, language });
    useApp.getState().handle({ channel: "notice", event: { level: "warning", message: "web_tools_unsupported" } });
    expect(useApp.getState().notices.map((notice) => notice.message)).toEqual([message]);
  });

  it("the same message is shown once", async () => {
    await freshApp({ signedIn: true });
    const msg = "O atalho para abrir o Aura está em uso por outro app.";
    useApp.getState().notify("warning", msg);
    useApp.getState().notify("warning", msg);
    useApp.getState().notify("warning", "outro aviso");
    expect(useApp.getState().notices.map((n) => n.message)).toEqual([msg, "outro aviso"]);
  });
});

describe("reminders (020)", () => {
  it("a due reminder becomes one Windows notification in the Overlay window", async () => {
    await freshApp({ signedIn: true });
    const notify = vi.spyOn(api, "notify").mockResolvedValue(undefined);
    useApp.getState().handle({ channel: "reminder", event: { id: "r1", text: "Ligar para o João" } });
    expect(notify).toHaveBeenCalledWith("Aura", "Ligar para o João");
    window.location.hash = "#/settings/general";
    useApp.getState().handle({ channel: "reminder", event: { id: "r2", text: "Outro" } });
    expect(notify).toHaveBeenCalledTimes(1);
    window.location.hash = "";
  });

  it("in broadcast mode the reminder stays inside Aura, with no Windows notification", async () => {
    await freshApp({ signedIn: true });
    const notify = vi.spyOn(api, "notify").mockResolvedValue(undefined);
    useApp.getState().setSettings({ ...useApp.getState().settings!, broadcastMode: true });
    useApp.getState().handle({ channel: "reminder", event: { id: "r3", text: "Ligar para a Ana" } });
    expect(notify).not.toHaveBeenCalled();
    expect(useApp.getState().notices.map((n) => n.message)).toContain("Ligar para a Ana");
  });
});
