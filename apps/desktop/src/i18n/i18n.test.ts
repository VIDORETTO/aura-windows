import { en } from "./en";
import { ptBR } from "./pt-BR";
import { translate } from ".";

const placeholders = (s: string) => (s.match(/\{\w+\}/g) ?? []).sort();

describe("i18n", () => {
  it("has the same keys in every language", () => {
    expect(Object.keys(en).sort()).toEqual(Object.keys(ptBR).sort());
  });

  it("keeps placeholders consistent", () => {
    for (const k of Object.keys(ptBR) as (keyof typeof ptBR)[]) {
      expect(placeholders(en[k]), k).toEqual(placeholders(ptBR[k]));
    }
  });

  it("interpolates variables", () => {
    expect(translate("ptBr", "context.tokens", { n: 765 })).toBe("~765 tokens");
    expect(translate("en", "error.usageLimitRetry", { s: 30 })).toBe("Provider limit reached. Try again in 30 s.");
  });
});
