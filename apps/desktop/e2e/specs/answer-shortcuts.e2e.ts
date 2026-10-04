// QA-035: in the Overlay, Ctrl+Shift+L reads the last answer and
// Ctrl+Shift+Enter inserts it into the app the Overlay was opened over. The
// target is a QA WinForms text box (insert-target.ps1), never a personal app;
// the clipboard is restored by Aura and the user's clipboard by the test.
import { spawn, spawnSync } from "node:child_process";
import { existsSync, readFileSync, rmSync } from "node:fs";
import { createServer, type IncomingMessage } from "node:http";
import os from "node:os";
import path from "node:path";
import { responsesUpstream } from "../responses";

const ANSWER = "Use cargo test --workspace.";
const read = (req: IncomingMessage) => new Promise<string>((r) => { let d = ""; req.on("data", (c) => (d += c)); req.on("end", () => r(d)); });

async function speechServer() {
  const bodies: any[] = [];
  const server = createServer(async (req, res) => {
    if (req.url === "/v1/audio/speech") {
      bodies.push(JSON.parse(await read(req)));
      res.writeHead(200, { "content-type": "audio/wav" });
      res.end(Buffer.from("UklGRiQAAABXQVZFZm10IBAAAAABAAEAgD4AAAB9AAACABAAZGF0YQAAAAA=", "base64"));
      return;
    }
    res.writeHead(404); res.end();
  });
  await new Promise<void>((r) => server.listen(0, "127.0.0.1", r));
  const a = server.address() as { port: number };
  return { baseUrl: `http://127.0.0.1:${a.port}/v1`, bodies, close: () => new Promise<void>((r) => server.close(() => r())) };
}

const ps = (cmd: string) => spawnSync("powershell", ["-NoProfile", "-STA", "-Command", cmd], { encoding: "utf8", timeout: 30_000 });
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

describe("Answer shortcuts (QA-035)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  let tts: Awaited<ReturnType<typeof speechServer>>;
  // The user's clipboard is saved before and put back after the test (not logged).
  const saved = path.join(os.tmpdir(), `aura-qa-clip-${process.pid}.txt`);
  before(async () => {
    up = await responsesUpstream(() => ANSWER, [{ id: "qa-model", architecture: { input_modalities: ["text"] }, supported_parameters: ["tools"] }]);
    tts = await speechServer();
    ps(`Get-Clipboard -Raw | Set-Content -NoNewline -Encoding UTF8 '${saved}'`);
  });
  after(async () => {
    await up.close();
    await tts.close();
    if (existsSync(saved)) {
      ps(`$t = Get-Content -Raw -Encoding UTF8 '${saved}'; if ($t) { Set-Clipboard -Value $t }`);
      rmSync(saved, { force: true });
    }
  });

  it("reads with Ctrl+Shift+L and inserts with Ctrl+Shift+Enter into the previous app", async () => {
    const ids = await browser.execute(async (model, speech) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en", autoRead: false, ttsVoice: null, ttsProvider: null } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      const m = await i("providers_save", { draft: { name: "QA model", preset: "ollama", baseUrl: model }, credential: null });
      await i("providers_test", { id: m.id });
      const s = await i("providers_save", { draft: { name: "QA Voice", preset: "custom", baseUrl: speech }, credential: null });
      await i("settings_update", { patch: { ttsProvider: s.id } });
      await i("speech_consent");
      return m.id;
    }, up.baseUrl, tts.baseUrl);
    void ids;
    await browser.refresh();
    const box = await $("textarea");
    await box.waitForDisplayed({ timeout: 30_000 });
    await browser.keys(["Control", "ArrowDown"]);
    await $('button[aria-label^="Model "]').click();
    const picker = await $('[role="dialog"][aria-label="Model and mode"]');
    await picker.waitForDisplayed();
    await (await picker.$("button=QA model")).click();
    await (await picker.$('[role="menu"][aria-label="Model"]')).$("button*=qa-model").click();
    await browser.keys("Escape");
    await box.setValue("QA how do I run the tests?");
    await browser.keys("Enter");
    await browser.waitUntil(async () => (await browser.execute(() => document.body.innerText)).includes(ANSWER), { timeout: 180_000, timeoutMsg: "no answer" });

    // Ctrl+Shift+L: the last answer goes to the (consented) voice.
    await box.click();
    await browser.keys(["Control", "Shift", "l"]);
    await browser.keys(["Control", "Shift"]);
    await browser.waitUntil(() => tts.bodies.length === 1, { timeout: 15_000, timeoutMsg: "Ctrl+Shift+L did not read" });
    console.log("Native Ctrl+Shift+L read", JSON.stringify(tts.bodies[0].input));
    expect(tts.bodies[0].input).toBe(ANSWER);

    // Open the Overlay over the QA text box with the global shortcut.
    const out = path.join(os.tmpdir(), `aura-qa-insert-${process.pid}.txt`);
    rmSync(out, { force: true });
    ps("Set-Clipboard -Value 'QA original clipboard'");
    await invoke("overlay_hide");
    const target = spawn("powershell", ["-NoProfile", "-STA", "-ExecutionPolicy", "Bypass", "-File", path.resolve("insert-target.ps1"), out, "90"], { stdio: "ignore" });
    try {
      await browser.waitUntil(() => existsSync(out), { timeout: 20_000, timeoutMsg: "target window not up" });
      await browser.pause(800);
      // Reopen the Overlay on this conversation while the QA box is the
      // foreground app (same path as the access-log link).
      const thread = (await invoke("conversation_history", { query: {} })).items[0].id;
      await invoke("privacy_open_conversation", { threadId: thread });
      await browser.waitUntil(async () => (await invoke("previous_app"))?.title === "QA Insert Target", { timeout: 15_000 }).catch(async (e) => {
        console.log("Native previous app (unexpected)", JSON.stringify(await invoke("previous_app")));
        throw e;
      });
      console.log("Native previous app", JSON.stringify(await invoke("previous_app")));
      await (await $("textarea")).click();
      await browser.keys(["Control", "Shift", "Enter"]);
      await browser.keys(["Control", "Shift"]);
      await browser.waitUntil(() => readFileSync(out, "utf8").includes(ANSWER), { timeout: 15_000, timeoutMsg: "answer not inserted" });
      const typed = readFileSync(out, "utf8");
      await browser.pause(1500);
      const clip = ps("Get-Clipboard -Raw").stdout.trim();
      console.log("Native insert", JSON.stringify({ typed, clipboardAfter: clip }));
      expect(typed).toBe(ANSWER);
      expect(clip).toBe("QA original clipboard");
    } finally {
      target.kill();
      rmSync(out, { force: true });
      await invoke("settings_update", { patch: { ttsProvider: null } }).catch(() => undefined);
    }
  });
});
