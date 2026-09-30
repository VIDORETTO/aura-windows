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
    await $("li*=Tela").waitForDisplayed({ timeout: 10_000 });
  });

  it("closes with Esc", async () => {
    await browser.keys("Escape");
    await browser.waitUntil(async () => !(await $("textarea").isDisplayed()), { timeout: 5_000 });
  });
});
