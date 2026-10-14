# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `pnpm -C apps/desktop exec vitest run src/settings/SettingsApp.test.tsx; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck`
- Execution: `executed`
- Environment: Windows 11 Pro 26200; Node 25.2.1; Vitest/jsdom
- Tested revision: `local:0d248c473363e59d0c12b557e1726855f99fab246d34b2385f139b77e1b26c36`
- Timestamp: `2026-10-14T02:50:19+00:00`
- Observations: Red: amostrador Padrão era rgb(91, 91, 247). Green: teste 043 passa; UI 254/254; tsc sem erros.
- Evidence refs: none
- Limitations: E2E WebDriver accent-color não executado (sem msedgedriver).

## EV-002 — passed

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `pnpm -C apps/desktop tauri icon ../../docs/assets/logo.svg -o <tmp>; copiar arquivos versionados; grep -i 5B5BF7/A24BF2 nos logos`
- Execution: `executed`
- Environment: Windows 11 Pro 26200; Tauri CLI 2
- Tested revision: `local:79829ac29f8960500bd81a23de7c8d7fc3d18a904cbc31671498589beae54005`
- Timestamp: `2026-10-14T02:50:19+00:00`
- Observations: Logos sem cores roxas; 15 ícones regenerados; logo.png 512x512 conferido visualmente.
- Evidence refs: none
- Limitations: none recorded

## EV-003 — passed

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `prepare-sidecars.ps1 -Release -DirectML; tauri build --ci --bundles nsis,msi com a chave permanente; updater-verify (permanente aceita, aposentada e adulterado rejeitados); ProductVersion; cargo test --workspace --exclude aura-desktop; pnpm -C apps/desktop test`
- Execution: `executed`
- Environment: Windows 11 Pro 26200; Rust 1.98.1; Tauri CLI 2; Node 25.2.1
- Tested revision: `local:3d556511637243e78b744de40de0563bad614aa82ac5143bac3e141de4eac7df`
- Timestamp: `2026-10-14T10:23:01+00:00`
- Observations: Build 0.3.2 exit 0; 2 assinaturas PASS; pública embutida = permanente; ProductVersion 0.3.2; Rust 519 passed; UI 254/254.
- Evidence refs: none
- Limitations: Instalação em Windows limpo não executada.
