import { freshApp } from "../test/harness";
import { useApp } from "./app";

describe("notices", () => {
  it("the same message is shown once", async () => {
    await freshApp({ signedIn: true });
    const msg = "O atalho para abrir o Aura está em uso por outro app.";
    useApp.getState().notify("warning", msg);
    useApp.getState().notify("warning", msg);
    useApp.getState().notify("warning", "outro aviso");
    expect(useApp.getState().notices.map((n) => n.message)).toEqual([msg, "outro aviso"]);
  });
});
