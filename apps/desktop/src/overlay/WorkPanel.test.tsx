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

  it("classifies diff lines", () => {
    expect(diffLines("--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new\n same").map((l) => l.kind)).toEqual(["file", "file", "hunk", "del", "add", "ctx"]);
    expect(sandboxedHtml("<p>x</p>")).toContain("<body><p>x</p></body>");
  });
});
