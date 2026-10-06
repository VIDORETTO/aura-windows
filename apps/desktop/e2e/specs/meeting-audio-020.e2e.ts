// Real WASAPI -> Parakeet worker -> meeting transcript. Synthetic SAPI speech
// routed only through VB-Cable; no ambient microphone, account or cloud ASR.
import { spawnSync } from "node:child_process";
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

describe("Real meeting audio (0.2.0)", () => {
  it("transcribes synthetic system speech, drops microphone echo and saves notes after stopping", async () => {
    const mic = "CABLE Output (VB-Audio Virtual Cable)";
    const output = "CABLE Input (VB-Audio Virtual Cable)";
    expect((await invoke("audio_devices", { system: false }) as any[]).map(d => d.id)).toContain(mic);
    expect((await invoke("audio_devices", { system: true }) as any[]).map(d => d.id)).toContain(output);
    await invoke("settings_update", { patch: { language: "en", asrLanguage: "en", microphoneDeviceId: mic, systemAudioDeviceId: output, cloudAsrProvider: null } });
    await invoke("voice_select", { id: "parakeet-tdt-0.6b-v3" });
    const meeting = await invoke("meeting_start", { title: "QA synthetic meeting", kind: "other", briefing: "QA" }) as any;
    try {
      await invoke("meeting_note", { text: "QA meeting note" });
      const result = spawnSync("powershell", ["-NoProfile", "-Command", [
        "$v = New-Object -ComObject SAPI.SpVoice",
        `$v.AudioOutput = @($v.GetAudioOutputs() | Where-Object { $_.GetDescription() -eq '${output}' })[0]`,
        `if (-not $v.AudioOutput -or $v.AudioOutput.GetDescription() -ne '${output}') { throw 'VB-Cable missing' }`,
        "$v.Voice = @($v.GetVoices() | Where-Object { $_.GetDescription() -like '*English*' })[0]",
        "[void]$v.Speak('The quick brown fox jumps over the lazy dog.')",
      ].join("; ")], { encoding: "utf8", timeout: 30000 });
      expect(result.status).toBe(0);
      let rows: any[] = [];
      await browser.waitUntil(async () => {
        rows = await invoke("meeting_utterances", { id: meeting.id }) as any[];
        return rows.some(r => r.speaker === "them" && /brown.*fox/i.test(r.text));
      }, { timeout: 90000, interval: 1000, timeoutMsg: "known speech not transcribed in the meeting" });
      console.log("Synthetic meeting transcript", rows.map(r => ({ speaker: r.speaker, text: r.text })));
      expect(rows.filter(r => r.speaker === "you" && /brown.*fox/i.test(r.text))).toHaveLength(0);
      await invoke("meeting_pause", { paused: true });
      await invoke("meeting_pause", { paused: false });
    } finally {
      await invoke("meeting_stop");
    }
    expect(await invoke("meeting_active")).toBe(null);
    await browser.reloadSession();
    const rows = await invoke("meeting_utterances", { id: meeting.id }) as any[];
    expect(rows.map(r => r.text)).toContain("QA meeting note");
    expect(rows.some(r => /brown.*fox/i.test(r.text))).toBe(true);
  });
});
