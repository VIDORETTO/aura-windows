// QA-023: MCP arguments typed with quotes reach the real server process as
// the intended argv (path with spaces, value with spaces, empty argument).
import { existsSync, readFileSync, rmSync } from "node:fs";
import path from "node:path";
import { writeMcpServer } from "../mcp-server";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

describe("MCP arguments (QA-023)", () => {
  it("spawns the server with the parsed argv", async () => {
    const server = writeMcpServer(path.resolve(process.env.AURA_HOME!, ".."));
    rmSync(path.join(server.dir, "argv.json"), { force: true });
    const overlay = await browser.getWindowHandle();
    await browser.execute(async () => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const s of await i("mcp_list")) await i("mcp_delete", { name: s.name });
      await i("settings_open", { section: "extensions" });
    });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    await (await $("button=Add server")).click();
    await (await $("label=Name").$("input")).setValue("qa-args");
    await (await $("label=Command").$("input")).setValue("node");
    const typed = `"${server.file}" --label "QA label" --empty ""`;
    await (await $("label*=Arguments").$("input")).setValue(typed);
    const parsed = await browser.execute(() => [...document.querySelectorAll('ol[aria-label="Parsed arguments"] li')].map((li) => li.textContent));
    console.log("Native parsed preview", JSON.stringify(parsed));
    await (await $("button=Save")).click();
    await browser.waitUntil(async () => ((await invoke("mcp_list")) as any[]).some((s) => s.name === "qa-args"), { timeout: 10_000 });
    const spec = ((await invoke("mcp_list")) as any[]).find((s) => s.name === "qa-args");
    console.log("Native saved args", JSON.stringify(spec.transport.args));
    expect(spec.transport.args).toEqual([server.file, "--label", "QA label", "--empty", ""]);

    await invoke("agent_restart");
    await browser.waitUntil(async () => {
      const status = (await invoke("mcp_status").catch(() => [])) as any[];
      return status.some((s) => s.name === "qa-args" && s.tools.length > 0);
    }, { timeout: 120_000, interval: 2000, timeoutMsg: "server did not start with tools" });
    await browser.waitUntil(() => existsSync(path.join(server.dir, "argv.json")), { timeout: 10_000 });
    const argv = JSON.parse(readFileSync(path.join(server.dir, "argv.json"), "utf8"));
    console.log("Native server argv", JSON.stringify(argv));
    expect(argv).toEqual(["--label", "QA label", "--empty", ""]);
    await invoke("mcp_delete", { name: "qa-args" });
  });
});
