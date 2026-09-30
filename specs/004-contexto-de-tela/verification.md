# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-policy -p aura-capture -p aura-mcp -p aura-app (consentimento, exclusões, pausa); cargo check -p aura-win --target x86_64-pc-windows-msvc`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:23266c8d30310bb8a7d9c0db2e66b0e24bb851554f5c1298ab8d8fbc9d564de5`
- Timestamp: `2026-09-30T02:11:41+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-002 — partial

- Ticket: `TK-003`
- Acceptance: `AC-003`, `AC-005`, `AC-006`, `AC-008`
- Procedure: `cargo test -p aura-policy -p aura-capture -p aura-mcp -p aura-app (consentimento, exclusões, pausa); cargo check -p aura-win --target x86_64-pc-windows-msvc`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:23266c8d30310bb8a7d9c0db2e66b0e24bb851554f5c1298ab8d8fbc9d564de5`
- Timestamp: `2026-09-30T02:11:42+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-003 — partial

- Ticket: `TK-004`
- Acceptance: `AC-007`, `AC-010`, `AC-011`
- Procedure: `cargo test -p aura-policy -p aura-capture -p aura-mcp -p aura-app (consentimento, exclusões, pausa); cargo check -p aura-win --target x86_64-pc-windows-msvc`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:23266c8d30310bb8a7d9c0db2e66b0e24bb851554f5c1298ab8d8fbc9d564de5`
- Timestamp: `2026-09-30T02:11:42+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-004 — partial

- Ticket: `TK-005`
- Acceptance: `AC-009`, `AC-013`
- Procedure: `cargo test -p aura-policy -p aura-capture -p aura-mcp -p aura-app (consentimento, exclusões, pausa); cargo check -p aura-win --target x86_64-pc-windows-msvc`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:23266c8d30310bb8a7d9c0db2e66b0e24bb851554f5c1298ab8d8fbc9d564de5`
- Timestamp: `2026-09-30T02:11:43+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-005 — partial

- Ticket: `TK-006`
- Acceptance: `AC-015`, `AC-016`
- Procedure: `cargo test -p aura-policy -p aura-capture -p aura-mcp -p aura-app (consentimento, exclusões, pausa); cargo check -p aura-win --target x86_64-pc-windows-msvc`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:23266c8d30310bb8a7d9c0db2e66b0e24bb851554f5c1298ab8d8fbc9d564de5`
- Timestamp: `2026-09-30T02:11:43+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-006 — partial

- Ticket: `TK-007`
- Acceptance: `AC-012`, `AC-014`
- Procedure: `cargo test -p aura-policy -p aura-capture -p aura-mcp -p aura-app (consentimento, exclusões, pausa); cargo check -p aura-win --target x86_64-pc-windows-msvc`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:23266c8d30310bb8a7d9c0db2e66b0e24bb851554f5c1298ab8d8fbc9d564de5`
- Timestamp: `2026-09-30T02:11:43+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-007 — partial

- Ticket: `TK-002`
- Acceptance: `AC-004`
- Procedure: `cargo test -p aura-app region_selection_crops_the_frozen_screen; vitest RegionSelector.test.ts`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:b3f376764925568fdd7a4fdac0737aa1ddde9184b2a4da7c1e47e8ae75c77791`
- Timestamp: `2026-09-30T03:13:11+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-008 — partial

- Ticket: `TK-004`
- Acceptance: `AC-007`, `AC-010`, `AC-011`
- Procedure: `real_app_server: Codex real lista e chama aura.active_window_info via /mcp com token`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:80eba4a142d1b258c167e5fea765d588112b1162780fde4f9dd855402f1f71af`
- Timestamp: `2026-09-30T03:13:11+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-009 — partial

- Ticket: `TK-005`
- Acceptance: `AC-009`, `AC-013`
- Procedure: `cargo test -p aura-app recent_buffers_feed_the_agent_tools (segmentos cifrados, keyframes)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:689adc1a0853c795b3383ac6d92985a6ce695e7f15ca728c520d1f98812a8223`
- Timestamp: `2026-09-30T03:13:12+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-010 — partial

- Ticket: `TK-006`
- Acceptance: `AC-015`, `AC-016`
- Procedure: `cargo test -p aura-app manual_recording_lifecycle; vitest Settings (gravação manual)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:738c567b6668323bc96edc651faa1254d382da02bfbc6d4699830b592f1ec31c`
- Timestamp: `2026-09-30T03:13:12+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-011 — partial

- Ticket: `TK-007`
- Acceptance: `AC-012`, `AC-014`
- Procedure: `cargo test -p aura-app recent_buffers_feed_the_agent_tools (screen_frames rotulados)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:689adc1a0853c795b3383ac6d92985a6ce695e7f15ca728c520d1f98812a8223`
- Timestamp: `2026-09-30T03:13:12+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.
