// 020: list and delete reminders, notes and saved texts in Settings.
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { SettingsApp } from "./SettingsApp";

describe("reminders and notes page", () => {
  it("lists them and deletes one", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.reminderList = [{ id: "r1", text: "Ligar para o João", dueAt: 1_900_000_000, repeat: "none", prompt: null }];
    bridge.state.noteList = [
      { id: "n1", kind: "note", text: "Ideia do projeto azul", createdAt: 1_900_000_000 },
      { id: "n2", kind: "saved", text: "Resposta salva sobre impostos", createdAt: 1_900_000_100 },
    ];
    window.location.hash = "#/settings/personal";
    const user = userEvent.setup();
    render(<SettingsApp />);
    expect(await screen.findByText("Ligar para o João")).toBeInTheDocument();
    expect(await screen.findByText("Ideia do projeto azul")).toBeInTheDocument();
    expect(screen.getByText("Resposta salva sobre impostos")).toBeInTheDocument();
    await user.type(screen.getByRole("textbox", { name: "Buscar: Notas" }), "azul");
    expect(screen.queryByText("Ideia do projeto azul")).toBeInTheDocument();
    expect(screen.getByText("Resposta salva sobre impostos")).toBeInTheDocument();
    await user.click(await screen.findByRole("button", { name: "Apagar: Ligar para o João" }));
    expect(screen.queryByText("Ligar para o João")).not.toBeInTheDocument();
    expect(bridge.state.reminderList).toEqual([]);
  });
});
