// QA-029: the recent-buffer length is chosen in Privacy (1–30 minutes) and
// survives a restart.
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

describe("Recent buffer length (QA-029)", () => {
  it("sets 2 minutes for the screen buffer, clamps 45 to 30 and keeps it after restart", async () => {
    const overlay = await browser.getWindowHandle();
    await invoke("settings_update", { patch: { language: "en" } });
    await invoke("privacy_set_source", { source: "screen", mode: { type: "onDemand" }, agent: "ask" });
    await invoke("settings_open", { section: "privacy" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    await $('select[aria-label="Capture"]').waitForDisplayed({ timeout: 15_000 });
    const mode = (await $$('select[aria-label="Capture"]'))[0];
    await mode.selectByAttribute("value", "recentBuffer");
    const minutes = await $('input[aria-label="Screen buffer minutes"]');
    await minutes.waitForDisplayed({ timeout: 10_000 });
    const type = async (value: string) => {
      await minutes.click();
      await browser.keys(["Control", "a"]);
      await browser.keys(value);
      await browser.keys("Tab");
    };
    await type("2");
    await browser.waitUntil(async () => ((await invoke("privacy_get")) as any).screen.mode.minutes === 2, { timeout: 10_000 });
    await type("45");
    await browser.waitUntil(async () => ((await invoke("privacy_get")) as any).screen.mode.minutes === 30, { timeout: 10_000 });
    expect(await minutes.getValue()).toBe("30");
    await type("2");
    await browser.waitUntil(async () => ((await invoke("privacy_get")) as any).screen.mode.minutes === 2, { timeout: 10_000 });

    await browser.reloadSession();
    await browser.waitUntil(() => browser.execute(() => "__TAURI_INTERNALS__" in window), { timeout: 20_000 });
    const after = (await invoke("privacy_get")) as any;
    console.log("Native screen policy after restart", JSON.stringify(after.screen));
    expect(after.screen.mode).toEqual({ type: "recentBuffer", minutes: 2 });
    await invoke("privacy_set_source", { source: "screen", mode: { type: "onDemand" }, agent: "ask" });
  });
});
