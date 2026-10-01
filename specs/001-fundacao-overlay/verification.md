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

## EV-007 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `Start-Process aura.exe --background; após 4 s EnumWindows/IsWindowVisible no PID`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; 2 monitores (1920x1080 100% + 1366x768); WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release --features demo; AURA_HOME isolado
- Tested revision: `local:7fa4e62a8c03224013eebd6f2921d413ccdb2f7ad00dd47467679f6f4d0b594d`
- Timestamp: `2026-10-01T00:06:34+00:00`
- Observations: 1 processo; janela overlay vis=False; nenhuma janela visível. Sem --background o Overlay aparece de propósito (main.rs: saudação/onboarding da primeira execução, docs/qa/instalacao.md passo 3) — conflita com o texto do AC-001.
- Evidence refs: none
- Limitations: Ícone da bandeja não conferido visualmente; conflito AC-001 x onboarding 010 a decidir na spec.

## EV-008 — passed

- Ticket: `TK-001`
- Acceptance: `AC-002`
- Procedure: `Com a instância --background rodando, Start-Process aura.exe; após 2 s Get-Process aura e EnumWindows`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; 2 monitores (1920x1080 100% + 1366x768); WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release --features demo; AURA_HOME isolado
- Tested revision: `local:f357a3fb077b7f01d78cfaed1f7be56f37c1654844ffd21a3963d996ed435341`
- Timestamp: `2026-10-01T00:06:34+00:00`
- Observations: Segunda execução saiu; 1 processo (mesmo PID); Overlay da instância original passou a vis=True (tauri-plugin-single-instance → overlay::show).
- Evidence refs: none
- Limitations: none recorded

## EV-009 — partial

- Ticket: `TK-002`
- Acceptance: `AC-008`
- Procedure: `Atalho Ctrl+Shift+Space (SendInput) → EnumWindows; Alt+Tab mantido e capturado; captura da WebView via WebDriver`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; 2 monitores (1920x1080 100% + 1366x768); WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release --features demo; AURA_HOME isolado
- Tested revision: `local:cca9fcf091c4e8ba81b213adfa22d1b7bc77cdb169ca7581e56535b7e35ac1e3`
- Timestamp: `2026-10-01T00:06:57+00:00`
- Observations: Overlay visível, em primeiro plano, WS_EX_TOPMOST. Alt+Tab não lista 'Aura' (lista outras 5 janelas). WebView renderiza cartão translúcido de cantos arredondados.
- Evidence refs: none
- Limitations: Acrylic/sombra e ausência na taskbar não conferidos a olho; Win10 e vídeo em tela cheia não testados.

## EV-010 — partial

- Ticket: `TK-002`
- Acceptance: `AC-009`
- Procedure: `Overlay visível e focado → GetWindowDisplayAffinity + captura GDI (Graphics.CopyFromScreen, mesmo caminho BitBlt do Print Screen) recortada no retângulo do Overlay`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; 2 monitores (1920x1080 100% + 1366x768); WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release --features demo; AURA_HOME isolado
- Tested revision: `local:d7ef327e6e4d972563fd47ff2bd8ed3e9420384cd570256cce080c7bee296d36`
- Timestamp: `2026-10-01T00:06:57+00:00`
- Observations: affinity=0x11 (WDA_EXCLUDEFROMCAPTURE) com o Overlay visível; o recorte mostra só a janela de baixo — Overlay ausente da captura.
- Evidence refs: none
- Limitations: Sem o modo de cor de teste #FF00FF nem captura WGC; Ferramenta de Captura/Teams não testados.

## EV-011 — partial

- Ticket: `TK-001`
- Acceptance: `AC-003`
- Procedure: `Modo real via WebDriver: models_list sobe codex-app-server.exe (filho adotado pelo Job Object) → taskkill /F /IM aura.exe → após 3 s Get-Process pelo PID do filho`
- Execution: `executed`
- Environment: Windows 11 Pro 10.0.26200; WebView2 154.0.4258.37; MSVC 17.14; Rust 1.98.1; aura.exe release (modo real, sem demo); codex-app-server rust-v0.159.0 baixado pelo app; AURA_HOME isolado; sem conta ChatGPT
- Tested revision: `local:9c3fa42c3720d25e533d1494e2411f0b903c7bbf95142c738713982702bc2650`
- Timestamp: `2026-10-01T00:15:36+00:00`
- Observations: Primeira execução baixou e verificou o app-server fixado (bin/codex/rust-v0.159.0); diagnostics: appServer ready, 1 launch; model/list respondeu em 105–565 ms. Após o kill forçado o filho sumiu em < 3 s; nenhum codex-app-server órfão. Encerramento normal (fim da sessão WebDriver) também não deixou órfãos.
- Evidence refs: none
- Limitations: Item Sair da bandeja não clicado (encerramento forçado é o caso mais forte do Job Object); filho de teste ping/debug_spawn_child não existe — usado o app-server real; worker ASR não exercitado.
