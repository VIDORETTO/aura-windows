# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-audio (hub/gravação com áudio sintético); cargo check -p aura-win --target x86_64-pc-windows-msvc (WASAPI)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:db39c2e79eec796f27bda8749db463aaaf22984829a105f4a4417fc57ad27ce1`
- Timestamp: `2026-09-30T02:11:44+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-002 — partial

- Ticket: `TK-002`
- Acceptance: `AC-003`, `AC-004`, `AC-006`
- Procedure: `cargo test -p aura-audio (hub/gravação com áudio sintético); cargo check -p aura-win --target x86_64-pc-windows-msvc (WASAPI)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:db39c2e79eec796f27bda8749db463aaaf22984829a105f4a4417fc57ad27ce1`
- Timestamp: `2026-09-30T02:11:44+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-003 — partial

- Ticket: `TK-003`
- Acceptance: `AC-005`
- Procedure: `cargo test -p aura-audio (hub/gravação com áudio sintético); cargo check -p aura-win --target x86_64-pc-windows-msvc (WASAPI)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:db39c2e79eec796f27bda8749db463aaaf22984829a105f4a4417fc57ad27ce1`
- Timestamp: `2026-09-30T02:11:45+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-004 — partial

- Ticket: `TK-004`
- Acceptance: `AC-007`, `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-audio (hub/gravação com áudio sintético); cargo check -p aura-win --target x86_64-pc-windows-msvc (WASAPI)`
- Execution: `executed`
- Environment: Linux VPS x86_64 (AlmaLinux/Ubuntu kernel 5.15), Rust 1.98.1, Node 22, sem Windows
- Tested revision: `local:db39c2e79eec796f27bda8749db463aaaf22984829a105f4a4417fc57ad27ce1`
- Timestamp: `2026-09-30T02:11:45+00:00`
- Observations: Testes automatizados da camada independente de SO passaram (224 Rust, 28 UI).
- Evidence refs: none
- Limitations: Adaptadores Windows só verificados por tipo; shell Tauri não compilado; sem conta ChatGPT/provedores reais.

## EV-005 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-audio; cargo clippy -p aura-win --target x86_64-pc-windows-msvc`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:d7b4510b5884bdb79009b04f5bbeb112bd646b29f4459f32e82b5c3f06ca2387`
- Timestamp: `2026-09-30T03:13:12+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-006 — partial

- Ticket: `TK-002`
- Acceptance: `AC-003`, `AC-004`, `AC-006`
- Procedure: `cargo test -p aura-app recent_buffers_feed_the_agent_tools, manual_recording_lifecycle`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:87a3611417bd93bb3fcab3e4bcaee9b30c4c8cdc044c5baa4d443ca254aa1878`
- Timestamp: `2026-09-30T03:13:12+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-007 — partial

- Ticket: `TK-003`
- Acceptance: `AC-005`
- Procedure: `vitest (indicador REC no cabeçalho, estado recording); cargo test -p aura-app (pausa para gravadores)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:e8ee0b0f315f34572f856d3ecd66286f9ce30e4df01cac76a5749d1a88a4bc2d`
- Timestamp: `2026-09-30T03:13:12+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-008 — partial

- Ticket: `TK-004`
- Acceptance: `AC-007`, `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-app recent_buffers_feed_the_agent_tools (audio_transcript)`
- Execution: `executed`
- Environment: VPS Linux x86_64, Rust 1.98.1, Node 22; Codex app-server real rust-v0.159.0 (build Linux)
- Tested revision: `local:87a3611417bd93bb3fcab3e4bcaee9b30c4c8cdc044c5baa4d443ca254aa1878`
- Timestamp: `2026-09-30T03:13:12+00:00`
- Observations: Testes automatizados passaram (246 Rust + 53 UI; e2e com app-server real).
- Evidence refs: none
- Limitations: Sem Windows/WebView2; adaptadores Windows só verificados por tipo.

## EV-009 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `IPC audio_devices(system=false/true) no app real`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; AMD GPU (MFTs AMDh264Encoder, Microsoft AVC DX12, H264 Encoder MFT); WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release modo real; AURA_HOME isolado
- Tested revision: `local:30c77b46c1f8de5b127dc258cd096ef0bff74dee2288fdd32ce30b41bfa86fd4`
- Timestamp: `2026-10-01T00:31:40+00:00`
- Observations: cpal/WASAPI listou 2 entradas (fifine, VB-CABLE) e 5 saídas com padrão correto; loopback da saída padrão gravou 7 s em system.wav (ver evidência 004 TK-006).
- Evidence refs: none
- Limitations: Medidores de nível não observados; fala no microfone não testada (microfone deixado desligado de propósito).
