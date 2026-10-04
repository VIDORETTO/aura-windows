// QA-018: Ctrl+V with an image in the Windows clipboard creates an image chip
// and the image reaches the model input. The clipboard gets a generated 40×20
// PNG; a previous text clipboard is restored afterwards.
import { spawnSync } from "node:child_process";
import { responsesUpstream } from "../responses";

const ps = (script: string) => spawnSync("powershell", ["-NoProfile", "-STA", "-Command", script], { encoding: "utf8", timeout: 30_000 });

describe("Paste image (QA-018)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  let savedText: string | null = null;
  before(async () => {
    up = await responsesUpstream(() => "QA saw the image");
    const saved = ps("Add-Type -AssemblyName System.Windows.Forms; if ([Windows.Forms.Clipboard]::ContainsText()) { [Windows.Forms.Clipboard]::GetText() }");
    savedText = saved.stdout ? saved.stdout.replace(/\r?\n$/, "") : null;
  });
  after(async () => {
    await up.close();
    if (savedText) ps(`Add-Type -AssemblyName System.Windows.Forms; [Windows.Forms.Clipboard]::SetText(@'\n${savedText}\n'@)`);
    else ps("Add-Type -AssemblyName System.Windows.Forms; [Windows.Forms.Clipboard]::Clear()");
  });

  it("attaches the clipboard image and sends it to the model", async () => {
    await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      const p = await i("providers_save", { draft: { name: "QA paste", preset: "ollama", baseUrl: url }, credential: null });
      await i("providers_test", { id: p.id });
    }, up.baseUrl);
    await browser.refresh();
    const set = ps([
      "Add-Type -AssemblyName System.Windows.Forms, System.Drawing",
      "$b = New-Object System.Drawing.Bitmap 40, 20",
      "$g = [System.Drawing.Graphics]::FromImage($b); $g.Clear([System.Drawing.Color]::FromArgb(255, 200, 30, 30)); $g.Dispose()",
      "[Windows.Forms.Clipboard]::SetImage($b)",
      "[Windows.Forms.Clipboard]::ContainsImage()",
    ].join("; "));
    expect(set.stdout.trim()).toBe("True");
    const box = await $("textarea");
    await box.waitForDisplayed({ timeout: 20_000 });
    await box.click();
    await browser.keys(["Control", "v"]);
    const chips = () => browser.execute(() => (document.querySelector('ul[aria-label="Context"]') as HTMLElement | null)?.innerText ?? "");
    await browser.waitUntil(async () => (await chips()).includes("clipboard-"), { timeout: 15_000, timeoutMsg: "no pasted image chip" });
    console.log("Native pasted chip", JSON.stringify(await chips()));
    expect(await box.getValue()).toBe("");

    await box.setValue("What color is it?");
    await browser.keys("Enter");
    await browser.waitUntil(() => up.requests.some((r) => r.url === "/v1/responses"), { timeout: 150_000, timeoutMsg: "no model request" });
    const input = JSON.stringify(up.requests.find((r) => r.url === "/v1/responses")!.body.input);
    const image = /"type":"input_image","image_url":"data:image\/png;base64,([A-Za-z0-9+/=]{20})/.exec(input);
    console.log("Native model input has image", !!image, image?.[1]);
    expect(image).not.toBeNull();
  });
});
