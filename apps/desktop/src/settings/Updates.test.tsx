import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { AboutSection } from "./Diagnostics";
import { checkForUpdate } from "../lib/updates";

vi.mock("../lib/updates", () => ({ checkForUpdate: vi.fn(), installUpdate: vi.fn() }));

it("keeps an update error in About and clears it on a successful retry", async () => {
  await freshApp({ signedIn: true });
  vi.mocked(checkForUpdate).mockRejectedValueOnce(new Error("HTTP 404 updater endpoint"));
  vi.mocked(checkForUpdate).mockResolvedValueOnce({ kind: "none" });
  const user = userEvent.setup();
  render(<AboutSection />);
  await user.click(await screen.findByRole("button", { name: "Verificar atualizações" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("HTTP 404 updater endpoint");
  await user.click(screen.getByRole("button", { name: "Verificar atualizações" }));
  await waitFor(() => expect(screen.queryByRole("alert")).not.toBeInTheDocument());
  expect(await screen.findByText("Você está na versão mais recente.")).toBeInTheDocument();
});
