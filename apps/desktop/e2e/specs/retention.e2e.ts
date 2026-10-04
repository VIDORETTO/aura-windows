// QA-030: retention days/space are edited in Privacy and persist.
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

describe("Retention (QA-030)", () => {
  it("saves 3 days, 5 GB and manual recordings, and keeps them after restart", async () => {
    const overlay = await browser.getWindowHandle();
    await invoke("settings_update", { patch: { language: "en" } });
    await invoke("privacy_set_retention", { days: 7, maxGb: 20, applyToManual: false });
    await invoke("settings_open", { section: "privacy" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    const days = await $('input[aria-label="Keep recordings for (days)"]');
    await days.waitForDisplayed({ timeout: 15_000 });
    const type = async (el: WebdriverIO.Element, value: string) => {
      await el.click();
      await browser.keys(["Control", "a"]);
      await browser.keys(value);
    };
    await type(days, "3");
    await type(await $('input[aria-label="Maximum space (GB)"]'), "5");
    await (await $('button[role="switch"][aria-label="Also apply to manual recordings"]')).click();
    await (await $("button=Save retention")).click();
    await browser.waitUntil(async () => ((await invoke("privacy_get")) as any).retention.days === 3, { timeout: 10_000 });
    await browser.reloadSession();
    await browser.waitUntil(() => browser.execute(() => "__TAURI_INTERNALS__" in window), { timeout: 20_000 });
    const after = ((await invoke("privacy_get")) as any).retention;
    console.log("Native retention after restart", JSON.stringify(after));
    expect(after).toEqual({ days: 3, maxGb: 5, applyToManual: true });
    await invoke("privacy_set_retention", { days: 7, maxGb: 20, applyToManual: false });
  });
});
