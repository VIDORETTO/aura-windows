// Packaged native frontend. Optional loopback upstream runs the production host.
import { localProvider } from "../provider";

describe("Overlay panels (QA-002)", () => {
  let upstream: Awaited<ReturnType<typeof localProvider>> | undefined;
  before(async () => {
    if (!process.env.AURA_E2E_REAL_PROVIDER) return;
    upstream = await localProvider();
    await browser.execute(async (baseUrl) => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      for (const p of await invoke("providers_list")) await invoke("providers_remove", { id: p.id });
      const p = await invoke("providers_save", { draft: { name: "QA controlled provider", preset: "ollama", baseUrl }, credential: null });
      await invoke("providers_test", { id: p.id });
    }, upstream.baseUrl);
    await $("textarea").waitForDisplayed();
    await $("textarea").click();
    await browser.keys(["Control", "ArrowDown"]);
    await $('button[aria-label^="Modelo "]').click();
    await $('button[role="menuitemradio"]*=QA literal model').click();
    await browser.keys("Escape");
  });
  after(async () => { if (upstream) await upstream.close(); });
  it("keeps the conversation readable when alternating History and Files", async () => {
    await $("textarea").waitForDisplayed({ timeout: 20_000 });
    await $("textarea").setValue("QA-002 synthetic demo message");
    await $("textarea").click();
    await $('button[aria-label="Enviar"]').click();
    try {
      await $('button[aria-label="Arquivos e alterações"]').waitForDisplayed({ timeout: 5000 });
    } catch (error) {
      console.log("Isolated QA preparation state", await browser.execute(() => document.body.innerText));
      throw error;
    }
    await $("textarea").setValue("rascunho QA-002");
    for (const width of [480, 640, 1024]) {
      await browser.execute(async (w) => (window as any).__TAURI_INTERNALS__.invoke("plugin:window|set_size", {
        label: "overlay", value: { Logical: { width: w, height: 600 } },
      }), width);
      await browser.pause(300);
      await $('button[aria-label="Histórico"]').click();
      await $('aside[aria-label="Histórico"]').waitForDisplayed();
      await $('button[aria-label="Arquivos e alterações"]').click();
      await $('aside[aria-label="Arquivos e alterações"]').waitForDisplayed();
      const conversationWidth = await browser.execute(() => {
        const body = document.querySelector('aside[aria-label="Arquivos e alterações"]')!.parentElement!;
        return body.querySelector('[class*="min-w-0 flex-1 flex-col"]')!.getBoundingClientRect().width;
      });
      console.log("QA-002 conversation width", { width, conversationWidth });
      expect(conversationWidth).toBeGreaterThanOrEqual(320);
      expect(await $('aside[aria-label="Histórico"]').isExisting()).toBe(false);
      expect(await $("textarea").getValue()).toBe("rascunho QA-002");
      await $("textarea").click();
      await browser.keys(["Control", "h"]);
      await $('aside[aria-label="Histórico"]').waitForDisplayed();
      expect(await $('aside[aria-label="Arquivos e alterações"]').isExisting()).toBe(false);
      const widthWithHistory = await browser.execute(() => {
        const body = document.querySelector('aside[aria-label="Histórico"]')!.parentElement!;
        return body.querySelector('[class*="min-w-0 flex-1 flex-col"]')!.getBoundingClientRect().width;
      });
      console.log("QA-002 conversation width with History", { width, widthWithHistory });
      expect(widthWithHistory).toBeGreaterThanOrEqual(320);
      await browser.keys("Escape");
      expect(await $('aside[aria-label="Histórico"]').isExisting()).toBe(false);
      expect(await $("textarea").getValue()).toBe("rascunho QA-002");
    }
  });
});
