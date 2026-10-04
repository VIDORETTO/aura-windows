// 012 AC-001: with the Overlay open, selecting text in another app and coming
// back to the Overlay brings that selection as a chip, without hiding and
// showing the Overlay. The other app is a QA WinForms text box
// (selection-target.ps1) that also gives the foreground back to the Overlay.
import { spawn } from "node:child_process";
import { appendFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

const selectionChips = () =>
  browser.execute(() => [...document.querySelectorAll("li")].map((li) => li.textContent ?? "").filter((t) => t.includes("❝")));

describe("Selection when returning to the Overlay (012)", () => {
  it("brings the current selection of the other app as one chip", async () => {
    // Aura starts in the tray; launching it again opens the Overlay.
    spawn(process.env.AURA_E2E_APP!, [], { stdio: "ignore", detached: true }).unref();
    await invoke("settings_update", { patch: { language: "en" } });
    await invoke("privacy_set_paused", { paused: false }).catch(() => undefined);
    await browser.refresh();
    await $("textarea").waitForDisplayed({ timeout: 30_000 });
    for (const c of await invoke("tray_list", { tray: "draft" }).catch(() => [])) await invoke("tray_remove", { tray: "draft", chipId: c.id }).catch(() => undefined);
    await browser.refresh();
    await $("textarea").waitForDisplayed({ timeout: 30_000 });

    const cmds = path.join(os.tmpdir(), `aura-qa-selection-${process.pid}.txt`);
    writeFileSync(cmds, "");
    const target = spawn("powershell", ["-NoProfile", "-STA", "-ExecutionPolicy", "Bypass", "-File", path.resolve("selection-target.ps1"), cmds, "90", path.basename(process.env.AURA_E2E_APP!, ".exe")], { stdio: "ignore" });
    try {
      await browser.pause(2500);
      for (const text of ["Aura QA seleção", "outro trecho selecionado"]) {
        appendFileSync(cmds, `select ${text}\n`, "utf8");
        await browser.pause(1500);
        appendFileSync(cmds, "focus-aura\n", "utf8");
        await browser.pause(1500);
        await browser.waitUntil(async () => (await selectionChips()).some((t) => t.includes(text.slice(0, 20))), {
          timeout: 10_000,
          timeoutMsg: `selection "${text}" did not reach the Overlay`,
        });
        const chips = await selectionChips();
        console.log("Native selection chips", JSON.stringify(chips));
        expect(chips).toHaveLength(1);
      }
      console.log("Native previous app", JSON.stringify((await invoke("previous_app"))?.title));
    } finally {
      target.kill();
      rmSync(cmds, { force: true });
    }
  });
});
