// QA-017: "/compactar" compacts the real conversation through the app-server
// (never sent as a prompt) and shows a compaction item.
import { responsesUpstream } from "../responses";

describe("/compactar (QA-017)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  // The app-server's compaction request carries its summarization prompt.
  before(async () => { up = await responsesUpstream((body) => (JSON.stringify(body).includes("seamlessly continue the work") ? "QA summary of the conversation" : "QA answer one")); });
  after(async () => { await up.close(); });

  it("runs compaction for the open conversation instead of sending the text", async () => {
    await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      const p = await i("providers_save", { draft: { name: "QA compact", preset: "ollama", baseUrl: url }, credential: null });
      await i("providers_test", { id: p.id });
    }, up.baseUrl);
    await browser.refresh();
    const box = await $("textarea");
    await box.waitForDisplayed({ timeout: 20_000 });
    await box.setValue("QA compact turn");
    await browser.keys("Enter");
    await browser.waitUntil(async () => (await browser.execute(() => document.body.innerText)).includes("QA answer one"), { timeout: 150_000, timeoutMsg: "no assistant reply" });
    const before = up.requests.length;
    console.log("Native first turn requests", before);

    await box.setValue("/compactar");
    await browser.keys("Enter");
    await $('[role="separator"][aria-label="Conversation compacted"]').waitForDisplayed({ timeout: 90_000 });
    const after = up.requests.slice(before);
    console.log("Native compaction requests", JSON.stringify(after.map((r) => ({ url: r.url, input: JSON.stringify(r.body?.input ?? "").slice(-300) }))));
    expect(after.length).toBeGreaterThan(0);
    // The command itself never reaches the model as user text.
    expect(after.some((r) => JSON.stringify(r.body?.input ?? "").includes("/compactar"))).toBe(false);
    expect(await box.getValue()).toBe("");

    // Context was reduced: the next turn carries the summary, not the old exchange.
    const mark = up.requests.length;
    await box.setValue("QA after compact");
    await browser.keys("Enter");
    await browser.waitUntil(() => up.requests.length > mark, { timeout: 60_000, timeoutMsg: "no turn after compaction" });
    const next = JSON.stringify(up.requests[mark].body?.input ?? "");
    console.log("Native turn after compaction", { hasSummary: next.includes("QA summary of the conversation"), hasOldAnswer: next.includes("QA answer one"), chars: next.length });
    expect(next).toContain("QA summary of the conversation");
    expect(next).not.toContain("QA answer one");
  });
});
