// QA-015/016: History rename and paging against the real app-server.
// Conversations are created through the app with a loopback upstream that
// answers every model request with an error (no inference, no account).
import { createServer } from "node:http";

async function upstream() {
  const server = createServer((req, res) => {
    req.resume();
    if (req.url === "/v1/models") {
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify({ data: [{ id: "qa-history-model" }] }));
      return;
    }
    res.writeHead(400, { "content-type": "application/json" });
    res.end(JSON.stringify({ error: { message: "QA history upstream", type: "invalid_request_error" } }));
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("no port");
  return { baseUrl: `http://127.0.0.1:${address.port}/v1`, close: () => new Promise<void>((r) => server.close(() => r())) };
}

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

async function useLoopbackProvider(baseUrl: string) {
  await browser.execute(async (url) => {
    const i = (window as any).__TAURI_INTERNALS__.invoke;
    await i("settings_update", { patch: { language: "en" } });
    for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
    const p = await i("providers_save", { draft: { name: "QA history", preset: "ollama", baseUrl: url }, credential: null });
    await i("providers_test", { id: p.id });
  }, baseUrl);
}

/** Starts a conversation through the Host and sends one turn. */
async function createConversation(text: string) {
  return browser.execute(async (t) => {
    const i = (window as any).__TAURI_INTERNALS__.invoke;
    const providers = await i("providers_list");
    const started = await i("conversation_start", { options: { mode: { mode: "chat" }, provider: `aura-${providers[0].id}`, model: "qa-history-model", ephemeral: false } });
    await i("conversation_send", { request: { threadId: started.threadId, text: t, tray: started.threadId } }).catch(() => undefined);
    return started.threadId as string;
  }, text);
}

describe("History rename (QA-015)", () => {
  let up: Awaited<ReturnType<typeof upstream>>;
  before(async () => { up = await upstream(); });
  after(async () => { await up.close(); });

  it("renames a real conversation from the History panel and keeps the name after restart", async () => {
    await useLoopbackProvider(up.baseUrl);
    const id = await createConversation("QA rename original");
    let summary: any;
    await browser.waitUntil(async () => {
      const page = (await invoke("conversation_history", { query: { limit: 50 } })) as any;
      summary = page.items.find((c: any) => c.id === id);
      return !!summary;
    }, { timeout: 60_000, timeoutMsg: "conversation not in history" });
    console.log("Native history item before", JSON.stringify(summary));
    await browser.refresh();
    await $("textarea").waitForDisplayed({ timeout: 20_000 });
    await browser.keys(["Control", "h"]);
    const panel = await $('aside[aria-label="History"]');
    await panel.waitForDisplayed();
    const title = summary.title || "…";
    const row = await panel.$(`button*=${title}`);
    await row.waitForDisplayed({ timeout: 10_000 });
    await row.moveTo();
    await (await panel.$(`button[aria-label="Rename ${title}"]`)).click();
    const field = await panel.$('input[aria-label="New conversation name"]');
    await field.waitForDisplayed();
    // Keyboard only: WebDriver clear() blurs the field, which ends the edit.
    await browser.keys(["Control", "a"]);
    await browser.keys("QA renamed via UI");
    await browser.keys("Enter");
    await (await panel.$("button*=QA renamed via UI")).waitForDisplayed({ timeout: 10_000 });

    await browser.reloadSession();
    await browser.waitUntil(() => browser.execute(() => "__TAURI_INTERNALS__" in window), { timeout: 20_000 });
    let after: any;
    await browser.waitUntil(async () => {
      const page = (await invoke("conversation_history", { query: { limit: 50 } })) as any;
      after = page.items.find((c: any) => c.id === id);
      return after?.title === "QA renamed via UI";
    }, { timeout: 60_000, timeoutMsg: "renamed title not persisted" });
    console.log("Native history item after restart", JSON.stringify(after));
  });
});

describe("History paging (QA-016)", () => {
  let up: Awaited<ReturnType<typeof upstream>>;
  before(async () => { up = await upstream(); });
  after(async () => { await up.close(); });

  it("reaches conversations beyond the first 50 with Load more", async () => {
    await useLoopbackProvider(up.baseUrl);
    for (let n = 1; n <= 55; n++) await createConversation(`QA page ${String(n).padStart(3, "0")}`);
    await browser.waitUntil(async () => {
      const first = (await invoke("conversation_history", { query: { limit: 50 } })) as any;
      return first.items.length === 50 && first.nextCursor;
    }, { timeout: 60_000, timeoutMsg: "app-server did not page" });
    await browser.refresh();
    await $("textarea").waitForDisplayed({ timeout: 20_000 });
    await browser.keys(["Control", "h"]);
    const panel = await $('aside[aria-label="History"]');
    await panel.waitForDisplayed();
    const count = () => browser.execute(() => document.querySelectorAll('aside[aria-label="History"] li button.min-w-0').length);
    await browser.waitUntil(async () => (await count()) === 50, { timeout: 10_000, timeoutMsg: "first page is not 50" });
    const firstPage = await panel.getText();
    expect(firstPage).not.toContain("QA page 001");
    await (await panel.$("button=Load more")).click();
    await (await panel.$("button*=QA page 001")).waitForDisplayed({ timeout: 10_000 });
    const total = await count();
    console.log("Native history rows after Load more", total);
    expect(total).toBeGreaterThan(50);
    // Every conversation is reachable, including those created in the same
    // second as the last row of the first page (whole-second cursor).
    const text = await panel.getText();
    const missing = Array.from({ length: 55 }, (_, i) => `QA page ${String(i + 1).padStart(3, "0")}`).filter((t) => !text.includes(t));
    expect(missing).toEqual([]);
  });
});
