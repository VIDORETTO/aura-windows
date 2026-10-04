// QA-031: an agent screen capture is logged with exact time, the consent
// outcome, a thumbnail of what was delivered and a link to the conversation.
import { responsesUpstream } from "../responses";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

describe("Access log (QA-031)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  let toolName = "";
  before(async () => {
    up = await responsesUpstream((body) => {
      const input = JSON.stringify(body?.input ?? "");
      if (input.includes("function_call_output")) return "QA capture received";
      // MCP tools come grouped: {"type":"namespace","name":"mcp__aura","tools":[..]}.
      const aura = (body?.tools ?? []).find((t: any) => t.type === "namespace" && t.name === "mcp__aura");
      const tool = (aura?.tools ?? []).map((t: any) => t.name).find((n: string) => n === "screen_capture");
      if (!tool) return "QA no capture tool";
      toolName = `${aura.name}.${tool}`;
      return { functionCall: { namespace: aura.name, name: tool, arguments: JSON.stringify({ reason: "QA access log", target: "window" }) } };
    }, [{ id: "qa-model", architecture: { input_modalities: ["text", "image"] }, supported_parameters: ["tools"] }]);
  });
  after(async () => { await up.close(); });

  it("logs the consented capture with thumbnail and opens its conversation", async () => {
    const overlay = await browser.getWindowHandle();
    await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      const p = await i("providers_save", { draft: { name: "QA access", preset: "ollama", baseUrl: url }, credential: null });
      await i("providers_test", { id: p.id });
    }, up.baseUrl);
    await invoke("privacy_clear_access_log");
    await browser.refresh();
    const box = await $("textarea");
    await box.waitForDisplayed({ timeout: 20_000 });
    // The loopback model (tools + images), not a model left over in the profile.
    await browser.keys(["Control", "ArrowDown"]);
    await $('button[aria-label^="Model "]').click();
    const dialog = await $('[role="dialog"][aria-label="Model and mode"]');
    await dialog.waitForDisplayed();
    await (await dialog.$('[role="menu"][aria-label="Model"]')).$("button*=qa-model").click();
    await browser.keys("Escape");
    await box.setValue("QA access turn");
    await browser.keys("Enter");
    const allow = await $("button=Allow in this conversation");
    await allow.waitForDisplayed({ timeout: 150_000, timeoutMsg: "no consent request" });
    await allow.click();
    await browser.waitUntil(async () => (await browser.execute(() => document.body.innerText)).includes("QA capture received"), { timeout: 90_000, timeoutMsg: "no reply after capture" });
    console.log("Native tool name", toolName);

    const log = await invoke("privacy_access_log", { limit: 5 });
    const entry = log.find((e: any) => e.tool === "screen_capture");
    console.log("Native access entry", JSON.stringify({ ...entry, conversation: entry?.conversation ? "<uuid>" : null }));
    expect(entry).toMatchObject({ decision: "allow", reason: "consent", hasThumbnail: true });
    expect(typeof entry.threadId).toBe("string");

    await invoke("settings_open", { section: "privacy" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    const row = await $(`li[aria-label^="screen_capture"]`);
    await row.waitForDisplayed({ timeout: 15_000 });
    const text = await row.getText();
    console.log("Native access row", text.replace(/\n/g, " | "));
    expect(text).toMatch(/\d{2}\/\d{2}\/\d{4}.*\d{2}:\d{2}:\d{2}/);
    expect(text).toContain("Allowed by the user");
    await (await row.$("button=Show thumbnail")).click();
    const img = await row.$('img[alt="Thumbnail of what the agent received"]');
    await img.waitForDisplayed({ timeout: 10_000 });
    const size = await browser.execute((el: any) => ({ w: el.naturalWidth, h: el.naturalHeight, src: el.src.slice(0, 22) }), img);
    console.log("Native thumbnail", JSON.stringify(size));
    expect(size.src).toBe("data:image/png;base64,");
    expect(Math.max(size.w, size.h)).toBeLessThanOrEqual(240);
    expect(size.w).toBeGreaterThan(0);

    // Start a fresh conversation so "Open conversation" has to switch back.
    await browser.switchToWindow(overlay);
    await (await $('button[aria-label="New conversation"]')).click();
    await browser.waitUntil(async () => !(await browser.execute(() => document.body.innerText)).includes("QA capture received"), { timeout: 10_000, timeoutMsg: "new conversation still shows the old one" });
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    await (await row.$("button=Open conversation")).click();
    await browser.switchToWindow(overlay);
    await browser.waitUntil(async () => (await browser.execute(() => document.body.innerText)).includes("QA capture received"), { timeout: 30_000, timeoutMsg: "conversation not reopened" });
    console.log("Native conversation reopened", await browser.execute(() => document.visibilityState));
    await invoke("privacy_clear_access_log");
  });
});
