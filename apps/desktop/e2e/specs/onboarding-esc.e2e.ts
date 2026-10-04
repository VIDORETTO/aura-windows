// QA-040: the first-run guide describes how the Overlay really closes
// (shortcut or Minimize; Esc only for menus/history/voice).
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

describe("Onboarding shortcut text (QA-040)", () => {
  it("matches the Escape behaviour", async () => {
    await invoke("settings_update", { patch: { language: "en", onboarded: false } });
    await browser.refresh();
    const guide = await $('section[aria-label="Getting started"]');
    await guide.waitForDisplayed({ timeout: 30_000 });
    await (await guide.$("button=Next")).click();
    const text = await guide.getText();
    console.log("Native shortcut step", JSON.stringify(text.split("\n").slice(0, 3)));
    expect(text).toContain("Use the shortcut to open and close Aura, or Minimize to the tray. Esc only closes menus, history and voice recording.");
    expect(text).not.toContain("Esc to close");
    await browser.keys("Escape");
    await browser.pause(500);
    expect(await guide.isDisplayed()).toBe(true);
    await invoke("settings_update", { patch: { onboarded: true } });
  });
});
