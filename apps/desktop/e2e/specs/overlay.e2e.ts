// Journeys of 001/002 in demo mode (fake agent, synthetic OS).
describe("Overlay", () => {
  it("opens on first run and answers a question with streaming", async () => {
    const input = await $("textarea");
    await input.waitForDisplayed({ timeout: 20_000 });
    await input.setValue("explique este erro");
    await browser.keys("Enter");
    const answer = await $("div.md");
    await answer.waitForDisplayed({ timeout: 20_000 });
    await browser.waitUntil(async () => (await answer.getText()).length > 20, { timeout: 20_000 });
  });

  it("captures the screen into a chip with Ctrl+Shift+S", async () => {
    const input = await $("textarea");
    await input.click();
    await browser.keys(["Control", "Shift", "s"]);
    const chip = await $("li*=Tela");
    await chip.waitForDisplayed({ timeout: 10_000 });
    // The hover preview is served by the asset protocol (scope follows AURA_HOME).
    await chip.moveTo();
    const thumb = await $("img[data-chip-preview]");
    await thumb.waitForExist({ timeout: 5_000 });
    await browser.waitUntil(async () => (await thumb.getProperty("naturalWidth")) > 0, { timeout: 5_000 });
  });

  it("Esc keeps it open; Minimize sends it to the tray", async () => {
    // Hiding the native window leaves the DOM "displayed": ask the window itself.
    const visible = () =>
      browser.execute(() => (window as any).__TAURI_INTERNALS__.invoke("plugin:window|is_visible", { label: "overlay" }));
    await browser.keys("Escape");
    await browser.pause(500);
    expect(await visible()).toBe(true);
    await $('button[aria-label="Minimizar para a bandeja"]').click();
    await browser.waitUntil(async () => (await visible()) === false, { timeout: 5_000 });
  });
});
