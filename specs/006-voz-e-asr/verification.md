# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-asr -p aura-worker -p aura-app (push_to_talk_dictation)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:70a671f4099dbfceb4ad4706c00bd2d5f18882ca0287254084ce0de8137b6924`
- Timestamp: `2026-09-30T02:11:45+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-002 — partial

- Ticket: `TK-002`
- Acceptance: `AC-003`, `AC-004`, `AC-005`
- Procedure: `cargo test -p aura-asr -p aura-worker -p aura-app (push_to_talk_dictation)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:70a671f4099dbfceb4ad4706c00bd2d5f18882ca0287254084ce0de8137b6924`
- Timestamp: `2026-09-30T02:11:45+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-003 — partial

- Ticket: `TK-003`
- Acceptance: `AC-006`
- Procedure: `cargo test -p aura-asr -p aura-worker -p aura-app (push_to_talk_dictation)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:70a671f4099dbfceb4ad4706c00bd2d5f18882ca0287254084ce0de8137b6924`
- Timestamp: `2026-09-30T02:11:46+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-004 — partial

- Ticket: `TK-004`
- Acceptance: `AC-007`, `AC-008`, `AC-013`
- Procedure: `cargo test -p aura-asr -p aura-worker -p aura-app (push_to_talk_dictation)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:70a671f4099dbfceb4ad4706c00bd2d5f18882ca0287254084ce0de8137b6924`
- Timestamp: `2026-09-30T02:11:46+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-005 — partial

- Ticket: `TK-005`
- Acceptance: `AC-009`, `AC-011`
- Procedure: `cargo test -p aura-asr -p aura-worker -p aura-app (push_to_talk_dictation)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:70a671f4099dbfceb4ad4706c00bd2d5f18882ca0287254084ce0de8137b6924`
- Timestamp: `2026-09-30T02:11:46+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-006 — partial

- Ticket: `TK-006`
- Acceptance: `AC-010`
- Procedure: `cargo test -p aura-asr -p aura-worker -p aura-app (push_to_talk_dictation)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:70a671f4099dbfceb4ad4706c00bd2d5f18882ca0287254084ce0de8137b6924`
- Timestamp: `2026-09-30T02:11:46+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-007 — partial

- Ticket: `TK-005`
- Acceptance: `AC-009`, `AC-011`
- Procedure: `cargo test -p aura-core (vocabulário/idioma); cargo test -p aura-app push_to_talk_dictation`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:278c2ea79152b97934772888823bb164d5d65cc25ad7a18ed608e6886c0c923e`
- Timestamp: `2026-09-30T03:13:12+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-008 — partial

- Ticket: `TK-006`
- Acceptance: `AC-010`
- Procedure: `cargo test -p aura-app (FallbackTranscriber local→nuvem)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:fec9c794607e8446f257b75aa21f47d355128074e686bf3ced7a642bb1a5683b`
- Timestamp: `2026-09-30T03:13:13+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-009 — partial

- Ticket: `TK-007`
- Acceptance: `AC-012`
- Procedure: `cargo test -p aura-asr partials_are_emitted_while_listening`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:2e56b5b3b18163ea25167a3110bf0ce3658b51614b473164bfe1aef6acde0f64`
- Timestamp: `2026-09-30T03:13:13+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.
