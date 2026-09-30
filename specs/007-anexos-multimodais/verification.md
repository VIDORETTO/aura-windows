# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`
- Procedure: `cargo test -p aura-ingest -p aura-app (attachments_quick_commands_and_selection)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:2008eee205e600201a57a29c7c3a9488b399ac4eb4f17bfbc16ca8a31c7a9da6`
- Timestamp: `2026-09-30T02:11:47+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-002 — partial

- Ticket: `TK-002`
- Acceptance: `AC-005`
- Procedure: `cargo test -p aura-ingest -p aura-app (attachments_quick_commands_and_selection)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:2008eee205e600201a57a29c7c3a9488b399ac4eb4f17bfbc16ca8a31c7a9da6`
- Timestamp: `2026-09-30T02:11:47+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-003 — partial

- Ticket: `TK-003`
- Acceptance: `AC-006`
- Procedure: `cargo test -p aura-ingest -p aura-app (attachments_quick_commands_and_selection)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:2008eee205e600201a57a29c7c3a9488b399ac4eb4f17bfbc16ca8a31c7a9da6`
- Timestamp: `2026-09-30T02:11:47+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-004 — partial

- Ticket: `TK-006`
- Acceptance: `AC-009`, `AC-010`
- Procedure: `cargo test -p aura-ingest -p aura-app (attachments_quick_commands_and_selection)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:2008eee205e600201a57a29c7c3a9488b399ac4eb4f17bfbc16ca8a31c7a9da6`
- Timestamp: `2026-09-30T02:11:48+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-005 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`
- Procedure: `cargo test -p aura-ingest (pdf::tests)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:f918c03697dcb59d81b55e2301a486641696d325a34a958ca6dfc9660f78ff43`
- Timestamp: `2026-09-30T03:13:13+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-006 — partial

- Ticket: `TK-004`
- Acceptance: `AC-007`
- Procedure: `cargo test -p aura-app wav_attachment_is_transcribed_and_readable_by_time, video_and_compressed_audio_attachments`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:2d9425f735934c2f609a466182e2bc4eff5e7cf4c7bb568cd64aa5d8802af462`
- Timestamp: `2026-09-30T03:13:13+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-007 — partial

- Ticket: `TK-005`
- Acceptance: `AC-008`
- Procedure: `cargo test -p aura-app video_and_compressed_audio_attachments; clippy aura-win media.rs (MSVC)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:116007f96ee6e4a5382a11e2a13836239154ebd56868d3e7bb856d2a7cb49054`
- Timestamp: `2026-09-30T03:13:13+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.
