// Performance budget for the built page (PLANO.md §7). Run after `pnpm build`.
import fs from "node:fs";
import path from "node:path";
import { gzipSync } from "node:zlib";
import { fileURLToPath } from "node:url";

const dist = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../dist");
const html = fs.readFileSync(path.join(dist, "index.html"), "utf8");
const gz = (s) => gzipSync(s).length;
const kb = (n) => `${(n / 1024).toFixed(1)} KB`;

const css = [...html.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]).join("");
const assets = fs.readdirSync(path.join(dist, "_astro"));
const js = assets.filter((f) => f.endsWith(".js")).map((f) => fs.readFileSync(path.join(dist, "_astro", f)));
const inlineJs = [...html.matchAll(/<script(?![^>]*application\/ld\+json)[^>]*>([\s\S]*?)<\/script>/g)].map((m) => m[1]).join("");
const fonts = assets.filter((f) => f.endsWith(".woff2"));

const checks = [
  ["JS (gzip)", gz(Buffer.concat([...js, Buffer.from(inlineJs)])), 12 * 1024],
  ["CSS (gzip)", gz(css), 20 * 1024],
  ["HTML (gzip)", gz(html), 40 * 1024],
];
let failed = false;
for (const [name, size, limit] of checks) {
  const ok = size <= limit;
  failed ||= !ok;
  console.log(`${ok ? "ok  " : "FAIL"} ${name.padEnd(12)} ${kb(size).padStart(9)} / ${kb(limit)}`);
}
const fontsOk = fonts.length <= 4;
failed ||= !fontsOk;
console.log(`${fontsOk ? "ok  " : "FAIL"} ${"WOFF2 files".padEnd(12)} ${String(fonts.length).padStart(9)} / 4`);
process.exit(failed ? 1 : 0);
