// 038: meeting privacy options in Settings › Privacy.
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { useApp } from "../state/app";
import { SettingsApp } from "./SettingsApp";

describe("meeting privacy", () => {
  it("only text stays by default; both options can be turned on", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/privacy";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const keep = await screen.findByRole("switch", { name: "Guardar o áudio depois da reunião" });
    const redact = screen.getByRole("switch", { name: "Ocultar dados pessoais do modelo" });
    expect(keep).not.toBeChecked();
    expect(redact).not.toBeChecked();
    await user.click(keep);
    await user.click(redact);
    expect(useApp.getState().settings).toMatchObject({ meetingKeepAudio: true, meetingRedactPii: true });
  });

  it("watched words are saved one per line", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/privacy";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const box = await screen.findByRole("textbox", { name: "Palavras para vigiar na reunião" });
    await user.type(box, "preço{enter}prazo final");
    await user.tab();
    expect(useApp.getState().settings?.meetingWatchWords).toEqual(["preço", "prazo final"]);
  });
});
