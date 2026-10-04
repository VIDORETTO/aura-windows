// QA-026: review, forget and edit remembered facts; new conversations only get
// what is left. Memory files are seeded as Codex would write them (generating
// them needs real model calls); the real app-server reads them.
import { existsSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { responsesUpstream } from "../responses";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

describe("Memories (QA-026)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA"); });
  after(async () => { await up.close(); });

  const instructions = async (providerId: string) => {
    const before = up.requests.filter((r) => r.url === "/v1/responses").length;
    await browser.execute(async (id) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      const s = await i("conversation_start", { options: { mode: { mode: "chat" }, provider: `aura-${id}`, model: "qa-model", ephemeral: true } });
      await i("conversation_send", { request: { threadId: s.threadId, text: "QA memory probe", tray: s.threadId } }).catch(() => undefined);
    }, providerId);
    await browser.waitUntil(() => up.requests.filter((r) => r.url === "/v1/responses").length > before, { timeout: 150_000, timeoutMsg: "no model request" });
    return JSON.stringify(up.requests.filter((r) => r.url === "/v1/responses")[before].body);
  };

  it("forgets one fact and then everything, for new conversations", async () => {
    const dir = path.join(process.env.AURA_HOME!, "codex-home", "memories");
    mkdirSync(path.join(dir, "rollout_summaries"), { recursive: true });
    writeFileSync(path.join(dir, "memory_summary.md"), "# User\n- QA fact alpha: prefers metric units\n- QA fact beta: lives in Lisbon\n");
    writeFileSync(path.join(dir, "MEMORY.md"), "# Registry\n- QA fact beta: lives in Lisbon\n");
    const providerId = await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en", memories: true } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      for (const s of await i("mcp_list")) await i("mcp_delete", { name: s.name });
      const p = await i("providers_save", { draft: { name: "QA memories", preset: "ollama", baseUrl: url }, credential: null });
      await i("agent_restart");
      return p.id as string;
    }, up.baseUrl);
    const first = await instructions(providerId);
    console.log("Native memory in request", { alpha: first.includes("QA fact alpha"), beta: first.includes("QA fact beta") });
    expect(first).toContain("QA fact alpha");
    expect(first).toContain("QA fact beta");

    const overlay = await browser.getWindowHandle();
    await invoke("settings_open", { section: "general" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    const list = await $('ul[aria-label="Remembered facts"]');
    await list.waitForDisplayed({ timeout: 15_000 });
    console.log("Native facts", JSON.stringify(await list.getText()));
    await (await $('button[aria-label="Forget: QA fact beta: lives in Lisbon"]')).click();
    await browser.waitUntil(async () => !(await list.getText()).includes("QA fact beta"), { timeout: 10_000 });
    const notes = readdirSync(path.join(dir, "extensions", "ad_hoc", "notes"));
    console.log("Native consolidation notes", JSON.stringify(notes));
    expect(notes.some((n) => n.endsWith("-aura-forget.md"))).toBe(true);

    const second = await instructions(providerId);
    console.log("Native after forgetting beta", { alpha: second.includes("QA fact alpha"), beta: second.includes("QA fact beta") });
    expect(second).toContain("QA fact alpha");
    expect(second).not.toContain("QA fact beta");

    await browser.execute(() => { window.confirm = () => true; });
    await (await $("button=Forget everything")).click();
    await $("p=Nothing remembered yet.").waitForDisplayed({ timeout: 10_000 });
    expect(existsSync(dir)).toBe(false);
    const third = await instructions(providerId);
    console.log("Native after forgetting everything", { alpha: third.includes("QA fact alpha") });
    expect(third).not.toContain("QA fact alpha");
  });
});
