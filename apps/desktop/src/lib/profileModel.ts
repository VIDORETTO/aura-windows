// An app profile's default model (QA-036): `aura-<provider>::<model>` for a
// BYOK provider model; a bare id is a ChatGPT-plan model (older profiles).
export const PLAN_PROVIDER = "aura-chatgpt-plan";
const SEP = "::";

export function encodeProfileModel(provider: string, model: string): string {
  return provider === PLAN_PROVIDER ? model : `${provider}${SEP}${model}`;
}

export function decodeProfileModel(value: string): { provider: string; model: string } {
  const i = value.indexOf(SEP);
  return i > 0 && value.startsWith("aura-") ? { provider: value.slice(0, i), model: value.slice(i + SEP.length) } : { provider: PLAN_PROVIDER, model: value };
}
