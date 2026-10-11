// Native window -> Host -> pinned process -> gateway -> MCP -> WebService.
// Only the remote model and DNS/HTTP adapters use deterministic QA fixtures.
import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { responsesUpstream, type FunctionCall } from "../responses";

const FIRST = "https://news.example/report";
const SECOND = "https://independent.example/report";
const ANSWER = "Production: 42; North: 12; South: 30 [[aura-source:W1]]. Second production: 30 [[aura-source:W2]].";
const invoke = async (cmd: string, args: Record<string, unknown> = {}): Promise<any> => {
  const result = await browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args);
  return JSON.parse(String(result));
};

function call(body: any, wanted: string, args: Record<string, unknown>): FunctionCall {
  for (const tool of body.tools ?? []) {
    if (tool.type === "namespace" && tool.name === "mcp__aura" && tool.tools?.some((t: any) => t.name === wanted)) {
      return { functionCall: { name: wanted, namespace: "mcp__aura", arguments: JSON.stringify(args) } };
    }
    if (tool.name === `mcp__aura__${wanted}`) {
      return { functionCall: { name: tool.name, arguments: JSON.stringify(args) } };
    }
  }
  throw new Error(`Pinned process did not advertise ${wanted}`);
}

function outputs(body: any): any[] {
  return (body.input ?? []).filter((item: any) => item.type === "function_call_output")
    .flatMap((item: any) => Array.isArray(item.output) ? item.output : [{ text: item.output }])
    .flatMap((part: any) => {
      try { return [JSON.parse(part.text)]; } catch { return []; }
    });
}

