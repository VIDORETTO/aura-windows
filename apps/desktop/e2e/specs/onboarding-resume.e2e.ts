// QA-039: "Resume getting started" in Settings shows the first-run guide in
// the Overlay again.
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

describe("Resume onboarding (QA-039)", () => {
  it("shows the guide again from Settings › General", async () => {
    const overlay = await browser.getWindowHandle();
    await invoke("settings_update", { patch: { language: "en", onboarded: true } });
    await browser.refresh();
    await $("textarea").waitForDisplayed({ timeout: 30_000 });
    expect(await $('section[aria-label="Getting started"]').isExisting()).toBe(false);
    await invoke("overlay_hide");
    await invoke("settings_open", { section: "general" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    await (await $("button=Resume getting started")).click();
    await browser.waitUntil(async () => (await invoke("settings_get")).onboarded === false, { timeout: 10_000 });
    await browser.switchToWindow(overlay);
    const guide = await $('section[aria-label="Getting started"]');
    await guide.waitForDisplayed({ timeout: 15_000 });
    console.log("Native onboarding guide", JSON.stringify((await guide.getText()).split("\n").slice(0, 2)));
    expect(await guide.getText()).toContain("1 of 3");
    await invoke("settings_update", { patch: { onboarded: true } });
  });
});
