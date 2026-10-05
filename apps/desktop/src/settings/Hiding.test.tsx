// 021: "Ocultar em transmissões" and "Testar ocultação".
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";
import { api } from "../ipc/commands";
import { freshApp } from "../test/harness";
import { SettingsApp } from "./SettingsApp";

describe("hide from screen sharing", () => {
  it("is on by default and the test reports what Windows says", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/general";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const sw = await screen.findByRole("switch", { name: "Ocultar em transmissões e gravações" });
    expect(sw).toBeChecked();
    vi.spyOn(api, "captureHidingCheck").mockResolvedValue([
      { window: "overlay", hidden: true },
      { window: "settings", hidden: true },
    ]);
    await user.click(screen.getByRole("button", { name: "Testar agora" }));
    expect(await screen.findByText(/Tudo oculto/)).toBeInTheDocument();
    expect(screen.getByText(/Overlay: oculta/)).toBeInTheDocument();
  });

  it("warns when a window is still visible", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/general";
    const user = userEvent.setup();
    render(<SettingsApp />);
    vi.spyOn(api, "captureHidingCheck").mockResolvedValue([{ window: "overlay", hidden: false }]);
    await user.click(await screen.findByRole("button", { name: "Testar agora" }));
    expect(await screen.findByText(/não bate com a configuração/)).toBeInTheDocument();
  });
});
