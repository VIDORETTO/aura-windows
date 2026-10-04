// QA-037: Diagnostics shows the whole pipeline with real values: Gateway
// reachability, MCP servers of the running app-server, speech worker,
// active captures, account (masked) and disk space.
import { spawnSync } from "node:child_process";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

describe("Diagnostics (QA-037)", () => {
  it("reports gateway, MCP, worker, captures, account and disk", async () => {
    const overlay = await browser.getWindowHandle();
    await invoke("settings_update", { patch: { language: "en", microphoneDeviceId: "CABLE Output (VB-Audio Virtual Cable)" } });
    await invoke("privacy_set_paused", { paused: false }).catch(() => undefined);
    await invoke("privacy_set_source", { source: "mic", mode: { type: "recentBuffer", minutes: 1 }, agent: "never" });
    try {
      await browser.waitUntil(async () => (await invoke("diagnostics")).appServer.state === "ready", { timeout: 180_000, timeoutMsg: "app-server not ready" });
      const d = await invoke("diagnostics");
      const home = process.env.AURA_HOME!;
      const drive = home.slice(0, 1);
      const free = Number(spawnSync("powershell", ["-NoProfile", "-Command", `(Get-PSDrive ${drive}).Free`], { encoding: "utf8" }).stdout.trim());
      console.log("Native diagnostics", JSON.stringify({ gateway: d.gateway, mcp: d.mcp, worker: d.worker, capture: d.capture, account: d.account ? { ...d.account, email: d.account.email ? "<masked>" : null } : null, disk: d.disk, freeByPowerShell: free }));
      expect(d.gateway.reachable).toBe(true);
      expect(d.mcp.find((m: any) => m.name === "aura")?.tools).toBeGreaterThan(0);
      expect(d.worker.installed).toBe(true);
      expect(d.capture.active).toContain("mic");
      if (d.account?.email) expect(d.account.email).toMatch(/^.\*\*\*@/);
      expect(Math.abs(d.disk.freeBytes - free) / free).toBeLessThan(0.01);
      expect(d.disk.auraBytes).toBeGreaterThan(0);

      await invoke("settings_open", { section: "diagnostics" });
      await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
      await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
      await browser.waitUntil(async () => (await browser.execute(() => document.body.innerText)).includes("Speech worker"), { timeout: 15_000 });
      const text = await browser.execute(() => document.body.innerText);
      const lines = text.split("\n").filter((l) => /responding|tools|Installed|Microphone|signed|free ·|None/.test(l));
      console.log("Native diagnostics screen", JSON.stringify(lines.map((l) => l.replace(/^.\*\*\*@\S+/, "<masked>"))));
      expect(text).toContain(`127.0.0.1:${d.gateway.port} · responding`);
      expect(text).toMatch(/aura · \d+ tools/);
      expect(text).toMatch(/Installed · (running|idle) · (GPU|CPU)/);
      expect(text).toContain("Microphone");
      expect(text).toMatch(/free · Aura uses/);
    } finally {
      await invoke("privacy_set_source", { source: "mic", mode: { type: "onDemand" }, agent: "ask" }).catch(() => undefined);
      await invoke("settings_update", { patch: { microphoneDeviceId: null } }).catch(() => undefined);
    }
  });
});
