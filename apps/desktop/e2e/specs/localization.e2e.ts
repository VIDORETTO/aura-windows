import { writeFile } from "node:fs/promises";
import path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

describe("Locale consistency (QA-004)", () => {
  it("localizes the actual Settings window title when English is selected", async () => {
    await browser.execute(async () => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      await invoke("settings_update", { patch: { language: "en" } });
      await invoke("settings_open", { section: "diagnostics" });
    });
    const handles = await browser.getWindowHandles();
    console.log("Native QA localization window handles", handles);
    const title = await browser.execute(async () => {
      try {
        return await (window as any).__TAURI_INTERNALS__.invoke("plugin:window|title", { label: "settings" });
      } catch (error) { return { problem: JSON.stringify(error) }; }
    });
    console.log("Native Settings title", title);
    expect(title).toBe("Aura — Settings");
    await browser.execute(async () => {
      await (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language: "ptBr" } });
    });
    const portuguese = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("plugin:window|title", { label: "settings" }));
    expect(portuguese).toBe("Aura — Configurações");
    await browser.execute(async () => {
      await (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language: "en" } });
    });
    const englishAgain = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("plugin:window|title", { label: "settings" }));
    expect(englishAgain).toBe("Aura — Settings");
  });

  it("localizes mounted Settings fields, builtins, attachment counts and accessible menus", async () => {
    const overlay = await browser.getWindowHandle();
    const file = path.resolve(import.meta.dirname, "../../../../target/qa-tools/1 linhas.txt");
    await writeFile(file, "one line\n", "utf8");
    await browser.execute(async (filePath) => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      await invoke("attach_file", { tray: "draft", threadId: null, path: filePath });
      await invoke("quick_save", { name: "qa-custom", template: "Meu texto permanece em português: {texto}", replace: true });
    }, file);
    // Mount the persisted synthetic chip once, then change language without reload.
    await browser.refresh();
    await $('ul[aria-label="Context"]').waitForDisplayed();
    expect(await $('ul[aria-label="Context"]').getText()).toContain("1 linhas.txt · 1 line");
    await $("textarea").click();
    await browser.keys(["Control", "a"]);
    await browser.keys("Backspace");
    await $("textarea").addValue("/tldr");
    console.log("Native localized draft", await $("textarea").getValue());
    await $('ul[role="listbox"][aria-label="Suggestions"]').waitForDisplayed();
    expect(await $('ul[role="listbox"]').getText()).toContain("Summarize in up to three sentences");
    await $("textarea").setValue("");
    const settings = (await browser.getWindowHandles()).find((handle) => handle !== overlay)!;
    await browser.switchToWindow(settings);
    const diagnostics = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("diagnostics"));
    const state = (diagnostics as any).appServer.state as string;
    const stateText: Record<string, string> = { ready: "Ready (", failed: "Failed:", stopped: "Stopped", starting: "Starting", downloading: "Downloading", restarting: "Restarting (" };
    await browser.waitUntil(async () => (await $("main").getText()).includes(stateText[state]));
    console.log("Native Diagnostics localized state", state, stateText[state]);
    await $('a=Extensions').click();
    await $('button=Add server').click();
    for (const text of ["Name", "Transport", "Secret variable (optional)", "Secret value"]) {
      expect(await $("body").getText()).toContain(text);
    }
    await $('select').selectByAttribute("value", "http");
    expect(await $("body").getText()).toContain("Token (optional)");
    expect(await $("body").getText()).toContain("Summarize in up to three sentences");
    expect(await $("body").getText()).toContain("Meu texto permanece em português");
    await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language: "ptBr" } }));
    await $('a=Extensões').waitForDisplayed();
    await browser.waitUntil(async () => (await $("body").getText()).includes("Resuma em até três frases"));
    expect(await $("body").getText()).toContain("Meu texto permanece em português");
    await browser.switchToWindow(overlay);
    await $('ul[aria-label="Contexto"]').waitForDisplayed();
    expect(await $('ul[aria-label="Contexto"]').getText()).toContain("1 linhas.txt · 1 linha");
    await $("textarea").click();
    await browser.keys(["Control", "a"]);
    await browser.keys("Backspace");
    await $("textarea").addValue("/tldr");
    console.log("Native localized draft", await $("textarea").getValue());
    await $('ul[role="listbox"][aria-label="Sugestões"]').waitForDisplayed();
    expect(await $('ul[role="listbox"]').getText()).toContain("Resuma em até três frases");
    await $("textarea").setValue("");
    await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language: "en" } }));
    await $('ul[aria-label="Context"]').waitForDisplayed();
    expect(await $('ul[aria-label="Context"]').getText()).toContain("1 linhas.txt · 1 line");
  });

  it("updates the existing native tray menu without restarting", async () => {
    const probe = path.resolve(import.meta.dirname, "../../../../target/debug/examples/qa_tray.exe");
    const processName = path.basename(process.env.AURA_E2E_APP!);
    for (const [language, expected] of [
      ["en", ["Open Aura", "New conversation", "Pause/resume privacy", "Settings…", "Quit"]],
      ["ptBr", ["Abrir Aura", "Nova conversa", "Pausar/retomar privacidade", "Configurações…", "Sair"]],
      ["en", ["Open Aura", "New conversation", "Pause/resume privacy", "Settings…", "Quit"]],
    ] as const) {
      await browser.execute(async (lang) => (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language: lang } }), language);
      const { stdout } = await promisify(execFile)(probe, [processName], { timeout: 10_000 });
      const texts = stdout.trim().split(/\r?\n/);
      console.log("Native QA tray", language, texts);
      expect(texts).toEqual(expected);
    }
  });
});
