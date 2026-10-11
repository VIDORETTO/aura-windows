// 038: actual WebView2/window geometry and keyboard, with an isolated QA profile.
import { execFile, spawn } from "node:child_process";
import path from "node:path";
import { promisify } from "node:util";

const modelName = "QA reasoner com um nome muito longo para verificar a escolha do modelo no cabeçalho do Aura";
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

async function compact() {
  await $("textarea").click();
  await browser.keys(["Control", "ArrowUp"]);
  await $('button[aria-label^="Modo:"]').waitForDisplayed({ timeout: 10_000 });
}

async function selectModel() {
  await $('header button[aria-label^="Modelo"]').click();
  const picker = await $('[role="dialog"][aria-label="Modelo e esforço"]');
  await picker.waitForDisplayed();
  await (await picker.$('[role="menu"][aria-label="Modelo"]')).$(`button*=${modelName}`).click();
  await (await picker.$('[role="menu"][aria-label="Esforço de raciocínio"]')).$("button=Máximo").click();
  await browser.keys("Escape");
}

describe("Visible model/effort and keyboard in the native Overlay (038)", () => {
  before(async () => {
    await invoke("settings_update", { patch: { language: "ptBr", onboarded: true, attachScreenOnOpen: false, effortPresets: {} } });
    for (const p of await invoke("providers_list")) await invoke("providers_remove", { id: p.id });
    const p = await invoke("providers_save", { draft: { name: "QA", preset: "custom", wire: "responses", baseUrl: "http://127.0.0.1:9/v1", auth: "none" }, credential: null });
    await invoke("providers_model_save", { id: p.id, model: { id: "qa-reasoner", displayName: modelName, contextWindow: null, maxOutput: null, supportsImages: false, supportsTools: true, supportsReasoning: true, estimated: false, manual: true, efforts: ["low", "high", "max"], defaultEffort: "high" } });
    await browser.refresh();
    spawn(process.env.AURA_E2E_APP!, [], { stdio: "ignore", detached: true }).unref();
    await $("textarea").waitForDisplayed({ timeout: 20_000 });
  });

  it("keeps the model label, arrow and menu actionable at 480 logical pixels (AC-008)", async () => {
    await compact();
    await selectModel();
    await $('button[aria-label="Modo: Chat"]').click();
    await (await $('[role="dialog"][aria-label="Modo"]')).$("button*=Tarefa").click();
    await selectModel();
    await invoke("privacy_set_paused", { paused: true });
    const monitors = await invoke("plugin:window|available_monitors");
    if (monitors.length) {
      const smallest = [...monitors].sort((a, b) => a.size.width * a.size.height - b.size.width * b.size.height)[0];
      const position = await invoke("plugin:window|outer_position", { label: "overlay" });
      const helper = path.resolve(import.meta.dirname, "../../../../target/debug/examples/qa_resize.exe");
      await promisify(execFile)(helper, [path.basename(process.env.AURA_E2E_APP!), "Move", String(smallest.position.x + 40 - position.x), String(smallest.position.y + 40 - position.y)], { timeout: 10_000 });
    }
    // Apply synthetic OS context after moving/focusing the actual window.
    await invoke("plugin:event|emit", { event: "aura://overlay", payload: { previousApp: { processName: "Aplicativo anterior com nome extenso", title: "Relatório com um título muito longo para validar o espaço do seletor de modelo", window: 1, pid: 1, monitorId: "\\\\.\\DISPLAY1" } } });
    await browser.waitUntil(() => browser.execute(() => document.querySelector("header")?.textContent?.includes("Relatório com um título muito longo") ?? false), { timeout: 10_000 });
    for (const expanded of [false, true]) {
      if (expanded) await $('button[aria-label="Expandir"]').click();
      await invoke("plugin:window|set_size", { label: "overlay", value: { Logical: { width: 480, height: expanded ? 650 : 160 } } });
      await browser.pause(500);
      const geometry = await browser.execute(async () => {
        const button = document.querySelector<HTMLButtonElement>('header button[aria-label^="Modelo"]')!;
        const b = button.getBoundingClientRect();
        const label = button.querySelector("span")!.getBoundingClientRect();
        const arrow = button.querySelector("svg")!.getBoundingClientRect();
        const hit = document.elementFromPoint(b.left + b.width / 2, b.top + b.height / 2);
        const nativeScale = await (window as any).__TAURI_INTERNALS__.invoke("plugin:window|scale_factor", { label: "overlay" });
        const controls = [...document.querySelectorAll('header button, header span[title]')]
          .map((el) => ({ name: el.getAttribute("aria-label") ?? el.getAttribute("title"), rect: el.getBoundingClientRect() }))
          .filter(({ rect }) => rect.width > 0 && rect.height > 0)
          .sort((a, b) => a.rect.left - b.rect.left);
        const overlaps = controls.slice(1).filter(({ rect }, i) => controls[i].rect.right > rect.left + 1).map(({ name }) => name);
        return { width: innerWidth, height: innerHeight, dpi: nativeScale * 100, dpr: devicePixelRatio, buttonWidth: b.width, labelWidth: label.width, left: b.left, right: b.right, arrowRight: arrow.right, clickable: !!hit && button.contains(hit), name: button.getAttribute("aria-label"), overlaps };
      });
      console.log("038 native geometry", JSON.stringify({ expanded, ...geometry }));
      if (process.env.AURA_E2E_SCREENSHOT_DIR) await browser.saveScreenshot(path.join(process.env.AURA_E2E_SCREENSHOT_DIR, `model-picker-${expanded ? "expanded" : "compact"}.png`));
      expect(geometry.buttonWidth).toBeGreaterThan(0);
      expect(geometry.labelWidth).toBeGreaterThan(0);
      expect(geometry.left).toBeGreaterThanOrEqual(0);
      expect(geometry.right).toBeLessThanOrEqual(geometry.width);
      expect(geometry.arrowRight).toBeLessThanOrEqual(geometry.right + 1);
      expect(geometry.clickable).toBe(true);
      expect(geometry.name).toContain(modelName);
      expect(geometry.overlaps).toEqual([]);
      await $('header button[aria-label^="Modelo"]').click();
      const menu = await $('[role="dialog"][aria-label^="Modelo e"]');
      await menu.waitForDisplayed();
      await browser.waitUntil(() => browser.execute(() => {
        const r = document.querySelector('[role="dialog"][aria-label^="Modelo e"]')!.getBoundingClientRect();
        return r.top >= 0 && r.bottom <= innerHeight + 1 && r.left >= 0 && r.right <= innerWidth + 1;
      }), { timeout: 10_000, timeoutMsg: "model/effort menu is clipped by the native window" });
      const monitorFit = await browser.execute(async () => {
        const ipc = (window as any).__TAURI_INTERNALS__.invoke;
        const position = await ipc("plugin:window|outer_position", { label: "overlay" });
        const scale = await ipc("plugin:window|scale_factor", { label: "overlay" });
        const all = await ipc("plugin:window|available_monitors");
        const monitor = all.find((m: any) => position.x >= m.position.x && position.x < m.position.x + m.size.width && position.y >= m.position.y && position.y < m.position.y + m.size.height);
        const menuRect = document.querySelector('[role="dialog"][aria-label^="Modelo e"]')!.getBoundingClientRect();
        const area = monitor.workArea;
        return { fits: position.x + menuRect.left * scale >= area.position.x && position.x + menuRect.right * scale <= area.position.x + area.size.width && position.y + menuRect.top * scale >= area.position.y && position.y + menuRect.bottom * scale <= area.position.y + area.size.height, monitor: monitor.name };
      });
      expect(monitorFit.fits).toBe(true);
      await (await menu.$('[role="menu"][aria-label="Esforço de raciocínio"]')).$("button=Alto").click();
      await browser.keys("Escape");
    }
  });

  it("opens, selects effort and closes by keyboard in pt-BR and en (AC-009)", async () => {
    for (const language of ["ptBr", "en"]) {
      await invoke("settings_update", { patch: { language } });
      await browser.refresh();
      await $("textarea").waitForDisplayed({ timeout: 20_000 });
      const prefix = language === "ptBr" ? "Modelo" : "Model";
      await $(`header button[aria-label^="${prefix}"]`).click();
      const modelMenu = language === "ptBr" ? "Modelo" : "Model";
      await (await $(`[role="menu"][aria-label="${modelMenu}"]`)).$(`button*=${modelName}`).click();
      await browser.keys("Escape");
      await $("textarea").click();
      let focused = false;
      for (let n = 0; n < 30; n++) {
        await browser.keys("Tab");
        focused = await browser.execute((p) => document.activeElement?.getAttribute("aria-label")?.startsWith(p) ?? false, prefix);
        if (focused) break;
      }
      expect(focused).toBe(true);
      await browser.keys("Enter");
      await $(`[role="dialog"][aria-label="${language === "ptBr" ? "Modelo e esforço" : "Model and effort"}"]`).waitForDisplayed();
      const low = language === "ptBr" ? "Baixo" : "Low";
      let effortFocused = false;
      for (let n = 0; n < 50; n++) {
        await browser.keys("Tab");
        effortFocused = await browser.execute((text) => document.activeElement?.getAttribute("role") === "menuitemradio" && document.activeElement?.textContent?.trim() === text, low);
        if (effortFocused) break;
      }
      expect(effortFocused).toBe(true);
      await browser.keys("Enter");
      await browser.keys("Escape");
      expect(await $('[role="dialog"]').isExisting()).toBe(false);
      const current = await browser.execute(() => ({ focused: document.activeElement?.getAttribute("aria-label"), input: !!document.querySelector("textarea") }));
      expect(current.focused).toContain(modelName);
      expect(current.focused).toContain(low);
      expect(current.input).toBe(true);
    }
  });
});
