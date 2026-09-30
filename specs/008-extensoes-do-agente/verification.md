# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`
- Procedure: `cargo test -p aura-extensions -p aura-app (skills_and_mcp_servers)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:e27c3538f1b00abc9e91e99998c8dc718f1abd6580780abc8fb80762701f6f26`
- Timestamp: `2026-09-30T02:11:48+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-002 — partial

- Ticket: `TK-002`
- Acceptance: `AC-005`, `AC-006`, `AC-007`, `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-extensions -p aura-app (skills_and_mcp_servers)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:e27c3538f1b00abc9e91e99998c8dc718f1abd6580780abc8fb80762701f6f26`
- Timestamp: `2026-09-30T02:11:48+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-003 — partial

- Ticket: `TK-003`
- Acceptance: `AC-010`, `AC-011`
- Procedure: `cargo test -p aura-extensions -p aura-app (skills_and_mcp_servers)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:e27c3538f1b00abc9e91e99998c8dc718f1abd6580780abc8fb80762701f6f26`
- Timestamp: `2026-09-30T02:11:48+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-004 — partial

- Ticket: `TK-004`
- Acceptance: `AC-012`, `AC-013`
- Procedure: `cargo test -p aura-extensions -p aura-app (skills_and_mcp_servers)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:e27c3538f1b00abc9e91e99998c8dc718f1abd6580780abc8fb80762701f6f26`
- Timestamp: `2026-09-30T02:11:49+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-005 — partial

- Ticket: `TK-005`
- Acceptance: `AC-014`
- Procedure: `cargo test -p aura-extensions -p aura-app (skills_and_mcp_servers)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:e27c3538f1b00abc9e91e99998c8dc718f1abd6580780abc8fb80762701f6f26`
- Timestamp: `2026-09-30T02:11:49+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-006 — partial

- Ticket: `TK-006`
- Acceptance: `AC-015`, `AC-016`
- Procedure: `cargo test -p aura-extensions -p aura-app (skills_and_mcp_servers)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:e27c3538f1b00abc9e91e99998c8dc718f1abd6580780abc8fb80762701f6f26`
- Timestamp: `2026-09-30T02:11:49+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-007 — partial

- Ticket: `TK-002`
- Acceptance: `AC-005`, `AC-006`, `AC-007`, `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-app mcp_status_and_workspace_files; vitest Extensions`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:42479c167ebeddc2bac0783db942fc2bd9ff6e16d5745960f4b605e0565c6b14`
- Timestamp: `2026-09-30T03:13:13+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-008 — partial

- Ticket: `TK-005`
- Acceptance: `AC-014`
- Procedure: `vitest WorkPanel.test.tsx (prévia HTML isolada, diff); cargo test -p aura-app (read_workspace_file bloqueia ..)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:e667d3c85f6308749c68ae654c866c9ddd1cf4b53482c5104f50e8d001a89320`
- Timestamp: `2026-09-30T03:13:13+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-009 — partial

- Ticket: `TK-006`
- Acceptance: `AC-015`, `AC-016`
- Procedure: `cargo test -p aura-core; vitest (toggle Memórias)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:1baebc62e69f2f6ec4edf130e54358046cda90b5a503d133c15e288dc73589e8`
- Timestamp: `2026-09-30T03:13:13+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.
