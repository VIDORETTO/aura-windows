// 040: real OS window + public IPC, without changing the user's Aura profile.
import { execFile, spawn } from "node:child_process";
import { promisify } from "node:util";
import path from "node:path";
import { createServer } from "node:http";
import { responsesUpstream } from "../responses";

const invoke = (command: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), command, args)
    .then((value) => JSON.parse(String(value)));

async function position() {
  return invoke("plugin:window|outer_position", { label: "overlay" });
}

async function dragTo(monitor: any, x = 80, y = 80) {
  const before = await position();
  const start = await browser.execute(async () => {
    const region = document.querySelector('header > div[data-tauri-drag-region]')!;
    const r = region.getBoundingClientRect();
    const x = r.left + r.width / 2;
    const y = r.top + r.height / 2;
    const hit = document.elementFromPoint(x, y);
    if (!hit?.closest('[data-tauri-drag-region]') || hit.closest('button')) throw new Error('QA drag start is not an observed non-button drag region');
    const scale = await (window as any).__TAURI_INTERNALS__.invoke('plugin:window|scale_factor', { label: 'overlay' });
    return { x: Math.round(x * scale), y: Math.round(y * scale) };
  });
  const helper = path.resolve(import.meta.dirname, "../../../../target/debug/examples/qa_resize.exe");
  const { stdout } = await promisify(execFile)(helper, [
    path.basename(process.env.AURA_E2E_APP!), "Move",
    String(monitor.workArea.position.x + x - before.x),
    String(monitor.workArea.position.y + y - before.y),
    String(start.x), String(start.y),
  ], { timeout: 10_000 });
  console.log("040 real drag", stdout.trim());
  await browser.pause(600);
}

async function compact() {
  await $("textarea").click();
  await browser.keys(["Control", "ArrowUp"]);
  await browser.pause(500);
}

async function assertOn(monitor: any) {
  const p = await position();
  expect(p.x).toBeGreaterThanOrEqual(monitor.workArea.position.x);
  expect(p.x).toBeLessThan(monitor.workArea.position.x + monitor.workArea.size.width);
  expect(p.y).toBeGreaterThanOrEqual(monitor.workArea.position.y);
  expect(p.y).toBeLessThan(monitor.workArea.position.y + monitor.workArea.size.height);
  return p;
}

async function foregroundAt(monitor: any) {
  const helper = path.resolve(import.meta.dirname, "../../../../target/debug/examples/qa_foreground.exe");
  const context = spawn(helper, [String(monitor.workArea.position.x + 100), String(monitor.workArea.position.y + 100)], { stdio: ["ignore", "pipe", "pipe"] });
  let diagnostic = "";
  context.stderr.on("data", (data) => { diagnostic += String(data); });
  try {
    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("QA foreground fixture readiness timeout")), 25_000);
      context.stdout.once("data", (data) => {
        clearTimeout(timer);
        if (String(data).includes("READY")) resolve();
        else reject(new Error("Unexpected QA foreground fixture readiness"));
      });
      context.once("exit", (code) => { clearTimeout(timer); reject(new Error(`QA foreground fixture exited ${code}: ${diagnostic.trim()}`)); });
      context.once("error", (error) => { clearTimeout(timer); reject(error); });
    });
    return context;
  } catch (error) {
    context.kill();
    throw error;
  }
}

async function reopenOn(monitor: any) {
  await invoke("overlay_hide");
  await browser.pause(200); // let native hide/focus events finish before the fixture
  const context = await foregroundAt(monitor);
  try {
    await promisify(execFile)(process.env.AURA_E2E_APP!, [], { timeout: 10_000 });
    await browser.pause(600);
    expect((await invoke("previous_app")).monitorId).toBe(monitor.name);
  } finally {
    context.kill();
  }
}

