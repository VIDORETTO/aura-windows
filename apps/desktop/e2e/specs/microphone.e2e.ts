import path from "node:path";

describe("Preferred native microphone (QA-007)", () => {
  it("uses both available microphones and preserves the preference across app restarts", async () => {
    const fifine = "Microfone (fifine Microphone)";
    const cable = "CABLE Output (VB-Audio Virtual Cable)";
    const overlay = await browser.getWindowHandle();
    await browser.execute(async () => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      await invoke("settings_update", { patch: { language: "en" } });
      await invoke("settings_open", { section: "voice" });
    });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((handle) => handle !== overlay)!);
    const devices = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("audio_devices", { system: false }));
    expect((devices as any[]).map((device) => device.id)).toContain(fifine);
    expect((devices as any[]).map((device) => device.id)).toContain(cable);
    const select = await $('select[aria-label="Microphone"]');
    await select.waitForDisplayed();
    const eventId = await browser.execute(async () => {
      const internals = (window as any).__TAURI_INTERNALS__;
      (window as any).qaVoiceStates = [];
      return internals.invoke("plugin:event|listen", {
        event: "aura://event", target: { kind: "Any" },
        handler: internals.transformCallback((event: any) => {
          if (event.payload.channel === "voice") (window as any).qaVoiceStates.push(event.payload.event.state);
        }),
      });
    });
    const listenAndCancel = async () => {
      const before = await browser.execute(() => (window as any).qaVoiceStates.length);
      await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("ptt_press", { device: null }));
      await browser.waitUntil(async () => (await browser.execute(() => (window as any).qaVoiceStates)).slice(before).includes("listening"));
      // No ASR call, retained recording or external transmission: close the real WASAPI hub.
      await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("ptt_cancel"));
      await browser.waitUntil(async () => (await browser.execute(() => (window as any).qaVoiceStates)).slice(before).includes("cancelled"));
    };
    try {
      if (process.env.AURA_E2E_MIC_STAGE === "restart") {
        const persisted = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_get"));
        expect((persisted as any).microphoneDeviceId).toBe(cable);
        expect(await select.getValue()).toBe(cable);
        await listenAndCancel();
        await select.selectByAttribute("value", "");
        await browser.waitUntil(async () => (await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_get")) as any).microphoneDeviceId === null);
        expect(await select.getValue()).toBe("");
        await listenAndCancel();
      } else {
        for (const device of [fifine, cable]) {
          await select.selectByAttribute("value", device);
          await browser.waitUntil(async () => (await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_get")) as any).microphoneDeviceId === device);
          expect(await select.getValue()).toBe(device);
          await listenAndCancel();
          console.log("Native QA microphone opened and cancelled", device);
        }
      }
      expect(await browser.execute(() => (window as any).qaVoiceStates)).toEqual(["listening", "cancelled", "listening", "cancelled"]);
      await select.scrollIntoView();
      await browser.saveScreenshot(path.resolve(import.meta.dirname, `../../../../target/qa-tools/microphone-native-${process.env.AURA_E2E_MIC_STAGE ?? "initial"}.png`));
    } finally {
      await browser.execute(async (id) => (window as any).__TAURI_INTERNALS__.invoke("plugin:event|unlisten", { event: "aura://event", eventId: id }), eventId);
    }
  });
});
