// Real updater and About page, including an opt-in installed-version transition.
import { spawnSync } from "node:child_process";

describe("Signed update check (0.2.0)", () => {
  const openAbout = async () => {
    const handles = await browser.getWindowHandles();
    const overlay = handles[0]!;
    await browser.switchToWindow(overlay);
    await browser.execute(async () => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      await i("settings_open", { section: "about" });
    });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find(h => h !== overlay)!);
  };

  it("reports the actual public endpoint outcome persistently", async function () {
    await openAbout();
    expect(await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("updater_configured"))).toBe(true);
    await $("button=Check for updates").click();
    if (process.env.AURA_E2E_UPDATER_EXPECT_ERROR === "1") {
      const error = await $('[role="alert"]');
      await error.waitForDisplayed({ timeout: 25000 });
      expect(await error.getText()).toContain("Could not fetch a valid release JSON");
      await browser.pause(5500);
      expect(await error.isDisplayed()).toBe(true);
    } else if (process.env.AURA_E2E_UPDATE_EXPECT_AVAILABLE === "1") {
      await browser.waitUntil(async () => (await browser.execute(() => document.body.textContent ?? "")).includes("Version 0.2.0 is available."), {
        timeout: 25_000,
        timeoutMsg: "the installed app did not show the 0.2.0 update",
      });
      expect(await $("button=Install and restart").isDisplayed()).toBe(true);
    } else {
      await browser.waitUntil(async () => (await browser.execute(() => document.body.textContent ?? "")).includes("You are on the latest version."), {
        timeout: 25_000,
        timeoutMsg: "the app did not report its update status",
      });
    }
    expect(await browser.execute(() => document.body.textContent)).not.toContain("Automatic updates are not set up");
    const button = process.env.AURA_E2E_UPDATE_EXPECT_AVAILABLE === "1"
      ? await $("button=Install and restart")
      : await $("button=Check for updates");
    expect(await button.isEnabled()).toBe(true);
  });

  if (process.env.AURA_E2E_UPDATE_INSTALL === "1") {
    it("installs the signed transition release from the running 0.1.0 app", async () => {
    const installedExe = process.env.AURA_E2E_INSTALLED_EXE;
    if (!installedExe) throw new Error("AURA_E2E_INSTALLED_EXE is required for the install journey");
    await openAbout();
    const alreadyAvailable = await browser.execute(() => (document.body.textContent ?? "").includes("Version 0.2.0 is available."));
    if (!alreadyAvailable) {
      await $("button=Check for updates").click();
      await browser.waitUntil(async () => (await browser.execute(() => document.body.textContent ?? "")).includes("Version 0.2.0 is available."), {
        timeout: 25_000,
        timeoutMsg: "the installed 0.1.0 app did not show the 0.2.0 update",
      });
    }
    await $("button=Install and restart").click();
    await browser.waitUntil(() => {
      const result = spawnSync("powershell", ["-NoProfile", "-Command", "(Get-Item -LiteralPath $env:AURA_E2E_INSTALLED_EXE).VersionInfo.ProductVersion"], {
        encoding: "utf8",
        env: process.env,
        timeout: 10_000,
      });
      return result.status === 0 && result.stdout.trim().startsWith("0.2.0");
    }, { timeout: 180_000, interval: 2_000, timeoutMsg: "the installed app did not advance to 0.2.0" });
  });
  }
});
