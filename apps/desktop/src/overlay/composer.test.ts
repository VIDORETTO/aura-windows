import { describe, expect, it } from "vitest";
import { filterMenu, findTrigger, quickHint, replaceTrigger, templatePreview } from "./composer";

const item = (id: string, label: string, aliases: string[] = []) => ({ id, label, hint: "", aliases });

describe("findTrigger (015 AC-001, AC-002)", () => {
  it("finds @ at the caret, in the middle of the text", () => {
    const value = "veja @ agora";
    expect(findTrigger(value, 6)).toEqual({ kind: "@", query: "", start: 5, end: 6 });
  });

  it("reads the query up to the caret", () => {
    expect(findTrigger("fala com o @joao", 16)).toEqual({ kind: "@", query: "joao", start: 11, end: 16 });
  });

  it("ignores @ inside a word (e-mail) and after a space", () => {
    expect(findTrigger("a@b.com", 7)).toBeNull();
    expect(findTrigger("@tela ", 6)).toBeNull();
  });

  it("accepts / only at the start of the message", () => {
    expect(findTrigger("/tl", 3)).toEqual({ kind: "/", query: "tl", start: 0, end: 3 });
    expect(findTrigger("a /tl", 5)).toBeNull();
    expect(findTrigger("/traduzir ", 10)).toBeNull();
  });
});

describe("filterMenu", () => {
  const items = [item("tela", "@tela"), item("regiao", "@região"), item("janela", "@janela")];

  it("matches without accents or case", () => {
    expect(filterMenu(items, "REGIAO").map((i) => i.id)).toEqual(["regiao"]);
  });

  it("puts prefix matches before substring matches", () => {
    expect(filterMenu(items, "ela").map((i) => i.id)).toEqual(["tela", "janela"]);
    expect(filterMenu(items, "j").map((i) => i.id)).toEqual(["janela"]);
  });

  it("returns nothing when no item matches", () => {
    expect(filterMenu(items, "joao")).toEqual([]);
  });

  it("matches aliases", () => {
    expect(filterMenu([item("plan", "/plano", ["plan"])], "pla").map((i) => i.id)).toEqual(["plan"]);
    expect(filterMenu([item("plan", "/plano", ["plan"])], "plan").map((i) => i.id)).toEqual(["plan"]);
  });
});

describe("replaceTrigger", () => {
  it("swaps the trigger word for the text and places the caret after it", () => {
    expect(replaceTrigger("veja @te agora", { kind: "@", query: "te", start: 5, end: 8 }, "")).toEqual(["veja agora", 5]);
    expect(replaceTrigger("/tl", { kind: "/", query: "tl", start: 0, end: 3 }, "/tldr ")).toEqual(["/tldr ", 6]);
  });
});

describe("quickHint (015 AC-005)", () => {
  it("names the argument default and the text source", () => {
    expect(quickHint("Traduza para {args:inglês}:\n\n{selecao}")).toEqual({ arg: "inglês", source: "selection", screen: false });
  });

  it("reports typed text and screen", () => {
    expect(quickHint("{tela}Resuma.\n\n{texto}")).toEqual({ arg: null, source: "typed", screen: true });
    expect(quickHint("Diga oi {args}")).toEqual({ arg: "", source: null, screen: false });
  });
});

describe("templatePreview", () => {
  it("shows the argument default and hides the context placeholders", () => {
    expect(templatePreview("Traduza para {args:inglês}:\n\n{selecao}")).toBe("Traduza para ‹inglês›:");
    expect(templatePreview("{tela}Resuma o que está na tela.\n\n{texto}")).toBe("Resuma o que está na tela.");
    expect(templatePreview("Diga {args} agora")).toBe("Diga ‹…› agora");
  });
});
