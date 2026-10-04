// QA-036: an app profile chooses the default model in Settings; opening the
// Overlay (global shortcut) over that app starts the conversation with it.
// The app is the QA WinForms window (insert-target.ps1).
import { spawn, spawnSync } from "node:child_process";
import { existsSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { responsesUpstream } from "../responses";

const ps = (cmd: string) => spawnSync("powershell", ["-NoProfile", "-STA", "-Command", cmd], { encoding: "utf8", timeout: 30_000 });
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

describe("Profile default model (QA-036)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA profile answer", [{ id: "qa-model", architecture: { input_modalities: ["text"] }, supported_parameters: ["tools"] }, { id: "qa-other", architecture: { input_modalities: ["text"] }, supported_parameters: ["tools"] }]); });
  after(async () => { await up.close(); });

  it("saves the model in the profile and uses it when the Overlay opens over the app", async () => {
    const overlay = await browser.getWindowHandle();
    await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en", ttsProvider: null, autoRead: false } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      for (const p of await i("profiles_list")) await i("profiles_delete", { id: p.id });
      const m = await i("providers_save", { draft: { name: "QA model", preset: "ollama", baseUrl: url }, credential: null });
      await i("providers_test", { id: m.id });
    }, up.baseUrl);

    // Settings › Profiles: create the profile and pick the model.
    await invoke("settings_open", { section: "profiles" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    const settings = (await browser.getWindowHandles()).find((h) => h !== overlay)!;
    await browser.switchToWindow(settings);
    await (await $("button=Add")).click();
    const fields = await $$("input");
    await fields[0].setValue("QA target");
    await fields[1].setValue("powershell.exe");
    await fields[2].setValue("QA Insert Target");
    const model = await $('select[aria-label="Default model"]');
    await browser.waitUntil(async () => (await model.getText()).includes("QA model · qa-other"), { timeout: 15_000 });
    await model.selectByVisibleText("QA model · qa-other");
    await (await $("button=Save")).click();
    await browser.waitUntil(async () => (await invoke("profiles_list")).length === 1);
    const saved = (await invoke("profiles_list"))[0];
    console.log("Native profile", JSON.stringify({ processPattern: saved.processPattern, titleGlob: saved.titleGlob, defaultModel: saved.defaultModel.replace(/aura-[^:]+::/, "aura-<provider>::") }));
    expect(saved.defaultModel).toMatch(/^aura-qa-model-[0-9a-f]+::qa-other$/);
    await browser.closeWindow();

    // Overlay opened over the QA app with the global shortcut.
    await browser.switchToWindow(overlay);
    await browser.refresh();
    await $("textarea").waitForDisplayed({ timeout: 30_000 });
    await invoke("overlay_hide");
    const out = path.join(os.tmpdir(), `aura-qa-profile-${process.pid}.txt`);
    rmSync(out, { force: true });
    const target = spawn("powershell", ["-NoProfile", "-STA", "-ExecutionPolicy", "Bypass", "-File", path.resolve("insert-target.ps1"), out, "90"], { stdio: "ignore" });
    try {
      await browser.waitUntil(() => existsSync(out), { timeout: 20_000, timeoutMsg: "target window not up" });
      await browser.pause(800);
      ps("(New-Object -ComObject WScript.Shell).SendKeys('^+ ')");
      await browser.waitUntil(async () => (await invoke("previous_app"))?.title === "QA Insert Target", { timeout: 15_000, timeoutMsg: "Overlay not opened over the QA app" });
      const active = await invoke("profile_active");
      console.log("Native active profile", active?.name);
      expect(active?.name).toBe("QA target");
      await (await $("textarea")).click();
      await browser.keys(["Control", "ArrowDown"]);
      await browser.keys(["Control"]);
      const button = await $('button[aria-label^="Model "]');
      await button.waitForExist({ timeout: 10_000 });
      await browser.waitUntil(async () => (await button.getAttribute("aria-label")).includes("qa-other"), { timeout: 10_000, timeoutMsg: "profile model not selected" });
      console.log("Native model button", await button.getAttribute("aria-label"));
      const box = await $("textarea");
      await box.setValue("QA which model?");
      await browser.keys("Enter");
      await browser.waitUntil(() => up.requests.some((r) => r.url === "/v1/responses"), { timeout: 180_000, timeoutMsg: "no turn" });
      const sent = up.requests.find((r) => r.url === "/v1/responses")!;
      console.log("Native turn model", sent.body?.model);
      expect(sent.body?.model).toBe("qa-other");
    } finally {
      target.kill();
      rmSync(out, { force: true });
      for (const p of await invoke("profiles_list")) await invoke("profiles_delete", { id: p.id });
    }
  });
});
