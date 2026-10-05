// History: deletion asks inside the app (no browser dialog); archived
// conversations can be shown again and restored.
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { setBridge } from "../ipc/bridge";
import { OverlayApp } from "./OverlayApp";

describe("History", () => {
  it("deletes only after the in-app confirmation", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.history = [{ id: "thr_a", title: "Plano de viagem", preview: "…", updatedAt: 1, pinned: false, archived: false }];
    const deleted: unknown[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "conversation_delete") deleted.push(args.threadId);
      return bridge.invoke<R>(cmd, args);
    } });
    const confirm = vi.spyOn(window, "confirm");
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(screen.getByRole("button", { name: "Histórico" }));
    const panel = screen.getByRole("complementary", { name: "Histórico" });
    await user.click(await within(panel).findByRole("button", { name: "Excluir" }));
    expect(within(panel).getByText("Excluir esta conversa e seus arquivos?")).toBeInTheDocument();
    await user.click(within(panel).getByRole("button", { name: "Cancelar" }));
    expect(deleted).toEqual([]);
    await user.click(within(panel).getByRole("button", { name: "Excluir" }));
    await user.click(within(panel).getByRole("button", { name: "Confirmar" }));
    await waitFor(() => expect(deleted).toEqual(["thr_a"]));
    expect(confirm).not.toHaveBeenCalled();
    confirm.mockRestore();
  });

  it("shows archived conversations and restores them", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.history = [
      { id: "thr_a", title: "Ativa", preview: "…", updatedAt: 2, pinned: false, archived: false },
      { id: "thr_b", title: "Antiga", preview: "…", updatedAt: 1, pinned: false, archived: true },
    ];
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(screen.getByRole("button", { name: "Histórico" }));
    const panel = screen.getByRole("complementary", { name: "Histórico" });
    expect(await within(panel).findByText("Ativa")).toBeInTheDocument();
    expect(within(panel).queryByText("Antiga")).toBeNull();
    await user.click(within(panel).getByRole("button", { name: "Arquivadas" }));
    expect(await within(panel).findByText("Antiga")).toBeInTheDocument();
    expect(within(panel).queryByText("Ativa")).toBeNull();
    await user.click(within(panel).getByRole("button", { name: "Desarquivar" }));
    await waitFor(() => expect(within(panel).queryByText("Antiga")).toBeNull());
    await user.click(within(panel).getByRole("button", { name: "Arquivadas" }));
    expect(await within(panel).findByText("Antiga")).toBeInTheDocument();
  });
});
