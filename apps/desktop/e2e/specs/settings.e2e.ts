// Settings opens as its own window; privacy pause toggles the indicator.
describe("Settings", () => {
  it("pauses privacy from the Privacy page", async () => {
    const before = await browser.getWindowHandles();
    // Same command the Overlay's gear button uses; it must resolve (a sync
    // command that builds a window never answers on Windows).
    await browser.execute(() => (window as any).__TAURI_INTERNALS__.invoke("settings_open", { section: "privacy" }));
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length > before.length, { timeout: 10_000 });
    const settings = (await browser.getWindowHandles()).find((h) => !before.includes(h))!;
    await browser.switchToWindow(settings);
    const pause = await $('button[role="switch"][aria-label="Pausar toda captura"]');
    await pause.waitForDisplayed({ timeout: 10_000 });
    await pause.click();
    await browser.waitUntil(async () => (await pause.getAttribute("aria-checked")) === "true", { timeout: 5_000 });
  });
});
