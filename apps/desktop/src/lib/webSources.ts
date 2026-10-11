import type { WebSource } from "../ipc/types";

/** Click destinations remain HTTP(S), without credentials or alternate ports. */
export function webSourceUrl(source: WebSource): string | null {
  if (!/^W[1-9]\d*$/.test(source.sourceId) || source.url.length > 2048
    || /[\x00-\x20\\]/.test(source.url)) return null;
  try {
    const url = new URL(source.url);
    const authority = source.url.split("//")[1]?.split(/[/?#]/)[0] ?? "";
    if ((url.protocol !== "https:" && url.protocol !== "http:")
      || url.username || url.password || authority.includes("@")
      || (url.port && url.port !== (url.protocol === "https:" ? "443" : "80"))) return null;
    return url.href;
  } catch { return null; }
}

export function answerSources(text: string, consulted: readonly WebSource[], known: readonly WebSource[]): WebSource[] {
  const sources = new Map(consulted.map((source) => [source.sourceId, source]));
  for (const match of text.matchAll(/\[\[aura-source:(W[1-9]\d*)\]\]/g)) {
    const source = known.find((entry) => entry.sourceId === match[1]);
    if (source) sources.set(source.sourceId, source);
  }
  return [...sources.values()].filter((source) => webSourceUrl(source) !== null);
}
