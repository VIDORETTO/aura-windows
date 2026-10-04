// QA-034: Voice settings choose the reading voice (installed Windows voices,
// offline) or a cloud voice of a BYOK provider (one-time consent before any
// text is sent) and "read answers automatically". The cloud voice is a
// loopback OpenAI-compatible /audio/speech; the model is a loopback too.
import { createServer, type IncomingMessage } from "node:http";
import { responsesUpstream } from "../responses";

const read = (req: IncomingMessage) =>
  new Promise<string>((resolve) => {
    let data = "";
    req.on("data", (c) => (data += c));
    req.on("end", () => resolve(data));
  });

/** 0.5 s 440 Hz mono 16 kHz WAV. */
function toneWav(): Buffer {
  const n = 8000;
  const b = Buffer.alloc(44 + n * 2);
  b.write("RIFF", 0); b.writeUInt32LE(36 + n * 2, 4); b.write("WAVE", 8); b.write("fmt ", 12);
  b.writeUInt32LE(16, 16); b.writeUInt16LE(1, 20); b.writeUInt16LE(1, 22); b.writeUInt32LE(16000, 24);
  b.writeUInt32LE(32000, 28); b.writeUInt16LE(2, 32); b.writeUInt16LE(16, 34); b.write("data", 36); b.writeUInt32LE(n * 2, 40);
  for (let i = 0; i < n; i++) b.writeInt16LE(Math.round(Math.sin((2 * Math.PI * 440 * i) / 16000) * 8000), 44 + i * 2);
  return b;
}

async function speechServer() {
  const bodies: any[] = [];
  const server = createServer(async (req, res) => {
    if (req.url === "/v1/models") { res.writeHead(200, { "content-type": "application/json" }); res.end(JSON.stringify({ data: [] })); return; }
    if (req.url === "/v1/audio/speech") {
      bodies.push(JSON.parse(await read(req)));
      res.writeHead(200, { "content-type": "audio/wav" });
      res.end(toneWav());
      return;
    }
    res.writeHead(404); res.end();
  });
  await new Promise<void>((r) => server.listen(0, "127.0.0.1", r));
  const a = server.address() as { port: number };
  return { baseUrl: `http://127.0.0.1:${a.port}/v1`, bodies, close: () => new Promise<void>((r) => server.close(() => r())) };
}

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));
const sha = (b64: string) => browser.execute(async (s) => {
  const bytes = Uint8Array.from(atob(s), (c) => c.charCodeAt(0));
  const d = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(d)].slice(0, 6).map((x) => x.toString(16).padStart(2, "0")).join("");
}, b64);

describe("Read aloud (QA-034)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  let tts: Awaited<ReturnType<typeof speechServer>>;
  before(async () => {
    up = await responsesUpstream(() => "The meeting moved to Friday.", [{ id: "qa-model", architecture: { input_modalities: ["text"] }, supported_parameters: ["tools"] }]);
    tts = await speechServer();
  });
  after(async () => { await up.close(); await tts.close(); });

  it("uses the chosen Windows voice, asks before the cloud voice and reads answers automatically", async () => {
    const overlay = await browser.getWindowHandle();
    const providers = await browser.execute(async (model, speech) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en", ttsVoice: null, ttsProvider: null, autoRead: false } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      const m = await i("providers_save", { draft: { name: "QA model", preset: "ollama", baseUrl: model }, credential: null });
      await i("providers_test", { id: m.id });
      const s = await i("providers_save", { draft: { name: "QA Voice", preset: "custom", baseUrl: speech }, credential: null });
      return JSON.stringify({ voice: s.id });
    }, up.baseUrl, tts.baseUrl).then((s) => JSON.parse(s as string));
    try {
      // Offline: real installed voices; two voices give different audio.
      const opts = await invoke("speech_options");
      console.log("Native voices", JSON.stringify(opts.voices.map((v: any) => `${v.name} (${v.language})`)), "cloud", JSON.stringify(opts.cloud.map((c: any) => c.name)));
      expect(opts.voices.length).toBeGreaterThan(1);
      expect(opts.cloud.map((c: any) => c.name)).toEqual(["QA Voice"]);
      await invoke("settings_open", { section: "voice" });
      await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
      const settings = (await browser.getWindowHandles()).find((h) => h !== overlay)!;
      await browser.switchToWindow(settings);
      const select = await $('select[aria-label="Reading voice"]');
      await select.waitForDisplayed({ timeout: 15_000 });
      const hashes: string[] = [];
      for (const v of opts.voices.slice(0, 2)) {
        await select.selectByAttribute("value", v.id);
        await browser.waitUntil(async () => (await invoke("settings_get")).ttsVoice === v.id);
        const [b64, mime] = await invoke("speak", { text: "Testing the reading voice." });
        expect(mime).toBe("audio/wav");
        hashes.push(await sha(b64));
      }
      console.log("Native offline voice audio", JSON.stringify(hashes));
      expect(hashes[0]).not.toBe(hashes[1]);
      expect(tts.bodies).toHaveLength(0);

      // Cloud voice: confirmation first, nothing sent before it.
      await select.selectByAttribute("value", `cloud:${providers.voice}`);
      await $("*=will be sent to QA Voice").waitForDisplayed({ timeout: 10_000 }).catch(() => undefined);
      expect(await browser.execute(() => document.body.innerText)).toContain("Answer text will be sent to QA Voice. Aura asks for confirmation on first use.");
      await (await $("button=Test voice")).click();
      const dialog = await $('[role="dialog"][aria-label="Send the answer to QA Voice?"]');
      await dialog.waitForDisplayed({ timeout: 10_000 });
      expect(tts.bodies).toHaveLength(0);
      await (await dialog.$("button=Send and listen")).click();
      await browser.waitUntil(() => tts.bodies.length === 1, { timeout: 15_000, timeoutMsg: "cloud voice not called" });
      console.log("Native cloud request", JSON.stringify(tts.bodies[0]));
      expect(tts.bodies[0]).toMatchObject({ model: "tts-1", voice: "alloy", input: "Hello! This is the voice Aura uses to read answers.", response_format: "wav" });
      expect((await invoke("settings_get")).ttsCloudConsent).toEqual([providers.voice]);

      // Auto-read: the finished answer goes to the voice without a click.
      await (await $('button[role="switch"][aria-label="Read answers automatically"]')).click();
      await browser.waitUntil(async () => (await invoke("settings_get")).autoRead === true);
      await browser.switchToWindow(overlay);
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
      await box.setValue("QA when is the meeting?");
      await browser.keys("Enter");
      await browser.waitUntil(() => tts.bodies.length === 2, { timeout: 180_000, timeoutMsg: "answer not read automatically" });
      console.log("Native auto-read request", JSON.stringify(tts.bodies[1]));
      expect(tts.bodies[1].input).toBe("The meeting moved to Friday.");
      // No second confirmation for the same provider.
      expect(await $('[role="dialog"][aria-label="Send the answer to QA Voice?"]').isExisting()).toBe(false);
    } finally {
      await invoke("settings_update", { patch: { ttsVoice: null, ttsProvider: null, autoRead: false } }).catch(() => undefined);
    }
  });
});
