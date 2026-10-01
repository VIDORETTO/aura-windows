# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`, `AC-005`
- Procedure: `cargo test -p aura-gateway (fixtures SSE escritas à mão) -p aura-app (providers_privacy_and_diagnostics)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:042b322d522a04bf5a993707702ff9098871ab28fab096ba9fb7e60921355481`
- Timestamp: `2026-09-30T02:11:38+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-002 — partial

- Ticket: `TK-002`
- Acceptance: `AC-006`
- Procedure: `cargo test -p aura-gateway (fixtures SSE escritas à mão) -p aura-app (providers_privacy_and_diagnostics)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:042b322d522a04bf5a993707702ff9098871ab28fab096ba9fb7e60921355481`
- Timestamp: `2026-09-30T02:11:39+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-003 — partial

- Ticket: `TK-003`
- Acceptance: `AC-007`, `AC-011`
- Procedure: `cargo test -p aura-gateway (fixtures SSE escritas à mão) -p aura-app (providers_privacy_and_diagnostics)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:042b322d522a04bf5a993707702ff9098871ab28fab096ba9fb7e60921355481`
- Timestamp: `2026-09-30T02:11:39+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-004 — partial

- Ticket: `TK-004`
- Acceptance: `AC-008`
- Procedure: `cargo test -p aura-gateway (fixtures SSE escritas à mão) -p aura-app (providers_privacy_and_diagnostics)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:042b322d522a04bf5a993707702ff9098871ab28fab096ba9fb7e60921355481`
- Timestamp: `2026-09-30T02:11:39+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-005 — partial

- Ticket: `TK-005`
- Acceptance: `AC-009`
- Procedure: `cargo test -p aura-gateway (fixtures SSE escritas à mão) -p aura-app (providers_privacy_and_diagnostics)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:042b322d522a04bf5a993707702ff9098871ab28fab096ba9fb7e60921355481`
- Timestamp: `2026-09-30T02:11:40+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-006 — partial

- Ticket: `TK-006`
- Acceptance: `AC-010`
- Procedure: `cargo test -p aura-gateway (fixtures SSE escritas à mão) -p aura-app (providers_privacy_and_diagnostics)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:042b322d522a04bf5a993707702ff9098871ab28fab096ba9fb7e60921355481`
- Timestamp: `2026-09-30T02:11:40+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-007 — partial

- Ticket: `TK-007`
- Acceptance: `AC-012`, `AC-013`, `AC-014`
- Procedure: `cargo test -p aura-gateway (fixtures SSE escritas à mão) -p aura-app (providers_privacy_and_diagnostics)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:042b322d522a04bf5a993707702ff9098871ab28fab096ba9fb7e60921355481`
- Timestamp: `2026-09-30T02:11:41+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-008 — partial

- Ticket: `TK-004`
- Acceptance: `AC-008`
- Procedure: `real_app_server: tool call Chat Completions → namespace mcp__aura → Codex executa; cargo test -p aura-gateway (namespace_tests, emitter)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:be679aed009d4f5a86e421ec10da4efd8ea8a3044b2961bb5ac37960a12efa12`
- Timestamp: `2026-09-30T03:13:11+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-009 — partial

- Ticket: `TK-001`
- Acceptance: `AC-004`, `AC-005`
- Procedure: `providers_save (preset openai) com chave de 3006 bytes → cmdkey /list → providers_remove → cmdkey /list; Select-String da chave em todo o AURA_HOME`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; AMD GPU (MFTs AMDh264Encoder, Microsoft AVC DX12, H264 Encoder MFT); WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release modo real; AURA_HOME isolado
- Tested revision: `local:545bfd22459a8f835499d26f0deb3a8acf641a54ca8d558d00959d78f21eaa38`
- Timestamp: `2026-10-01T00:31:39+00:00`
- Observations: Credential Manager: chave fragmentada em 3 credenciais (Aura/provider/<id>, ~0, ~1); remover apaga as 3. Varredura de 95 arquivos do AURA_HOME (db, wal, logs, codex-home): 0 ocorrências da chave.
- Evidence refs: none
- Limitations: Leitura de volta da chave pelo gateway não exercitada (exigiria rede com chave falsa); varredura feita com AURA_HOME isolado, não em %LOCALAPPDATA%\Aura.
