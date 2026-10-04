// QA-011: dictation without a local model offers the recommended download.
// Needs a profile without speech models (AURA_HOME) and the real `engines`
// worker next to the app. Downloads the real recommended model (~640 MB), so
// pass a long mocha timeout (--mochaOpts.timeout).
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import path from "node:path";

// Literal values from crates/aura-asr/models.toml for the recommendation on
// the QA machine: CPU-only worker (no GPU inference), English UI → Parakeet V3
// (006 AC-002). Independent of the UI code.
const MODEL = {
  id: "parakeet-tdt-0.6b-v3",
  name: "Parakeet V3",
  files: {
    "encoder-model.int8.onnx": "6139d2fa7e1b086097b277c7149725edbab89cc7c7ae64b23c741be4055aff09",
    "decoder_joint-model.int8.onnx": "eea7483ee3d1a30375daedc8ed83e3960c91b098812127a0d99d1c8977667a70",
    "nemo128.onnx": "a9fde1486ebfcc08f328d75ad4610c67835fea58c73ba57e3209a6f6cf019e9f",
    "vocab.txt": "d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d",
    "config.json": "666903c76b9798caf2c210afd4f6cd60b08a8dbf9800ec8d7a3bc0d2148ac466",
  } as Record<string, string>,
  // 670,619,803 bytes = 639.55 MiB → rounded up.
  downloadLabel: "Download (640 MB)",
};

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

const listenVoice = () =>
  browser.execute(async () => {
    const internals = (window as any).__TAURI_INTERNALS__;
    (window as any).qaVoice = [];
    (window as any).qaDownloads = [];
    (window as any).qaVoiceRaw = [];
    return internals.invoke("plugin:event|listen", {
      event: "aura://event",
      target: { kind: "Any" },
      handler: internals.transformCallback((event: any) => {
        if (event.payload.channel === "voice") (window as any).qaVoice.push(event.payload.event.state);
        if (event.payload.channel === "voice" || event.payload.channel === "notice") (window as any).qaVoiceRaw.push(event.payload);
        if (event.payload.channel === "download") (window as any).qaDownloads.push(event.payload.event);
      }),
    });
  });

const voiceStates = () => browser.execute(() => (window as any).qaVoice as string[]);
// Serialized: WebDriver treats a returned object with an `error` field as a protocol error.
const lastDownload = async () => JSON.parse(await browser.execute((id) => JSON.stringify(((window as any).qaDownloads as any[]).filter((d) => d.id === id).at(-1) ?? null), MODEL.id));
const cardText = () => browser.execute(() => document.querySelector('section[aria-label="Install local dictation"]')?.textContent ?? null);

const card = () => $('section[aria-label="Install local dictation"]');

async function sha256(file: string) {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest("hex");
}