describe("Native internet research (039)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  let stage = 0;
  let scenario: "research" | "pending" | "direct" = "research";
  let pendingReadProposals = 0;

  before(async () => {
    if (process.env.AURA_E2E_WEB !== "1" || !process.env.AURA_HOME || !process.env.AURA_CODEX_BIN) {
      throw new Error("Requires explicit native QA feature, isolated AURA_HOME and pinned AURA_CODEX_BIN");
    }
    up = await responsesUpstream((body) => {
      if (scenario === "direct") {
        if (stage++ === 0) return call(body, "web_fetch", { url: SECOND });
        return "Reenabled production: 30 [[aura-source:W1]].";
      }
      if (scenario === "pending") {
        if (stage++ === 0) {
          const proposed = call(body, "web_fetch", { url: "https://news.example/pending" });
          pendingReadProposals++;
          return proposed;
        }
        return "QA internet was disabled during the page read. No page content was received or cited.";
      }
      switch (stage++) {
        case 0: return call(body, "web_search", { objective: "Compare regional and independent production", queries: ["regional production"] });
        case 1: return call(body, "web_fetch", { url: FIRST });
        case 2: return call(body, "web_fetch", { url: SECOND });
        default: return ANSWER;
      }
    }, [{ id: "qa-model", supported_parameters: ["tools"] }]);
    await invoke("settings_update", { patch: { language: "en", onboarded: true, attachScreenOnOpen: false, webEnabled: true, hideOnBlur: false, ttsProvider: null, autoRead: false } });
    for (const p of await invoke("providers_list")) await invoke("providers_remove", { id: p.id });
    const p = await invoke("providers_save", { draft: { name: "QA web", preset: "custom", wire: "responses", baseUrl: up.baseUrl, auth: "none" }, credential: null });
    await invoke("providers_model_save", { id: p.id, model: { id: "qa-model", displayName: "QA web model", contextWindow: null, maxOutput: null, supportsImages: false, supportsTools: true, supportsReasoning: false, estimated: false, manual: true, efforts: [], defaultEffort: null } });
    await browser.refresh();
    spawn(process.env.AURA_E2E_APP!, [], { stdio: "ignore", detached: true }).unref();
    await $("textarea").waitForDisplayed({ timeout: 20_000 });
    await $('header button[aria-label^="Model"]').click();
    await (await $('[role="menu"][aria-label="Provider"]')).$("button*=QA web").click();
    await (await $('[role="menu"][aria-label="Model"]')).$("button*=QA web model").click();
    await browser.keys("Escape");
  });

  after(async () => { await up?.close(); });
  afterEach(async function () {
    if (this.currentTest?.state === "failed") {
      await browser.saveScreenshot(path.join(process.env.AURA_HOME!, "native-failure.png"));
      console.log("QA failure view", await browser.execute(() => ({
        text: document.body.innerText.slice(-1600),
        markdown: document.querySelector('[data-answer] .md')?.innerHTML,
        focus: document.activeElement?.outerHTML.slice(0,300),
        groups: [...document.querySelectorAll('[role="group"][aria-label="Consulted sources"]')].map((el) => ({ text: (el as HTMLElement).innerText, width: el.getBoundingClientRect().width, height: el.getBoundingClientRect().height })),
      })));
      console.log("QA recent tool outputs", up.requests.filter((r) => r.url === "/v1/responses").slice(-2).map((r) => (r.body.input ?? []).filter((i: any) => i.type === "function_call_output").map((i: any) => JSON.stringify(i.output).slice(0,400))));
    }
  });

  it("shows actual research, two read sources and restores their metadata through history", async () => {
    await $("textarea").setValue("QA native research: compare production with two independent pages.");
    await browser.keys("Enter");
    await $('[role="group"][aria-label="Consulted sources"]').waitForDisplayed({ timeout: 45_000 });
    await browser.waitUntil(async () => !(await $('button[aria-label="Stop"]').isExisting()), { timeout: 10_000 });
    const sources = await $('[role="group"][aria-label="Consulted sources"]');
    expect(await sources.getText()).toContain(FIRST);
    expect(await sources.getText()).toContain(SECOND);
    expect((await sources.getText()).match(/Page read/g)?.length).toBe(2);
    expect((await sources.$$("li")).length).toBe(2);
    await $('button[data-web-source="W1"]').waitForDisplayed({ timeout: 10_000 });
    await $('button[data-web-source="W2"]').waitForDisplayed({ timeout: 10_000 });
    const text = await $("body").getText();
    expect(text).toContain("Internet search");
    expect(text).toContain("Page reading");
    expect(text).toContain("Production: 42; North: 12; South: 30");
    expect(await $('header button[aria-label^="Model"]').getText()).toContain("QA web model");
    await browser.waitUntil(async () => !(await $('button[aria-label="Stop"]').isExisting()), { timeout: 10_000 });

    const delivered = up.requests.filter((r) => r.url === "/v1/responses").flatMap((r) => outputs(r.body));
    const pages = delivered.filter((o) => typeof o.text === "string" && o.sourceId);
    expect(pages.some((p) => p.sourceId === "W1" && p.finalUrl === FIRST && p.text.includes("42") && p.text.includes("12") && p.text.includes("30"))).toBe(true);
    expect(pages.some((p) => p.sourceId === "W2" && p.finalUrl === SECOND && p.text.includes("30"))).toBe(true);
    expect(pages.every((p) => !p.text.includes("sendCredentials") && !p.text.includes("DISCARD THIS MENU"))).toBe(true);

    const history = await invoke("conversation_history", { query: { search: "QA native research", limit: 10 } });
    expect(history.items.length).toBe(1);
    const transcript = await invoke("conversation_open", { threadId: history.items[0].id });
    const answer = transcript.find((m: any) => m.role === "assistant");
    expect(answer.text).toBe(ANSWER);
    expect(answer.sources.map((s: any) => s.url)).toEqual([FIRST, SECOND]);
    const requestsBefore = readFileSync(path.join(process.env.AURA_HOME!, "e2e-web-requests.jsonl"), "utf8");
    await $('button[aria-label="New conversation"]').click();
    await $('button[aria-label="History"]').click();
    const historyEntry = await $('aside[aria-label="History"] li > button:not([aria-label])');
    expect(await historyEntry.getText()).toContain("QA native research");
    // Hover reveals row actions and can move their hit targets. Navigate the
    // public history controls with the keyboard, then verify actual focus.
    await $('aside[aria-label="History"] input').click();
    await browser.keys("Tab");
    await browser.keys("Tab");
    expect(await browser.execute(() => document.activeElement === document.querySelector('aside[aria-label="History"] li > button:not([aria-label])'))).toBe(true);
    await browser.keys("Enter");
    await $('[role="group"][aria-label="Consulted sources"]').waitForDisplayed();
    expect(await $('[role="group"][aria-label="Consulted sources"]').getText()).toContain(FIRST);
    expect(readFileSync(path.join(process.env.AURA_HOME!, "e2e-web-requests.jsonl"), "utf8")).toBe(requestsBefore);
    await browser.saveScreenshot(path.join(process.env.AURA_HOME!, "native-research.png"));
    // Real user action, actual native opener. Confirm the opened browser page
    // independently when collecting QA evidence; a click alone is not proof.
    await $('button[data-web-source="W1"]').click();
  });

  it("stops an actual pending page stream without a read source or another page", async () => {
    await $('button[aria-label="New conversation"]').click();
    scenario = "pending";
    stage = 0;
    const responseCount = up.requests.filter((r) => r.url === "/v1/responses").length;
    const tracePath = path.join(process.env.AURA_HOME!, "e2e-web-requests.jsonl");
    const traceBefore = readFileSync(tracePath, "utf8");
    await $("textarea").setValue("QA cancel pending read");
    await browser.keys("Enter");
    await browser.waitUntil(() => readFileSync(tracePath, "utf8").slice(traceBefore.length).includes("https://news.example/pending"), { timeout: 15_000 });
    expect(await $("body").getText()).toContain("Reading page");
    await $('button[aria-label="Stop"]').click();
    await browser.waitUntil(async () => !(await $('button[aria-label="Stop"]').isExisting()), { timeout: 10_000 });
    expect(await $('[role="group"][aria-label="Consulted sources"]').isExisting()).toBe(false);
    expect(up.requests.filter((r) => r.url === "/v1/responses").length).toBe(responseCount + 1);
    expect(readFileSync(tracePath, "utf8").slice(traceBefore.length)).not.toContain(SECOND);
    const history = await invoke("conversation_history", { query: { search: "QA cancel pending read", limit: 10 } });
    const transcript = await invoke("conversation_open", { threadId: history.items[0].id });
    expect(transcript.every((m: any) => !m.sources?.length)).toBe(true);
    expect(transcript.some((m: any) => m.role === "assistant" && m.text.includes("No page content"))).toBe(false);
    await browser.saveScreenshot(path.join(process.env.AURA_HOME!, "native-cancel.png"));
  });

  it("turns internet off in Settings, cancels the pending read, denies new reads and allows reenable", async () => {
    await $('button[aria-label="New conversation"]').click();
    scenario = "pending";
    stage = 0;
    const tracePath = path.join(process.env.AURA_HOME!, "e2e-web-requests.jsonl");
    const traceBefore = readFileSync(tracePath, "utf8");
    const responsesBefore = up.requests.length;
    await $("textarea").setValue("QA disable internet during pending read");
    await browser.keys("Enter");
    await browser.waitUntil(() => readFileSync(tracePath, "utf8").slice(traceBefore.length).includes("https://news.example/pending"), { timeout: 15_000 });
    expect(await $("body").getText()).toContain("Reading page");
    const overlay = await browser.getWindowHandle();
    await invoke("settings_open", { section: "general" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    const settings = (await browser.getWindowHandles()).find((h) => h !== overlay)!;
    await browser.switchToWindow(settings);
    const toggle = await $('[role="switch"][aria-label="Internet access"]');
    await toggle.waitForDisplayed();
    expect(await toggle.getAttribute("aria-checked")).toBe("true");
    await toggle.click();
    await browser.waitUntil(async () => (await invoke("settings_get")).webEnabled === false);
    expect(await toggle.getAttribute("aria-checked")).toBe("false");
    await browser.saveScreenshot(path.join(process.env.AURA_HOME!, "native-internet-off.png"));
    await browser.switchToWindow(overlay);
    await browser.waitUntil(async () => !(await $('button[aria-label="Stop"]').isExisting()), { timeout: 10_000 });
    expect(await $("body").getText()).toContain("No page content was received or cited");
    expect(await $('[role="group"][aria-label="Consulted sources"]').isExisting()).toBe(false);
    const errors = up.requests.slice(responsesBefore).flatMap((r) => outputs(r.body));
    expect(errors.some((o) => o.code === "cancelled" || o.code === "web_disabled")).toBe(true);

    const traceWhileOff = readFileSync(tracePath, "utf8");
    await $('button[aria-label="New conversation"]').click();
    stage = 0;
    const disabledResponses = up.requests.length;
    const proposalsBefore = pendingReadProposals;
    await $("textarea").setValue("QA attempt page reading while internet is off");
    await browser.keys("Enter");
    await browser.waitUntil(() => up.requests.slice(disabledResponses).filter((r) => r.url === "/v1/responses").length >= 2, { timeout: 15_000 });
    await browser.waitUntil(async () => (await $("body").getText()).includes("No page content was received or cited"), { timeout: 15_000 });
    expect(pendingReadProposals).toBe(proposalsBefore + 1);
    expect(await $('[role="group"][aria-label="Consulted sources"]').isExisting()).toBe(false);
    expect(readFileSync(tracePath, "utf8")).toBe(traceWhileOff);

    await browser.switchToWindow(settings);
    await $('[role="switch"][aria-label="Internet access"]').click();
    await browser.waitUntil(async () => (await invoke("settings_get")).webEnabled === true);
    await browser.closeWindow();
    await browser.switchToWindow(overlay);
    await $('button[aria-label="New conversation"]').click();
    scenario = "direct";
    stage = 0;
    await $("textarea").setValue("QA read independent page after reenable");
    await browser.keys("Enter");
    await $('[role="group"][aria-label="Consulted sources"]').waitForDisplayed({ timeout: 15_000 });
    expect(await $('[role="group"][aria-label="Consulted sources"]').getText()).toContain(SECOND);
    expect(await $("body").getText()).toContain("Reenabled production: 30");
    expect(readFileSync(tracePath, "utf8").slice(traceWhileOff.length)).toContain(SECOND);
    await browser.saveScreenshot(path.join(process.env.AURA_HOME!, "native-reenabled.png"));
  });
});
