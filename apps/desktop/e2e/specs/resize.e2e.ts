// Exercises the packaged WebView2 and native window, never a browser mock.
import { execFile, spawn } from "node:child_process";
import path from "node:path";
import { promisify } from "node:util";

async function drag(edge: string, dx: number, dy = 0) {
  const probe = path.resolve(import.meta.dirname, "../../../../target/debug/examples/qa_resize.exe");
  const process = path.basename(globalThis.process.env.AURA_E2E_APP!);
  const { stdout } = await promisify(execFile)(probe, [process, edge, String(dx), String(dy)], { timeout: 10_000 });
  console.log("Native drag", stdout.trim());
}
async function dimensions() {
  return browser.execute(async () => {
    const invoke = (window as any).__TAURI_INTERNALS__.invoke;
    const size = await invoke("plugin:window|inner_size", { label: "overlay" });
    const scale = await invoke("plugin:window|scale_factor", { label: "overlay" });
    const shell = document.querySelector(".overlay-shell")!.getBoundingClientRect();
    const input = document.querySelector("textarea")!.getBoundingClientRect();
    const menu = document.querySelector('[role="listbox"]')?.getBoundingClientRect();
    return { nativeWidth: size.width / scale, nativeHeight: size.height / scale,
      width: innerWidth, height: innerHeight, shellWidth: shell.width,
      shellHeight: shell.height, inputBottom: input.bottom, menuBottom: menu?.bottom, scale };
  });
}

async function resize(width: number, height: number) {
  await browser.execute(async (w, h) => {
    await (window as any).__TAURI_INTERNALS__.invoke("plugin:window|set_size", {
      label: "overlay", value: { Logical: { width: w, height: h } },
    });
  }, width, height);
  await browser.pause(500);
}

async function moveIntoView() {
  const p = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("plugin:window|outer_position", { label: "overlay" }));
  await drag("Move", 80 - (p as any).x, 80 - (p as any).y);
}