describe("Overlay monitor stability (040)", () => {
  let monitors: any[];
  let previous: any;
  let other: any;

  before(async function () {
    if (!process.env.AURA_HOME || !process.env.AURA_E2E_APP?.endsWith("aura-qa.exe")) {
      throw new Error("Requires an isolated QA profile and executable");
    }
    await invoke("settings_update", { patch: { language: "ptBr", onboarded: true, attachScreenOnOpen: false, hideOnBlur: false } });
    await browser.refresh();
    monitors = await invoke("plugin:window|available_monitors", { label: "overlay" });
    if (monitors.length < 2) throw new Error("Requires two real monitors; no emulated geometry");
    await reopenOn(monitors[0]);
    await $("textarea").waitForDisplayed({ timeout: 20_000 });
    previous = await invoke("previous_app");
    other = monitors.find((m) => m.name !== previous?.monitorId);
    if (!previous?.monitorId || !monitors.some((m) => m.name === previous.monitorId) || !other) {
      throw new Error("Previous-app monitor must match the native monitor inventory");
    }
    console.log("040 native context", JSON.stringify({ previousMonitor: previous.monitorId, other: other.name, monitors }));
  });

  it("keeps the dragged monitor and anchor when expanding and compacting", async () => {
    await compact();
    // Seed a different expanded anchor on B before the user moves compact B.
    // A saved mode must supply its size without moving the current window.
    await browser.keys(["Control", "ArrowDown"]);
    await browser.pause(500);
    await invoke("plugin:window|set_size", { label: "overlay", value: { Logical: { width: 640, height: 500 } } });
    await browser.pause(600);
    await dragTo(other);
    expect((await invoke("plugin:window|inner_size", { label: "overlay" })).height).toBe(500);
    await invoke("overlay_moved");
    await compact();
    await dragTo(other, 160, 60);
    const anchor = await assertOn(other);
    expect(anchor.x).toBe(other.workArea.position.x + 160);
    expect(anchor.y).toBe(other.workArea.position.y + 60);
    expect((await invoke("previous_app")).monitorId).toBe(previous.monitorId);
    for (const expanded of [true, false, true, false]) {
      await $("textarea").click();
      await browser.keys(["Control", expanded ? "ArrowDown" : "ArrowUp"]);
      await browser.pause(600);
      const p = await position();
      console.log("040 mode position", JSON.stringify({ expanded, anchor, actual: p, expectedMonitor: other.name }));
      await assertOn(other);
      expect(p.x).toBe(anchor.x);
      // Height can require clamping near a work-area boundary; this anchor has room.
      expect(p.y).toBe(anchor.y);
    }
  });

  it("does not overwrite the previous monitor's saved compact placement", async () => {
    await compact();
    await reopenOn(monitors[0]);
    const previousNow = await invoke("previous_app");
    const originalMonitor = monitors.find((m) => m.name === previousNow?.monitorId);
    if (!originalMonitor) throw new Error("No contextual monitor after reopening");
    const target = monitors.find((m) => m.name !== originalMonitor.name)!;
    await dragTo(originalMonitor);
    await invoke("plugin:window|set_size", { label: "overlay", value: { Logical: { width: 700, height: 160 } } });
    await browser.pause(600);
    const original = await position();
    await invoke("overlay_moved");
    await dragTo(target);
    await invoke("plugin:window|set_size", { label: "overlay", value: { Logical: { width: 820, height: 160 } } });
    await browser.pause(600);
    await invoke("overlay_moved");
    await reopenOn(target);
    const restoredTarget = await position();
    expect(restoredTarget.x).toBe(target.workArea.position.x + 80);
    expect(restoredTarget.y).toBe(target.workArea.position.y + 80);
    expect(await browser.execute(() => innerWidth)).toBe(820);
    await reopenOn(originalMonitor);
    const restored = await position();
    const restoredWidth = await browser.execute(() => innerWidth);
    console.log("040 saved placement", JSON.stringify({ original, restored, restoredWidth }));
    expect(restored.x).toBe(original.x);
    expect(restored.y).toBe(original.y);
    expect(restoredWidth).toBe(700);
  });

  it("stays on the chosen monitor while sending and returning from a real Minibar", async () => {
    const up = await responsesUpstream(() => "QA040 stable conversation reply");
    let releaseReply!: () => void;
    const replyGate = new Promise<void>((resolve) => { releaseReply = resolve; });
    let waiting = false;
    // Only the external model's network response is held; Host/UI/OS are real.
    const proxy = createServer(async (req, res) => {
      const chunks: Buffer[] = [];
      for await (const chunk of req) chunks.push(Buffer.from(chunk));
      if (req.url === "/v1/responses") { waiting = true; await replyGate; }
      const response = await fetch(`${up.baseUrl.replace(/\/v1$/, "")}${req.url}`, {
        method: req.method,
        headers: { "content-type": "application/json" },
        ...(req.method === "POST" ? { body: Buffer.concat(chunks) } : {}),
      });
      res.writeHead(response.status, { "content-type": response.headers.get("content-type") ?? "application/json" });
      res.end(Buffer.from(await response.arrayBuffer()));
    });
    await new Promise<void>((resolve) => proxy.listen(0, "127.0.0.1", resolve));
    const address = proxy.address();
    if (!address || typeof address === "string") throw new Error("QA model proxy has no port");
    let context: Awaited<ReturnType<typeof foregroundAt>> | undefined;
    try {
      await invoke("settings_update", { patch: { hideOnBlur: false } });
      const p = await invoke("providers_save", { draft: { name: "QA040 model", preset: "custom", wire: "responses", baseUrl: `http://127.0.0.1:${address.port}/v1`, auth: "none" }, credential: null });
      await invoke("providers_model_save", { id: p.id, model: { id: "qa-model", displayName: "QA040 model", contextWindow: null, maxOutput: null, supportsImages: false, supportsTools: true, supportsReasoning: false, estimated: false, manual: true, efforts: [], defaultEffort: null } });
      await browser.refresh();
      await reopenOn(monitors[0]);
      await $("textarea").waitForDisplayed({ timeout: 20_000 });
      await $('header button[aria-label^="Modelo"]').click();
      await (await $('[role="menu"][aria-label="Provedor"]')).$("button*=QA040 model").click();
      await (await $('[role="menu"][aria-label="Modelo"]')).$("button*=QA040 model").click();
      await browser.keys("Escape");
      await compact();
      await dragTo(other);
      const anchor = await assertOn(other);
      await $("textarea").setValue("QA040 monitor stability conversation");
      await browser.keys("Enter");
      await browser.waitUntil(() => waiting, { timeout: 30_000 });
      await assertOn(other);
      await invoke("settings_update", { patch: { hideOnBlur: true } });
      context = await foregroundAt(monitors[0]);
      await browser.waitUntil(async () => (await invoke("previous_app"))?.monitorId === monitors[0].name, { timeout: 10_000 });
      const minibar = await $('button[aria-label="Aura está respondendo…"]');
      await minibar.waitForDisplayed({ timeout: 10_000 });
      await assertOn(other);
      await minibar.click();
      await $("textarea").waitForDisplayed({ timeout: 10_000 });
      releaseReply();
      await browser.waitUntil(async () => (await $("body").getText()).includes("QA040 stable conversation reply"), { timeout: 30_000 });
      const restored = await assertOn(other);
      console.log("040 live Minibar position", JSON.stringify({ anchor, restored }));
      expect(restored.x).toBe(anchor.x);
      expect(restored.y).toBe(anchor.y);
    } finally {
      releaseReply();
      context?.kill();
      await invoke("settings_update", { patch: { hideOnBlur: false } });
      await new Promise<void>((resolve) => proxy.close(() => resolve()));
      await up.close();
    }
  });
});