describe("Recommended speech model install (QA-011)", () => {
  it("offers, cancels, retries, installs and selects the recommended model without starting capture", async function () {
    this.timeout(25 * 60_000);
    await invoke("settings_update", { patch: { language: "en", sendAfterDictation: false } });
    const models = (await invoke("voice_models")) as any[];
    console.log("Native catalog before install", models.map((m) => ({ id: m.entry.id, installed: m.installed, selected: m.selected, recommended: m.recommended })));
    expect(models.some((m) => m.installed)).toBe(false);
    expect(models.find((m) => m.recommended)?.entry.id).toBe(MODEL.id);

    const eventId = await listenVoice();
    try {
      const mic = await $('button[aria-label="Dictate"]');
      await mic.waitForDisplayed();
      await mic.click();
      await (await card()).waitForDisplayed({ timeout: 5000 });
      const text = await (await card()).getText();
      console.log("Native install card", JSON.stringify(text));
      expect(text).toContain(MODEL.name);
      const download = await (await card()).$(`button=${MODEL.downloadLabel}`);
      expect(await download.isDisplayed()).toBe(true);
      expect(await voiceStates()).not.toContain("listening");

      // Start, observe progress, cancel.
      await download.click();
      await browser.waitUntil(async () => ((await lastDownload())?.bytes ?? 0) > 0, { timeout: 60_000, timeoutMsg: "no download progress" });
      expect(await browser.execute(() => !!document.querySelector('section [role="progressbar"]'))).toBe(true);
      await (await card()).$("button=Cancel").click();
      await browser.waitUntil(async () => (await cardText())?.includes("Download cancelled") ?? false, { timeout: 15_000, timeoutMsg: "no cancelled status" });
      console.log("Native cancelled download event", await lastDownload());
      expect(await browser.execute(() => !!document.querySelector('section [role="alert"]'))).toBe(false);

      // Retry until the real install completes.
      await (await card()).$(`button=${MODEL.downloadLabel}`).click();
      let lastLog = 0;
      await browser.waitUntil(async () => {
        const d = await lastDownload();
        if (Date.now() - lastLog > 30_000) {
          lastLog = Date.now();
          console.log("Native download progress", d && { bytes: d.bytes, total: d.total, done: d.done, error: d.error });
        }
        if (d?.error && d.error !== "cancelled") throw new Error(`download failed: ${d.error}`);
        return !!d?.done;
      }, { timeout: 22 * 60_000, interval: 2000, timeoutMsg: "download did not finish" });
      await browser.waitUntil(async () => (await cardText()) === null, { timeout: 15_000, timeoutMsg: "card still open after install" });

      const after = (await invoke("voice_models")) as any[];
      const installed = after.find((m) => m.entry.id === MODEL.id);
      console.log("Native catalog after install", after.map((m) => ({ id: m.entry.id, installed: m.installed, selected: m.selected })));
      expect(installed.installed).toBe(true);
      expect(installed.selected).toBe(true);
      expect(await voiceStates()).not.toContain("listening");

      const home = process.env.AURA_HOME;
      if (!home) throw new Error("AURA_HOME is required to verify the installed file");
      for (const [file, expected] of Object.entries(MODEL.files)) {
        const digest = await sha256(path.join(home, "models", "asr", MODEL.id, file));
        console.log("Installed model sha256", file, digest);
        expect(digest).toBe(expected);
      }
    } finally {
      await browser.execute(async (id) => {
        await (window as any).__TAURI_INTERNALS__.invoke("plugin:event|unlisten", { event: "aura://event", eventId: id });
      }, eventId);
    }
  });

  it("keeps the selection after restart and starts the microphone", async () => {
    await browser.reloadSession();
    await browser.waitUntil(() => browser.execute(() => "__TAURI_INTERNALS__" in window), { timeout: 20_000 });
    const models = (await invoke("voice_models")) as any[];
    expect(models.find((m) => m.entry.id === MODEL.id)?.selected).toBe(true);
    const eventId = await listenVoice();
    try {
      const mic = await $('button[aria-label="Dictate"]');
      await mic.waitForDisplayed();
      await mic.click();
      await browser.waitUntil(async () => (await voiceStates()).includes("listening"), { timeout: 15_000, timeoutMsg: "microphone did not start" });
      expect(await (await card()).isExisting()).toBe(false);
      await browser.keys("Escape");
      await browser.waitUntil(async () => (await voiceStates()).includes("cancelled"), { timeout: 5000 });
      console.log("Native voice states after restart", await voiceStates());
    } finally {
      await browser.execute(async (id) => {
        const invoke = (window as any).__TAURI_INTERNALS__.invoke;
        await invoke("ptt_cancel");
        await invoke("plugin:event|unlisten", { event: "aura://event", eventId: id });
      }, eventId);
    }
  });

  // Acoustic check without a physical speaker or personal audio: Windows SAPI
  // speaks a known sentence into "CABLE Input"; the app records "CABLE Output".
  it("transcribes known synthetic speech through the UI with the real worker and model", async function () {
    this.timeout(3 * 60_000);
    const cable = "CABLE Output (VB-Audio Virtual Cable)";
    await invoke("settings_update", { patch: { language: "en", asrLanguage: "en", microphoneDeviceId: cable, sendAfterDictation: false } });
    await browser.refresh();
    await browser.waitUntil(() => browser.execute(() => "__TAURI_INTERNALS__" in window), { timeout: 20_000 });
    const eventId = await listenVoice();
    try {
      const textarea = await $("textarea");
      await textarea.waitForDisplayed();
      await textarea.setValue("");
      const mic = await $('button[aria-label="Dictate"]');
      await mic.click();
      await browser.waitUntil(async () => (await voiceStates()).includes("listening"), { timeout: 15_000 });
      const spoken = spawnSync("powershell", ["-NoProfile", "-Command", SPEAK], { encoding: "utf8", timeout: 60_000 });
      console.log("SAPI speak", spoken.status, spoken.stdout.trim(), spoken.stderr.trim());
      expect(spoken.status).toBe(0);
      await (await $('button[aria-label="Finish dictation"]')).click();
      await browser.waitUntil(async () => (await textarea.getValue()).trim().length > 0, { timeout: 90_000, timeoutMsg: "no transcript inserted" })
        .catch(async (e) => {
          console.log("Native voice states without transcript", JSON.stringify(await browser.execute(() => (window as any).qaVoiceRaw)));
          throw e;
        });
      const transcript = await textarea.getValue();
      console.log("Native transcript", JSON.stringify(transcript), await voiceStates());
      const words = transcript.toLowerCase().replace(/[^a-z ]/g, " ").split(/\s+/);
      for (const word of ["quick", "brown", "fox", "lazy", "dog"]) expect(words).toContain(word);
    } finally {
      await browser.execute(async (id) => {
        const invoke = (window as any).__TAURI_INTERNALS__.invoke;
        await invoke("ptt_cancel");
        await invoke("plugin:event|unlisten", { event: "aura://event", eventId: id });
      }, eventId);
    }
  });
});

const SPEAK = [
  "$v = New-Object -ComObject SAPI.SpVoice",
  "$v.AudioOutput = @($v.GetAudioOutputs() | Where-Object { $_.GetDescription() -eq 'CABLE Input (VB-Audio Virtual Cable)' })[0]",
  "if (-not $v.AudioOutput -or $v.AudioOutput.GetDescription() -ne 'CABLE Input (VB-Audio Virtual Cable)') { throw 'CABLE Input not found' }",
  "$v.Voice = @($v.GetVoices() | Where-Object { $_.GetDescription() -like '*English*' })[0]",
  "[void]$v.Speak('The quick brown fox jumps over the lazy dog.')",
  "$v.AudioOutput.GetDescription()",
].join("; ");
