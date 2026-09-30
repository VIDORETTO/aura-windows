import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { Minibar } from "./Minibar";
import { blurAction, useSession } from "./session";
import { emptyThread } from "../state/conversation";

describe("minibar", () => {
  it("decides what focus loss does", () => {
    expect(blurAction({ busy: false, keepOpen: false, pendingConsent: false })).toBe("hide");
    expect(blurAction({ busy: true, keepOpen: false, pendingConsent: false })).toBe("minibar");
    expect(blurAction({ busy: true, keepOpen: true, pendingConsent: false })).toBe("stay");
    expect(blurAction({ busy: false, keepOpen: false, pendingConsent: true })).toBe("stay");
  });

  it("shows progress and expands on click", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    useSession.setState({ minibar: true });
    const thread = { ...emptyThread("t"), running: true, blocks: [{ type: "assistant" as const, id: "a", text: "Olá, estou pensando", streaming: true }] };
    render(<Minibar thread={thread} />);
    expect(screen.getByText("Olá, estou pensando")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Aura está respondendo…" }));
    expect(useSession.getState().minibar).toBe(false);
  });
});