describe("Native Overlay resizing (QA-001/042)", () => {
  // Aura starts in the tray (001 AC-001); launching it again opens the
  // Overlay, like a user would before resizing it.
  before(async () => {
    await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language: "ptBr" } }));
    await browser.refresh();
    spawn(globalThis.process.env.AURA_E2E_APP!, [], { stdio: "ignore", detached: true }).unref();
    await browser.pause(2000);
  });

  it("keeps content synchronized after dragging both compact side handles", async () => {
    await $("textarea").waitForDisplayed({ timeout: 20_000 });
    await $("textarea").click();
    await browser.keys(["Control", "ArrowUp"]);
    await resize(900, 121);
    for (const [edge, delta] of [["East", -120], ["West", 120], ["East", 120], ["West", -120]] as const) {
      const before = await dimensions();
      await browser.execute(() => {
        (window as any).resizeSamples = [];
        (window as any).resizeTimer = setInterval(() => {
          (window as any).resizeSamples.push({ at: Date.now(), width: innerWidth, shell: document.querySelector(".overlay-shell")!.getBoundingClientRect().width });
        }, 20);
      });
      await drag(edge, delta);
      console.log("DOM drag samples", await browser.execute(() => {
        clearInterval((window as any).resizeTimer);
        return (window as any).resizeSamples;
      }));
      await browser.pause(500);
      const after = await dimensions();
      console.log("QA-001 drag", edge, { before, after });
      if ((edge === "East" && delta < 0) || (edge === "West" && delta > 0))
        expect(after.nativeWidth).toBeLessThan(before.nativeWidth - 50);
      else expect(after.nativeWidth).toBeGreaterThan(before.nativeWidth + 50);
      expect(Math.abs(after.shellWidth - after.nativeWidth)).toBeLessThanOrEqual(2);
    }
  });

  it("keeps compact content equal to the native viewport at multiple widths", async () => {
    const input = await $("textarea");
    await input.waitForDisplayed({ timeout: 20_000 });
    await input.click();
    await browser.keys(["Control", "ArrowUp"]);
    for (const width of [900, 480, 640]) {
      const before = await dimensions();
      await resize(width, before.height);
      const d = await dimensions();
      console.log("QA-001 dimensions", d);
      expect(Math.abs(d.width - width)).toBeLessThanOrEqual(2);
      expect(Math.abs(d.shellWidth - d.nativeWidth)).toBeLessThanOrEqual(2);
      expect(Math.abs(d.width - d.nativeWidth)).toBeLessThanOrEqual(2);
      expect(d.inputBottom).toBeLessThanOrEqual(d.height + 2);
    }
  });

  it("refits compact height after an external native resize", async () => {
    await $("textarea").click();
    await browser.keys(["Control", "ArrowUp"]);
    await resize(900, 400);
    const d = await dimensions();
    console.log("QA-001 compact height", d);
    expect(Math.abs(d.nativeHeight - d.shellHeight)).toBeLessThanOrEqual(2);
    expect(d.nativeWidth).toBe(900);
  });

  it("prevents expanded content from shrinking below its own minimum", async () => {
    await $("textarea").click();
    await browser.keys(["Control", "ArrowDown"]);
    await browser.pause(500);
    await resize(640, 600);
    await moveIntoView();
    await drag("South", 0, -536);
    const d = await dimensions();
    console.log("QA-042 dimensions", d);
    expect(d.nativeHeight).toBeGreaterThanOrEqual(360);
    expect(Math.abs(d.shellHeight - d.nativeHeight)).toBeLessThanOrEqual(2);
    expect(d.inputBottom).toBeLessThanOrEqual(d.height + 2);
  });

  it("resizes expanded content using every native edge and corner", async () => {
    await $("textarea").click();
    await browser.keys(["Control", "ArrowDown"]);
    for (const edge of ["East", "West", "North", "South", "NorthEast", "NorthWest", "SouthEast", "SouthWest"]) {
      await resize(900, 600);
      await moveIntoView();
      const before = await dimensions();
      const dx = edge.includes("East") ? -40 : edge.includes("West") ? 40 : 0;
      const dy = edge.includes("North") ? 40 : edge.includes("South") ? -40 : 0;
      await drag(edge, dx, dy);
      const d = await dimensions();
      console.log("QA-001 expanded edge", edge, d);
      if (dx) expect(d.nativeWidth).toBeLessThan(before.nativeWidth - 20);
      if (dy) expect(d.nativeHeight).toBeLessThan(before.nativeHeight - 20);
      expect(Math.abs(d.width - d.nativeWidth)).toBeLessThanOrEqual(2);
      expect(Math.abs(d.shellWidth - d.nativeWidth)).toBeLessThanOrEqual(2);
      expect(Math.abs(d.shellHeight - d.nativeHeight)).toBeLessThanOrEqual(2);
      expect(d.inputBottom).toBeLessThanOrEqual(d.height + 2);
    }
  });

  it("fits the compact context menu while resizing and returns to content height", async () => {
    await $("textarea").click();
    await browser.keys(["Control", "ArrowUp"]);
    await $('button[aria-label="Adicionar contexto"]').click();
    await $('[role="listbox"]').waitForDisplayed();
    for (const width of [900, 480, 640]) {
      await resize(width, 400);
      const d = await dimensions();
      expect(d.menuBottom).toBeDefined();
      expect(d.menuBottom!).toBeLessThanOrEqual(d.height + 2);
      expect(d.shellWidth).toBe(width);
    }
    await $("textarea").click();
    await browser.keys("Escape");
    await browser.pause(500);
    const d = await dimensions();
    expect(d.menuBottom).toBeNull();
    expect(Math.abs(d.nativeHeight - d.shellHeight)).toBeLessThanOrEqual(2);
    expect(d.nativeHeight).toBeLessThan(360);
  });

  it("remembers the compact width when hiding and reopening", async () => {
    await resize(850, 400);
    await browser.pause(600); // placement debounce is 400 ms
    const before = await dimensions();
    await browser.execute(async () => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      await invoke("overlay_hide");
    });
    // Launching a second instance reopens the existing Overlay (001 AC-002).
    await promisify(execFile)(process.env.AURA_E2E_APP!, [], { timeout: 10_000 });
    await browser.pause(500);
    const d = await dimensions();
    expect(d.nativeWidth).toBe(before.nativeWidth);
    expect(Math.abs(d.nativeHeight - d.shellHeight)).toBeLessThanOrEqual(2);
  });

  it("keeps dimensions synchronized after moving across available monitors", async function () {
    const monitors = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("plugin:window|available_monitors", { label: "overlay" })) as any[];
    console.log("Native monitor matrix", monitors);
    if (monitors.length < 2) this.skip();
    for (const monitor of monitors) {
      const p = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("plugin:window|outer_position", { label: "overlay" })) as any;
      await drag("Move", monitor.position.x + 80 - p.x, monitor.position.y + 80 - p.y);
      await resize(640, 400);
      const d = await dimensions();
      console.log("QA-001 monitor", monitor.name, d);
      expect(Math.abs(d.width - d.nativeWidth)).toBeLessThanOrEqual(2);
      expect(Math.abs(d.shellWidth - d.nativeWidth)).toBeLessThanOrEqual(2);
      expect(Math.abs(d.nativeHeight - d.shellHeight)).toBeLessThanOrEqual(2);
    }
  });

  it("clamps a legacy undersized expanded placement when restoring it", async () => {
    await $("textarea").click();
    await browser.keys(["Control", "ArrowDown"]);
    // The old app could persist a 64px expanded placement. The admin size API
    // seeds that legacy state; user resizing itself is tested with native mouse.
    await resize(640, 64);
    await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("overlay_moved"));
    await browser.keys(["Control", "ArrowUp"]);
    await browser.pause(500);
    await browser.keys(["Control", "ArrowDown"]);
    await browser.pause(500);
    const d = await dimensions();
    console.log("QA-042 legacy placement", d);
    expect(d.nativeHeight).toBeGreaterThanOrEqual(360);
    expect(d.inputBottom).toBeLessThanOrEqual(d.height + 2);
  });
});
