// 014: switching Chat → Task → Plan in the middle of a conversation shows a
// divider, the next turn tells the agent the new mode (the real pinned
// app-server keeps the start-time developer instructions) and the
// announcement never shows when the conversation is reopened.
import { responsesUpstream } from "../responses";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

describe("Mode switch mid-conversation (014)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA mode reply", [{ id: "qa-model", architecture: { input_modalities: ["text"] }, supported_parameters: ["tools"] }]); });
  after(async () => { await up.close(); });

  it("tells the agent about Task and Plan after a Chat start", async () => {
    await invoke("settings_update", { patch: { language: "en", effortPresets: {} } });
    for (const p of await invoke("providers_list")) await invoke("providers_remove", { id: p.id });
    const p = await invoke("providers_save", { draft: { name: "QA model", preset: "ollama", baseUrl: up.baseUrl }, credential: null });
    await invoke("providers_test", { id: p.id });
    await browser.refresh();
    const box = await $("textarea");
    await box.waitForDisplayed({ timeout: 30_000 });
    await browser.keys(["Control", "ArrowDown"]);
    const openPicker = async () => {
      await $('button[aria-label^="Model "]').click();
      const d = await $('[role="dialog"][aria-label="Model and mode"]');
      await d.waitForDisplayed();
      return d;
    };
    let picker = await openPicker();
    await (await picker.$("button=QA model")).click();
    await (await picker.$('[role="menu"][aria-label="Model"]')).$("button*=qa-model").click();
    await (await picker.$('[role="menu"][aria-label="Mode"]')).$("button*=Chat").click();
    await browser.keys("Escape");

    const turn = async (text: string) => {
      const before = up.requests.filter((r) => r.url === "/v1/responses").length;
      await (await $("textarea")).setValue(text);
      await browser.keys("Enter");
      await browser.waitUntil(() => up.requests.filter((r) => r.url === "/v1/responses").length > before, { timeout: 180_000, timeoutMsg: `no turn for ${text}` });
      await browser.waitUntil(async () => (await browser.execute(() => document.querySelectorAll("div.md").length)) > 0 && !(await $('button[aria-label="Stop"]').isExisting()), { timeout: 60_000 });
      const req = up.requests.filter((r) => r.url === "/v1/responses").at(-1)!;
      const input = JSON.stringify(req.body?.input ?? "");
      const lastUser = [...(req.body?.input ?? [])].reverse().find((i: any) => i.role === "user");
      return { input, last: JSON.stringify(lastUser ?? {}), instructions: JSON.stringify(req.body?.instructions ?? "") };
    };
    const first = await turn("QA first in chat");
    console.log("Native turn 1", JSON.stringify({ note: first.last.includes("<aura-mode>") }));
    expect(first.last).not.toContain("<aura-mode>");

    const switchTo = async (mode: string) => {
      picker = await openPicker();
      await (await picker.$('[role="menu"][aria-label="Mode"]')).$(`button*=${mode}`).click();
      await browser.keys("Escape");
      await $(`[role="separator"][aria-label="Mode changed to ${mode}"]`).waitForDisplayed({ timeout: 10_000 });
    };
    await switchTo("Task");
    const second = await turn("QA second in task");
    console.log("Native turn 2", JSON.stringify({ note: /<aura-mode>[^<]*Task mode[^<]*<\/aura-mode>/.test(second.last), chatInstructionStillThere: (second.input + second.instructions).includes("Chat mode") }));
    expect(second.last).toMatch(/<aura-mode>[^<]*Task mode/);

    const third = await turn("QA third still task");
    expect(third.last).not.toContain("<aura-mode>");

    await switchTo("Plan");
    const fourth = await turn("QA fourth in plan");
    console.log("Native turn 4", JSON.stringify({ note: /<aura-mode>[^<]*Plan mode/.test(fourth.last) }));
    expect(fourth.last).toMatch(/<aura-mode>[^<]*Plan mode/);

    // Reopened from history: the user messages have no announcement.
    const thread = (await invoke("conversation_history", { query: {} })).items[0].id;
    const transcript = await invoke("conversation_open", { threadId: thread });
    const users = transcript.filter((m: any) => m.role === "user").map((m: any) => m.text);
    console.log("Native reopened user messages", JSON.stringify(users));
    expect(users).toEqual(["QA first in chat", "QA second in task", "QA third still task", "QA fourth in plan"]);
  });
});
