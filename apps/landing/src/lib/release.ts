// Latest Aura release from the GitHub API (PLANO.md §5). Read at build time; the page
// refreshes it in the browser (src/scripts/release-client.ts) with the same parser.

export const REPO = "VIDORETTO/aura-windows";
export const REPO_URL = `https://github.com/${REPO}`;
export const LATEST_URL = `${REPO_URL}/releases/latest`;
export const API_LATEST = `https://api.github.com/repos/${REPO}/releases/latest`;

export interface Asset {
  name: string;
  url: string;
  bytes: number;
  sha256: string | null;
}

export interface Release {
  version: string;
  tag: string;
  publishedAt: string;
  notesUrl: string;
  setup: Asset | null;
  msi: Asset | null;
}

interface ApiAsset {
  name: string;
  browser_download_url: string;
  size: number;
  digest?: string | null;
}
interface ApiRelease {
  tag_name: string;
  html_url: string;
  published_at: string;
  draft?: boolean;
  assets: ApiAsset[];
}

const asset = (assets: ApiAsset[], pattern: RegExp): Asset | null => {
  const a = assets.find((x) => pattern.test(x.name));
  if (!a) return null;
  const digest = a.digest?.startsWith("sha256:") ? a.digest.slice(7) : null;
  return { name: a.name, url: a.browser_download_url, bytes: a.size, sha256: digest };
};

/** Turns the API payload into what the page shows; null when it is not a usable release. */
export function parseRelease(json: unknown): Release | null {
  const r = json as ApiRelease;
  if (!r || typeof r.tag_name !== "string" || !Array.isArray(r.assets) || r.draft) return null;
  return {
    version: r.tag_name.replace(/^v/, ""),
    tag: r.tag_name,
    publishedAt: r.published_at,
    notesUrl: r.html_url,
    setup: asset(r.assets, /^Aura_.*_x64-setup\.exe$/),
    msi: asset(r.assets, /^Aura_.*_x64.*\.msi$/),
  };
}

/** Fills missing hashes from the `sha256.txt` asset published with each release. */
async function withHashes(release: Release, assets: ApiAsset[]): Promise<Release> {
  if (release.setup?.sha256 && (!release.msi || release.msi.sha256)) return release;
  const sums = assets.find((a) => a.name === "sha256.txt");
  if (!sums) return release;
  try {
    const text = await (await fetch(sums.browser_download_url)).text();
    const map = new Map(text.split(/\r?\n/).map((l) => l.trim().split(/\s+/)).filter((p) => p.length === 2).map(([h, n]) => [n, h]));
    const fill = (a: Asset | null) => (a && !a.sha256 ? { ...a, sha256: map.get(a.name) ?? null } : a);
    return { ...release, setup: fill(release.setup), msi: fill(release.msi) };
  } catch {
    return release;
  }
}

/** Build-time fetch. Never fails the build: without the API the page links to /releases/latest. */
export async function latestRelease(): Promise<Release | null> {
  const headers: Record<string, string> = { Accept: "application/vnd.github+json", "User-Agent": "aura-landing" };
  const token = process.env.GITHUB_TOKEN;
  if (token) headers.Authorization = `Bearer ${token}`;
  try {
    const res = await fetch(API_LATEST, { headers });
    if (!res.ok) throw new Error(`GitHub API ${res.status}`);
    const json = (await res.json()) as ApiRelease;
    const release = parseRelease(json);
    return release && (await withHashes(release, json.assets));
  } catch (error) {
    console.warn(`[release] ${String(error)}; using ${LATEST_URL}`);
    return null;
  }
}

export const megabytes = (bytes: number) => (bytes / 1024 / 1024).toFixed(bytes > 100 * 1024 * 1024 ? 0 : 1).replace(".", ",");

export const releaseDate = (iso: string) =>
  new Intl.DateTimeFormat("pt-BR", { day: "numeric", month: "long", year: "numeric", timeZone: "America/Sao_Paulo" }).format(new Date(iso));

/** Compares dotted versions ("0.10.0" > "0.9.2"); pre-release suffixes sort before the release. */
export function newer(a: string, b: string): boolean {
  const parse = (v: string) => {
    const [core, pre] = v.split("-", 2);
    return { nums: core.split(".").map((n) => Number.parseInt(n, 10) || 0), pre: pre ?? null };
  };
  const x = parse(a);
  const y = parse(b);
  for (let i = 0; i < Math.max(x.nums.length, y.nums.length); i++) {
    const d = (x.nums[i] ?? 0) - (y.nums[i] ?? 0);
    if (d !== 0) return d > 0;
  }
  if (x.pre === y.pre) return false;
  if (x.pre === null) return true;
  if (y.pre === null) return false;
  return x.pre > y.pre;
}

/** Repository stars for the open-source strip (build time only; null without the API). */
export async function repoStars(): Promise<number | null> {
  const headers: Record<string, string> = { Accept: "application/vnd.github+json", "User-Agent": "aura-landing" };
  if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  try {
    const res = await fetch(`https://api.github.com/repos/${REPO}`, { headers });
    if (!res.ok) return null;
    const json = (await res.json()) as { stargazers_count?: number };
    return typeof json.stargazers_count === "number" ? json.stargazers_count : null;
  } catch {
    return null;
  }
}
