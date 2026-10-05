// Automated accessibility audit (010 TK-005): no serious/critical axe
// violations on the Overlay and every Settings page. Color contrast needs a
// real renderer and is covered by the manual Narrator script (docs/qa).
import axe from "axe-core";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "./test/harness";
import { OverlayApp } from "./overlay/OverlayApp";
import { PAGES, SettingsApp } from "./settings/SettingsApp";

async function audit(container: HTMLElement) {
  const r = await axe.run(container, { rules: { "color-contrast": { enabled: false }, region: { enabled: false } } });
  return r.violations.filter((v) => v.impact === "serious" || v.impact === "critical").map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
}

describe("accessibility", () => {
  it("overlay (empty and with a conversation) has no serious violations", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    const { container } = render(<OverlayApp />);
    await screen.findByRole("combobox");
    expect(await audit(container)).toEqual([]);
    await user.type(screen.getByRole("combobox"), "/aprovar{Enter}");
    await screen.findByRole("group", { name: "Executar comando" });
    expect(await audit(container)).toEqual([]);
  });

  it.each(PAGES.map((p) => p.id))("settings page %s has no serious violations", async (id) => {
    await freshApp({ signedIn: true });
    window.location.hash = `#/settings/${id}`;
    const { container } = render(<SettingsApp />);
    await screen.findAllByRole("heading");
    await new Promise((r) => setTimeout(r, 30));
    expect(await audit(container)).toEqual([]);
  });
});
