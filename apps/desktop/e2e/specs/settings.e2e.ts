// Settings opens as its own window; privacy pause toggles the indicator.
describe("Settings", () => {
  it("pauses privacy from the Privacy page", async () => {
    await browser.execute(() => window.location.assign("index.html#/settings/privacy"));
    const pause = await $('button[role="switch"][aria-label="Pausar toda captura"]');
    await pause.waitForDisplayed({ timeout: 10_000 });
    await pause.click();
    await browser.waitUntil(async () => (await pause.getAttribute("aria-checked")) === "true", { timeout: 5_000 });
  });
});
