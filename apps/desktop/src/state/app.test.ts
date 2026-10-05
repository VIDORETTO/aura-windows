import { vi } from "vitest";
import { api } from "../ipc/commands";
import { freshApp } from "../test/harness";
import { useApp } from "./app";

describe("notices", () => {
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
});
