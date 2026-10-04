// QA-024: turning off one MCP tool removes it from new conversations.
// Real MCP server (Node) + real app-server + loopback model upstream that
// records the tools offered to the model.
import path from "node:path";
import { writeMcpServer } from "../mcp-server";
import { responsesUpstream } from "../responses";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

describe("MCP tool toggles (QA-024)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA"); });
  after(async () => { await up.close(); });

  const toolsOffered = async (providerId: string) => {
    const before = up.requests.filter((r) => r.url === "/v1/responses").length;
    await browser.execute(async (id) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      const s = await i("conversation_start", { options: { mode: { mode: "chat" }, provider: `aura-${id}`, model: "qa-model", ephemeral: true } });
      await i("conversation_send", { request: { threadId: s.threadId, text: "QA tools probe", tray: s.threadId } }).catch(() => undefined);
    }, providerId);
    await browser.waitUntil(() => up.requests.filter((r) => r.url === "/v1/responses").length > before, { timeout: 150_000, timeoutMsg: "no model request" });
    const tools = JSON.stringify(up.requests.filter((r) => r.url === "/v1/responses")[before].body.tools ?? []);
    return { echo: tools.includes("qa_echo"), write: tools.includes("qa_write") };
  };

  it("removes a disabled tool from new conversations", async () => {
    const server = writeMcpServer(path.resolve(process.env.AURA_HOME!, ".."));
    const providerId = await browser.execute(async (url, file) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      for (const s of await i("mcp_list")) await i("mcp_delete", { name: s.name });
      await i("mcp_save", { spec: { name: "qa", transport: { type: "stdio", command: "node", args: [file], env: {}, cwd: null }, enabled: true, disabledTools: [], approvalMode: "auto", startupTimeoutSec: null, toolTimeoutSec: null }, secrets: [], bearer: null });
      const p = await i("providers_save", { draft: { name: "QA tools", preset: "ollama", baseUrl: url }, credential: null });
      await i("agent_restart");
      return p.id as string;
    }, up.baseUrl, server.file);
    await browser.waitUntil(async () => ((await invoke("mcp_status").catch(() => [])) as any[]).some((s) => s.name === "qa" && s.tools.length === 2), { timeout: 120_000, interval: 2000 });
    const baseline = await toolsOffered(providerId);
    console.log("Native tools before", JSON.stringify(baseline));
    expect(baseline).toEqual({ echo: true, write: true });

    const overlay = await browser.getWindowHandle();
    await invoke("settings_open", { section: "extensions" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    await (await $('button[aria-label="Tools of qa"]')).click();
    await (await $("label=qa_write").$("input")).click();
    await browser.waitUntil(async () => ((await invoke("mcp_list")) as any[]).find((s) => s.name === "qa")?.disabledTools?.includes("qa_write"), { timeout: 10_000 });
    expect(await (await $("label=qa_write").$("input")).isSelected()).toBe(false);

    await invoke("agent_restart");
    const after = await toolsOffered(providerId);
    console.log("Native tools after disabling qa_write", JSON.stringify(after));
    expect(after).toEqual({ echo: true, write: false });
    await invoke("mcp_delete", { name: "qa" });
  });
});
