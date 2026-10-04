// QA-020: a Custom provider configured from Settings with an API format and
// extra headers; requests use the chosen protocol path and carry the headers.
// Keyless loopback upstream (answers non-Responses paths with 404).
import { responsesUpstream } from "../responses";

describe("Custom provider format and headers (QA-020)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA", [{ id: "qa-custom-model" }]); });
  after(async () => { await up.close(); });

  it("sends Chat Completions with the extra header, then Anthropic Messages after editing the format", async () => {
    const overlay = await browser.getWindowHandle();
    await browser.execute(async () => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      await i("settings_open", { section: "providers" });
    });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    await (await $("button=Add provider")).click();
    const form = await $('[role="group"][aria-label="Add provider"]');
    await (await form.$("select")).selectByAttribute("value", "custom");
    await (await form.$('select[aria-label="API format"]')).selectByAttribute("value", "chat");
    await (await form.$("label=Name").$("input")).setValue("QA custom");
    await (await form.$("label=Base URL").$("input")).setValue(up.baseUrl);
    await (await form.$("button=Add header")).click();
    await (await form.$('input[aria-label="Header name 1"]')).setValue("X-QA-Tenant");
    await (await form.$('input[aria-label="Header value 1"]')).setValue("alpha");
    await (await form.$("button=Save")).click();
    await browser.waitUntil(() => up.requests.some((r) => r.url === "/v1/models"), { timeout: 15_000, timeoutMsg: "no discovery request" });

    const saved = ((await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("providers_list"))) as any[])[0];
    console.log("Native custom provider", JSON.stringify({ wire: saved.wire, headers: saved.extraHeaders, auth: saved.auth }));
    expect(saved.wire).toBe("chat");
    expect(saved.extraHeaders).toEqual({ "X-QA-Tenant": "alpha" });
    expect(up.requests.find((r) => r.url === "/v1/models")!.headers["x-qa-tenant"]).toBe("alpha");

    const turn = (thread?: string) => browser.execute(async (id, t) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      const threadId = t ?? (await i("conversation_start", { options: { mode: { mode: "chat" }, provider: `aura-${id}`, model: "qa-custom-model", ephemeral: false } })).threadId;
      await i("conversation_send", { request: { threadId, text: "QA format probe", tray: threadId } }).catch(() => undefined);
      return threadId as string;
    }, saved.id, thread);
    const thread = await turn();
    await browser.waitUntil(() => up.requests.some((r) => r.url === "/v1/chat/completions"), { timeout: 150_000, timeoutMsg: "no chat completions request" });
    expect(up.requests.find((r) => r.url === "/v1/chat/completions")!.headers["x-qa-tenant"]).toBe("alpha");

    // Edit: switch the same provider to Anthropic Messages.
    await (await $('button[aria-label="Edit QA custom"]')).click();
    const edit = await $('[role="group"][aria-label="Edit provider QA custom"]');
    await (await edit.$('select[aria-label="API format"]')).selectByAttribute("value", "anthropic");
    await (await edit.$("button=Save")).click();
    await browser.waitUntil(async () => ((await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("providers_list"))) as any[])[0].wire === "anthropic", { timeout: 10_000 });
    await turn(thread);
    await browser.waitUntil(() => up.requests.some((r) => r.url === "/v1/messages"), { timeout: 60_000, timeoutMsg: "no anthropic messages request" });
    const msg = up.requests.find((r) => r.url === "/v1/messages")!;
    console.log("Native protocol paths", JSON.stringify(up.requests.map((r) => r.url)), "tenant on messages", msg.headers["x-qa-tenant"], "anthropic-version", msg.headers["anthropic-version"]);
    expect(msg.headers["x-qa-tenant"]).toBe("alpha");
  });
});
