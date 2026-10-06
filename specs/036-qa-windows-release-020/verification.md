# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — stale

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `cargo test --workspace --exclude aura-desktop; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --all-targets -- -D warnings; cargo fmt --all -- --check`
- Execution: `executed`
- Environment: Windows 11, MSVC, WebView2 154
- Tested revision: `local:31e7ae517d94931eedfc1ddf1371c0238187ca58581ae9110847662fb03c27fd`
- Timestamp: `2026-10-06T00:28:45+00:00`
- Observations: Rust passou; UI 232/232; tipos, Clippy do shell Windows e rustfmt passaram. Clippy inicialmente falhou com 23 needless_question_mark, corrigidos.
- Evidence refs: none
- Limitations: Evidence invalidated because an input changed.

## EV-002 — stale

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `AURA_CODEX_BIN fixado: cargo test -p aura-app --test real_app_server -- --ignored; WebDriver release-020, meeting-audio-020 e migration-020 seed/verify`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e1b95aa95ec437e50c9a3d35c664dc73e3fc8fb665cb2c1f665b19655b8fcdca`
- Timestamp: `2026-10-06T00:28:46+00:00`
- Observations: 6/6 app-server; notas e contratos novos 2/2; reunião WASAPI/Parakeet/VB-Cable 1/1; migração do MSI 0.1.0 verificou preferências, provedor e comando. Transcrição literal correta sem eco e persistida.
- Evidence refs: none
- Limitations: Ainda falta jornada de criação/disparo de lembrete; instalação interativa/updater público, login ChatGPT e DPI 125/150 não executados. Evidence invalidated because an input changed.

## EV-003 — stale

- Ticket: `—`
- Acceptance: `AC-003`, `AC-005`
- Procedure: `pnpm -C apps/desktop test src/settings/Updates.test.tsx; WebDriver updates.e2e.ts com AURA_E2E_UPDATER_EXPECT_ERROR=1 no canal ainda retido`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e34b1cae2ba3468beead5c5195e3d079deaeea3936629f32ee1b200ffb0c38d0`
- Timestamp: `2026-10-06T00:30:37+00:00`
- Observations: Erro real de endpoint aparece e permanece além do timeout dos toasts; chave configurada verdadeira e botão habilitado após consulta. Caso UI red/green cobriu falha e nova tentativa bem-sucedida.
- Evidence refs: none
- Limitations: Passagem comprova tratamento de erro; não comprova atualização pública ou instalação assinada. Evidence invalidated because an input changed.

## EV-004 — stale

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `WebDriver regressão ampliada 13 specs; localização e providers-refresh repetidos após correção de setup`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:c221e0dc27edc27e58ad4418bcdfd56abdca2db83ed2342e3dc3364e4c186a33`
- Timestamp: `2026-10-06T00:32:43+00:00`
- Observations: 11/13 specs iniciais verdes; provider refresh verde após fixar idioma. Resize 8/9 inicialmente e 6/9 na repetição com recusa de foco. Demais comportamentos passaram; evidência não equivale a validação de todas as escalas.
- Evidence refs: none
- Limitations: Evidence invalidated because an input changed.

## EV-005 — stale

- Ticket: `—`
- Acceptance: `AC-004`
- Procedure: `pnpm -C apps/desktop tauri build com chave nova; verificador minisign no NSIS e MSI; alteração de byte e chave antiga como casos negativos; extração administrativa do MSI`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:162af4a4284e02cb71c6161a40375993e7f6be1f806ac979fc13147f81fcfaa9`
- Timestamp: `2026-10-06T00:34:36+00:00`
- Observations: NSIS/MSI e duas .sig gerados; nova chave verifica e chave aposentada/arquivo adulterado são rejeitados; pacote contém motores Parakeet/Whisper e DLLs DirectML/VC++.
- Evidence refs: none
- Limitations: Publicação interrompida; não houve instalação por cima, Windows limpo ou update público. Canal público não provisionado e CI deve ser adaptado. Evidence invalidated because an input changed.

## EV-006 — failed

