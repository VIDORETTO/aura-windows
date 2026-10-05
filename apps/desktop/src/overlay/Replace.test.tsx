// 019 AC-002: "Substituir seleção" with before → after, then "Desfazer".
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";
import { api } from "../ipc/commands";
import { freshApp } from "../test/harness";
import { emptyThread } from "../state/conversation";
import { MessageList } from "./Messages";

const thread = () => ({
  ...emptyThread("t"),
  blocks: [{ type: "assistant" as const, id: "a", text: "O texto estava ruim.", streaming: false }],
});

describe("replace selection", () => {
  it("shows the before → after and can be undone", async () => {
    await freshApp({ signedIn: true });
    vi.spyOn(api, "replaceTarget").mockResolvedValue("tava muito ruim o texto");
    const replace = vi.spyOn(api, "replaceSelection").mockResolvedValue("tava muito ruim o texto");
    const undo = vi.spyOn(api, "undoReplace").mockResolvedValue(true);
    const user = userEvent.setup();
    render(<MessageList thread={thread()} />);
    const button = await screen.findByRole("button", { name: "Substituir seleção" });
    expect(button.title).toContain("Antes: tava muito ruim o texto");
    expect(button.title).toContain("Depois: O texto estava ruim.");
    await user.click(button);
    expect(replace).toHaveBeenCalledWith("O texto estava ruim.");
    await user.click(await screen.findByRole("button", { name: "Desfazer substituição" }));
    expect(undo).toHaveBeenCalled();
    await waitFor(() => expect(screen.getByRole("button", { name: "Substituir seleção" })).toBeInTheDocument());
  });

  it("is hidden when nothing was selected", async () => {
    await freshApp({ signedIn: true });
    vi.spyOn(api, "replaceTarget").mockResolvedValue(null);
    render(<MessageList thread={thread()} />);
    await screen.findByText("O texto estava ruim.");
    expect(screen.queryByRole("button", { name: "Substituir seleção" })).toBeNull();
  });
});
