import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { OverlayApp } from "./OverlayApp";
import { filterMenu, insertAtCaret } from "./InputBar";

describe("Overlay", () => {
  it("first run shows the ChatGPT sign-in and the plan-usage notice", async () => {
    await freshApp({ signedIn: false });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const button = await screen.findByRole("button", { name: /Continuar com ChatGPT/ });
    await user.click(button);
    expect(await screen.findByRole("dialog", { name: /plano ChatGPT/ })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Entendi" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByRole("textbox")).toBeInTheDocument();
  });

  it("sends with Enter, streams the answer and expands", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("textbox");
    await user.type(box, "explique este erro{Enter}");
    expect(await screen.findByText("explique este erro")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByText(/tipos incompatíveis/)).toBeInTheDocument());
    // Tool calls appear as collapsible cards.
    expect(screen.getByText("aura.active_window_info")).toBeInTheDocument();
    // Header only exists in the expanded state.
    expect(screen.getByRole("button", { name: "Nova conversa" })).toBeInTheDocument();
  });

  it("Shift+Enter inserts a newline instead of sending", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = (await screen.findByRole("textbox")) as HTMLTextAreaElement;
    await user.type(box, "linha 1{Shift>}{Enter}{/Shift}linha 2");
    expect(box.value).toBe("linha 1\nlinha 2");
    expect(screen.queryByText("linha 1")).not.toBeInTheDocument();
  });

  it("approval card accepts with the A key", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("textbox"), "/aprovar{Enter}");
    const card = await screen.findByRole("group", { name: "Executar comando" });
    expect(within(card).getByText("pip install requests")).toBeInTheDocument();
    await waitFor(() => expect(card).toHaveFocus());
    await user.keyboard("a");
    await waitFor(() => expect(screen.getByText("Respondido")).toBeInTheDocument());
    await waitFor(() => expect(screen.getByText(/instalei/)).toBeInTheDocument());
  });

  it("agent questions are answered with a form", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("textbox"), "/pergunta{Enter}");
    const card = await screen.findByRole("group", { name: "O agente tem uma pergunta" });
    await user.click(within(card).getByLabelText("TypeScript"));
    await user.click(within(card).getByRole("button", { name: "Responder" }));
    await waitFor(() => expect(screen.getByText("Certo, vou usar TypeScript.")).toBeInTheDocument());
  });

  it("shows context usage after a turn", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("textbox"), "oi{Enter}");
    expect(await screen.findByRole("button", { name: /% do contexto usado/ })).toBeInTheDocument();
  });

  it("slash menu lists quick commands and completes with Tab", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = (await screen.findByRole("textbox")) as HTMLTextAreaElement;
    await user.type(box, "/tra");
    const list = await screen.findByRole("listbox");
    expect(within(list).getByText("/traduzir")).toBeInTheDocument();
    await user.keyboard("{Tab}");
    expect(box.value).toBe("/traduzir ");
  });

  it("screen capture adds a chip that can be removed", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("textbox");
    box.focus();
    await user.keyboard("{Control>}{Shift>}s{/Shift}{/Control}");
    const chip = await screen.findByText(/Tela · main.rs/);
    expect(chip).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Remover Tela/ }));
    await waitFor(() => expect(screen.queryByText(/Tela · main.rs/)).not.toBeInTheDocument());
  });

  it("consent requests from the agent are answered inline", async () => {
    const bridge = await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await screen.findByRole("textbox");
    act(() =>
      bridge.emitLocal!("aura://event", {
        channel: "consent",
        event: { id: "c1", conversation: "u", tool: "screen_capture", source: "screen", reason: "ver o erro", app: "Code.exe" },
      }),
    );
    const dialog = await screen.findByRole("alertdialog", { name: /a tela/ });
    await user.click(within(dialog).getByRole("button", { name: "Permitir uma vez" }));
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("dictation result lands at the caret", async () => {
    const bridge = await freshApp({ signedIn: true });
    render(<OverlayApp />);
    const box = (await screen.findByRole("textbox")) as HTMLTextAreaElement;
    act(() => bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "done", text: "olá mundo" } }));
    await waitFor(() => expect(box.value).toBe("olá mundo"));
  });
});

