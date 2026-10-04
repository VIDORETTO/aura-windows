// QA-038: while the build has the template updater key, "Check for updates"
// says updates are not set up (no network attempt, no confusing error).
describe("Updates without a signing key (QA-038)", () => {
  it("explains that automatic updates are not configured", async () => {
    const overlay = await browser.getWindowHandle();
    await browser.execute(async () => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      await i("settings_open", { section: "about" });
    });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    const configured = await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("updater_configured").catch((e: unknown) => `error: ${JSON.stringify(e)}`));
    console.log("Native updater_configured", JSON.stringify(configured));
    await (await $("button=Check for updates")).click();
    await browser.waitUntil(async () => /not set up|error|Error|failed/i.test(await browser.execute(() => document.body.innerText)), { timeout: 30_000, timeoutMsg: "no update outcome" });
    const line = (await browser.execute(() => document.body.innerText)).split("\n").find((l) => /not set up|error|Error|failed/i.test(l));
    console.log("Native update outcome", line);
    expect(line).toBe("Automatic updates are not set up in this build: the signing public key is missing.");
  });
});
