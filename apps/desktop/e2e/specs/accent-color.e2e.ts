// 012 AC-002: a custom accent (#e4572e) chosen in Settings › General colors
// the accent elements of Settings and the Overlay, survives a restart, and
// "Default" brings back #5b5bf7.
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

/** Computed accent of the current window: the CSS variable and a real element. */
const accentHere = () =>
  browser.execute(() => {
    const probe = document.createElement("div");
    probe.className = "bg-accent";
    document.body.appendChild(probe);
    const bg = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return { variable: getComputedStyle(document.documentElement).getPropertyValue("--accent").trim(), bg };
  });

describe("Accent color (012)", () => {
  it("applies a custom hex to Settings and the Overlay and keeps it after restart", async () => {
    const overlay = await browser.getWindowHandle();
    await invoke("settings_update", { patch: { language: "en", accentColor: null } });
    await invoke("settings_open", { section: "general" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    const settings = (await browser.getWindowHandles()).find((h) => h !== overlay)!;
    await browser.switchToWindow(settings);
    const hex = await $('input[aria-label="Color hex code"]');
    await hex.waitForDisplayed({ timeout: 15_000 });
    await hex.click();
    await browser.keys(["Control", "a"]);
    await browser.keys("#e4572e");
    await browser.keys("Enter");
    await browser.waitUntil(async () => (await invoke("settings_get")).accentColor === "#e4572e", { timeout: 10_000 });
    const inSettings = await accentHere();
    const primary = await browser.execute(() => {
      const b = [...document.querySelectorAll("button")].find((x) => x.className.includes("bg-accent"));
      return b ? { bg: getComputedStyle(b).backgroundColor, color: getComputedStyle(b).color } : null;
    });
    console.log("Native accent in Settings", JSON.stringify({ inSettings, primary }));
    expect(inSettings.bg).toBe("rgb(228, 87, 46)");

    await browser.switchToWindow(overlay);
    await browser.waitUntil(async () => (await accentHere()).bg === "rgb(228, 87, 46)", { timeout: 10_000, timeoutMsg: "Overlay did not follow the accent" });
    console.log("Native accent in Overlay", JSON.stringify(await accentHere()));

    await browser.reloadSession();
    await browser.waitUntil(() => browser.execute(() => "__TAURI_INTERNALS__" in window), { timeout: 30_000 });
    await browser.waitUntil(async () => (await accentHere()).bg === "rgb(228, 87, 46)", { timeout: 15_000, timeoutMsg: "accent lost after restart" });
    console.log("Native accent after restart", JSON.stringify(await accentHere()));

    await invoke("settings_open", { section: "general" });
    const main = await browser.getWindowHandle();
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== main)!);
    const reset = await $('button[role="radio"][aria-label="Default"]');
    await reset.waitForDisplayed({ timeout: 15_000 });
    await reset.click();
    await browser.waitUntil(async () => (await invoke("settings_get")).accentColor === null, { timeout: 10_000 });
    const back = await accentHere();
    console.log("Native accent after Default", JSON.stringify(back));
    expect(back.bg).toBe("rgb(91, 91, 247)");
  });
});
