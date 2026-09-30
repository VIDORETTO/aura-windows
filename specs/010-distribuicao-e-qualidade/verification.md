# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `cargo test -p aura-bench; cargo run -p aura-bench -- all (cenários portáveis); pnpm -C apps/desktop test (i18n)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b9a2ff29140820c7ba19e0588ae7a12b3d0fe88d7340d815cb9031eb6c2fb50e`
- Timestamp: `2026-09-30T02:11:50+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-002 — partial

- Ticket: `TK-003`
- Acceptance: `AC-006`
- Procedure: `cargo test -p aura-bench; cargo run -p aura-bench -- all (cenários portáveis); pnpm -C apps/desktop test (i18n)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b9a2ff29140820c7ba19e0588ae7a12b3d0fe88d7340d815cb9031eb6c2fb50e`
- Timestamp: `2026-09-30T02:11:50+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-003 — partial

- Ticket: `TK-004`
- Acceptance: `AC-007`, `AC-008`
- Procedure: `cargo test -p aura-bench; cargo run -p aura-bench -- all (cenários portáveis); pnpm -C apps/desktop test (i18n)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b9a2ff29140820c7ba19e0588ae7a12b3d0fe88d7340d815cb9031eb6c2fb50e`
- Timestamp: `2026-09-30T02:11:50+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-004 — partial

- Ticket: `TK-005`
- Acceptance: `AC-009`, `AC-010`
- Procedure: `cargo test -p aura-bench; cargo run -p aura-bench -- all (cenários portáveis); pnpm -C apps/desktop test (i18n)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b9a2ff29140820c7ba19e0588ae7a12b3d0fe88d7340d815cb9031eb6c2fb50e`
- Timestamp: `2026-09-30T02:11:51+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-005 — partial

- Ticket: `TK-006`
- Acceptance: `AC-011`
- Procedure: `cargo test -p aura-bench; cargo run -p aura-bench -- all (cenários portáveis); pnpm -C apps/desktop test (i18n)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b9a2ff29140820c7ba19e0588ae7a12b3d0fe88d7340d815cb9031eb6c2fb50e`
- Timestamp: `2026-09-30T02:11:51+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-006 — partial

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`
- Procedure: `vite build + tsc (lib/updates.ts); tauri.conf updater (não executado)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:da0546cf01507a81e5972ab0d94c94a7aa18c22ed8d046d16e281174b08dda47`
- Timestamp: `2026-09-30T03:13:14+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-007 — partial

- Ticket: `TK-003`
- Acceptance: `AC-006`
- Procedure: `vitest OverlayApp 'first run' (privacidade, atalho, voz)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:a29558c840da4447029b14dddaca10fadcde7df51ff780a5e2082b16446d9692`
- Timestamp: `2026-09-30T03:13:14+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-008 — partial

- Ticket: `TK-005`
- Acceptance: `AC-009`, `AC-010`
- Procedure: `vitest a11y.test.tsx (axe, todas as páginas) + i18n.test.ts`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:c84f4bec9a58492804407f96516eec30f3f8598e8477f4d9a89f735d94d25403`
- Timestamp: `2026-09-30T03:13:15+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-009 — partial

- Ticket: `TK-006`
- Acceptance: `AC-011`
- Procedure: `cargo test -p aura-app diagnostics_package_is_redacted`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:60217b7929f86f3122a34a07690ba3c27fa2966d3813b2133b0f4a91a5572218`
- Timestamp: `2026-09-30T03:13:15+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.
