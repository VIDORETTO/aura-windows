describe("Microphone button toggle (QA-010)", () => {
  it("supports Enter, Space, two mouse clicks and Escape in the native app", async () => {
    await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language: "en", sendAfterDictation: false } }));
    const mic = await $('button[aria-label="Hold to talk"],button[aria-label="Dictate"]');
    await mic.waitForDisplayed();
    const eventId = await browser.execute(async () => {
      const internals = (window as any).__TAURI_INTERNALS__;
      (window as any).qaToggleStates = [];
      return internals.invoke("plugin:event|listen", { event: "aura://event", target: { kind: "Any" },
        handler: internals.transformCallback((event: any) => {
          if (event.payload.channel === "voice") (window as any).qaToggleStates.push(event.payload.event.state);
        }),
      });
    });
    const waitState = async (state: string, from: number) => browser.waitUntil(async () => (await browser.execute(() => (window as any).qaToggleStates)).slice(from).includes(state), { timeout: 3000 });
    try {
      await browser.execute((element: any) => element.focus(), mic);
      await browser.keys("Enter");
      await waitState("listening", 0);
      expect(await mic.getAttribute("aria-pressed")).toBe("true");
      console.log("Native microphone focus after start", await browser.execute((element: any) => ({ tag: document.activeElement?.tagName, sameButton: document.activeElement === element }), mic));
      await browser.waitUntil(async () => (await mic.getAttribute("aria-disabled")) === "false");
      await browser.keys(" ");
      await waitState("empty", 0);
      const before = await browser.execute(() => (window as any).qaToggleStates.length);
      await mic.click();
      await waitState("listening", before);
      await $("textarea").moveTo();
      expect(await mic.getAttribute("aria-pressed")).toBe("true");
      await browser.waitUntil(async () => (await mic.getAttribute("aria-disabled")) === "false");
      await mic.click();
      await waitState("empty", before);
      const last = await browser.execute(() => (window as any).qaToggleStates.length);
      await mic.click();
      await waitState("listening", last);
      await browser.keys("Escape");
      await waitState("cancelled", last);
      expect(await $("textarea").getValue()).toBe("");
    } finally {
      await browser.execute(async (id) => {
        const invoke = (window as any).__TAURI_INTERNALS__.invoke;
        await invoke("ptt_cancel");
        await invoke("plugin:event|unlisten", { event: "aura://event", eventId: id });
      }, eventId);
    }
  });
});
