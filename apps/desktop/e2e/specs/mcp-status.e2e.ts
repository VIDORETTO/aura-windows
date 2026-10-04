// QA-025: MCP status shows connected/error clearly, and a failing server's
// own last log lines. Real Node MCP servers + real app-server.
import path from "node:path";
import { writeMcpServer } from "../mcp-server";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

describe("MCP status and logs (QA-025)", () => {
  it("shows Connected, Error with the reason, and the failing server's log", async () => {
    const server = writeMcpServer(path.resolve(process.env.AURA_HOME!, ".."));
    await browser.execute(async (file) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const s of await i("mcp_list")) await i("mcp_delete", { name: s.name });
      const spec = (name: string, args: string[]) => ({ name, transport: { type: "stdio", command: "node", args, env: {}, cwd: null }, enabled: true, disabledTools: [], approvalMode: "askForWrites", startupTimeoutSec: null, toolTimeoutSec: null });
      await i("mcp_save", { spec: spec("qa-ok", [file]), secrets: [], bearer: null });
      await i("mcp_save", { spec: spec("qa-fail", [file, "--fail"]), secrets: [], bearer: null });
      await i("agent_restart");
    }, server.file);
    await browser.waitUntil(async () => {
      const st = (await invoke("mcp_status").catch(() => [])) as any[];
      return st.some((s) => s.name === "qa-ok" && s.tools.length) && st.some((s) => s.name === "qa-fail" && s.error);
    }, { timeout: 120_000, interval: 2000, timeoutMsg: "statuses not available" });

    const overlay = await browser.getWindowHandle();
    await invoke("settings_open", { section: "extensions" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    const ok = await $('[role="status"][aria-label="State of qa-ok"]');
    await ok.waitForDisplayed({ timeout: 15_000 });
    await browser.waitUntil(async () => (await ok.getText()) === "Connected", { timeout: 20_000, timeoutMsg: "state did not refresh" });
    const failState = await $('[role="status"][aria-label="State of qa-fail"]');
    console.log("Native states", JSON.stringify({ ok: await ok.getText(), fail: await failState.getText() }));
    expect(await ok.getText()).toBe("Connected");
    expect(await failState.getText()).toMatch(/^Error: MCP startup failed/);
    await (await $('button[aria-label="Show log of qa-fail"]')).click();
    const log = await $('[role="log"][aria-label="Log of qa-fail"]');
    await browser.waitUntil(async () => (await log.getText()).includes("Exited with code"), { timeout: 20_000 });
    console.log("Native log", JSON.stringify(await log.getText()));
    expect(await log.getText()).toContain("Exited with code 3");
    expect(await log.getText()).toContain("QA MCP server failed on purpose: missing QA_TOKEN");
    expect(await $('button[aria-label="Show log of qa-ok"]').isExisting()).toBe(false);
    for (const name of ["qa-ok", "qa-fail"]) await invoke("mcp_delete", { name });
  });
});
