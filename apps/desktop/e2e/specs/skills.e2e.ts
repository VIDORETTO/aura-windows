// QA-022: Skills list origin, enable/disable (persisted by the real
// app-server and applied to new conversations) and editing of Aura skills.
import { readFileSync } from "node:fs";
import path from "node:path";
import { responsesUpstream } from "../responses";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

async function newConversationRequest(up: Awaited<ReturnType<typeof responsesUpstream>>, providerId: string) {
  const before = up.requests.filter((r) => r.url === "/v1/responses").length;
  await browser.execute(async (id) => {
    const i = (window as any).__TAURI_INTERNALS__.invoke;
    const s = await i("conversation_start", { options: { mode: { mode: "chat" }, provider: `aura-${id}`, model: "qa-model", ephemeral: true } });
    await i("conversation_send", { request: { threadId: s.threadId, text: "QA skills probe", tray: s.threadId } }).catch(() => undefined);
  }, providerId);
  await browser.waitUntil(() => up.requests.filter((r) => r.url === "/v1/responses").length > before, { timeout: 150_000, timeoutMsg: "no model request" });
  return JSON.stringify(up.requests.filter((r) => r.url === "/v1/responses")[before].body);
}

describe("Skills management (QA-022)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA"); });
  after(async () => { await up.close(); });

  it("creates, lists by origin, disables for new conversations and edits an Aura skill", async () => {
    const overlay = await browser.getWindowHandle();
    const providerId = await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      for (const s of await i("skills_list")) await i("skills_delete", { name: s.manifest.name });
      const p = await i("providers_save", { draft: { name: "QA skills", preset: "ollama", baseUrl: url }, credential: null });
      await i("settings_open", { section: "extensions" });
      return p.id as string;
    }, up.baseUrl);
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    await (await $("button=New Skill")).click();
    await (await $("label*=Name").$("input")).setValue("qa-revisar-contrato");
    await (await $("label*=Description").$("input")).setValue("QA reviews contracts");
    await (await $("label=Instructions").$("textarea")).setValue("QA instructions body.");
    await (await $("button=Save")).click();
    const row = await $('li[aria-label="qa-revisar-contrato"]');
    await row.waitForDisplayed({ timeout: 15_000 });
    expect(await row.getText()).toContain("Aura");
    // Only counts per origin are logged: user skills belong to the person.
    const catalog = (await invoke("skills_catalog")) as any[];
    const counts = catalog.reduce((acc: Record<string, number>, s) => ({ ...acc, [s.origin]: (acc[s.origin] ?? 0) + 1 }), {});
    console.log("Native skills by origin", JSON.stringify(counts));
    expect(counts.system).toBeGreaterThan(0);
    expect(catalog.find((s) => s.name === "qa-revisar-contrato")?.origin).toBe("aura");

    expect(await newConversationRequest(up, providerId)).toContain("qa-revisar-contrato");

    await (await row.$('button[role="switch"]')).click();
    await browser.waitUntil(async () => (await (await $('li[aria-label="qa-revisar-contrato"]').$('button[role="switch"]')).getAttribute("aria-checked")) === "false", { timeout: 10_000 });
    expect(await newConversationRequest(up, providerId)).not.toContain("qa-revisar-contrato");

    await (await $('button[aria-label="Edit qa-revisar-contrato"]')).click();
    const desc = await $("label*=Description").$("input");
    await desc.click();
    await browser.keys(["Control", "a"]);
    await browser.keys("QA reviews rental contracts");
    await (await $("button=Save")).click();
    await browser.waitUntil(async () => (await $('li[aria-label="qa-revisar-contrato"]').getText()).includes("QA reviews rental contracts"), { timeout: 10_000 });
    const home = process.env.AURA_HOME!;
    const md = readFileSync(path.join(home, "skills", "qa-revisar-contrato", "SKILL.md"), "utf8");
    console.log("Native SKILL.md", JSON.stringify(md));
    expect(md).toContain("QA reviews rental contracts");

    await browser.reloadSession();
    await browser.waitUntil(() => browser.execute(() => "__TAURI_INTERNALS__" in window), { timeout: 20_000 });
    const after = ((await invoke("skills_catalog")) as any[]).find((s) => s.name === "qa-revisar-contrato");
    console.log("Native skill after restart", JSON.stringify(after));
    expect(after.enabled).toBe(false);
    expect(after.origin).toBe("aura");
    await invoke("skills_set_enabled", { path: after.path, enabled: true });
  });
});
