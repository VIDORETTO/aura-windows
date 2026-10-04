// QA-014: the model picker offers only supported reasoning efforts, shows
// capabilities and the chosen effort reaches the provider. Loopback upstream
// only: it records the request body and answers with an error (no inference).
import { createServer, type IncomingMessage } from "node:http";

const MODELS = [
  { id: "qa-reasoner", name: "QA reasoner", architecture: { input_modalities: ["text", "image"] }, supported_parameters: ["tools", "reasoning"] },
  { id: "qa-plain", name: "QA plain", architecture: { input_modalities: ["text"] }, supported_parameters: [] },
];

async function upstream() {
  const bodies: any[] = [];
  const read = (req: IncomingMessage) => new Promise<string>((resolve) => {
    let data = "";
    req.on("data", (c) => (data += c));
    req.on("end", () => resolve(data));
  });
  const server = createServer(async (req, res) => {
    if (req.url === "/v1/models") {
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify({ data: MODELS }));
      return;
    }
    if (req.method === "POST") {
      const body = await read(req);
      try { bodies.push({ url: req.url, body: JSON.parse(body) }); } catch { bodies.push({ url: req.url, raw: body.slice(0, 200) }); }
      res.writeHead(400, { "content-type": "application/json" });
      res.end(JSON.stringify({ error: { message: "QA captured request", type: "invalid_request_error" } }));
      return;
    }
    res.writeHead(404);
    res.end();
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("no port");
  return { baseUrl: `http://127.0.0.1:${address.port}/v1`, bodies, close: () => new Promise<void>((r) => server.close(() => r())) };
}

describe("Reasoning effort and capabilities (QA-014)", () => {
  let up: Awaited<ReturnType<typeof upstream>>;
  before(async () => { up = await upstream(); });
  after(async () => { await up.close(); });

  it("offers efforts only for the reasoning model, shows capabilities and sends the chosen effort upstream", async () => {
    const saved = await browser.execute(async (baseUrl) => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      await invoke("settings_update", { patch: { language: "en", defaultEffort: null } });
      for (const p of await invoke("providers_list")) await invoke("providers_remove", { id: p.id });
      const p = await invoke("providers_save", { draft: { name: "QA effort", preset: "ollama", baseUrl }, credential: null });
      return invoke("providers_test", { id: p.id });
    }, up.baseUrl);
    console.log("Native provider", JSON.stringify((saved as any).models.map((m: any) => ({ id: m.id, images: m.supportsImages, tools: m.supportsTools, reasoning: m.supportsReasoning }))));
    expect((saved as any).status).toBe("verified");
    await $("textarea").waitForDisplayed({ timeout: 10_000 });
    await browser.keys(["Control", "ArrowDown"]);
    await $('button[aria-label^="Model "]').click();
    const dialog = await $('[role="dialog"][aria-label="Model and mode"]');
    await dialog.waitForDisplayed();
    const describe = (name: string) => browser.execute((n) => {
      const b = [...document.querySelectorAll('[role="menu"][aria-label="Model"] button[role="menuitemradio"]')].find((x) => document.getElementById(x.getAttribute("aria-labelledby")!)?.textContent === n);
      return b ? document.getElementById(b.getAttribute("aria-describedby") ?? "")?.textContent ?? "" : null;
    }, name);
    expect(await describe("QA reasoner")).toBe("Images · Tools · Reasoning");
    expect(await describe("QA plain")).toBe("Text only");

    await (await dialog.$('[role="menu"][aria-label="Model"]')).$("button*=QA reasoner").click();
    const efforts = () => browser.execute(() => [...document.querySelectorAll('[role="menu"][aria-label="Reasoning effort"] button')].map((b) => b.textContent));
    expect(await efforts()).toEqual(["Model default", "Low", "Medium", "High"]);
    await (await dialog.$('[role="menu"][aria-label="Reasoning effort"]')).$("button=High").click();
    await (await dialog.$('[role="menu"][aria-label="Model"]')).$("button*=QA plain").click();
    expect(await efforts()).toEqual([]);
    expect(await dialog.getText()).toContain("This model has no configurable reasoning effort.");
    await (await dialog.$('[role="menu"][aria-label="Model"]')).$("button*=QA reasoner").click();
    // Switching away dropped "High"; choose it again for the turn.
    await (await dialog.$('[role="menu"][aria-label="Reasoning effort"]')).$("button=High").click();
    await browser.keys("Escape");
    await $("textarea").setValue("QA-014 effort probe");
    await browser.keys("Enter");
    await browser.waitUntil(() => up.bodies.length > 0, { timeout: 90_000, timeoutMsg: "no upstream request" }).catch(async (e) => {
      console.log("Native conversation without upstream", JSON.stringify(await browser.execute(() => document.body.innerText.slice(0, 1500))));
      await browser.saveScreenshot("../../../target/qa-tools/model-effort-failure.png");
      throw e;
    });
    const sent = up.bodies[0];
    console.log("Native upstream request", JSON.stringify({ url: sent.url, model: sent.body?.model, reasoning: sent.body?.reasoning, reasoning_effort: sent.body?.reasoning_effort }));
    expect(sent.body.model).toBe("qa-reasoner");
    expect(sent.body.reasoning?.effort ?? sent.body.reasoning_effort).toBe("high");
  });
});
