// QA-027: HTML preview works in the packaged release (CSP) without running
// scripts or touching the network.
import { readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { responsesUpstream } from "../responses";

describe("HTML preview (QA-027)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA reply"); });
  after(async () => { await up.close(); });

  it("renders the file in a sandbox with no scripts and no network", async () => {
    await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      await i("providers_save", { draft: { name: "QA preview", preset: "ollama", baseUrl: url }, credential: null });
    }, up.baseUrl);
    await browser.refresh();
    const box = await $("textarea");
    await box.waitForDisplayed({ timeout: 20_000 });
    await box.setValue("QA html preview");
    await browser.keys("Enter");
    await browser.waitUntil(async () => (await browser.execute(() => document.body.innerText)).includes("QA reply"), { timeout: 150_000 });

    const root = path.join(process.env.AURA_HOME!, "workspaces");
    const workspace = readdirSync(root).map((d) => path.join(root, d)).filter((d) => statSync(d).isDirectory() && !d.endsWith("_ephemeral")).sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs)[0];
    const port = new URL(up.baseUrl).port;
    writeFileSync(path.join(workspace, "qa.html"), `<h1 id="qa">QA HTML preview</h1><p style="color:rgb(200,30,30)">styled</p><script>document.getElementById("qa").textContent = "SCRIPT RAN"; fetch("http://127.0.0.1:${port}/beacon");</script><img src="http://127.0.0.1:${port}/pixel.png">`);
    const before = up.requests.length;

    await (await $('button[aria-label="Files and changes"]')).click();
    const file = await $("button*=qa.html");
    await file.waitForDisplayed({ timeout: 15_000 });
    await file.click();
    const frame = await $('iframe[title="qa.html"]');
    await frame.waitForDisplayed({ timeout: 10_000 });
    await browser.pause(1500);
    await browser.switchFrame(frame);
    const inside = await browser.execute(() => ({ h1: document.getElementById("qa")?.textContent ?? null, color: getComputedStyle(document.querySelector("p")!).color }));
    await browser.switchFrame(null);
    console.log("Native iframe content", JSON.stringify(inside), "requests after preview", JSON.stringify(up.requests.slice(before).map((r) => r.url)));
    expect(inside.h1).toBe("QA HTML preview");
    expect(inside.color).toBe("rgb(200, 30, 30)");
    expect(up.requests.slice(before).filter((r) => /beacon|pixel/.test(r.url))).toHaveLength(0);
    await browser.saveScreenshot(path.resolve(process.cwd(), "../../../target/qa-tools/html-preview-native.png"));
  });
});
