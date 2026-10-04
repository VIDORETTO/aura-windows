// QA-019: edit a saved provider (name, URL) from Settings without recreating it;
// conversations that reference it keep working against the new URL.
// Keyless provider: nothing goes to the Windows Credential Manager.
import { responsesUpstream } from "../responses";

describe("Edit provider (QA-019)", () => {
  let a: Awaited<ReturnType<typeof responsesUpstream>>;
  let b: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => {
    a = await responsesUpstream(() => "QA from A", [{ id: "qa-model-a" }]);
    b = await responsesUpstream(() => "QA from B", [{ id: "qa-model-b" }]);
  });
  after(async () => { await a.close(); await b.close(); });

  it("renames and repoints the provider in place and an existing conversation follows it", async () => {
    const before = await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      const p = await i("providers_save", { draft: { name: "QA edit", preset: "ollama", baseUrl: url }, credential: null });
      await i("providers_test", { id: p.id });
      const conv = await i("conversation_start", { options: { mode: { mode: "chat" }, provider: `aura-${p.id}`, model: "qa-model-a", ephemeral: false } });
      return { id: p.id, thread: conv.threadId };
    }, a.baseUrl);
    const overlay = await browser.getWindowHandle();
    await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_open", { section: "providers" }));
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    await (await $('button[aria-label="Edit QA edit"]')).click();
    const form = await $('[role="group"][aria-label="Edit provider QA edit"]');
    await form.waitForDisplayed();
    const name = await form.$("label=Name").$("input");
    await name.click();
    await browser.keys(["Control", "a"]);
    await browser.keys("QA edited");
    const url = await form.$("label=Base URL").$("input");
    await url.click();
    await browser.keys(["Control", "a"]);
    await browser.keys(b.baseUrl);
    await (await form.$("button=Save")).click();
    await browser.waitUntil(async () => (await $("body").getText()).includes("QA edited"), { timeout: 10_000 });

    const after = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("providers_list"));
    console.log("Native providers after edit", JSON.stringify((after as any[]).map((p) => ({ id: p.id, name: p.name, baseUrl: p.baseUrl, status: p.status, models: p.models.map((m: any) => m.id) }))));
    expect((after as any[]).length).toBe(1);
    expect((after as any[])[0].id).toBe(before.id);
    expect((after as any[])[0].name).toBe("QA edited");
    expect((after as any[])[0].baseUrl).toBe(b.baseUrl);
    await browser.waitUntil(async () => ((await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("providers_list"))) as any[])[0].models.some((m: any) => m.id === "qa-model-b"), { timeout: 15_000, timeoutMsg: "re-test did not discover models from the new URL" });

    const aBefore = a.requests.filter((r) => r.url === "/v1/responses").length;
    await browser.execute(async (thread) => (window as any).__TAURI_INTERNALS__.invoke("conversation_send", { request: { threadId: thread, text: "QA after edit", tray: thread } }), before.thread);
    await browser.waitUntil(() => b.requests.some((r) => r.url === "/v1/responses"), { timeout: 150_000, timeoutMsg: "existing conversation did not reach the new URL" });
    expect(a.requests.filter((r) => r.url === "/v1/responses").length).toBe(aBefore);
    console.log("Native existing conversation reached new URL", b.requests.filter((r) => r.url === "/v1/responses").length);
  });
});
