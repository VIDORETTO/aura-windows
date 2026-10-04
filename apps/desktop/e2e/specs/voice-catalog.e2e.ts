// QA-012: the speech model catalog shows comparable metadata (006 AC-001).
// Oracles are literals from crates/aura-asr/models.toml.
describe("Speech model catalog metadata (QA-012)", () => {
  it("shows speed/accuracy bars, languages and CPU/GPU/RAM requirements in English and Portuguese", async () => {
    const overlay = await browser.getWindowHandle();
    await browser.execute(async () => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      await invoke("settings_update", { patch: { language: "en" } });
      await invoke("settings_open", { section: "voice" });
    });
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    await browser.switchToWindow((await browser.getWindowHandles()).find((h) => h !== overlay)!);
    const row = (id: string) => $(`[data-model="${id}"]`);
    await (await row("parakeet-tdt-0.6b-v3")).waitForDisplayed({ timeout: 10_000 });
    const facts = (id: string) => browser.execute((model) => {
      const el = document.querySelector(`[data-model="${model}"]`)!;
      const meters = Object.fromEntries([...el.querySelectorAll('[role="meter"]')].map((m) => [m.getAttribute("aria-label"), m.getAttribute("aria-valuenow")]));
      return { meters, text: (el as HTMLElement).innerText };
    }, id);

    const parakeet = await facts("parakeet-tdt-0.6b-v3");
    console.log("Native catalog EN parakeet", JSON.stringify(parakeet));
    expect(parakeet.meters).toEqual({ Speed: "85", Accuracy: "85" });
    expect(parakeet.text).toContain("Languages: bg, hr, cs");
    expect(parakeet.text).toContain(", pt,");
    expect(parakeet.text).toContain("CPU · min. 2 GB RAM");
    expect(parakeet.text).toContain("CC-BY-4.0");
    expect(parakeet.text).toContain("Fast on CPU, 25 European languages including Portuguese.");
    expect(parakeet.text).not.toContain("Rápido");

    const turbo = await facts("whisper-turbo");
    console.log("Native catalog EN turbo", JSON.stringify(turbo));
    expect(turbo.meters).toEqual({ Speed: "40", Accuracy: "82" });
    expect(turbo.text).toContain("Languages: multilingual");
    expect(turbo.text).toContain("GPU recommended · min. 4 GB RAM");

    await browser.execute(async () => (window as any).__TAURI_INTERNALS__.invoke("settings_update", { patch: { language: "ptBr" } }));
    await browser.waitUntil(async () => (await facts("whisper-turbo")).text.includes("GPU recomendada · mín. 4 GB de RAM"), { timeout: 5000 });
    const turboPt = await facts("whisper-turbo");
    console.log("Native catalog PT turbo", JSON.stringify(turboPt));
    expect(turboPt.meters).toEqual({ Velocidade: "40", "Precisão": "82" });
    expect(turboPt.text).toContain("Idiomas: multilíngue");
    expect(turboPt.text).toContain("Máxima precisão com GPU dedicada; 99 idiomas.");
  });
});
