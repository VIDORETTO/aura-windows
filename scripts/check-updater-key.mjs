// Release guard (QA-038): refuses to build a release whose updater public key
// is still the template placeholder, or without the signing private key.
// Usage: node scripts/check-updater-key.mjs [path/to/tauri.conf.json]
import { readFileSync } from "node:fs";

const file = process.argv[2] ?? "apps/desktop/src-tauri/tauri.conf.json";
const key = JSON.parse(readFileSync(file, "utf8"))?.plugins?.updater?.pubkey ?? "";
const k = String(key).trim();
const configured = k.length >= 40 && !k.includes("REPLACE_WITH") && /^[A-Za-z0-9+/=]+$/.test(k);
if (!configured) {
  console.error(`${file}: plugins.updater.pubkey is not a real key. Generate the pair with "pnpm tauri signer generate", put the public key in tauri.conf.json and the private key in the TAURI_SIGNING_PRIVATE_KEY secret.`);
  process.exit(1);
}
if (process.env.CI && !process.env.TAURI_SIGNING_PRIVATE_KEY) {
  console.error("TAURI_SIGNING_PRIVATE_KEY is not set: update artifacts would be unsigned.");
  process.exit(1);
}
console.log("updater key configured");
