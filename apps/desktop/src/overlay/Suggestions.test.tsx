// 028: the empty Overlay suggests what to do for the app in front.
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { OverlayApp } from "./OverlayApp";
import { useSession } from "./session";

describe("empty Overlay suggestions", () => {
  it("shows suggestions and 'Preparar reunião' opens the Meeting panel", async () => {
    await freshApp({ signedIn: true });
    useSession.setState({ overlayMode: "expanded" });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const group = await screen.findByRole("group", { name: "Sugestões para este app" });
    expect(within(group).getAllByRole("button").length).toBeGreaterThan(1);
    await user.click(within(group).getByRole("button", { name: "Preparar reunião" }));
    expect(useSession.getState().meetingOpen).toBe(true);
    expect(await screen.findByText("Nova reunião")).toBeInTheDocument();
  });
});
