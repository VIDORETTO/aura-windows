// QA-032: a manual recording shows its duration, plays in Aura's own player
// and can be attached to the conversation. Audio comes from VB-Cable (SAPI
// speech into "CABLE Input", recorded as the microphone "CABLE Output"): no
// room audio. Recordings are deleted at the end. Run with a profile that
// has a speech model installed (attached audio is transcribed).
import { spawn } from "node:child_process";

const CABLE_MIC = "CABLE Output (VB-Audio Virtual Cable)";
const CABLE_RENDER = "CABLE Input (VB-Audio Virtual Cable)";
const SPEAK = [
  "$v = New-Object -ComObject SAPI.SpVoice",
  `$v.AudioOutput = @($v.GetAudioOutputs() | Where-Object { $_.GetDescription() -eq '${CABLE_RENDER}' })[0]`,
  "[void]$v.Speak('Aura recording test. One, two, three, four, five, six, seven, eight.')",
].join("; ");

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

async function record(seconds: number) {
  const startBtn = await $("button=Start recording");
  await startBtn.waitForClickable({ timeout: 10_000 });
  await startBtn.click();
  await (await $("button=Stop recording")).waitForDisplayed({ timeout: 10_000 });
  const t0 = Date.now();
  const voice = spawn("powershell", ["-NoProfile", "-Command", SPEAK], { stdio: "ignore" });
  await browser.pause(seconds * 1000);
  await (await $("button=Stop recording")).click();
  const wall = Date.now() - t0;
  voice.kill();
  await (await $("button=Start recording")).waitForDisplayed({ timeout: 15_000 });
  const recs = await invoke("recordings_list");
  return { rec: recs[0], wall };
}

describe("Recordings (QA-032)", () => {
  it("shows the duration, plays in Aura and attaches to the conversation", async () => {
    const overlay = await browser.getWindowHandle();
    // QA profile only: leftovers of earlier runs (they may hold screen video).
    for (const r of await invoke("recordings_list")) await invoke("recording_delete", { id: r.id });
    await invoke("settings_update", { patch: { language: "en", microphoneDeviceId: CABLE_MIC } });
    await invoke("privacy_set_paused", { paused: false }).catch(() => undefined);
    await invoke("privacy_set_source", { source: "mic", mode: { type: "manual" }, agent: "never" });
    await invoke("privacy_set_source", { source: "screen", mode: { type: "manual" }, agent: "never" });
    await invoke("settings_open", { section: "privacy" });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    const settings = (await browser.getWindowHandles()).find((h) => h !== overlay)!;
    await browser.switchToWindow(settings);
    try {
      // Microphone + screen, ~10 s: duration and player.
      const { rec, wall } = await record(10);
      console.log("Native recording A", JSON.stringify({ source: rec.source, durationMs: rec.durationMs, wallMs: wall, bytes: rec.bytes }));
      expect(Math.abs(rec.durationMs - wall)).toBeLessThanOrEqual(1500);
      const item = await $(`li[aria-label="${rec.title}"]`);
      const shown = await item.$('[aria-label="Duration"]').getText();
      console.log("Native duration shown", shown);
      expect(shown).toBe(`0:${String(Math.round(rec.durationMs / 1000)).padStart(2, "0")}`);
      await (await item.$("button=Play")).click();
      const audio = await item.$(`audio[aria-label="Microphone — ${rec.title}"]`);
      await audio.waitForExist({ timeout: 20_000 });
      const video = await item.$(`video[aria-label="Screen 1 — ${rec.title}"]`);
      await video.waitForExist({ timeout: 5_000 });
      const meta = async (el: WebdriverIO.Element) => {
        await browser.waitUntil(() => browser.execute((e: any) => e.readyState >= 1 || !!e.error, el), { timeout: 20_000, timeoutMsg: "media metadata not loaded" });
        return browser.execute((e: any) => ({ duration: e.duration, error: e.error?.code ?? null, src: e.currentSrc.slice(0, 24) }), el);
      };
      const a = await meta(audio);
      const v = await meta(video);
      console.log("Native player", JSON.stringify({ audio: a, video: v }));
      expect(a.error).toBeNull();
      expect(Math.abs(a.duration * 1000 - rec.durationMs)).toBeLessThanOrEqual(1000);
      expect(v.error).toBeNull();
      expect(v.duration).toBeGreaterThan(0);
      // Plays: the clock advances.
      await browser.execute((e: any) => { e.muted = true; return e.play(); }, audio);
      await browser.pause(1500);
      const t = await browser.execute((e: any) => e.currentTime, audio);
      console.log("Native audio currentTime after 1.5 s", t);
      expect(t).toBeGreaterThan(0.5);
      await invoke("recording_delete", { id: rec.id });

      // Microphone only (no screen in the conversation): attach.
      await invoke("privacy_set_source", { source: "screen", mode: { type: "onDemand" }, agent: "ask" });
      await browser.refresh();
      const b = await record(4);
      console.log("Native recording B", JSON.stringify({ source: b.rec.source, durationMs: b.rec.durationMs }));
      const itemB = await $(`li[aria-label="${b.rec.title}"]`);
      await (await itemB.$("button=Attach to conversation")).click();
      const body = () => browser.execute(() => document.body.innerText);
      // Audio is transcribed on attach (007): needs the installed speech model.
      await browser.waitUntil(async () => (await body()).includes("Recording attached"), { timeout: 120_000 }).catch(async (e) => {
        console.log("Native settings text", JSON.stringify((await body()).slice(-600)));
        throw e;
      });
      const notice = (await body()).split("\n").find((l) => l.includes("Recording attached"));
      console.log("Native attach notice", notice);
      expect(notice).toBe("Recording attached to the conversation: 1 file(s).");
      await browser.switchToWindow(overlay);
      const chip = await $('//*[contains(text(), "mic.wav")]');
      await chip.waitForDisplayed({ timeout: 30_000 }).catch(async (e) => {
        console.log("Native overlay text", JSON.stringify((await body()).slice(0, 800)));
        throw e;
      });
      console.log("Native chip", await chip.getText());
      await browser.switchToWindow(settings);
      await invoke("recording_delete", { id: b.rec.id });
    } finally {
      await browser.switchToWindow(settings).catch(() => undefined);
      await invoke("privacy_set_source", { source: "mic", mode: { type: "onDemand" }, agent: "ask" });
      await invoke("privacy_set_source", { source: "screen", mode: { type: "onDemand" }, agent: "ask" });
      await invoke("settings_update", { patch: { microphoneDeviceId: null } });
    }
  });
});
