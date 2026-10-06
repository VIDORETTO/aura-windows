import { localProvider } from "../provider";

describe("Provider catalog invalidation (QA-003)", () => {
  let upstream: Awaited<ReturnType<typeof localProvider>>;
  before(async () => { upstream = await localProvider(); });
  after(async () => { await upstream.close(); });
  it("unlocks an already mounted Overlay after saving the first keyless provider", async () => {
    await browser.execute(async () => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      await invoke("settings_update", { patch: { language: "ptBr" } });
      for (const p of await invoke("providers_list")) await invoke("providers_remove", { id: p.id });
    });
    // Only initial empty-profile preparation reloads; the tested mutation does not.
    await browser.refresh();
    await $('button=Continuar com ChatGPT').waitForDisplayed();
    const saved = await browser.execute(async (baseUrl) => (window as any).__TAURI_INTERNALS__.invoke("providers_save", {
      draft: { name: "QA local isolado", preset: "ollama", baseUrl }, credential: null,
    }), upstream.baseUrl);
    await $("textarea").waitForDisplayed({ timeout: 5000 });
    expect(await $('button=Continuar com ChatGPT').isExisting()).toBe(false);
    expect((saved as any).name).toBe("QA local isolado");
    await $("textarea").setValue("rascunho QA-003");
    await browser.keys(["Control", "ArrowDown"]);
    await $('button[aria-label^="Modelo "]').click();
    const selected = await $('button[role="menuitemradio"][aria-checked="true"]');
    expect(await selected.getText()).toBe("QA local isolado");
    const tested = await browser.execute(async (id) => (window as any).__TAURI_INTERNALS__.invoke("providers_test", { id }), (saved as any).id);
    expect((tested as any).status).toBe("verified");
    await $('button[role="menuitemradio"]*=QA literal model').waitForDisplayed();
    expect(await $("textarea").getValue()).toBe("rascunho QA-003");
    upstream.fail();
    const failed = await browser.execute(async (id) => (window as any).__TAURI_INTERNALS__.invoke("providers_test", { id }), (saved as any).id);
    expect((failed as any).status).toBe("error");
    await browser.waitUntil(async () => (await $('button[role="menuitemradio"]*=QA local isolado').getText()).includes("Erro"), { timeout: 5000 });
    expect(await $("textarea").getValue()).toBe("rascunho QA-003");
    await browser.execute(async (id) => (window as any).__TAURI_INTERNALS__.invoke("providers_remove", { id }), (saved as any).id);
    await $('button=Continuar com ChatGPT').waitForDisplayed({ timeout: 5000 });
  });
});
