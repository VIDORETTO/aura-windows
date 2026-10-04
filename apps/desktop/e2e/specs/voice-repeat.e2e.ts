describe("Repeated dictation results (QA-008)", () => {
  it("inserts equal results from new cycles, after clearing, and ignores terminal duplication", async () => {
    const box = await $("textarea");
    await box.waitForDisplayed();
    await box.click();
    await browser.keys(["Control", "a"]);
    await browser.keys("Backspace");
    const emit = async (state: string, text?: string) => browser.execute(async (state, text) => {
      await (window as any).__TAURI_INTERNALS__.invoke("plugin:event|emit", {
        event: "aura://event", payload: { channel: "voice", event: text === undefined ? { state } : { state, text } },
      });
    }, state, text);
    const cycle = async () => { await emit("listening"); await emit("done", "mesma frase"); };
    // ASR output is controlled at the event boundary. UI, IPC and WebView are real.
    await cycle();
    await browser.waitUntil(async () => (await box.getValue()) === "mesma frase");
    await cycle();
    await browser.waitUntil(async () => (await box.getValue()) === "mesma frase mesma frase", { timeout: 3000 });
    await emit("done", "mesma frase");
    expect(await box.getValue()).toBe("mesma frase mesma frase");
    await box.click();
    await browser.keys(["Control", "a"]);
    await browser.keys("Backspace");
    expect(await box.getValue()).toBe("");
    await cycle();
    await browser.waitUntil(async () => (await box.getValue()) === "mesma frase");
    await emit("done", "mesma frase");
    expect(await box.getValue()).toBe("mesma frase");
  });
});
