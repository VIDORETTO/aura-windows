import { vi } from "vitest";
import { api } from "../ipc/commands";
import { freshApp } from "../test/harness";
import { readSelectionAloud, stopSpeaking } from "./speech";

describe("read selection aloud (019)", () => {
  it("speaks the remembered selection, and says so when there is none", async () => {
    await freshApp({ signedIn: true });
    const speak = vi.spyOn(api, "speak").mockResolvedValue(["", "audio/wav"]);
    vi.spyOn(api, "replaceTarget").mockResolvedValueOnce("  Olá, mundo  ").mockResolvedValueOnce(null);
    expect(await readSelectionAloud()).toBe(true);
    expect(speak).toHaveBeenCalledWith("Olá, mundo");
    stopSpeaking();
    expect(await readSelectionAloud()).toBe(false);
    expect(speak).toHaveBeenCalledTimes(1);
  });
});
