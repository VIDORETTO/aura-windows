// 022 AC-003: the last first-run step hands "how I work" to the agent.
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";
import { api } from "../ipc/commands";
import { freshApp } from "../test/harness";
import { FirstRun } from "./FirstRun";

describe("first run: tell me how you work", () => {
  it("one sentence goes to the configure skill; the grill-me path too", async () => {
    await freshApp({ signedIn: true });
    const task = vi.spyOn(api, "agentTask").mockResolvedValue(undefined);
    const user = userEvent.setup();
    render(<FirstRun />);
    for (let i = 0; i < 3; i++) await user.click(await screen.findByRole("button", { name: /Próximo|Avançar|Next/ }));
    expect(await screen.findByRole("heading", { name: "Conte como você trabalha" })).toBeInTheDocument();
    const send = screen.getByRole("button", { name: "Configurar com IA" });
    expect(send).toBeDisabled();
    await user.type(screen.getByRole("textbox"), "sou advogada e uso o Teams");
    await user.click(send);
    expect(task).toHaveBeenCalledWith(expect.stringContaining("$aura-configurar"), "task");
    expect(task.mock.calls[0][0]).toContain("sou advogada e uso o Teams");
  });
});
