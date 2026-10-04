// Runs on Windows only: `cargo install tauri-driver`, Microsoft Edge WebDriver
// matching the installed WebView2 (msedgedriver.exe on PATH), then:
//   pnpm -C apps/desktop tauri build --no-bundle --features demo
//   pnpm -C apps/desktop/e2e install && pnpm -C apps/desktop/e2e test
import { spawn, type ChildProcess } from "node:child_process";
import path from "node:path";
import { createServer, type Server } from "node:http";

// AURA_E2E_APP: another build (e.g. while target/release/aura.exe is in use).
const app = process.env.AURA_E2E_APP ?? path.resolve(import.meta.dirname, "../../../target/release/aura.exe");
let driver: ChildProcess | undefined;
let authorization: Server | undefined;

export const config: WebdriverIO.Config = {
  runner: "local",
  specs: ["./specs/**/*.e2e.ts"],
  maxInstances: 1,
  hostname: "127.0.0.1",
  port: 4444,
  capabilities: [{ "tauri:options": { application: app } } as WebdriverIO.Capabilities],
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: { timeout: 60_000 },
  beforeSession: async () => {
    if (process.env.AURA_E2E_LOCAL_AUTH === "1") {
      authorization = createServer((_req, res) => {
        // Do not log the OAuth query, and never complete a callback or request tokens.
        res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
        res.end("<title>Aura QA authorization</title><p>Local cancellation test. No account or identity requested.</p>");
      });
      await new Promise<void>((resolve) => authorization!.listen(0, "127.0.0.1", resolve));
      const address = authorization.address();
      if (!address || typeof address === "string") throw new Error("QA authorization has no loopback port");
      process.env.AURA_E2E_AUTH_PORT = String(address.port);
    }
    driver = spawn("tauri-driver", [], { stdio: [null, process.stdout, process.stderr] });
  },
  // A fresh profile opens on the login card, and demo mode has no fake ChatGPT
  // sign-in. Through the app's own IPC: a keyless provider (so nothing goes to the
  // Credential Manager) counts as signed in; `onboarded` skips the first-run guide.
  before: async () => {
    await browser.waitUntil(() => browser.execute(() => "__TAURI_INTERNALS__" in window), { timeout: 20_000 });
    await browser.execute(async () => {
      const invoke = (window as any).__TAURI_INTERNALS__.invoke;
      const existing: unknown[] = await invoke("providers_list");
      if (existing.length === 0) {
        await invoke("providers_save", { draft: { name: "Ollama (E2E)", preset: "ollama" }, credential: null });
      }
      await invoke("settings_update", { patch: { onboarded: true } });
    });
    await browser.refresh();
  },
  afterSession: () => {
    driver?.kill();
    authorization?.close();
    delete process.env.AURA_E2E_AUTH_PORT;
  },
};
