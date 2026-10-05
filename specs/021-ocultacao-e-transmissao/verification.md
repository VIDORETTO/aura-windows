# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `cargo test -p aura-core (padrão ligado, patch, configurações antigas); typecheck e testes de UI`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:0ed65de67fb7d88c2a33a53a28bc0a7f807f727bf7620a16b66822419a54e20e`
- Timestamp: `2026-10-05T16:28:10+00:00`
- Observations: Opção hideFromCapture testada em lógica e UI; WDA aplicado só no shell Windows
- Evidence refs: none
- Limitations: Afinidade da janela e apps reais (Meet, Teams, Discord, AnyDesk, OBS) não verificados: exigem Windows

## EV-002 — partial

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `cargo clippy -p aura-win --target x86_64-pc-windows-msvc; vitest Hiding.test.tsx`
- Execution: `executed`
- Environment: Linux x86_64, alvo x86_64-pc-windows-msvc só de tipos
- Tested revision: `local:f5c8fa3930c40b0a089fe5c3a2b8ba151e6b0d156a78f824df44b25a04250805`
- Timestamp: `2026-10-05T17:27:28+00:00`
- Observations: is_excluded_from_capture (GetWindowDisplayAffinity) compila e passa no clippy; botão Testar ocultação na UI com resultado por janela
- Evidence refs: none
- Limitations: Leitura real da afinidade, teste por WGC/DXGI/GDI e Modo Transmissão (silenciar notificações) não implementados/verificados; shell Tauri não compila no Linux

## EV-003 — partial

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `cargo test -p aura-core (broadcast_mode); vitest state/app.test.ts`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:49c028e0ab6292a11093b26d092d773a96ba94c2474592cc2f0609773425ccf6`
- Timestamp: `2026-10-05T17:52:12+00:00`
- Observations: Modo transmissão (broadcastMode): sem notificações do Windows (comando notify do shell e toast de cópia de texto) e lembretes ficam dentro do Overlay; a IA pode ligar/desligar
- Evidence refs: none
- Limitations: Ativação manual (sem detecção automática de compartilhamento); toast de inicialização e do instalador não cobertos; teste por WGC/DXGI/GDI com miniaturas e roteiro nos apps reais pendentes