describe("first run", () => {
  it("walks through privacy, shortcut and voice, then never shows again", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.settings.onboarded = false;
    const { useApp } = await import("../state/app");
    useApp.getState().setSettings({ ...bridge.state.settings });
    const user = userEvent.setup();
    render(<OverlayApp />);
    expect(await screen.findByText("Quando o agente pode ver sua tela?")).toBeInTheDocument();
    await user.click(screen.getByRole("radio", { name: /Nunca/ }));
    await waitFor(() => expect(useApp.getState().privacy?.screen.agent).toBe("never"));
    await user.click(screen.getByRole("button", { name: "Próximo" }));
    expect(screen.getByText("Abra o Aura de qualquer lugar")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Próximo" }));
    expect(screen.getByText("Fale em vez de digitar")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Concluir" }));
    await waitFor(() => expect(useApp.getState().settings?.onboarded).toBe(true));
    expect(screen.queryByText("Fale em vez de digitar")).not.toBeInTheDocument();
  });
});

describe("input helpers", () => {
  it("inserts with a separating space", () => {
    expect(insertAtCaret("abc", 3, "def")).toEqual(["abc def", 7]);
    expect(insertAtCaret("", 0, "x")).toEqual(["x", 1]);
    expect(insertAtCaret("a b", 2, "c")).toEqual(["a c b", 3]);
  });
  it("filters menu items case-insensitively", () => {
    const items = [{ id: "1", label: "/Traduzir", hint: "", insert: "" }, { id: "2", label: "/tldr", hint: "", insert: "" }];
    expect(filterMenu(items, "tra").map((i) => i.id)).toEqual(["1"]);
  });
});

describe("window behaviour (001 revision 2)", () => {
  function spyCalls(bridge: Awaited<ReturnType<typeof freshApp>>) {
    const calls: string[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = ((cmd: string, args?: Record<string, unknown>) => {
      calls.push(cmd);
      return invoke(cmd, args);
    }) as typeof bridge.invoke;
    return calls;
  }

  it("Esc never hides the Overlay; Minimize to tray does", async () => {
    const bridge = await freshApp({ signedIn: true });
    const calls = spyCalls(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    (await screen.findByRole("textbox")).focus();
    await user.keyboard("{Escape}");
    expect(calls).not.toContain("overlay_hide");
    await user.click(screen.getByRole("button", { name: "Minimizar para a bandeja" }));
    expect(calls).toContain("overlay_hide");
  });

  it("compact Overlay has window controls", async () => {
    await freshApp({ signedIn: true });
    render(<OverlayApp />);
    await screen.findByRole("textbox");
    expect(screen.getByRole("button", { name: "Expandir" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Aparência" })).toBeInTheDocument();
  });

  it("opacity is adjusted from the Overlay itself", async () => {
    await freshApp({ signedIn: true });
    const { useApp } = await import("../state/app");
    const { fireEvent } = await import("@testing-library/react");
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Aparência" }));
    const slider = within(screen.getByRole("dialog", { name: "Aparência" })).getByRole("slider", { name: "Opacidade do Overlay" });
    fireEvent.change(slider, { target: { value: "60" } });
    expect(document.documentElement.style.getPropertyValue("--overlay-opacity")).toBe("0.6");
    await waitFor(() => expect(useApp.getState().settings?.opacity).toBe(0.6), { timeout: 2000 });
  });

  it("model picker switches the mode", async () => {
    await freshApp({ signedIn: true });
    const { useSession } = await import("./session");
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(await screen.findByRole("button", { name: /modo Chat/ }));
    await user.click(within(screen.getByRole("dialog", { name: "Modelo e modo" })).getByRole("menuitemradio", { name: /Tarefa/ }));
    expect(useSession.getState().mode).toBe("task");
  });
});
