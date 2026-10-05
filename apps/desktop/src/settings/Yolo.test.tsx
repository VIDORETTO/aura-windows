// 018: YOLO mode is turned on in Settings only after typing the word, and the
// Overlay shows it on the Task badge.
import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { SettingsApp } from "./SettingsApp";
import { OverlayApp } from "../overlay/OverlayApp";

describe("YOLO mode (018 AC-005)", () => {
  it("needs ACEITO typed, warns about the risks and turns off with one click", async () => {
    const bridge = await freshApp({ signedIn: true });
    window.location.hash = "#/settings/general";
    const user = userEvent.setup();
    render(<SettingsApp />);
    expect(await screen.findByRole("status", { name: "Estado do modo YOLO" })).toHaveTextContent("Desligado");
    await user.click(screen.getByRole("button", { name: "Ligar modo YOLO…" }));
    const warning = screen.getByRole("group", { name: "Atenção: o agente vai agir sem pedir permissão" });
    expect(within(warning).getByText(/Executa qualquer comando/)).toBeInTheDocument();
    const confirm = within(warning).getByRole("button", { name: "Ligar YOLO" });
    const box = within(warning).getByRole("textbox", { name: "Para ligar, escreva ACEITO" });
    await user.type(box, "ACEIT");
    expect(confirm).toBeDisabled();
    expect(bridge.state.settings.yolo).toBe(false);
    await user.type(box, "o");
    expect(confirm).toBeEnabled();
    await user.click(confirm);
    expect(bridge.state.settings.yolo).toBe(true);
    expect(screen.getByRole("status", { name: "Estado do modo YOLO" })).toHaveTextContent("Ligado");
    await user.click(screen.getByRole("button", { name: "Desligar modo YOLO" }));
    expect(bridge.state.settings.yolo).toBe(false);
  });

  it("the host refuses a wrong word and a settings patch cannot turn it on", async () => {
    const bridge = await freshApp({ signedIn: true });
    await expect(bridge.invoke("yolo_set", { enabled: true, confirmation: "sim" })).rejects.toMatchObject({ code: "invalid" });
    await bridge.invoke("settings_update", { patch: { yolo: true } });
    expect(bridge.state.settings.yolo).toBe(false);
    await bridge.invoke("yolo_set", { enabled: true, confirmation: "accept" });
    expect(bridge.state.settings.yolo).toBe(true);
  });

  it("the Overlay marks Task mode as YOLO", async () => {
    const bridge = await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Modo: Chat" }));
    await user.click(within(screen.getByRole("dialog", { name: "Modo" })).getByRole("menuitemradio", { name: /Tarefa/ }));
    expect(await screen.findByRole("button", { name: "Modo: Tarefa" })).toBeInTheDocument();
    await act(async () => { await bridge.invoke("yolo_set", { enabled: true, confirmation: "ACEITO" }); });
    expect(await screen.findByRole("button", { name: "Modo: Tarefa · YOLO" })).toBeInTheDocument();
  });
});
