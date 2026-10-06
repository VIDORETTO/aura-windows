// Production Windows shell, real persistence and IPC; no account or inference.
import { spawn } from "node:child_process";
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

describe("0.2.0 Windows release journeys", () => {
  before(async () => {
    spawn(process.env.AURA_E2E_APP!, [], { stdio: "ignore", detached: true }).unref();
    await invoke("settings_update", { patch: { language: "en" } });
    await browser.refresh();
  });

  it("keeps notes and saved answers across a process restart and searches without accents", async () => {
    await invoke("note_add", { kind: "note", text: "QA renovar seguro em março" });
    await invoke("note_add", { kind: "saved", text: "QA resposta salva" });
    expect((await invoke("notes_all", { kind: "note", query: "marco" }) as any[]).map(n => n.text)).toContain("QA renovar seguro em março");
    await browser.reloadSession();
    expect((await invoke("notes_all", { kind: "saved", query: "resposta" }) as any[]).map(n => n.text)).toContain("QA resposta salva");
    const overlay = await browser.getWindowHandle();
    await invoke("settings_open", { section: "personal" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find(h => h !== overlay)!);
    await $("div=QA renovar seguro em março").waitForDisplayed();
    await $("div=QA resposta salva").waitForDisplayed();
    expect(await invoke("reminders_all")).toEqual([]);
    await $('button[aria-label="Delete: QA renovar seguro em março"]').click();
    await browser.waitUntil(async () => (await invoke("notes_all", { kind: "note", query: "marco" }) as any[]).length === 0);
  });

  it("exposes the new meeting, recipes and project contracts through the native shell", async () => {
    expect(await invoke("meeting_active")).toBe(null);
    expect((await invoke("recipes_list") as any[]).length).toBeGreaterThanOrEqual(8);
    expect(await invoke("meetings_list")).toEqual([]);
    expect(await invoke("projects_list")).toEqual([]);
    await invoke("meeting_set_brief", { text: "QA briefing sem iniciar gravação" });
    expect(await invoke("meeting_brief")).toBe("QA briefing sem iniciar gravação");
    expect(await invoke("meeting_active")).toBe(null);
  });
});
