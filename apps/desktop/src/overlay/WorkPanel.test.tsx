import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { WorkPanel, diffLines, sandboxedHtml } from "./WorkPanel";

describe("work panel", () => {
  it("previews HTML only inside a script-less, network-less sandbox", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<WorkPanel threadId="t" onClose={() => undefined} />);
    await user.click(await screen.findByRole("button", { name: "pagina.html" }));
    const frame = await screen.findByTitle("pagina.html");
    expect(frame.getAttribute("sandbox")).toBe("");
    expect(frame.getAttribute("srcdoc")).toContain("default-src 'none'");
    await user.click(screen.getByRole("button", { name: "relatorio.md" }));
    await waitFor(() => expect(screen.getByRole("heading", { name: "Relatório" })).toBeInTheDocument());
  });

  it("previews the text of a PDF and explains a PDF without text", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<WorkPanel threadId="t" onClose={() => undefined} />);
    await user.click(await screen.findByRole("button", { name: "relatorio.pdf" }));
    const preview = await screen.findByRole("region", { name: "Prévia de relatorio.pdf" });
    expect(preview).toHaveTextContent("Prévia do texto: 5 de 7 páginas");
    expect(screen.getByRole("heading", { name: "Página 1" })).toBeInTheDocument();
    expect(preview).toHaveTextContent("Texto da página 5");
    await user.click(screen.getByRole("button", { name: "scan.pdf" }));
    expect(await screen.findByText("Este PDF não tem texto selecionável (provavelmente digitalizado). Use Abrir para ver as páginas.")).toBeInTheDocument();
  });

  it("classifies diff lines", () => {
    expect(diffLines("--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new\n same").map((l) => l.kind)).toEqual(["file", "file", "hunk", "del", "add", "ctx"]);
    expect(sandboxedHtml("<p>x</p>")).toContain("<body><p>x</p></body>");
  });
});
