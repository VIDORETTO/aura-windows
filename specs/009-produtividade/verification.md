# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-app (seleção, inserir no app com plataforma falsa); cargo check -p aura-win --target x86_64-pc-windows-msvc (UIA, clipboard)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:47e321d46aa7394250dc4bb715478541c2c05ada135eb9ea87816791feafcd57`
- Timestamp: `2026-09-30T02:11:49+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-002 — partial

- Ticket: `TK-005`
- Acceptance: `AC-009`, `AC-010`
- Procedure: `cargo test -p aura-app (seleção, inserir no app com plataforma falsa); cargo check -p aura-win --target x86_64-pc-windows-msvc (UIA, clipboard)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:47e321d46aa7394250dc4bb715478541c2c05ada135eb9ea87816791feafcd57`
- Timestamp: `2026-09-30T02:11:50+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-003 — partial

- Ticket: `TK-002`
- Acceptance: `AC-003`, `AC-004`
- Procedure: `vitest Minibar.test.tsx (blurAction, expandir)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:2093398f6f5db189949ea6d0cfe2ec46dc5cafd9b811303f817ed46238bbc4a4`
- Timestamp: `2026-09-30T03:13:14+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-004 — partial

- Ticket: `TK-003`
- Acceptance: `AC-005`, `AC-006`
- Procedure: `cargo test -p aura-app speak_returns_playable_audio, speech::tests`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:d3ac5bdf1c9a9de311ed971ca4ddc769cb0342fe2e9bac5a10e1202d34fcd77b`
- Timestamp: `2026-09-30T03:13:14+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-005 — partial

- Ticket: `TK-004`
- Acceptance: `AC-007`, `AC-008`
- Procedure: `cargo test -p aura-app app_profile_applies_to_new_conversations, profiles::tests; vitest perfis`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:e919921d5354276e8fa7166bd6477bcc9a4715353a631bec56758df28eb0da4a`
- Timestamp: `2026-09-30T03:13:14+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.
