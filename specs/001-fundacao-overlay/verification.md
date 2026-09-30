# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `TK-005`
- Acceptance: `AC-011`, `AC-012`, `AC-013`
- Procedure: `cargo test -p aura-core`
- Execution: `executed`
- Environment: Linux
- Tested revision: `local:9195977538d39bfdda66254d00625c6b474fa18f1d89036518e2da75aa42b046`
- Timestamp: `2026-09-29T23:48:46+00:00`
- Observations: ok
- Evidence refs: none
- Limitations: sem Windows

## EV-002 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `cargo test -p aura-core -p aura-store; cargo check/clippy -p aura-win --target x86_64-pc-windows-msvc; pnpm -C apps/desktop test (Settings)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b5c0eec6ec0672b58a13066a45b9efaa2a44afe05ac1ed96bcdffc498ba05c66`
- Timestamp: `2026-09-30T02:11:31+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-003 — partial

- Ticket: `TK-002`
- Acceptance: `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-core -p aura-store; cargo check/clippy -p aura-win --target x86_64-pc-windows-msvc; pnpm -C apps/desktop test (Settings)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b5c0eec6ec0672b58a13066a45b9efaa2a44afe05ac1ed96bcdffc498ba05c66`
- Timestamp: `2026-09-30T02:11:32+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-004 — partial

- Ticket: `TK-003`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-007`
- Procedure: `cargo test -p aura-core -p aura-store; cargo check/clippy -p aura-win --target x86_64-pc-windows-msvc; pnpm -C apps/desktop test (Settings)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b5c0eec6ec0672b58a13066a45b9efaa2a44afe05ac1ed96bcdffc498ba05c66`
- Timestamp: `2026-09-30T02:11:33+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-005 — partial

- Ticket: `TK-004`
- Acceptance: `AC-010`, `AC-014`, `AC-015`
- Procedure: `cargo test -p aura-core -p aura-store; cargo check/clippy -p aura-win --target x86_64-pc-windows-msvc; pnpm -C apps/desktop test (Settings)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b5c0eec6ec0672b58a13066a45b9efaa2a44afe05ac1ed96bcdffc498ba05c66`
- Timestamp: `2026-09-30T02:11:33+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-006 — partial

- Ticket: `TK-006`
- Acceptance: `AC-016`, `AC-017`
- Procedure: `cargo test -p aura-core -p aura-store; cargo check/clippy -p aura-win --target x86_64-pc-windows-msvc; pnpm -C apps/desktop test (Settings)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:b5c0eec6ec0672b58a13066a45b9efaa2a44afe05ac1ed96bcdffc498ba05c66`
- Timestamp: `2026-09-30T02:11:34+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.
