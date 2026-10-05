// QA-033: the user attaches the last minutes of the recent buffer (screen +
// Microphone + System) from the context menu; the turn carries keyframes and
// the transcript of the same interval, labelled by source. Audio is SAPI
// speech into VB-Cable "CABLE Input", heard as the microphone "CABLE Output"
// and as System audio (loopback of CABLE Input): no room audio. Needs a
// profile with a speech model. Screen segments and clip files are removed.
import { spawn } from "node:child_process";
import { existsSync, readdirSync, rmSync, statSync } from "node:fs";
import path from "node:path";
import { responsesUpstream } from "../responses";

const CABLE_MIC = "CABLE Output (VB-Audio Virtual Cable)";
const CABLE_RENDER = "CABLE Input (VB-Audio Virtual Cable)";
const SPEAK = [
  "$v = New-Object -ComObject SAPI.SpVoice",
  `$v.AudioOutput = @($v.GetAudioOutputs() | Where-Object { $_.GetDescription() -eq '${CABLE_RENDER}' })[0]`,
  "$v.Rate = -1",
  "[void]$v.Speak('The quarterly budget review starts on Thursday morning.')",
].join("; ");

const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

function find(dir: string, name: RegExp, out: string[] = []): string[] {
  if (!existsSync(dir)) return out;
  for (const e of readdirSync(dir)) {
    const p = path.join(dir, e);
    if (statSync(p).isDirectory()) find(p, name, out);
    else if (name.test(e)) out.push(p);
  }
  return out;
}

describe("Recent buffer clip (QA-033)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  const home = process.env.AURA_HOME!;
  before(async () => { up = await responsesUpstream(() => "QA clip received", [{ id: "qa-model", architecture: { input_modalities: ["text", "image"] }, supported_parameters: ["tools"] }]); });
  after(async () => { await up.close(); });

  it("attaches screen keyframes and the labelled transcript of the last minute", async () => {
    await browser.execute(async (url, mic, render) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en", microphoneDeviceId: mic, systemAudioDeviceId: render } });
      await i("privacy_set_paused", { paused: false }).catch(() => undefined);
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      const p = await i("providers_save", { draft: { name: "QA clip", preset: "ollama", baseUrl: url }, credential: null });
      await i("providers_test", { id: p.id });
    }, up.baseUrl, CABLE_MIC, CABLE_RENDER);
    try {
      for (const source of ["screen", "mic", "systemAudio"]) {
        await invoke("privacy_set_source", { source, mode: { type: "recentBuffer", minutes: 2 }, agent: "never" });
      }
      // Speech lands in both audio buffers; the screen buffer keeps frames.
      await browser.pause(3000);
      await new Promise<void>((resolve) => spawn("powershell", ["-NoProfile", "-Command", SPEAK], { stdio: "ignore" }).on("exit", () => resolve()));
      await browser.pause(4000);

      await browser.refresh();
      const box = await $("textarea");
      await box.waitForDisplayed({ timeout: 30_000 });
      await browser.keys(["Control", "ArrowDown"]);
      await $('button[aria-label^="Model "]').click();
      const picker = await $('[role="dialog"][aria-label="Model and mode"]');
      await picker.waitForDisplayed();
      await (await picker.$('[role="menu"][aria-label="Model"]')).$("button*=qa-model").click();
      await browser.keys("Escape");

      await (await $('button[aria-label="Add context"]')).click();
      await (await $('//*[@role="option"][contains(., "@recent")]')).click();
      const dialog = await $('[role="dialog"][aria-label="Attach recent buffer"]');
      await dialog.waitForDisplayed({ timeout: 10_000 });
      const minutes = await dialog.$('input[aria-label="Last minutes"]');
      await minutes.click();
      await browser.keys(["Control", "a"]);
      await browser.keys("1");
      expect(await dialog.$('input[type="checkbox"]').isSelected()).toBe(true);
      expect(await dialog.$('select[aria-label="Audio"]').getValue()).toBe("both");
      const t0 = Date.now();
      await (await dialog.$("button=Attach")).click();
      const chip = await $('//*[contains(text(), "Últimos 1 min")]');
      await chip.waitForDisplayed({ timeout: 120_000 });
      console.log("Native clip chip", await chip.getText(), `${Date.now() - t0} ms`);

      await box.setValue("QA what happened in the last minute?");
      await browser.keys("Enter");
      await browser.waitUntil(async () => (await browser.execute(() => document.body.innerText)).includes("QA clip received"), { timeout: 180_000, timeoutMsg: "no reply" });
      const turn = up.requests.filter((r) => r.url === "/v1/responses").at(-1)!;
      const input = JSON.stringify(turn.body?.input ?? "");
      const images = (input.match(/"type":"input_image"/g) ?? []).length;
      const transcript = /Você:[^"\\]*/.exec(input)?.[0] ?? "";
      const system = /Sistema:[^"\\]*/.exec(input)?.[0] ?? "";
      console.log("Native clip turn", JSON.stringify({ images, frames: (input.match(/Quadro \d+/g) ?? []).length, you: transcript, system }));
      expect(images).toBeGreaterThan(0);
      expect(images).toBeLessThanOrEqual(8);
      expect(transcript.toLowerCase()).toMatch(/budget|thursday/);
      expect(system.toLowerCase()).toMatch(/budget|thursday/);

      const frames = find(path.join(home, "workspaces"), /^frame-\d+\.png$/);
      const wavs = find(path.join(home, "workspaces"), /^(mic|system)\.wav$/).filter((p) => p.includes(`${path.sep}clips${path.sep}`));
      console.log("Native clip files", JSON.stringify({ frames: frames.length, wavs: wavs.map((p) => path.basename(p)) }));
      expect(frames.length).toBe(images);
      expect(wavs.map((p) => path.basename(p)).sort()).toEqual(["mic.wav", "system.wav"]);
    } finally {
      for (const source of ["screen", "mic", "systemAudio"]) {
        await invoke("privacy_set_source", { source, mode: { type: source === "systemAudio" ? "off" : "onDemand" }, agent: "ask" }).catch(() => undefined);
      }
      await invoke("settings_update", { patch: { microphoneDeviceId: null, systemAudioDeviceId: null } }).catch(() => undefined);
      // Screen content of this run: clip frames and buffer segments.
      for (const dir of find(path.join(home, "workspaces"), /^frame-\d+\.png$/).map((p) => path.dirname(p))) rmSync(dir, { recursive: true, force: true });
      rmSync(path.join(home, "captures", "screen"), { recursive: true, force: true });
    }
  });
});
