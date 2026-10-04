import path from "node:path";

const nativeLocalAuth = process.env.AURA_E2E_LOCAL_AUTH === "1";
(nativeLocalAuth ? describe : describe.skip)("Native login cancellation (QA-006)", () => {
  it("returns to the initial card twice without emitting a technical failure", async () => {
    // This named artifact is built with e2e,tauri/custom-protocol, never demo.
    expect(path.basename(process.env.AURA_E2E_APP!)).toBe("aura-auth-qa.exe");
    await browser.execute(async () => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      for (const provider of await invoke("providers_list")) await invoke("providers_remove", { id: provider.id });
      await invoke("settings_update", { patch: { language: "ptBr" } });
    });
    await browser.refresh();
    const eventId = await browser.execute(async () => {
      const internals = (window as any).__TAURI_INTERNALS__;
      (window as any).qaLoginStates = [];
      const handler = internals.transformCallback((event: any) => {
        if (event.payload.channel === "login") (window as any).qaLoginStates.push(event.payload.event.state);
      });
      return internals.invoke("plugin:event|listen", { event: "aura://event", target: { kind: "Any" }, handler });
    });
    try {
      for (const [language, login, cancel] of [["ptBr", "Continuar com ChatGPT", "Cancelar"], ["en", "Continue with ChatGPT", "Cancel"]]) {
        await browser.execute(async (lang) => (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language: lang } }), language);
        await $(`button=${login}`).waitForDisplayed();
        await $(`button=${login}`).click();
        await $(`button=${cancel}`).waitForDisplayed();
        await $(`button=${cancel}`).click();
        await $(`button=${login}`).waitForDisplayed();
        expect(await $("body").getText()).not.toContain("sign-in cancelled");
        expect(await $("body").getText()).not.toContain("Não foi possível entrar");
        expect(await $("body").getText()).not.toContain("Could not sign in");
      }
      await browser.pause(300);
      const states = await browser.execute(() => (window as any).qaLoginStates);
      console.log("Native QA login states (no URLs)", states);
      expect(states).toEqual(["waitingBrowser", "cancelled", "waitingBrowser", "cancelled"]);
      const auth = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("auth_status"));
      expect((auth as any).active).toBeNull();
      await browser.saveScreenshot(path.resolve(import.meta.dirname, "../../../../target/qa-tools/auth-cancel-native-green.png"));
    } finally {
      await browser.execute(async (id) => (window as any).__TAURI_INTERNALS__.invoke("plugin:event|unlisten", { event: "aura://event", eventId: id }), eventId);
    }
  });
});
