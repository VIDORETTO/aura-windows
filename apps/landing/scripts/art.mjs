// Brand art generated from code: favicons and the Open Graph image (IMG-04/05/06 in IMAGENS.md).
// Run after `pnpm capture`: `pnpm -C apps/landing art`.
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import sharp from "sharp";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "..");
const repo = path.resolve(root, "../..");
const pub = path.join(root, "public");
const logo = await fs.readFile(path.join(repo, "docs/assets/logo.svg"), "utf8");

await fs.mkdir(pub, { recursive: true });
await fs.writeFile(path.join(pub, "favicon.svg"), logo);
await fs.copyFile(path.join(repo, "apps/desktop/src-tauri/icons/icon.ico"), path.join(pub, "favicon.ico"));
await sharp(Buffer.from(logo)).resize(512, 512).png().toFile(path.join(pub, "icon-512.png"));
// Apple touch icon: no transparency, page background around the mark.
const mark = await sharp(Buffer.from(logo)).resize(148, 148).png().toBuffer();
await sharp({ create: { width: 180, height: 180, channels: 4, background: "#0A0909" } })
  .composite([{ input: mark, top: 16, left: 16 }])
  .png()
  .toFile(path.join(pub, "apple-touch-icon.png"));

// Open Graph 1200x630: rendered from HTML with the real conversation capture.
const shot = (await fs.readFile(path.join(root, "src/assets/screens/conversa.png"))).toString("base64");
const font = async (f) => `data:font/woff2;base64,${(await fs.readFile(path.join(root, "node_modules/@fontsource", f))).toString("base64")}`;
const sans = await font("geist-sans/files/geist-sans-latin-600-normal.woff2");
const mono = await font("geist-mono/files/geist-mono-latin-500-normal.woff2");
const html = `<!doctype html><html><head><meta charset="utf-8"><style>
@font-face{font-family:Geist;font-weight:600;src:url("${sans}") format("woff2")}
@font-face{font-family:"Geist Mono";font-weight:500;src:url("${mono}") format("woff2")}
*{margin:0;box-sizing:border-box}
body{width:1200px;height:630px;overflow:hidden;background:#0A0909;font-family:Geist;color:#F6F3F2;position:relative}
.halo{position:absolute;right:-120px;top:-60px;width:820px;height:820px;border-radius:50%;background:radial-gradient(circle,rgba(240,64,68,.45),rgba(168,27,38,.18) 45%,transparent 70%)}
.ring{position:absolute;border:1px solid rgba(255,255,255,.08);border-radius:50%}
.logo{position:absolute;left:64px;top:64px;display:flex;align-items:center;gap:16px;font-size:30px}
.logo svg{width:56px;height:56px}
h1{position:absolute;left:64px;top:190px;width:600px;font-size:64px;line-height:1.02;letter-spacing:-.035em}
h1 em{font-style:normal;background:linear-gradient(90deg,#FF8A8A,#F04044);-webkit-background-clip:text;color:transparent}
.keys{position:absolute;left:64px;bottom:72px;display:flex;gap:10px}
.keys span{font-family:"Geist Mono";font-size:20px;padding:10px 16px;border-radius:10px;background:#181515;border:1px solid rgba(255,255,255,.12);box-shadow:inset 0 -3px 0 rgba(0,0,0,.5)}
.shot{position:absolute;left:700px;top:110px;width:620px;border-radius:14px;border:1px solid rgba(255,255,255,.1);transform:rotate(-4deg);box-shadow:0 30px 80px rgba(0,0,0,.6),0 0 90px rgba(240,64,68,.25)}
</style></head><body>
<div class="halo"></div>
<div class="ring" style="right:40px;top:40px;width:540px;height:540px"></div>
<div class="ring" style="right:-60px;top:-60px;width:740px;height:740px"></div>
<div class="logo">${logo.replace(/<\?xml[^>]*>/, "")}<span>Aura</span></div>
<h1>Sua IA no Windows, a <em>um atalho</em> de distância.</h1>
<div class="keys"><span>Ctrl</span><span>Shift</span><span>Espaço</span></div>
<img class="shot" src="data:image/png;base64,${shot}">
</body></html>`;
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1200, height: 630 }, deviceScaleFactor: 1 });
await page.setContent(html, { waitUntil: "load" });
await page.evaluate(() => document.fonts.ready);
const og = await page.screenshot({ type: "png" });
await browser.close();
await sharp(og).png({ compressionLevel: 9, palette: false }).toFile(path.join(pub, "og.png"));
console.log("art ready in public/");
