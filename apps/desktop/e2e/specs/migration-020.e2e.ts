// Same isolated profile, first with the published 0.1.0 executable, then 0.2.0.
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => (window as any).__TAURI_INTERNALS__.invoke(c, a), cmd, args);

describe("Published 0.1.0 data migration", () => {
  it("preserves settings, providers and custom commands when the executable changes", async () => {
    if (process.env.AURA_E2E_MIGRATION_STAGE === "seed") {
      expect((await invoke("diagnostics") as any).version).toBe("0.1.0");
      await invoke("settings_update", { patch: { language: "en", accentColor: "#e4572e", asrLanguage: "en" } });
      await invoke("quick_save", { name: "qa-migration", template: "QA persistent command: {texto}", replace: true });
      return;
    }
    expect((await invoke("diagnostics") as any).version).toBe("0.2.0");
    const settings = await invoke("settings_get") as any;
    expect(settings.language).toBe("en");
    expect(settings.accentColor).toBe("#e4572e");
    expect(settings.asrLanguage).toBe("en");
    expect((await invoke("providers_list") as any[]).map(p => p.name)).toContain("Ollama (E2E)");
    expect((await invoke("quick_list") as any[]).find(q => q.name === "qa-migration")?.template).toBe("QA persistent command: {texto}");
    await invoke("note_add", { kind: "note", text: "QA new table after migration" });
    expect((await invoke("notes_all", { kind: "note", query: "migration" }) as any[]).map(n => n.text)).toContain("QA new table after migration");
  });
});
