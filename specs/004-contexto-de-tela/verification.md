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

## EV-012 — partial

- Ticket: `TK-003`
- Acceptance: `AC-006`
- Procedure: `pnpm -C apps/desktop/e2e test (settings.e2e.ts): settings_open → janela Configurações → switch 'Pausar toda captura'; cargo test -p aura-app --test host privacy_commands_work_outside_the_runtime`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; 2 monitores (1920x1080 100% + 1366x768); WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release --features demo; AURA_HOME isolado
- Tested revision: `local:b9cd0e160e72de7e261dca01ec41f3fdf5eaeca152017d969aa9680ae7e36142`
- Timestamp: `2026-10-01T00:06:58+00:00`
- Observations: Antes da correção c7d3890 pausar fechava o aura.exe (tokio::spawn fora do runtime); depois o switch fica aria-checked=true e o processo segue vivo. Teste novo cobre a chamada fora do runtime (caminho da bandeja/atalho).
- Evidence refs: none
- Limitations: Troca do ícone da bandeja e atalho Ctrl+Shift+Alt+P não conferidos a olho.

## EV-013 — partial

- Ticket: `TK-006`
- Acceptance: `AC-015`
- Procedure: `IPC no app real: tela e Áudio do sistema em Manual → recording_start, 7 s, recording_stop, recordings_list, recording_export; ffprobe/ffmpeg nos arquivos`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; AMD GPU (MFTs AMDh264Encoder, Microsoft AVC DX12, H264 Encoder MFT); WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release modo real; AURA_HOME isolado
- Tested revision: `local:0d75aa7324969a4754c9bb9f545748adec843c7c1c90da412e67b10f0111d561`
- Timestamp: `2026-10-01T00:31:39+00:00`
- Observations: Antes de 480aa73 o codificador MF falhava (BeginWriting E_INVALIDARG; available()=false). Depois: gravação 'screen,system' 545 KB; export screen-0001.mp4 H.264 1920x1080 8 s decodifica sem erro, quadro correto (stride RGB32 ok); system.wav PCM 16 kHz mono 7 s.
- Evidence refs: none
- Limitations: Gravação de 7 s (não 30 s/3 segmentos); exclusão e player não exercitados.

## EV-014 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `IPC capture_screen(windowOnly=false/true) no app real com Firefox em primeiro plano; PNG inspecionado`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; 2 monitores (1920x1080 + 1366x768); WebView2 154.0.4258.37; aura.exe release modo real; AURA_HOME isolado
- Tested revision: `local:e32976096daba5fbeabcbc2732c4fd8420261ba5a9c567db0440217e79bf8f2d`
- Timestamp: `2026-10-01T00:31:52+00:00`
- Observations: Tela inteira em 49 ms, janela em 39 ms; capturou só o monitor do Aplicativo anterior (DISPLAY1); conteúdo do Firefox renderizado (não preto); Chip 'Tela · <título>'. previous_app correto (firefox.exe, título, monitor).
- Evidence refs: none
- Limitations: Sem modo magenta nem 20 execuções para p95; janela parcialmente coberta (AC-002) não testada.
