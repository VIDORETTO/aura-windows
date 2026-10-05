import { suggestionsFor } from "./suggestions";

describe("suggestions by app (028)", () => {
  const ids = (p: string, t = "") => suggestionsFor(p, t).map((s) => s.id);

  it("meeting apps and meeting pages offer to prepare a meeting first", () => {
    expect(ids("ms-teams.exe")[0]).toBe("meeting");
    expect(ids("Zoom.exe")[0]).toBe("meeting");
    expect(ids("chrome.exe", "Reunião semanal - Google Meet")[0]).toBe("meeting");
  });

  it("browsers, mail, editors and Office get what they usually need", () => {
    expect(ids("chrome.exe", "Notícias")).toEqual(["summarize", "scam", "doc", "help"]);
    expect(ids("OUTLOOK.EXE")[0]).toBe("reply");
    expect(ids("Code.exe")[0]).toBe("error");
    expect(ids("WINWORD.EXE")).toContain("doc");
  });

  it("anything else gets the generic three and never more than four", () => {
    expect(ids("calc.exe")).toEqual(["summarize", "help", "meeting"]);
    expect(ids("")).toEqual(["summarize", "help", "meeting"]);
    for (const p of ["chrome.exe", "code.exe", "outlook.exe", "x.exe"]) expect(suggestionsFor(p, "").length).toBeLessThanOrEqual(4);
  });

  it("commands only reference slash commands that exist", () => {
    for (const p of ["chrome.exe", "outlook.exe", "x.exe"]) {
      for (const s of suggestionsFor(p, "")) {
        if (s.action.kind === "send" && s.action.text.startsWith("/")) expect(["/resumir-tela", "/ajuda", "/responder", "/golpe", "/documento"]).toContain(s.action.text);
      }
    }
  });
});
