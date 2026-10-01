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

## EV-008 — partial

- Ticket: `TK-005`
- Acceptance: `AC-008`
- Procedure: `attach_file no app real com MP4 de 5 s do x264 (1 keyframe) e com GOP de 1 s; listar quadros extraídos`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; AMD GPU (MFTs AMDh264Encoder, Microsoft AVC DX12, H264 Encoder MFT); WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release modo real; AURA_HOME isolado
- Tested revision: `local:36ce5ced16a88e4fa312cc07d41f67b8075b453e6a81e99c96caed69cc36eb80`
- Timestamp: `2026-10-01T00:31:39+00:00`
- Observations: Antes de de216be/dae0a7b: 1 keyframe → 8 pedidos no mesmo instante, resumo '~00:00 · 8 quadros' e 1 PNG. Depois: 8 quadros distintos 0,32–4,72 s nos dois clipes, resumo '~00:04'. Teste aura-win video_frames_are_spread_over_the_clip cobre.
- Evidence refs: none
- Limitations: Sem o vídeo numerado de 60 s/12 quadros do AC; legenda de tempo não conferida; transcrição do áudio do vídeo exige modelo ASR (não instalado).
