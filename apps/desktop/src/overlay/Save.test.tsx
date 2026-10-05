// 020 AC-003: the star keeps an answer in "Salvos".
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";
import { api } from "../ipc/commands";
import { freshApp } from "../test/harness";
import { emptyThread } from "../state/conversation";
import { MessageList } from "./Messages";

describe("save an answer", () => {
  it("the star saves the text once and shows it was saved", async () => {
    await freshApp({ signedIn: true });
    vi.spyOn(api, "replaceTarget").mockResolvedValue(null);
    const add = vi.spyOn(api, "noteAdd").mockResolvedValue(undefined);
    const user = userEvent.setup();
    const thread = { ...emptyThread("t"), blocks: [{ type: "assistant" as const, id: "a", text: "Receita de bolo de cenoura", streaming: false }] };
    render(<MessageList thread={thread} />);
    await user.click(await screen.findByRole("button", { name: /Salvar/ }));
    expect(add).toHaveBeenCalledWith("saved", "Receita de bolo de cenoura");
    expect(await screen.findByRole("button", { name: /Salvo/ })).toHaveAttribute("aria-pressed", "true");
  });
});
