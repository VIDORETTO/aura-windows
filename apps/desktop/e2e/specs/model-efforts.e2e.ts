// 013: GPT-6.1 Sol is recognized with its efforts and limits; a custom
// provider model declares its efforts in Settings; the Overlay remembers the
// effort per model and mode and sends it. Loopback provider only.
import { responsesUpstream } from "../responses";

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

describe("Models and reasoning effort (013)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA effort reply", [{ id: "gpt-6.1-sol" }, { id: "qa-plain" }]); });
  after(async () => { await up.close(); });

  it("recognizes GPT-6.1 Sol, saves custom model efforts and remembers the effort per mode", async () => {
    const overlay = await browser.getWindowHandle();
    await invoke("settings_update", { patch: { language: "en", effortPresets: {} } });
    for (const p of await invoke("providers_list")) await invoke("providers_remove", { id: p.id });
    const saved = await invoke("providers_save", { draft: { name: "QA", preset: "custom", wire: "responses", baseUrl: up.baseUrl }, credential: null });
    const tested = await invoke("providers_test", { id: saved.id });
    const sol = tested.models.find((m: any) => m.id === "gpt-6.1-sol");
    console.log("Native discovered GPT-6.1 Sol", JSON.stringify(sol));
    expect(sol).toMatchObject({ displayName: "GPT-6.1 Sol", contextWindow: 1050000, maxOutput: 128000, supportsImages: true, supportsTools: true, supportsReasoning: true, efforts: ["low", "medium", "high", "xhigh", "max"], defaultEffort: "medium" });

    // Settings › Providers: manual model with Low, High, Max; default High.
    await invoke("settings_open", { section: "providers" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    const settings = (await browser.getWindowHandles()).find((h) => h !== overlay)!;
    await browser.switchToWindow(settings);
    await (await $('button[aria-label="Models of QA"]')).click();
    await (await $("button=Add model")).click();
    await (await $('//label[span="Model ID"]//input')).setValue("qa-reasoner");
    const check = async (name: string) => (await $(`//label[normalize-space(.)="${name}"]/input`)).click();
    await check("Reasoning");
    for (const e of ["Low", "High", "Max"]) await check(e);
    await (await $('select[aria-label="Default effort"]')).selectByAttribute("value", "high");
    await (await $("button=Save model")).click();
    await browser.waitUntil(async () => (await invoke("providers_list"))[0].models.some((m: any) => m.id === "qa-reasoner"), { timeout: 10_000 });
    const manual = (await invoke("providers_list"))[0].models.find((m: any) => m.id === "qa-reasoner");
    console.log("Native manual model", JSON.stringify({ efforts: manual.efforts, defaultEffort: manual.defaultEffort }));
    expect(manual).toMatchObject({ efforts: ["low", "high", "max"], defaultEffort: "high" });

    // Settings › Account and models: the table lists GPT-6.1 Sol up to Max.
    await invoke("settings_open", { section: "account" });
    const solChat = await $('select[aria-label="QA · GPT-6.1 Sol — Chat"]');
    await solChat.waitForExist({ timeout: 10_000 });
    const options = await browser.execute((el: any) => [...el.options].map((o: any) => o.textContent), solChat);
    console.log("Native GPT-6.1 Sol effort options", JSON.stringify(options));
    expect(options).toEqual(["Model default (medium)", "Low", "Medium", "High", "Extra high", "Max"]);

    // Overlay picker: Low in Chat, Max in Task.
    await browser.switchToWindow(overlay);
    await browser.refresh();
    const box = await $("textarea");
    await box.waitForDisplayed({ timeout: 30_000 });
    await browser.keys(["Control", "ArrowDown"]);
    const openPicker = async () => {
      await $('button[aria-label^="Model "]').click();
      const p = await $('[role="dialog"][aria-label="Model and mode"]');
      await p.waitForDisplayed();
      return p;
    };
    let picker = await openPicker();
    await (await picker.$("button=QA")).click();
    await (await picker.$('[role="menu"][aria-label="Model"]')).$("button*=qa-reasoner").click();
    const effortItems = async () => browser.execute(() => [...document.querySelectorAll('[role="menu"][aria-label="Reasoning effort"] [role="menuitemradio"]')].map((b) => ({ text: b.textContent, checked: b.getAttribute("aria-checked") })));
    console.log("Native picker efforts", JSON.stringify(await effortItems()));
    expect((await effortItems()).map((e) => e.text)).toEqual(["Model default (high)", "Low", "High", "Max"]);
    const pick = async (menu: string, text: string) => (await picker.$(`[role="menu"][aria-label="${menu}"]`)).$(`button*=${text}`).click();
    await pick("Reasoning effort", "Low");
    await pick("Mode", "Task");
    await pick("Reasoning effort", "Max");
    await browser.waitUntil(async () => JSON.stringify((await invoke("settings_get")).effortPresets) === JSON.stringify({ [`aura-${saved.id}::qa-reasoner`]: { chat: "low", task: "max" } }), { timeout: 10_000, timeoutMsg: "presets not saved" });
    await pick("Mode", "Chat");
    const checkedNow = async () => (await effortItems()).find((e) => e.checked === "true")?.text;
    expect(await checkedNow()).toBe("Low");
    await browser.keys("Escape");

    // Restart: the preference is still there and the Task turn sends "max".
    await browser.reloadSession();
    await browser.waitUntil(() => browser.execute(() => "__TAURI_INTERNALS__" in window), { timeout: 30_000 });
    await $("textarea").waitForDisplayed({ timeout: 30_000 });
    await browser.keys(["Control", "ArrowDown"]);
    picker = await openPicker();
    await (await picker.$("button=QA")).click();
    await (await picker.$('[role="menu"][aria-label="Model"]')).$("button*=qa-reasoner").click();
    await pick("Mode", "Task");
    expect(await checkedNow()).toBe("Max");
    await browser.keys("Escape");
    const before = up.requests.length;
    await (await $("textarea")).setValue("QA task effort");
    await browser.keys("Enter");
    await browser.waitUntil(() => up.requests.slice(before).some((r) => r.url === "/v1/responses"), { timeout: 180_000, timeoutMsg: "no turn" });
    const turn = up.requests.slice(before).find((r) => r.url === "/v1/responses")!;
    console.log("Native task turn", JSON.stringify({ model: turn.body?.model, reasoning: turn.body?.reasoning }));
    expect(turn.body?.model).toBe("qa-reasoner");
    expect(turn.body?.reasoning?.effort).toBe("max");
    await invoke("settings_update", { patch: { effortPresets: {} } });
  });
});
