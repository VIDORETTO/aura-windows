describe("Automatic ASR language (QA-009)", () => {
  it("clears a fixed language and does not derive ASR language from the UI", async () => {
    const overlay = await browser.getWindowHandle();
    await browser.execute(async () => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      await invoke("settings_update", { patch: { language: "en" } });
      await invoke("settings_open", { section: "voice" });
    });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((handle) => handle !== overlay)!);
    const language = await $('select[aria-label="Dictation language"]');
    await language.waitForDisplayed();
    const saved = async () => browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_get"));
    if (process.env.AURA_E2E_ASR_STAGE === "restart") {
      expect((await saved() as any).asrLanguage).toBeNull();
      expect(await language.getValue()).toBe("");
    } else {
      await language.selectByAttribute("value", "es");
      await browser.waitUntil(async () => (await saved() as any).asrLanguage === "es");
      await language.selectByAttribute("value", "");
      await browser.waitUntil(async () => (await saved() as any).asrLanguage === null, { timeout: 3000 });
      expect(await language.getValue()).toBe("");
    }
    await browser.switchToWindow(overlay);
    const eventId = await browser.execute(async () => {
      const internals = (window as any).__TAURI_INTERNALS__;
      (window as any).qaLanguageVoiceStates = [];
      return internals.invoke("plugin:event|listen", {
        event: "aura://event", target: { kind: "Any" },
        handler: internals.transformCallback((event: any) => {
          if (event.payload.channel === "voice") (window as any).qaLanguageVoiceStates.push(event.payload.event.state);
        }),
      });
    });
    try {
      for (const ui of ["ptBr", "en"]) {
        await browser.execute(async (language) => (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language } }), ui);
        const mic = await $(`button[aria-label="${ui === "ptBr" ? "Ditar" : "Dictate"}"]`);
        await mic.waitForDisplayed();
        const before = await browser.execute(() => (window as any).qaLanguageVoiceStates.length);
        await mic.click();
        await browser.waitUntil(async () => (await browser.execute(() => (window as any).qaLanguageVoiceStates)).slice(before).includes("listening"));
        await browser.waitUntil(async () => (await mic.getAttribute("aria-disabled")) === "false");
        await mic.click();
        await browser.waitUntil(async () => {
          const states = (await browser.execute(() => (window as any).qaLanguageVoiceStates)).slice(before);
          return states.includes("listening") && states.includes("empty");
        });
        expect((await saved() as any).asrLanguage).toBeNull();
        await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("ptt_cancel"));
      }
      console.log("Native QA automatic settings and actual microphone click states", await browser.execute(() => (window as any).qaLanguageVoiceStates));
    } finally {
      await browser.execute(async (id) => {
        const invoke = (window as any).__TAURI_INTERNALS__.invoke;
        await invoke("ptt_cancel");
        await invoke("plugin:event|unlisten", { event: "aura://event", eventId: id });
      }, eventId);
    }
  });
});
