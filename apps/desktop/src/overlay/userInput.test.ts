import { formAnswer, parseInput, questionsAnswer } from "./userInput";

describe("agent requests", () => {
  it("parses Codex questions and builds the answers map", () => {
    const f = parseInput("agent", {
      questions: [{ id: "lang", header: "Idioma", question: "Qual linguagem?", isOther: true, options: [{ label: "Rust" }, { label: "Go", description: "rápido" }] }],
    });
    expect(f.kind).toBe("questions");
    if (f.kind !== "questions") return;
    expect(f.questions[0].options.map((o) => o.label)).toEqual(["Rust", "Go"]);
    expect(questionsAnswer({ lang: " Rust ", empty: "" })).toEqual({ type: "answer", content: { answers: { lang: { answers: ["Rust"] } } } });
  });

  it("maps MCP elicitation schemas to typed fields", () => {
    const f = parseInput("github", {
      message: "Confirme",
      requestedSchema: {
        type: "object",
        properties: { repo: { type: "string", title: "Repositório" }, stars: { type: "integer" }, private: { type: "boolean" }, tier: { type: "string", enum: ["a", "b"] } },
        required: ["repo", "stars"],
      },
    });
    expect(f.kind).toBe("form");
    if (f.kind !== "form") return;
    expect(f.fields.map((x) => x.kind)).toEqual(["string", "integer", "boolean", "enum"]);
    const ok = formAnswer(f.fields, { repo: "aura", stars: "12", private: true, tier: "b" });
    expect(ok.missing).toEqual([]);
    expect(ok.decision).toEqual({ type: "answer", content: { repo: "aura", stars: 12, private: true, tier: "b" } });
    expect(formAnswer(f.fields, { repo: "", stars: "1.5" }).missing).toEqual(["repo", "stars"]);
  });

  it("recognizes permission requests and unknown shapes", () => {
    expect(parseInput("permissions", { reason: "rede", permissions: { network: true } })).toEqual({ kind: "permissions", reason: "rede", permissions: { network: true } });
    expect(parseInput("agent", { foo: 1 }).kind).toBe("unknown");
  });
});
