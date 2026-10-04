// QA-021: a provider without /models gets a model id and capabilities by hand;
// the model survives a connection re-test, appears in the Overlay picker with
// its capabilities and turns use it.
import { responsesUpstream } from "../responses";

describe("Manual provider model (QA-021)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA", null); });
  after(async () => { await up.close(); });

  it("adds a model by hand and uses it for a turn", async () => {
    const overlay = await browser.getWindowHandle();
    const id = await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      const p = await i("providers_save", { draft: { name: "QA manual", preset: "custom", wire: "chat", baseUrl: url }, credential: null });
      await i("settings_open", { section: "providers" });
      return p.id as string;
    }, up.baseUrl);
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    await (await $('button[aria-label="Models of QA manual"]')).click();
    await (await $("button=Add model")).click();
    await (await $("label=Model ID").$("input")).setValue("qa-manual-model");
    await (await $("label=Display name").$("input")).setValue("QA Manual");
    await (await $("label=Tools").$("input")).click();
    await (await $("label=Reasoning").$("input")).click();
    await (await $("button=Save model")).click();
    await $('li[aria-label="QA Manual"]').waitForDisplayed({ timeout: 10_000 });

    // Re-testing a provider without /models keeps the manual entry.
    const tested = await browser.execute(async (pid) => (window as any).__TAURI_INTERNALS__.invoke("providers_test", { id: pid }), id);
    console.log("Native provider after re-test", JSON.stringify({ status: (tested as any).status, lastError: (tested as any).lastError, models: (tested as any).models }));
    expect((tested as any).status).toBe("unverified");
    expect((tested as any).models).toEqual([{ id: "qa-manual-model", displayName: "QA Manual", contextWindow: null, maxOutput: null, supportsImages: false, supportsTools: true, supportsReasoning: true, estimated: false, manual: true }]);

    await browser.switchToWindow(overlay);
    await browser.refresh();
    await $("textarea").waitForDisplayed({ timeout: 20_000 });
    await browser.keys(["Control", "ArrowDown"]);
    await $('button[aria-label^="Model "]').click();
    const option = await $('[role="menu"][aria-label="Model"]').$("button*=QA Manual");
    await option.waitForDisplayed();
    console.log("Native picker option", JSON.stringify(await option.getText()));
    expect(await option.getText()).toContain("Tools · Reasoning");
    await option.click();
    await browser.keys("Escape");
    await $("textarea").setValue("QA manual model turn");
    await browser.keys("Enter");
    await browser.waitUntil(() => up.requests.some((r) => r.url === "/v1/chat/completions"), { timeout: 150_000, timeoutMsg: "no turn request" });
    const body = up.requests.find((r) => r.url === "/v1/chat/completions")!.body;
    console.log("Native turn model", body?.model);
    expect(body.model).toBe("qa-manual-model");
  });
});
