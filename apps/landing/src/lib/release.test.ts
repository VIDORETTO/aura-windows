// node --test (Node ≥ 22.18 strips the types). Oracle: the published v0.3.1 release.
import { test } from "node:test";
import assert from "node:assert/strict";
import { megabytes, newer, parseRelease } from "./release.ts";

const api = {
  tag_name: "v0.3.1",
  html_url: "https://github.com/VIDORETTO/aura-windows/releases/tag/v0.3.1",
  published_at: "2026-10-13T20:00:00Z",
  assets: [
    { name: "Aura_0.3.1_x64-setup.exe.sig", browser_download_url: "https://x/sig", size: 420, digest: "sha256:5c7e" },
    { name: "Aura_0.3.1_x64-setup.exe", browser_download_url: "https://x/setup.exe", size: 31_457_280, digest: "sha256:def9e599324a9e0221dad7b75960ed302dcc840984fc2ee5ccb506efb2438197" },
    { name: "Aura_0.3.1_x64_en-US.msi", browser_download_url: "https://x/aura.msi", size: 33_000_000 },
    { name: "latest.json", browser_download_url: "https://x/latest.json", size: 900 },
  ],
};

test("picks the NSIS installer, not its signature, and keeps the API digest", () => {
  const r = parseRelease(api);
  assert.equal(r?.version, "0.3.1");
  assert.deepEqual(r?.setup, { name: "Aura_0.3.1_x64-setup.exe", url: "https://x/setup.exe", bytes: 31_457_280, sha256: "def9e599324a9e0221dad7b75960ed302dcc840984fc2ee5ccb506efb2438197" });
  assert.equal(r?.msi?.name, "Aura_0.3.1_x64_en-US.msi");
  assert.equal(r?.msi?.sha256, null);
});

test("rejects drafts and payloads that are not releases", () => {
  assert.equal(parseRelease({ ...api, draft: true }), null);
  assert.equal(parseRelease({ message: "API rate limit exceeded" }), null);
  assert.equal(parseRelease(null), null);
});

test("a release without the installer still parses, with no setup asset", () => {
  assert.equal(parseRelease({ ...api, assets: [] })?.setup, null);
});

test("compares versions numerically", () => {
  assert.equal(newer("0.10.0", "0.9.2"), true);
  assert.equal(newer("0.3.1", "0.3.1"), false);
  assert.equal(newer("0.3.0", "0.3.1"), false);
  assert.equal(newer("0.4.0", "0.4.0-beta.1"), true);
  assert.equal(newer("0.4.0-beta.1", "0.3.9"), true);
});

test("formats sizes in Brazilian megabytes", () => {
  assert.equal(megabytes(31_457_280), "30,0");
});
