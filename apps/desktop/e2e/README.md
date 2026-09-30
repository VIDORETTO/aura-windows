# E2E (Windows)

Critical journeys driven through the real app with WebdriverIO and
`tauri-driver`, against the **demo** build (fake agent, synthetic capture) so
no ChatGPT account is needed.

```powershell
cargo install tauri-driver
# msedgedriver.exe matching the installed WebView2 must be on PATH
pnpm -C apps/desktop tauri build --no-bundle --features demo
pnpm -C apps/desktop/e2e install
pnpm -C apps/desktop/e2e test
```

Add a spec per acceptance journey that needs the real window system
(focus, hotkeys, capture exclusion); keep everything else in Vitest.
