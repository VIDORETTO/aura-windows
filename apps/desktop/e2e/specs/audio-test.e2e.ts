// QA-013: device tests with live meters (005 AC-001/002). Uses VB-Cable and
// Windows SAPI: speech is played into "CABLE Input" (render) and read back as
// the microphone "CABLE Output" and as System audio loopback of "CABLE Input".
// No physical speaker, no personal audio, nothing recorded.
import { spawnSync } from "node:child_process";

const CABLE_MIC = "CABLE Output (VB-Audio Virtual Cable)";
const CABLE_RENDER = "CABLE Input (VB-Audio Virtual Cable)";
const SPEAK = [
  "$v = New-Object -ComObject SAPI.SpVoice",
  `$v.AudioOutput = @($v.GetAudioOutputs() | Where-Object { $_.GetDescription() -eq '${CABLE_RENDER}' })[0]`,
  `if ($v.AudioOutput.GetDescription() -ne '${CABLE_RENDER}') { throw 'CABLE Input not found' }`,
  "[void]$v.Speak('Testing the Aura level meter. One, two, three, four, five.')",
].join("; ");

const speak = () => {
  const r = spawnSync("powershell", ["-NoProfile", "-Command", SPEAK], { encoding: "utf8", timeout: 60_000 });
  expect(r.status).toBe(0);
};

async function listen() {
  return browser.execute(async () => {
    const internals = (window as any).__TAURI_INTERNALS__;
    (window as any).qaLevels = [];
    (window as any).qaNotices = [];
    return internals.invoke("plugin:event|listen", {
      event: "aura://event",
      target: { kind: "Any" },
      handler: internals.transformCallback((event: any) => {
        if (event.payload.channel === "audioLevel") (window as any).qaLevels.push({ ...event.payload.event, at: performance.now() });
        if (event.payload.channel === "notice") (window as any).qaNotices.push(event.payload.event.message);
      }),
    });
  });
}

const levels = (source: string) => browser.execute((s) => ((window as any).qaLevels as any[]).filter((l) => l.source === s), source);
const clear = () => browser.execute(() => { (window as any).qaLevels = []; (window as any).qaNotices = []; });
const meterValue = (name: string) => browser.execute((n) => Number(document.querySelector(`[role="meter"][aria-label="${n}"]`)?.getAttribute("aria-valuenow") ?? -1), name);

describe("Audio device tests (QA-013)", () => {
  it("shows live microphone and system-audio levels from the chosen devices and stops cleanly", async () => {
    const overlay = await browser.getWindowHandle();
    await browser.execute(async () => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      await invoke("settings_update", { patch: { language: "en", microphoneDeviceId: null, systemAudioDeviceId: null } });
      await invoke("privacy_set_paused", { paused: false }).catch(() => undefined);
      await invoke("settings_open", { section: "voice" });
    });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    const outputs = (await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("audio_devices", { system: true }))) as any[];
    console.log("Native outputs", outputs.map((d) => d.id));
    expect(outputs.map((d) => d.id)).toContain(CABLE_RENDER);
    const eventId = await listen();
    try {
      // Microphone: CABLE Output.
      await (await $('select[aria-label="Microphone"]')).selectByAttribute("value", CABLE_MIC);
      await (await $("button=Test microphone")).click();
      await clear();
      speak();
      const mic = await levels("mic");
      const span = (mic.at(-1).at - mic[0].at) / 1000;
      const rate = (mic.length - 1) / span;
      const peak = Math.max(...mic.map((l: any) => l.dbfs));
      console.log("Native mic levels", { count: mic.length, span, rate, peak });
      expect(rate).toBeGreaterThanOrEqual(20);
      expect(peak).toBeGreaterThan(-40);
      expect(await meterValue("Microphone level")).toBeGreaterThanOrEqual(0);
      await (await $("button=Stop microphone test")).click();
      await browser.pause(300);
      await clear();
      await browser.pause(600);
      expect(await levels("mic")).toHaveLength(0);

      // System audio: loopback of CABLE Input, saved as the preference.
      await (await $('select[aria-label="System audio"]')).selectByAttribute("value", CABLE_RENDER);
      await browser.waitUntil(async () => (await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_get"))) .systemAudioDeviceId === CABLE_RENDER, { timeout: 5000 });
      await (await $("button=Test system audio")).click();
      await browser.pause(300);
      const silent = Math.max(...(await levels("systemAudio")).map((l: any) => l.dbfs), -100);
      await clear();
      speak();
      const sys = await levels("systemAudio");
      const sysPeak = Math.max(...sys.map((l: any) => l.dbfs));
      console.log("Native system levels", { count: sys.length, silentPeak: silent, peak: sysPeak });
      expect(sys.length).toBeGreaterThan(20);
      expect(sysPeak).toBeGreaterThan(-40);
      await (await $("button=Stop system audio test")).click();

      // Missing chosen output: default device with a warning (005 AC-002).
      await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { systemAudioDeviceId: "QA missing output" } }));
      await clear();
      await (await $("button=Test system audio")).click();
      await browser.waitUntil(async () => (await browser.execute(() => (window as any).qaNotices as string[])).some((m) => m.includes("QA missing output")), { timeout: 5000 });
      console.log("Native fallback notice", await browser.execute(() => (window as any).qaNotices));
      await browser.waitUntil(async () => (await levels("systemAudio")).length > 5, { timeout: 5000 });
      await (await $("button=Stop system audio test")).click();

      // Dictation with a missing saved microphone also opens the default and warns.
      await clear();
      await browser.execute(async () => {
        const invoke = (window as any).__TAURI_INTERNALS__.invoke;
        await invoke("settings_update", { patch: { microphoneDeviceId: "QA missing mic" } });
        await invoke("ptt_press", { device: null });
      });
      await browser.waitUntil(async () => (await browser.execute(() => (window as any).qaNotices as string[])).some((m) => m.includes("QA missing mic")), { timeout: 5000 });
      console.log("Native dictation fallback notice", await browser.execute(() => (window as any).qaNotices));
      await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("ptt_cancel"));
    } finally {
      await browser.execute(async (id) => {
        const invoke = (window as any).__TAURI_INTERNALS__.invoke;
        await invoke("audio_test_stop", { source: "mic" });
        await invoke("audio_test_stop", { source: "systemAudio" });
        await invoke("settings_update", { patch: { microphoneDeviceId: null, systemAudioDeviceId: null } });
        await invoke("plugin:event|unlisten", { event: "aura://event", eventId: id });
      }, eventId);
    }
  });
});
