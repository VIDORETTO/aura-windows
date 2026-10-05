// 015: in the native window, the compact Overlay shows the mode, the `/` menu
// lists localized commands without /tela, Esc keeps the text and Enter on a
// message ending in "@word" sends it instead of capturing the screen.
const invoke = (cmd: string, args: Record<string, unknown> = {}) =>
  browser.execute(async (c, a) => JSON.stringify(await (window as any).__TAURI_INTERNALS__.invoke(c, a)), cmd, args).then((s) => JSON.parse(s as string));

describe("Composer menus, mode badge (015)", () => {
  it("compact mode badge, / menu and @word messages", async () => {
    await invoke("settings_update", { patch: { language: "ptBr" } });
    await browser.refresh();
    const box = await $("textarea");
    await box.waitForDisplayed({ timeout: 30_000 });

    const badge = await $('button[aria-label="Modo: Chat"]');
    await badge.waitForDisplayed({ timeout: 10_000 });
    await badge.click();
    await (await $('[role="dialog"][aria-label="Modo"]')).$("button*=Tarefa").click();
    await $('button[aria-label="Modo: Tarefa"]').waitForDisplayed({ timeout: 10_000 });
    await (await $('button[aria-label="Modo: Tarefa"]')).click();
    await (await $('[role="dialog"][aria-label="Modo"]')).$("button*=Chat").click();
    await $('button[aria-label="Modo: Chat"]').waitForDisplayed({ timeout: 10_000 });

    await box.click();
    await browser.keys("/");
    const list = await $('[role="listbox"]');
    await list.waitForDisplayed({ timeout: 10_000 });
    const options = await browser.execute(() => [...document.querySelectorAll('[role="option"]')].map((o) => o.textContent ?? ""));
    console.log("Native / menu", JSON.stringify(options));
    expect(options.some((o) => o.startsWith("/plano"))).toBe(true);
    expect(options.some((o) => o.startsWith("/compactar"))).toBe(true);
    expect(options.some((o) => o.startsWith("/tela"))).toBe(false);
    await browser.keys("Enter");
    expect(await box.getValue()).toBe("/");
    await browser.keys("Escape");
    expect(await box.getValue()).toBe("/");
    expect(await $('[role="listbox"]').isExisting()).toBe(false);

    await browser.keys("Backspace");
    for (const ch of "fala com o @joao") await browser.keys(ch);
    expect(await $('[role="listbox"]').isExisting()).toBe(false);
    await browser.keys("Enter");
    await browser.waitUntil(
      () => browser.execute(() => [...document.querySelectorAll("div")].some((d) => d.textContent === "fala com o @joao" && d.className.includes("rounded-br-sm"))),
      { timeout: 30_000, timeoutMsg: "message with @word was not sent" },
    );
    const screenChips = await browser.execute(() => [...document.querySelectorAll("li")].filter((li) => /^Tela/.test(li.textContent ?? "")).length);
    expect(screenChips).toBe(0);
  });
});