- Ticket: `—`
- Acceptance: `AC-006`
- Procedure: `Reunião real no MSI inicial; cargo test -p aura-capture --test pipeline concurrent_sources_keep_every_retained_segment_readable antes da correção`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:10a607aac6c8fe0f063c026701638a3279f40ed15c97c084c6bcb04a57beb0d7`
- Timestamp: `2026-10-06T00:40:39+00:00`
- Observations: Reunião perdeu a frase; log audio segment Acesso negado. Teste público de escrita+retenção de duas fontes reproduziu Io PermissionDenied antes da correção.
- Evidence refs: none
- Limitations: none recorded

## EV-007 — passed

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `cargo test --workspace --exclude aura-desktop após correção de segmentos; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --all-targets -- -D warnings`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:5ca1a4828ec11761e16c7da82230cb89f6c2dcc52695ee581aa090e53738b98f`
- Timestamp: `2026-10-06T00:40:40+00:00`
- Observations: Rust, 232 testes UI, tipos e Clippy passaram após lock compartilhado de publicação/limpeza. IPC e sidecar Codex sem mudanças.
- Evidence refs: none
- Limitations: none recorded

## EV-008 — passed

- Ticket: `—`
- Acceptance: `AC-006`
- Procedure: `cargo test -p aura-capture --test pipeline; cargo test -p aura-audio; reunião SAPI/VB-Cable/Parakeet no MSI reconstruído no perfil em que havia falhado`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:c8b30bce5c2f9b72e105966e373c327616aafdca6f6b3b0d3c96345d04805a8b`
- Timestamp: `2026-10-06T00:45:07+00:00`
- Observations: 400 segmentos concorrentes legíveis; captura/áudio verdes. Reunião real no pacote corrigido transcreveu frase literal, descartou eco e persistiu nota/fala após reiniciar. Falha anterior mantida como EV-006.
- Evidence refs: none
- Limitations: none recorded

## EV-009 — passed

- Ticket: `—`
- Acceptance: `AC-003`, `AC-005`
- Procedure: `WebDriver updates.e2e.ts no MSI reconstruído após correção de concorrência, AURA_E2E_UPDATER_EXPECT_ERROR=1`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:32d436f9e28d29891ba32a15ee6cb2820ce933f5a640e0ed9ccc3fd6bd93b004`
- Timestamp: `2026-10-06T00:46:07+00:00`
- Observations: Erro real do canal retido persistente após 5,5 s; chave configurada; botão de nova tentativa habilitado. Quatro specs, cinco casos passaram no pacote final.
- Evidence refs: none
- Limitations: Não houve update público nem instalação via updater.

## EV-010 — partial

- Ticket: `—`
- Acceptance: `AC-004`
- Procedure: `Build assinado após correção de áudio; verificador NSIS/MSI, chave antiga e byte alterado; WebDriver migration-020.e2e.ts com MSI final e perfil originado da 0.1.0`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:324407b72d5f9ad01fcc28fa029ae5461368508054d51b106755cbb60872dbeb`
- Timestamp: `2026-10-06T00:46:08+00:00`
- Observations: Dois instaladores assinados, manifest local e hashes; assinaturas válidas com chave nova, negativos rejeitados; dados antigos continuam preservados no pacote final.
- Evidence refs: none
- Limitations: Publicação interrompida; canal público não provisionado; instalação por cima, Windows limpo e updater entre versões novas ainda não rodaram.

## EV-011 — partial

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `App-server real 6 casos após correção de áudio; WebDriver notas/contratos novos, reunião real e migração no MSI final`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:1a83a35975641c4a9580a952d4a353054af69281c3ac57336cdf344023e438b5`
- Timestamp: `2026-10-06T00:46:45+00:00`
- Observations: 6/6 app-server e jornadas selecionadas passaram. Nota/fala reais persistidas, sem eco; notas/salvos, contratos de receitas/projetos e dados da 0.1.0 funcionam no pacote final.
- Evidence refs: none
- Limitations: Criação/disparo de lembrete pelo agente, login ChatGPT, transmissão, Narrador e DPI 125/150 não executados.
