// Real updater and real About page. A held release has no public manifest yet.
describe("Signed update check (0.2.0)", () => {
  it("reports the actual public endpoint outcome persistently", async () => {
    const overlay = await browser.getWindowHandle();
    await browser.execute(async () => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      await i("settings_open", { section: "about" });
    });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find(h => h !== overlay)!);
    expect(await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("updater_configured"))).toBe(true);
    await $("button=Check for updates").click();
    if (process.env.AURA_E2E_UPDATER_EXPECT_ERROR === "1") {
      const error = await $('[role="alert"]');
      await error.waitForDisplayed({ timeout: 25000 });
      expect(await error.getText()).toContain("Could not fetch a valid release JSON");
      await browser.pause(5500);
      expect(await error.isDisplayed()).toBe(true);
    } else {
      await $("*=You are on the latest version.").waitForDisplayed({ timeout: 25000 });
    }
    expect(await browser.execute(() => document.body.textContent)).not.toContain("Automatic updates are not set up");
    expect(await $("button=Check for updates").isEnabled()).toBe(true);
  });
});
