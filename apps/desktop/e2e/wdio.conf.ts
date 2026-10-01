// Runs on Windows only: `cargo install tauri-driver`, Microsoft Edge WebDriver
// matching the installed WebView2 (msedgedriver.exe on PATH), then:
//   pnpm -C apps/desktop tauri build --no-bundle --features demo
//   pnpm -C apps/desktop/e2e install && pnpm -C apps/desktop/e2e test
import { spawn, type ChildProcess } from "node:child_process";
import path from "node:path";

const app = path.resolve(import.meta.dirname, "../../../target/release/aura.exe");
let driver: ChildProcess | undefined;

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
  beforeSession: () => {
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
  afterSession: () => driver?.kill(),
};
