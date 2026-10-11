# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web --test search; cargo clippy -p aura-web --all-targets -- -D warnings; cargo fmt --all -- --check; cargo deny check licenses (cargo-deny0.20.2 oficial no TEMP/PATH somente processo)`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:269bfacf251288eafa1908a40d5ef51f3c40aad747877df898a1b8b3c496117f`
- Timestamp: `2026-10-08T00:43:16+00:00`
- Observations: 18 determinísticos passaram,1 live ignorado na suíte; Clippy/rustfmt/licenses passaram. TDD adicional URL>2048 red2fontes/green1; SSE fragmentado/pendingEOF,quota/concorrência,cancelamento,deadline,fallback,dedupe e privacy-wire comprovados.
- Evidence refs: none
- Limitations: Somente escopoTK001: AC001servico, AC012entrada/busca/concorrencia/resposta. Host/modelosTK003, fetchTK002, cache/global/UI/desativacaoTK004, qualidadeTK005 pendentes; nao aprova AC inteiro do esforco. Evidence invalidated because an input changed.

## EV-002 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-web --test search live_anonymous_search -- --ignored --nocapture`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:fb869210c2723b5d2f5d646709effc11cbae2feb8b7ac7b4a062528034ec2101`
- Timestamp: `2026-10-08T00:43:16+00:00`
- Observations: Consulta publica Rust docs: provider=parallel,degraded=false,5 fontes; sem conta/chave de busca.1passed em3s.
- Evidence refs: none
- Limitations: Smoke pontual do endpoint primario. Nao benchmark de12consultas, nao evidencia de fallbackDDGaoVivo, nao prova Host/modelo/inferencia. Evidence invalidated because an input changed.

## EV-003 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web; cargo test -p aura-web --test search live_anonymous_search -- --ignored --nocapture; cargo clippy -p aura-web --all-targets -- -D warnings; cargo fmt --all -- --check; cargo deny check licenses`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:9b8ac0ddf576fb750e48279468ee547fabb55968922c4481c873f68f1ac6299b`
- Timestamp: `2026-10-08T06:56:00+00:00`
- Observations: Renovacao planr3 depois de mudancas compartilhadas:18offline search verdes;liveparallel5fontes/degradedfalse passou;47offline total incluindo29fetch;clippy/fmt/licenses verdes.
- Evidence refs: none
- Limitations: EscopoTK001 e parcelas explicitas dos ACs compartilhados. Host/modelo,UI/desativacao/cache de busca e qualidade/fixado/Windows pendentes nos sucessores. Evidence invalidated because an input changed.

## EV-004 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web; cargo clippy -p aura-web --all-targets -- -D warnings; cargo fmt --all -- --check; cargo deny check licenses`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:a15989b460b6d9d274d08fafcf926e08797500eaad4f9f9bd7455ca3d477fde7`
- Timestamp: `2026-10-08T06:56:01+00:00`
- Observations: 29offline fetch+18search verdes; literal42/12/30/lista/tabela,ABC-DE/emoji,versao/expiry/LRU,DOM/100k/2MiB/30s/cancel,URL/pin/DNS/redirect,tipos/bloqueio/charset comprovados. Clippy/fmt/licencas verdes.
- Evidence refs: none
- Limitations: Somente TK002; cache de documentos provado, cache de busca/registry global/Settings/Host nao nesta fatia. Rebinding na seam observa IPs aprovados no request; production resolve_to_addrs e peer revisados, smoke positivo real separado. Gates qualidade/fixado/Windows pendentes. Evidence invalidated because an input changed.

## EV-005 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`
- Procedure: `cargo test -p aura-web --test fetch live_public_html -- --ignored --nocapture`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:eb21fbe318b353d739c700726087ff961eab924fe2bbf6f029e37053866369c6`
- Timestamp: `2026-10-08T06:56:01+00:00`
- Observations: Pagina publica Rust Ownership lida via transporte real:extractor=readability,20000chars,truncatedtrue,fatos ownership/String,semHTMLscript;1passed0.18s.
- Evidence refs: none
- Limitations: Uma pagina de documentacao; nao prova benchmark8/10 nem todas as variantes adversariais de rede. Nao integrado ao modelo/app ainda. Evidence invalidated because an input changed.

## EV-006 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:2c574e35af70ecf6490323ee6908964444335cdff712f546170e2e648549be9d`
- Timestamp: `2026-10-08T07:03:18+00:00`
- Observations: 47offline passaram,18search+29fetch; renovacao documental planr4 que altera somente seam/observador da integracao TK003, sem mudar codigo/audit dos servicos.
- Evidence refs: none
- Limitations: Mesmos limites de escopoTK001 do review. Live smoke r3 permanece historico; essa execucao e offline. Host/UI/qualidade pendentes. Evidence invalidated because an input changed.

## EV-007 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:8195e20bc8dda02c64069954cd03b89b6cace30136b4ae1a3e13c1510659235d`
- Timestamp: `2026-10-08T07:03:18+00:00`
- Observations: 29fetch+18search verdes; planr4 acrescenta seam de integracao/observador TK003, sem mudanca da leitura/transport/pin/oracles aprovados na revisao r3.
- Evidence refs: none
- Limitations: Mesmos limites escopoTK002; root completo nao aprovado. Smoke HTML anterior historico, essa execucao offline. Evidence invalidated because an input changed.

## EV-008 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:917daf23dfdc6b46b9fb811d43ca3b534dde3b78c010fd7829f32aef1ebde509`
- Timestamp: `2026-10-08T07:19:01+00:00`
- Observations: 47 testes offline passaram (18 busca,29 leitura),2 live ignorados. Renovacao devido aresta aura-app->aura-web e futures dev; servicos e oraculos inalterados.
- Evidence refs: none
- Limitations: Aprovacao somente escopo do ticket de servico; integracao agente,UI,cache agregado e gates finais pendentes. Evidence invalidated because an input changed.

## EV-009 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:917daf23dfdc6b46b9fb811d43ca3b534dde3b78c010fd7829f32aef1ebde509`
- Timestamp: `2026-10-08T07:19:02+00:00`
- Observations: 47 testes offline passaram (18 busca,29 leitura),2 live ignorados. Renovacao devido aresta aura-app->aura-web e futures dev; servicos e oraculos inalterados.
- Evidence refs: none
- Limitations: Aprovacao somente escopo do ticket de servico; integracao agente,UI,cache agregado e gates finais pendentes. Evidence invalidated because an input changed.

## EV-010 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-app --test web_tools --test host --test ipc_contract; pnpm -C apps/desktop test -- src/state/app.test.ts src/i18n/i18n.test.ts; pnpm -C apps/desktop typecheck`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:108824b494f05f07ec548b33134694423581931e12f1f5554c245c7d03f37d14`
- Timestamp: `2026-10-10T15:09:11+00:00`
- Observations: 10 web_tools,74 Host,2 IPC,8 UI passaram e typecheck verde. Red/green reais: contexto antes unknown_tool; close/shutdown demoravam; hosted search habilitado; aviso UI codigo cru. Casos ja verdes registrados somente como cobertura.
- Evidence refs: none
- Limitations: AC001 aviso modelo sem tools; AC007 registry/tools sem renderer; AC008 readonly/autoridade/aprovacao sem promessa LLM e W999 ainda TK004; AC011 cancelamento/lifecycle,cache busca/TTL agregado TK004. Nao aprova qualidade ou Windows. Evidence invalidated because an input changed.

## EV-011 — stale

- Ticket: `TK-003`
- Acceptance: `AC-007`, `AC-008`, `AC-014`
- Procedure: `AURA_CODEX_BIN=TEMP/aura-038-model-picker-production/profile/bin/codex/rust-v0.159.0/bin/codex-app-server.exe; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:b1ac809d467ee151546a7d475b6469c36b6d48106856db48c0c8f02b87c7dcf8`
- Timestamp: `2026-10-10T15:09:12+00:00`
- Observations: 12 casos reais passaram em9.54s:9 modos/transportes,comparacao42/30 W1/W2,retomada apos restart,write injetado exige aprovacao e recusa impede efeito. Outputs efetivos na rede de inferencia contem fatos/IDs/URLs;config disabled apesar override live. Nenhuma conta/custo de busca.
- Evidence refs: none
- Limitations: Inferencia e DNS/HTTP upstream scripted; sidecar,gateway,MCP e Host reais. Marcador textual ainda sem link/painel do TK004. Nao prova SIWC comercial,relevancia ao vivo ou WindowsUI. Evidence invalidated because an input changed.

## EV-012 — stale

- Ticket: `TK-003`
- Acceptance: `AC-008`, `AC-014`
- Procedure: `AURA_CODEX_BIN fixado; AURA_E2E_DIR=TEMP e AURA_SBX_OUTSIDE=TEMP/aura-039-sandbox-outside-UUID isolado; cargo test -p aura-app --test real_app_server real_app_server_task_mode_confines_writes -- --ignored`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:d419570638f6aadd7de4b631303178735e5cf72d342453971f42d9cc2e8bb602`
- Timestamp: `2026-10-10T15:09:13+00:00`
- Observations: Regressao real1/1 passou em47.98s: arquivo fora do workspace ausente,escalada pediu uma aprovacao e foi recusada.
- Evidence refs: none
- Limitations: Este caso preserva sandbox/aprovacao;nao substitui validacao Windows da interface,cache ou relevancia. Evidence invalidated because an input changed.

## EV-013 — stale

- Ticket: `TK-003`
- Acceptance: `AC-014`
- Procedure: `cargo test -p aura-codex -p aura-gateway -p aura-mcp; cargo clippy --workspace --exclude aura-desktop --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:6879dc178928396f94557d62bb2896ce2a41ad7b2ad53ddc50f5bf3f33b57616`
- Timestamp: `2026-10-10T15:09:56+00:00`
- Observations: 90 testes core/process/service/gateway/MCP verdes; Clippy all-targets sem shell,fmt,diff verdes. Resume RPC reforca web disabled e header UUID.
- Evidence refs: none
- Limitations: Clippy excluiu shell Tauri; build/WindowsE2E sao gate TK005. Probe bruto deliberadamente nao executado. Evidence invalidated because an input changed.

## EV-014 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:6695a0342b1c01087aa49886d7167515132105ab5b70e6fa6a0c03528597da24`
- Timestamp: `2026-10-10T15:49:03+00:00`
- Observations: 21 buscas offline passaram, incluindo repeticao cacheada, refresh, TTL15min, isolamento e budget; 29 leituras passaram na mesma execucao.
- Evidence refs: none
- Limitations: Renova escopo TK001; metadados do registry e byte budget agregado incluindo registry continuam TK004. Nao aprova qualidade ao vivo. Evidence invalidated because an input changed.

## EV-015 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-app --test web_tools --test host --test ipc_contract; cargo test -p aura-codex -p aura-gateway -p aura-mcp`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:74e57aacbd3d5a6e06c0873d9bc10813e90c8bf68eb5c968f72e8a5f29115695`
- Timestamp: `2026-10-10T15:49:04+00:00`
- Observations: 14 Host/MCP,74 Host,2 IPC e90 Codex/gateway/MCP passaram. Renovacao apos toggle/cache/eventos TK004; identidades, lifecycle, limites, schemas e readonly preservados.
- Evidence refs: none
- Limitations: Mesmo escopo TK003 da revisao; UX/historico e byte budget registry sao TK004. Nao prova qualidade geral de LLM ou Windows. Evidence invalidated because an input changed.

## EV-016 — stale

- Ticket: `TK-003`
- Acceptance: `AC-007`, `AC-008`, `AC-014`
- Procedure: `AURA_CODEX_BIN=executavel rust-v0.159.0 verificado em TEMP; AURA_E2E_DIR=TEMP; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:84d95dc0bd7b5a7a2a941ec0ae670938ad5fd1419ae13cc467d976e86aaf13f8`
- Timestamp: `2026-10-10T15:49:05+00:00`
- Observations: 12/12 com fixado real passaram em8.44s. Nove transportes/modos, duas fontes42/30, write injetado recusado e retomada; busca e leitura retomadas cached:true,5 requests, conforme AC011.
- Evidence refs: none
- Limitations: Upstreams de inferencia e paginas scripted; nao prova SIWC, disponibilidade comercial, qualidade ao vivo nem janela Windows. Evidence invalidated because an input changed.

## EV-017 — stale

- Ticket: `TK-003`
- Acceptance: `AC-008`, `AC-014`
- Procedure: `AURA_CODEX_BIN=fixado rust-v0.159.0 em TEMP; AURA_E2E_DIR=TEMP; AURA_SBX_OUTSIDE=TEMP/aura-039-sandbox-outside-UUID novo; cargo test -p aura-app --test real_app_server real_app_server_task_mode_confines_writes -- --ignored`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:669afd678015582bef6978f49f871c81303271d0b19c65e88cc24874d47ffef9`
- Timestamp: `2026-10-10T15:49:05+00:00`
- Observations: 1/1 em28.25s. Escrita fora do workspace impedida; escalada solicitou aprovacao e foi recusada. Target isolado proprio em TEMP.
- Evidence refs: none
- Limitations: Regressao de sandbox via processo real, sem alegar imunidade LLM geral; nao e validacao da janela Tauri. Evidence invalidated because an input changed.

## EV-018 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:2d25fa464a2f4ec72633cc3f7a1f0f69bba6ffd767c26efe6652a18cfff0bce7`
- Timestamp: `2026-10-10T15:49:42+00:00`
- Observations: 29 leituras offline passaram, incluindo SSRF,paginacao Unicode,limites de documento,cancelamento e eviccao de documentos.21 buscas passaram na mesma execucao.
- Evidence refs: none
- Limitations: Renova escopo TK002. Registry/historico/UI e agregacao incluindo registry continuam TK004; nao aprova qualidade ao vivo. Evidence invalidated because an input changed.

## EV-019 — stale

- Ticket: `TK-004`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`
- Procedure: `cargo test -p aura-app --test web_tools --test host --test ipc_contract; cargo test -p aura-web --test search --test fetch; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --exclude aura-desktop --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:f0354f683b851a365f3a917b4b5eb9ad393b29fba2a063092396cd46426574b3`
- Timestamp: `2026-10-10T15:49:43+00:00`
- Observations: 14+74+2 testes app,50 servico,251 UI/33files verdes; typecheck,Clippy,fmts e diff passaram. Defaults antigos true,persistencia false apos reinicio,cancelamento antes de cache,reenable mesmo turno preservando budget,TTL/refresh/isolamento/LRU compartilhado,source events e fontes/citacoes PT/en/teclado. Red comportamental confirmado antes das correcoes de default/cancel/TTL/eventos/painel/HTML falso/atividade.
- Evidence refs: none
- Limitations: TK004 incompleto: persistencia so de fontes citadas e restauracao no historico/efemeridade,retencao de outputs web nos rollouts Codex e memoria incluindo registry pendentes. UI sem build/E2E Windows desta versao. AC010 coberto nesta fatia; AC008/009/011/012 parciais. Live qualidade TK005 nao executada. Evidence invalidated because an input changed.

## EV-020 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:3751cc7fee9232b11356f060f12aaded96fbe6d57f5983b3e71d24642d5010bc`
- Timestamp: `2026-10-10T16:11:18+00:00`
- Observations: 50 casos offline:21 busca,29 leitura passaram no planr5. Sem mudanca no comportamento do servico nesta fatia; nova persistencia e privacidade sao TK004.
- Evidence refs: none
- Limitations: Qualidade live e registry/historico pertencem aos sucessores. Evidence invalidated because an input changed.

## EV-021 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:033fbcc836d2b4f3832198ff460e1ae1366ee30641ee3c858e61978064f81a33`
- Timestamp: `2026-10-10T16:11:19+00:00`
- Observations: 50 offline passaram,29 provas de leitura/SSRF/paginacao/cancel/limites; dois smokes live ignorados. Servico sem mudanca nesta fatia.
- Evidence refs: none
- Limitations: Memoria incluindo registry,UX,historico e qualidade live ainda pendentes. Evidence invalidated because an input changed.

## EV-022 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-app --test web_tools --test host --test ipc_contract; cargo test -p aura-codex -p aura-gateway -p aura-mcp; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_history_restores --skip web_ephemeral_research --skip web_normal_research; cargo test -p aura-app --test real_app_server real_app_server_task_mode_confines_writes -- --ignored`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:f536f86107d577d494959d8fbf1f8bf73c7ea7863d7052c15e5b0211ddb9cdcc`
- Timestamp: `2026-10-10T16:11:19+00:00`
- Observations: 90app+90Codex/gateway/MCP,12 integracoes web com fixado real em8.95s e sandbox separado1/1 em23.79s passaram. Tarefa mantem network false/approval; contexto e ferramentas sem mudanca. Somente ids internos do transcript acrescentados sem serializar.
- Evidence refs: none
- Limitations: Novos gates de historico/privacidade TK004 sao separados:historico e efemero passaram; rollout normal falhou. UI/memoria/live ainda pendentes. Nao aprova TK004. Evidence invalidated because an input changed.

## EV-023 — stale

- Ticket: `TK-004`
- Acceptance: `AC-009`, `AC-012`
- Procedure: `cargo test -p aura-app --test real_app_server web::web_normal_research -- --ignored`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e5c4d3d08d495ee606117323ff47d421f64ffc641a32bd7a48af1559d9484a4c`
- Timestamp: `2026-10-10T16:11:20+00:00`
- Observations: Fixado real rust-v0.159.0 concluiu pesquisa/leitura/resposta,shutdown; inspeccao dos arquivos isolados encontrou trecho de pagina nao citado no rollout. Falha comportamental0/1; sem dumps de conteudo/segredos.
- Evidence refs: none
- Limitations: Persistencia seletiva no Aura nao impede escrita do sidecar normal; requer solucao arquitetural antes de done. Nao alterar oraculo nem atualizar sidecar automaticamente. Evidence invalidated because an input changed.

## EV-024 — stale

- Ticket: `TK-004`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`
- Procedure: `cargo test -p aura-app --test real_app_server web::web_history_restores -- --ignored; cargo test -p aura-app --test real_app_server web::web_ephemeral_research -- --ignored; cargo test -p aura-app --test web_tools --test host --test ipc_contract; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --exclude aura-desktop --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:6b77ea5f758f967aca4ea227d35a137b38a75af03712156bb9f8f27a61105db5`
- Timestamp: `2026-10-10T16:11:20+00:00`
- Observations: Historico real2Hosts/banco em disco recupera somenteW1 citado,kind pageContent,snippetvazio,semHTTP; efemero semrollout/URL/pagina emarquivos. Red historico ausente eUI sempainel viraramgreen. 90app,253UI/33files,typecheck,Clippy,fmt,diff verdes.
- Evidence refs: none
- Limitations: TK004 incompleto:rollout normal falho,IDs entre reinicio/Turnos,citacao apos toggle/exclusao/isolamento metadata e memoria registry pendentes. Sem build/E2E Windows atual ou qualidade live. Evidence invalidated because an input changed.

## EV-025 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo clippy --workspace --exclude aura-desktop --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:bf77423b1b9fbaf812627ace4616d837fb3021746e5e5cc8a0a96ebb8f22dc8e`
- Timestamp: `2026-10-10T16:40:33+00:00`
- Observations: Workspace passou; servico 54 offline (25 buscas,29 leituras), Clippy/fmt/diff passaram. Renovacao no plano r6 do escopo TK001, revista em review.md. Admissao de metadata e slots nao alteram gratuidade/normalizacao/fallback.
- Evidence refs: none
- Limitations: ACs compartilhados aprovados somente no escopo TK001. Qualidade live, privacidade rollout normal e Windows atual continuam pendentes nos sucessores. Evidence invalidated because an input changed.

## EV-026 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo clippy --workspace --exclude aura-desktop --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:7e02b57065af12a828053c9c2c929946cf9e62a68591916049429363f91f6122`
- Timestamp: `2026-10-10T16:40:35+00:00`
- Observations: Workspace passou; servico54 offline (25busca,29leitura), checks passaram. Renovacao r6 do escopo leitura segura/paginacao/DNS/limits revisado em review.md.
- Evidence refs: none
- Limitations: Somente escopo TK002, sem aprovacao de qualidade live, UI Windows ou privacidade dos rollouts normais. Evidence invalidated because an input changed.

## EV-027 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test --workspace --exclude aura-desktop; AURA_CODEX_BIN=fixado159 cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_normal_research; AURA_SBX_OUTSIDE=TEMPunico cargo test -p aura-app --test real_app_server real_app_server_task_mode_confines_writes -- --ignored --test-threads=1`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:a4685d8db42dce0b9a5f89d71f552ddfc593fbc51a2e16871be276c7459f91fa`
- Timestamp: `2026-10-10T16:41:32+00:00`
- Observations: Workspace verde incluindo90 app e90 Codex/gateway/MCP. Fixado real16/16 em12.87s, sandbox1/1 em26.60s. Nove protocolos/modos, comparacao, recusa, resume, history/identidade/toggle/efemero passaram. Revisao r6 renova somente escopo original TK003.
- Evidence refs: none
- Limitations: Provedores e paginas scripted, sem prova SIWC/qualidade live. Gate de rollout normal continua falho e pertence TK004. Windows atual ainda pendente. Evidence invalidated because an input changed.

## EV-028 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch --quiet`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:10e517b20f7fd42e746027254de12979cc232e8126d0f4b31e031cb4142e07d3`
- Timestamp: `2026-10-10T16:49:36+00:00`
- Observations: 54offline(25busca,29leitura) passaram no r7, que apenas decompõe qualidade independente. Revisao Standards/Spec libera somente escopo TK001.
- Evidence refs: none
- Limitations: Qualidade, privacidade normal e Windows ainda pendentes; ACs compartilhados so escopo do ticket. Evidence invalidated because an input changed.

## EV-029 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch --quiet`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:becf7b2815f3fa1f2a5c236cf01b52bbbc72d7df90b450e2a4eb1af9f2e11ab5`
- Timestamp: `2026-10-10T16:49:37+00:00`
- Observations: 54offline passaram em planr7; leitura segura/SSRF/paginacao/charset/budgets renovados. Revisao libera somente escopo original.
- Evidence refs: none
- Limitations: Nao aprova relevancia/extração live nem retencao normal/UI Windows. Evidence invalidated because an input changed.

## EV-030 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-app --lib --test web_tools --test host --test ipc_contract --quiet; cargo test -p aura-codex -p aura-gateway -p aura-mcp --quiet; AURA_CODEX_BIN=fixado159 cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_normal_research; AURA_SBX_OUTSIDE=TEMPunico cargo test -p aura-app --test real_app_server real_app_server_task_mode_confines_writes -- --ignored --test-threads=1`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:c9116fb51f7e661cf91800221e61e49582283b8b43bcd4a1079f43a9a540cd7d`
- Timestamp: `2026-10-10T16:49:38+00:00`
- Observations: 144app+90Codex/gateway/MCP passaram r7; 16reais passaram12.37s, sandbox1/1. Revisao r7 renova so escopo TK003.
- Evidence refs: none
- Limitations: Provedores/paginas scripted, sem prova SIWC/qualidade live; rollout normal falho em TK004, Windows atual pendente. Evidence invalidated because an input changed.

## EV-031 — stale

- Ticket: `TK-004`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch --quiet; cargo test -p aura-app --lib --test web_tools --test host --test ipc_contract --quiet; AURA_CODEX_BIN=fixado159 cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_normal_research; cargo clippy --workspace --exclude aura-desktop --all-targets -- -D warnings`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e1fcfb6a4a61c4966de4cb5e29dcb77ca7e31248e3d1b324c651757a161209db`
- Timestamp: `2026-10-10T16:50:34+00:00`
- Observations: R7:54servico+144app,16reais12.37s,Clippy passaram. IdentidadeW1/W3aposrestart, citacao apos toggle, efemero disco, restauracao seletiva, orcamento registry/contextos8/32MiB, slots em nova vida,3casosmetadataSQLitedisco (isolamento/exclusao/restorebounded) passaram. Casosmetadata já verdes sao cobertura. UI253/types passaram nesta rodada antes da decomposicao r7, sem mudanca UI posterior; nao sao aprovacao final Windows.
- Evidence refs: none
- Limitations: Gate rollout normal falho; nao aprova TK004done. Windows atual, qualidade live, sintese e renovacao final UI/esforco038 pendentes. Evidence invalidated because an input changed.

## EV-032 — stale

- Ticket: `TK-004`
- Acceptance: `AC-009`, `AC-012`
- Procedure: `AURA_CODEX_BIN=TEMP/aura-038-model-picker-production/profile/bin/codex/rust-v0.159.0/bin/codex-app-server.exe; AURA_E2E_DIR=TEMP; cargo test -p aura-app --test real_app_server web::web_normal_research -- --ignored --test-threads=1`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:ece45f14598a61e76a426e19487f317be7b2f93b2b7a517f305b95d55e5977c3`
- Timestamp: `2026-10-10T16:50:35+00:00`
- Observations: 0/1 em0.69s no r7; teste falhou por Codex rollout persisted uncited web page content. Infra completou pesquisa/leitura/resposta; nao e erro de ambiente.
- Evidence refs: none
- Limitations: Nenhuma solucao produtiva para retencao normal nesta fatia; manter esperado e versao fixada. Efemero passou separadamente e nao dispensa normal. Evidence invalidated because an input changed.

## EV-033 — stale

- Ticket: `TK-006`
- Acceptance: `AC-001`, `AC-013`
- Procedure: `cargo run -p aura-web --example quality -- C:\Users\gabri\AppData\Local\Temp\aura-039-quality-20261010-r7-report.json C:\Users\gabri\AppData\Local\Temp\aura-039-quality-20261010-r7-targets.json; cargo clippy -p aura-web --all-targets -- -D warnings; cargo test -p aura-web --test search --test fetch --quiet; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:7d6f019defb03647f1d54a957882f713e188df5610bd55d5421b5c4c429ab011`
- Timestamp: `2026-10-10T17:02:20+00:00`
- Observations: 12consultas fixas/top5:11relevantes(>=10),Tauriqueryfalhoutema. 10HTML preselecionados/oraculos inspecionados antesfetch:8literal+origem(>=8), Tauriunsupported_content e SearXNGheaderausente mantidos no denominador. Processo exit0/complete,58HTTPAura/239s,sessaounica,semkey/conta;54offline eClippy/fmt/diff passaram. Reviewliberaapenasbackend.
- Evidence refs: none
- Limitations: ContadorHTTPmede somente transporteAura; browser da preparacao nao instrumentado. Nao aprova sintese3jornadas/UIWindows/SIWC/disponibilidadecontínua/privacidade normal. DoisachadosmenoresFD002/003 registrados. TK004blockedEV032/FD001. Evidence invalidated because an input changed.

## EV-034 — stale

- Ticket: `TK-007`
- Acceptance: `AC-009`, `AC-012`, `AC-014`
- Procedure: `AURA_CODEX_BIN fixado; python %TEMP%/aura-039-persistence-probe/private_result_probe.py; comparação da cópia em experiments com o script executado; fontes rust-v0.159.0 baixadas por curl.exe e inspecionadas`
- Execution: `executed`
- Environment: Windows 11; Python 3.14; app-server rust-v0.159.0; MCP e provider Responses locais scripted; CODEX_HOME isolado novo
- Tested revision: `local:5dc3d97f62d94dea2502bf207e69d08b690ab62affdcff0ca316b9bd810debdc`
- Timestamp: `2026-10-10T17:26:45+00:00`
- Observations: Protótipo completou turno; 2 requests simulados, 1 MCP, 1 expansão; literal integral de página ausente nos arquivos examinados, handle presente; mesmo threadId e resposta após reinício; headers thread-id/session-id correspondem ao ID. A aplicação não foi corrigida.
- Evidence refs: none
- Limitations: Prova de viabilidade somente: não demonstra escopo/cancelamento/TTL/budget/compactação/URLs/raciocínio, adapters do Aura, ausência em conteúdo comprimido nem gate integral. Parser recursivo do protótipo não será usado na implementação. Tentativas anteriores falharam na configuração MCP, sem aprovação de comportamento. Evidence invalidated because an input changed.

## EV-035 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; saída em TEMP/aura-039-r8-regression.log; observar search/fetch`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:fafc58bb4ed7c7396f932f07487e7662b32681b8f8ef8a578c21fccc649c8e3a`
- Timestamp: `2026-10-10T17:27:27+00:00`
- Observations: 54 testes offline WebService passaram: 25 search/29 fetch; 2 live ignorados. Plan r8 sem mudança no código nesta rodada; renovação do escopo original de TK001.
- Evidence refs: none
- Limitations: Não aprova a entrega transitória proposta TK007, retenção normal, qualidade live nem Windows. Evidence invalidated because an input changed.

## EV-036 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; saída TEMP/aura-039-r8-regression.log; escopo serviço search/fetch`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:fafc58bb4ed7c7396f932f07487e7662b32681b8f8ef8a578c21fccc649c8e3a`
- Timestamp: `2026-10-10T17:28:00+00:00`
- Observations: 54 offline WebService: 29 fetch/25 search, sem falhas. Código não mudou nesta rodada; plan r8 renova escopo original de TK002.
- Evidence refs: none
- Limitations: 2 live ignorados. FD002/FD003 menores continuam; sem aprovação do TK007, gate live, privacidade normal ou Windows. Evidence invalidated because an input changed.

## EV-037 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; AURA_CODEX_BIN fixado/AURA_E2E_DIR TEMP: cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_normal_research; AURA_SBX_OUTSIDE TEMP UUID: cargo test -p aura-app --test real_app_server real_app_server_task_mode_confines_writes -- --ignored --test-threads=1`
- Execution: `executed`
- Environment: Windows 11; fixado rust-v0.159.0 cached; upstream/páginas scripted; sandbox target TEMP aleatório próprio
- Tested revision: `local:3f49f6cecddbbd96399b3772bc43c2004408cf36b4f61a74f6093efb99397451`
- Timestamp: `2026-10-10T17:29:05+00:00`
- Observations: 288 testes dos cinco crates passaram (app144, Codex/gateway/MCP90, WebService54); integrações16/16 em22,17s, sandbox1/1 em23,02s. Escopo original da integração, código inalterado nesta rodada. Primeiro filtro de sandbox rodou0; só a execução corrigida1/1 aprova.
- Evidence refs: none
- Limitations: Não aprova TK007, retenção normal falha, qualidade live, SIWC real ou Windows/UI. Providers e páginas scripted; acurácia de síntese depende do modelo. Evidence invalidated because an input changed.

## EV-038 — stale

- Ticket: `TK-004`
- Acceptance: `AC-009`, `AC-012`
- Procedure: `AURA_CODEX_BIN fixado/AURA_E2E_DIR TEMP; cargo test -p aura-app --test real_app_server web::web_normal_research -- --ignored --test-threads=1; saída TEMP/aura-039-r8-normal-retention.log`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:c6763f5087800ed87bb5a1d71119f54fc02320d6fc135adf6ff40c3a3c5cd146`
- Timestamp: `2026-10-10T17:29:54+00:00`
- Observations: 0/1 em0,74s, exit101: Codex rollout persisted uncited web page content. Código de produção permanece anterior ao protótipo. Oráculo inalterado e sem falha de ambiente.
- Evidence refs: none
- Limitations: Prova da falha atual; não avalia a proposta TK007. Nenhum esperado foi dispensado. Evidence invalidated because an input changed.

## EV-039 — stale

- Ticket: `TK-004`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_normal_research; conferir relatórios r8`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:f7d5cb426b2cc2f9d3c6fffc7d4f3adabca92060629aeb8e02d22827fb9f7ab2`
- Timestamp: `2026-10-10T17:29:54+00:00`
- Observations: 288 offline e 16 integrações fixadas passaram; identidade/metadata/budget/efêmero/histórico/toggle do código atual mantidos. Gate normal reexecutado falhou EV038; protótipo EV034 é viabilidade apenas.
- Evidence refs: none
- Limitations: UI253/types são evidência histórica, não executados nesta rodada. Retenção normal, implementação TK007 e Windows/DPI/fluxo final ainda faltam; quality r7 aguarda renovação após invalidação. Evidence invalidated because an input changed.

## EV-040 — stale

- Ticket: `TK-007`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-014`
- Procedure: `cargo test -p aura-gateway --test tool_results`
- Execution: `executed`
- Environment: Windows; branch release/0.2.0; alterações locais sem commit; baseline ed9a9654c6e76e48186fc010a12eba77023b00f5
- Tested revision: `local:56013a19d86b2c7970c05ff48b684d488845d7d1e00e7a1081260f09c89ba9fd`
- Timestamp: `2026-10-10T17:56:39+00:00`
- Observations: 9/9 testes HTTP passaram em 0.01s: expansão exata, JSON escapado, escopo de thread, headers duplicados, mensagens/terceiros/outputs sem chamada, envelopes inexatos, namespace achatado, cancelamento e ausência de expansão recursiva. Nenhum processo cargo ficou ativo.
- Evidence refs: none
- Limitations: Provedor externo simulado; usa WebService real. Não cobre o gate integral com app-server fixado, reinício, compactação, nove combinações de protocolos/modos, URLs não citadas/argumentos/raciocínio nem armazenamento comprimido/SQLite. Regressoes anteriores afetadas marcadas stale; não aprova TK007, TK004 nem ADR0010. Evidence invalidated because an input changed.

## EV-041 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp`
- Execution: `executed`
- Environment: Windows11; rust-v0.159.0 cached, SHA256 executável68cc2280bfd6fd682f3399790526b35aae29c9395ff2bc4efd652a768fc97ac5; perfis/sandbox TEMP próprios
- Tested revision: `local:9c7cb438ec2b95bc8dce2ed1e5ad033f5149ecf99a7d980d8b95f762e2ec6bfa`
- Timestamp: `2026-10-10T18:03:30+00:00`
- Observations: 304 testes dos cinco crates passaram; WebService61 inclui busca25, leitura29 e entrega7. Renova somente o escopo original de busca, fallback, proveniência e limites. Revisão Standards/Spec em review.md.
- Evidence refs: none
- Limitations: Fixtures externas determinísticas. Não aprova qualidade live, entrega transitória inteira, retenção em rollout, UI ou Windows. Evidence invalidated because an input changed.

## EV-042 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp`
- Execution: `executed`
- Environment: Windows11; rust-v0.159.0 cached, SHA256 executável68cc2280bfd6fd682f3399790526b35aae29c9395ff2bc4efd652a768fc97ac5; perfis/sandbox TEMP próprios
- Tested revision: `local:33c6066ff0724a3e9119a34c0447f895878ae6eedb07808491ab4314627bbad8`
- Timestamp: `2026-10-10T18:03:30+00:00`
- Observations: WebService61/61 dentro das regressões304/304; leitura29 inclui extração, continuação, DNS/redirects e limites com o serviço real. Revisão Standards/Spec em review.md; escopo original preservado.
- Evidence refs: none
- Limitations: Rede e relógio controlados. Não aprova HTML ao vivo, rollout, entrega transitória completa ou Windows. Evidence invalidated because an input changed.

## EV-043 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; AURA_CODEX_BIN fixado/AURA_E2E_DIR TEMP: cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1; AURA_SBX_OUTSIDE TEMP UUID: cargo test -p aura-app --test real_app_server real_app_server_task_mode_confines_writes -- --ignored --test-threads=1; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck`
- Execution: `executed`
- Environment: Windows11; rust-v0.159.0 cached, SHA256 executável68cc2280bfd6fd682f3399790526b35aae29c9395ff2bc4efd652a768fc97ac5; perfis/sandbox TEMP próprios
- Tested revision: `local:e3f68d4ad7e49f8d940c1396d75e4ec9f9eb7fa1242154397fd97cb7a31df105`
- Timestamp: `2026-10-10T18:03:31+00:00`
- Observations: Rust304/304; fixado17/17 em14,59s incluindo nove protocolos/modos, retomada/cache e histórico; sandbox1/1 em22,42s; UI253/253; typecheck passou. Renova escopo original da integração conforme review.md.
- Evidence refs: none
- Limitations: Providers/páginas scripted. Passagem do literal de página não aprova privacidade integral de URLs não citadas/argumentos/raciocínio, compactação ou TK007/TK004. SIWC e QA nativo atuais não executados. Evidence invalidated because an input changed.

## EV-044 — stale

- Ticket: `TK-007`
- Acceptance: `AC-009`, `AC-012`
- Procedure: `AURA_CODEX_BIN fixado/AURA_E2E_DIR TEMP: cargo test -p aura-app --test real_app_server web_fetched_but_uncited_source_url -- --ignored --test-threads=1`
- Execution: `executed`
- Environment: Windows11; rust-v0.159.0; perfil sintético TEMP próprio; provider/rede scripted
- Tested revision: `local:5dbe722a8183915120f75a4baf7e0cd4701e3a0433bd7b6f574870cdd16fd66a`
- Timestamp: `2026-10-10T18:12:19+00:00`
- Observations: Red comportamental: 0/1 em0,75s. O provedor recebe W1/W2 e resposta cita só W1; URL de W2 persiste em2 arquivos, incluindo1 function_call. Perfil preservado somente para auditoria sintética. Nenhum problema de ambiente.
- Evidence refs: none
- Limitations: Não exercita raciocínio/compactação. Não reduzir esperado, trocar thread normal por efêmera ou aceitar persistência. Rollout e WAL foram localizados por auditoria independente. Evidence invalidated because an input changed.

## EV-045 — stale

- Ticket: `TK-007`
- Acceptance: `AC-009`, `AC-012`
- Procedure: `python specs/039-web-gratuita-do-agente/experiments/audit_synthetic_storage.py C:\Users\gabri\AppData\Local\Temp\.tmp6a0iUM\Aura specs/039-web-gratuita-do-agente/experiments/uncited-storage-result.json`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:f456b341f9d7787aef697bfec82c442475506b1e545c7bde8130d241e0acd245`
- Timestamp: `2026-10-10T18:12:19+00:00`
- Observations: Inspeção de88 arquivos e128 linhas SQLite confirmou uncited_url no rollout, no WAL e em thread_history_1.sqlite/thread_items/item_json. Nenhum corpo/snippet ou nonce foi incluído no relatório.
- Evidence refs: none
- Limitations: Somente4 literais sintéticos; zero frames comprimidos observados, logo branch gzip/zstd não comprovada nesta fixture; raciocínio e compactação pendentes. Diagnóstico do bloqueio, não aprovação. Evidence invalidated because an input changed.

## EV-046 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `AURA_CODEX_BIN fixado/AURA_E2E_DIR TEMP: cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_fetched_but_uncited_source_url`
- Execution: `executed`
- Environment: Windows11; rust-v0.159.0; providers/rede scripted
- Tested revision: `local:56746f328ff3bf52ee5830d19f6a31e410454ae633f382ad55358cccd4f4d184`
- Timestamp: `2026-10-10T18:12:20+00:00`
- Observations: 17/17 casos do escopo original passaram em13,39s após acrescentar o novo oracle. Nove protocolos/modos, recusa, cache/retomada e história preservados. Código produtivo não mudou nesta rodada; Rust304, sandbox1, UI253/typecheck executados antes da alteração do fixture. Revisão Standards/Spec separa os escopos.
- Evidence refs: none
- Limitations: O novo caso de retenção de URL falhou e está registrado separadamente EV044/EV045; foi excluído só desta verificação do predecessor, permanece gate obrigatório de TK007/TK004. Não aprova privacidade integral, qualidade live, compactação nem Windows. Evidence invalidated because an input changed.

## EV-047 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; cargo clippy -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: Windows11; app-server fixado rust-v0.159.0; fixtures externas sintéticas; código local após rustfmt
- Tested revision: `local:4b5e3f700693aa1bbc535096c0b9105a60891a79700352837bd1a4e421528b34`
- Timestamp: `2026-10-10T18:17:08+00:00`
- Observations: Após rustfmt,304/304 testes passaram; WebService61 inclui busca25/leitura29/entrega7. Clippy/fmt/diff check passaram. Reaprova somente escopo original da busca conforme revisão.
- Evidence refs: none
- Limitations: Não aprova retenção integral, raciocínio, compactação, qualidade live ou janela Windows atual. Sandbox1/1 eUI253/typecheck passaram antes da formatação, sem mudança produtiva funcional posterior. Evidence invalidated because an input changed.

## EV-048 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; cargo clippy -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: Windows11; app-server fixado rust-v0.159.0; fixtures externas sintéticas; código local após rustfmt
- Tested revision: `local:d11e86db2ca080d8d4c962739e414a6eb15871f2b4941ebc79e3a04f4153e24b`
- Timestamp: `2026-10-10T18:17:09+00:00`
- Observations: Após rustfmt,WebService61/61 e total304/304 passaram; leitura29 mantém extração,continuação,DNS/redirects/limites. Clippy/fmt/diff passaram. Escopo original apenas.
- Evidence refs: none
- Limitations: Não aprova retenção integral, raciocínio, compactação, qualidade live ou janela Windows atual. Sandbox1/1 eUI253/typecheck passaram antes da formatação, sem mudança produtiva funcional posterior. Evidence invalidated because an input changed.

## EV-049 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; AURA_CODEX_BIN fixado/AURA_E2E_DIR TEMP: cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_fetched_but_uncited_source_url; cargo clippy -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: Windows11; app-server fixado rust-v0.159.0; fixtures externas sintéticas; código local após rustfmt
- Tested revision: `local:b0cc12b389fdfd84089e1603cef98ff70177bdbccff31e6bb11a5543fa4df9b7`
- Timestamp: `2026-10-10T18:17:09+00:00`
- Observations: Após rustfmt,Rust304/304 e17/17 integrações originais em13,24s passaram; nove protocolos/modos e oráculos originais preservados. Clippy/fmt/diff passaram. O gate W2 foi executado separadamente e falhou0/1 em0,71s, permanece obrigatório em TK007/TK004.
- Evidence refs: none
- Limitations: Não aprova retenção integral, raciocínio, compactação, qualidade live ou janela Windows atual. Sandbox1/1 eUI253/typecheck passaram antes da formatação, sem mudança produtiva funcional posterior. Evidence invalidated because an input changed.

## EV-050 — stale

- Ticket: `TK-007`
- Acceptance: `AC-009`, `AC-012`
- Procedure: `AURA_CODEX_BIN fixado/AURA_E2E_DIR TEMP: cargo test -p aura-app --test real_app_server web_fetched_but_uncited_source_url -- --ignored --test-threads=1`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:d42ef68b472c90b34e19ebcbac5c677284507d4779a0217321244b0eaa0dc97e`
- Timestamp: `2026-10-10T18:17:10+00:00`
- Observations: Após rustfmt,0/1 em0,71s: o provedor recebe fatos de W1/W2 e final cita só W1; URL de W2 persiste em2 arquivos, incluindo1 chamada. Mesmo expected preservado; red comportamental.
- Evidence refs: none
- Limitations: Raciocínio/compactação pendentes; script auditou perfil sintético próprio separado. Nunca reduzir expected nem substituir thread normal por efêmera. Evidence invalidated because an input changed.

## EV-051 — stale

- Ticket: `TK-007`
- Acceptance: `AC-009`, `AC-012`
- Procedure: `python specs/039-web-gratuita-do-agente/experiments/audit_synthetic_storage.py C:\Users\gabri\AppData\Local\Temp\.tmpXDiIEH\Aura specs/039-web-gratuita-do-agente/experiments/uncited-storage-formatted-result.json`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:08be042bf50a13d85b2057fd1819b07d9e018405b1a378fc0c77e7263626fecd`
- Timestamp: `2026-10-10T18:17:11+00:00`
- Observations: Auditoria do perfil da execução após rustfmt confirma uncited_url no rollout, WAL e SQLite/thread_items/item_json. Report sanitizado, sem corpo/argumentos/nonce.
- Evidence refs: none
- Limitations: Quatro literais sintéticos,sem frames comprimidos observados e sem cenário de raciocínio/compactação. Diagnóstico de falha, não aprovação. Evidence invalidated because an input changed.

## EV-052 — stale

- Ticket: `TK-007`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`, `AC-014`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; AURA_CODEX_BIN fixado/AURA_E2E_DIR TEMP: cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_fetched_but_uncited_source_url; cargo clippy -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:82ed9f2ecb3330b2022d74135905448773bf7c78b9efa4b4a199aa92b1dac59b`
- Timestamp: `2026-10-10T18:18:27+00:00`
- Observations: 304/304 regressões, incluindo gateway HTTP9, entrega7 e Host/MCP14. Fixado17/17 dos casos originais, Clippy/fmt/diff passaram. Escopo,origem,budget e ciclo de vida dos outputs transitórios comprovados parcialmente.
- Evidence refs: none
- Limitations: EV050/051 falham o gate W2 em argumentos/SQLite; não aprova TK007/TK004/ADR0010. Raciocínio,compactação e caminhos comprimidos não exercitados. Nenhum aceite reduzido. Evidence invalidated because an input changed.

## EV-053 — stale

- Ticket: `TK-006`
- Acceptance: `AC-001`, `AC-013`
- Procedure: `cargo run -p aura-web --example quality -- 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-r8-1791656334857-report.json' 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-r8-1791656334857-targets.json'; inspeção independente e manifesto antes de extrair, conforme quality.md`
- Execution: `executed`
- Environment: Windows11; spec1/plan8; WebService/ReqwestTransport reais; uma sessão,sem conta/chave de busca
- Tested revision: `local:5cb02953762e486f23df4eddce01915321a0b57e8ad4df1d9d6af967dd6a119e`
- Timestamp: `2026-10-10T18:32:25+00:00`
- Observations: Nova execução terminal exit0/complete:10/12 fontes confirmadas,8/10 extrações,58 HTTP/242s. Doze buscas48 HTTP/33s,seguidas por10 leituras de alvos inspecionados/fixados antes de extrair. IBGE não verificado ficou fora da aprovação,Tauri tema incorreto; duas falhas de leitura mantidas. Review Standards/Spec em review.md.
- Evidence refs: none
- Limitations: Só backend; não aprova síntese,privacidade de rollout,compactação ou Windows. Inspeção pelo browser fora do contador do Aura. Não substituiu/repetiu queries/alvos depois de falha. Títulos de leitura limitados,sem snippets/corpos no report do git. Evidence invalidated because an input changed.

## EV-054 — stale

- Ticket: `TK-007`
- Acceptance: `AC-009`, `AC-014`
- Procedure: `python specs/039-web-gratuita-do-agente/experiments/private_arguments_probe.py --schema-conforming; mesma chamada com --reasoning --schema-conforming; executável fixado em AURA_CODEX_BIN, perfis TEMP exclusivos, provedor/MCP sintéticos.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:0cb70006cf2cda4f30906795b20825992a94f94d24b4b0b778cd910844591f68`
- Timestamp: `2026-10-10T18:58:45+00:00`
- Observations: Dois experimentos terminal exit0: schema conforme, 2 requests/1 MCP, argumentos e resultado restituídos upstream, mesmo thread/resposta após restart; 77 arquivos e 84/85 linhas SQLite sem quatro literais. Raciocínio sintético reidratado. Plano r9 escolhido por evidência real.
- Evidence refs: none
- Limitations: Protótipo, não aplicação. Não cobre blob criptográfico comercial, compactação, orçamento/autorização de produção nem compressão ausente. Gate real EV050/051 continua falho. Evidence invalidated because an input changed.

## EV-055 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; paraTK003 cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_fetched_but_uncited_source_url, AURA_CODEX_BIN fixado e AURA_E2E_DIR TEMP.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:bca801c16148e7c33873e5ce3702def50cde67b4b54e34d6454d1e6e944ba7e2`
- Timestamp: `2026-10-10T19:00:40+00:00`
- Observations: Busca sem chave, parsing/proveniência e budget:25 casos de search e suite304,0falhas. Renovação após plano9,sem mudança de comportamento produtivo nesta execução.
- Evidence refs: none
- Limitations: Somente escopo original deste predecessor; não aprova TK007,gate integralW2,raciocínio,compactação,qualidade ou Windows. Evidence invalidated because an input changed.

## EV-056 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; paraTK003 cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_fetched_but_uncited_source_url, AURA_CODEX_BIN fixado e AURA_E2E_DIR TEMP.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:bca801c16148e7c33873e5ce3702def50cde67b4b54e34d6454d1e6e944ba7e2`
- Timestamp: `2026-10-10T19:00:41+00:00`
- Observations: Leitura pública segura/continuação/budget:29fetch+7delivery,suite304,0falhas. Renovação após plano9,sem mudança de comportamento produtivo nesta execução.
- Evidence refs: none
- Limitations: Somente escopo original deste predecessor; não aprova TK007,gate integralW2,raciocínio,compactação,qualidade ou Windows. Evidence invalidated because an input changed.

## EV-057 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; paraTK003 cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 --skip web_fetched_but_uncited_source_url, AURA_CODEX_BIN fixado e AURA_E2E_DIR TEMP.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:438fc74b30b1d05c4e248285f41dc8cf1b2641b7827778d635c2f6a0d9e14c0a`
- Timestamp: `2026-10-10T19:00:42+00:00`
- Observations: Composição original: suite304 e17casos reais fixados em13,10s. Privacidade integral permanece falha separadaW2; sem redução deexpected. Renovação após plano9,sem mudança de comportamento produtivo nesta execução.
- Evidence refs: none
- Limitations: Somente escopo original deste predecessor; não aprova TK007,gate integralW2,raciocínio,compactação,qualidade ou Windows. Evidence invalidated because an input changed.

## EV-058 — stale

- Ticket: `TK-007`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`, `AC-014`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; cargo test -p aura-gateway --test private_calls após ajuste somente de comparação literal no teste; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin AURA_CODEX_BIN e TEMP; clippy cinco crates --all-targets -- -D warnings; fmt/diff check.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:7f5f3bb5e20aeb47b8e543ceaeecf3d510186ba5f999c198aff090ea4d9a5e6f`
- Timestamp: `2026-10-10T19:19:04+00:00`
- Observations: 316 regressões;10HTTP novos atuais;18/18 casos reais em14,46s, incluindo gateW2 antes falho. Args/resultados kind separados no mesmo budget; scope capturado antes deawait; semnonce/URL bruto no stream de ferramentas. RedW2 antes da correção0,81s/doisarquivos; green sem alterarexpected. Unicode ecall_id históricos corrigidos após reds próprios.
- Evidence refs: none
- Limitations: Parcial: raciocínio, blob criptográfico de provedor, compactação e gate integral ainda pendentes. Suíte316 antecedeu somente variáveis de comparação no novo teste; os10casos desse arquivo foram reexecutados após esse ajuste. UI/native/workspace inteiro não renovados. Evidence invalidated because an input changed.

## EV-059 — stale

- Ticket: `TK-007`
- Acceptance: `AC-009`, `AC-012`
- Procedure: `AURA_WEB_KEEP_SYNTHETIC_PROFILE=1: suíte real18/18 pós-fmt em14,06s, own TEMP .tmp9YHTtc; python specs/039-web-gratuita-do-agente/experiments/audit_synthetic_storage.py C:/Users/gabri/AppData/Local/Temp/.tmp9YHTtc/Aura specs/039-web-gratuita-do-agente/experiments/uncited-storage-private-arguments-formatted-result.json.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:259ca9ff8b61a371cdb5d23720e5f1c90de12fb5a918a72564d5257ece345849`
- Timestamp: `2026-10-10T19:19:05+00:00`
- Observations: 86 arquivos/128 linhasSQLite,zero quatro literais não citados. ContrastaEV051 mesmaestrutura comURLemrollout/WAL/SQLite. FonteW2 foi lida, fato recebido upstream e resposta citouW1; expectedmantido. Último ajuste de associação sequencialcall_id não muda bytes do sidecar,18reais passaram novamente14,46s.
- Evidence refs: none
- Limitations: Auditoria limitada a quatro literais. Nenhum frame comprimido, raciocínio ou compactação exercitados. Não aprova toda privacidade nem resolveFD001. Evidence invalidated because an input changed.

## EV-060 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; cargo clippy cinco crates --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:f161ce354fe8f0ba2c96ffb75322ff2526d58660d43b4ce21f208f7240f2c450`
- Timestamp: `2026-10-10T19:19:05+00:00`
- Observations: 316casos na rodada atual;search25,fetch29,delivery8;busca,normalização/proveniência e budget preservados. r9 técnico sem alteração na spec1. RevisãoStandards/Spec será registrada emreview.md.
- Evidence refs: none
- Limitations: Escopo originalTK001; gate integral/raciocínio/compactação e qualidade live não aprovados. Evidence invalidated because an input changed.

## EV-061 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; cargo clippy cinco crates --all-targets -- -D warnings; fmt/diff check.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:f161ce354fe8f0ba2c96ffb75322ff2526d58660d43b4ce21f208f7240f2c450`
- Timestamp: `2026-10-10T19:19:06+00:00`
- Observations: 316casos;fetch29/delivery8,paginação/DNS/redirects/oráculos e orçamento preservados. Nenhum parser/fonte/limiar de qualidade alterado.
- Evidence refs: none
- Limitations: Escopo originalTK002; privacidade integral/raciocínio/compactação e qualidade não aprovados. Evidence invalidated because an input changed.

## EV-062 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-web -p aura-app -p aura-codex -p aura-gateway -p aura-mcp; novos10HTTP reexecutados após ajuste deassert; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin eTEMP; clippy cinco crates/all-targets,fmt/diff check.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:5819c810ecc1c25fbde0bc16a2fdd0cfc1f8d0d99af204dae12a5c886e17e4a9`
- Timestamp: `2026-10-10T19:19:06+00:00`
- Observations: 316regressões,10novos atuais,18reais14,46s. Nove pares protocolo/modo,recusa/sandbox,retomada/histórico/identidade/toggle preservados. Host/MCP14 usam oráculos upstream observados,sem reimplementar expansão.
- Evidence refs: none
- Limitations: Somente escopo originalTK003. Não aprova o gate integral deTK007/004,raciocínio/compactação nemUI/nativo/qualidade.316+10 conformelimitaçãoEV058. Evidence invalidated because an input changed.

## EV-063 — stale

- Ticket: `TK-007`
- Acceptance: `AC-009`, `AC-012`
- Procedure: `cargo test -p aura-app --test real_app_server web_reasoning_does_not -- --ignored --test-threads=1 com pin AURA_CODEX_BIN e ownTEMP; audit_synthetic_storage.py no perfil .tmprigQbw/Aura`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e3a780d67f3906c72d78c98ee010e92a55b24a8d9b71a5474b03b090e7c7e0e7`
- Timestamp: `2026-10-10T19:27:23+00:00`
- Observations: Red real0,72s: mesmo fluxo4calls/6HTTP/fatos W1/W2/resposta somenteW1 confirmado, URLnão citada em2arquivos sem function_call persistido. Provedor adiciona reasoning summary eopaque field sintético após web; etapa deargumentos jáverde não protege raciocínio.
- Evidence refs: none
- Limitations: Fixture de raciocínio sintético,semcriptografia comercial/compactação. Auditor antigo temlimitação fixa reasoningnotexercised, inaplicável a esta variante; corpo das reasoning foi emitido pelo fixture eURLpersistiu. Registra falha real, não erro ambiente. Evidence invalidated because an input changed.

## EV-064 — stale

- Ticket: `TK-007`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`, `AC-014`
- Procedure: `cargo test -p aura-gateway --test private_calls; cargo test -p aura-web --test delivery; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com AURA_CODEX_BIN rust-v0.159.0 e AURA_E2E_DIR TEMP; audit_synthetic_storage.py no perfil sintetico .tmp6Zzqzt/Aura com --reasoning --compaction; python specs/039-web-gratuita-do-agente/experiments/check_storage_auditor.py`
- Execution: `executed`
- Environment: Windows 11; app-server rust-v0.159.0 fixado; provider e MCP loopback controlados; perfil sintetico exclusivo TEMP; nenhuma inferencia paga
- Tested revision: `local:ec8a1b5064cf492bccf98116c720484ebfd77f6ddbeb3dcb3f4a4c4c16eed21c`
- Timestamp: `2026-10-10T19:46:14+00:00`
- Observations: 14/14 HTTP,9/9 delivery e21/21 web real verdes; suite real19,07s. Original reasoning summary e campo opaco restaurados somente no provider. Reinicio,novo Turno e compactacao publica preservam resposta/fontes W1. Auditor:88arquivos/139registros SQLite/zero quatro marcadores. Controles positivos:2arquivos gzip/zstd e2celulas SQLite,3marcadores em cada local. Logs TEMP aura-039-status-native e aura-039-status-private.
- Evidence refs: none
- Limitations: Partial: campo opaco sintetico sem criptografia comercial; novos raciocinio/compactacao somente Responses; zero frames comprimidos na fixture Codex. Controle de compressao valida auditor separadamente. Faltam campo criptografado representativo,ciclo de vida adicional,regressoes finais e revisao. Nenhuma conclusao geral de ausencia de toda informacao derivada. Evidence invalidated because an input changed.

## EV-065 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-gateway -p aura-app -p aura-codex -p aura-mcp; cargo clippy -p aura-web -p aura-gateway -p aura-app -p aura-codex -p aura-mcp --all-targets -- -D warnings`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:bdc0f4ba19e22387054bf7a6e718ea7f9c78e3a1b8ce415bd3d0d81ad6c36ef5`
- Timestamp: `2026-10-10T19:48:49+00:00`
- Observations: 321 regressões passaram,29 ambientais ignoradas;Clippy all-targets semavisos. Busca/fallback/proveniencia/budget originais preservados apos kindReasoning e frescor. Revisao atual emreview.md.
- Evidence refs: none
- Limitations: Renova somente escopo original TK001,nao gate integral TK007/qualidade live/sintese/UI/Windows. Evidence invalidated because an input changed.

## EV-066 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web -p aura-gateway -p aura-app -p aura-codex -p aura-mcp; cargo clippy -p aura-web -p aura-gateway -p aura-app -p aura-codex -p aura-mcp --all-targets -- -D warnings`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:2ade1c88180e084b780184e9ebb8af66fc4bbd00af4aa7de510a90381d5ce229`
- Timestamp: `2026-10-10T19:48:49+00:00`
- Observations: 321 regressões passaram;fetch,continuacao,limites,DNS/redirect/rebinding,extracao adversa preservados. Clippy all-targets passou. Revisao atual emreview.md.
- Evidence refs: none
- Limitations: Renova somente escopo original TK002,nao gate integral TK007/live/sintese/UI/Windows. Evidence invalidated because an input changed.

## EV-067 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test -p aura-web -p aura-gateway -p aura-app -p aura-codex -p aura-mcp; cargo clippy -p aura-web -p aura-gateway -p aura-app -p aura-codex -p aura-mcp --all-targets -- -D warnings; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e ownTEMP`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:ac7d19bc5f92616bf72148928116cb82d6721c28ccbfd23265471aed973854b6`
- Timestamp: `2026-10-10T19:48:50+00:00`
- Observations: 321 regressões Rust e21/21 web reais19,07s passaram. Matriz9 pares,permissoes/cache/historico/reinicio/fontes preservados. Clippy all-targets semavisos. Revisao atual emreview.md.
- Evidence refs: none
- Limitations: Renova somente integracao original TK003. Raciocinio/compactacao novos somente Responses;gate integral TK007,qualidade live,UI/Windows seguem pendentes. Evidence invalidated because an input changed.

## EV-068 — stale

- Ticket: `TK-007`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`, `AC-014`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e ownTEMP; cargo test -p aura-gateway --test private_calls apos ultimo ajuste somente da fixture de erro; pwsh -NoProfile -File specs/039-web-gratuita-do-agente/experiments/check_encrypted_reasoning.ps1 -CodexBin <pin> -Output experiments/encrypted-reasoning-compaction-final-result.json; python specs/039-web-gratuita-do-agente/experiments/check_storage_auditor.py; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: Windows11;app-server rust-v0.159.0 fixado;provider/MCP loopback;perfis TEMP proprios;AES-256-GCM independente .NET;sem inferencia paga
- Tested revision: `local:1c21ad19168329d8f6d279d6803140e4502da86a315b3c198fb2c5a2b0bf46e7`
- Timestamp: `2026-10-10T20:03:04+00:00`
- Observations: 504workspace verdes,34 ambientais ignoradas;25/25web reais26,17s;16HTTP e9delivery;diagnostico isolado1/1. Prova cifrada final1,74s:bytes originais no provider,decryptliteral/tamper rejection,reinicio/compactacao/historico. Auditor88arquivos/139registros/zero quatro marcadores;scan nativo semciphertext. Controle gzip/zstd2arquivos+2celulas. UIReasoningDelta/ToolCall semprivados/referencias. Erro upstream comconteudo/nonce sanitizado;somente controle publico produz dump. Clippy/fmt/diff e revisao Standards/Spec passaram.
- Evidence refs: none
- Limitations: Aprova somente entrega transitoria TK007. Fixture criptografica real representativa,sem modelo comercial/formato proprietario/replay deassinaturas nativas. Semframes comprimidos Codex;controle doauditor separado. Scan literal nao garante ausencia matematica detodainformacao derivada. UI/fontes TK004,sintese/Windows TK005,live TK006,DPI038 pendentes. Suite workspace precede ultimo ajuste somente da fixture erro;16HTTP afetados ecaso diagnostico reexecutados apos ajuste. Evidence invalidated because an input changed.

## EV-069 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e ownTEMP; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e610d5eabba750dbc82f6ba4480a37160a460787795750edf465622789b7d6f1`
- Timestamp: `2026-10-10T20:03:31+00:00`
- Observations: 504workspace verdes,34 ambientais ignoradas;25/25web reais26,17s;Clippy/fmt/diff verdes. Escopo original busca/fallback/proveniencia/limites preservado no codigo pos-fmt. Revisao atual emreview.md;gate sucessor TK007 separado emEV068.
- Evidence refs: none
- Limitations: Renovacao somente do escopo original do predecessor. UI/live/sintese/Windows/DPI e inferencia comercial nao aprovados. Ultimo ajuste so na fixture de erro deprivate_calls teve16HTTP afetados ecaso diagnostico reexecutados explicitamente. Evidence invalidated because an input changed.

## EV-070 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e ownTEMP; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:3cd91b09744dd459d793071e74ddce6fc17258fc93a0363efcb011d0cab51b4e`
- Timestamp: `2026-10-10T20:03:32+00:00`
- Observations: 504workspace verdes,34 ambientais ignoradas;25/25web reais26,17s;Clippy/fmt/diff verdes. Escopo original leitura/continuacao/DNS/redirect/limites preservado no codigo pos-fmt. Revisao atual emreview.md;gate sucessor TK007 separado emEV068.
- Evidence refs: none
- Limitations: Renovacao somente do escopo original do predecessor. UI/live/sintese/Windows/DPI e inferencia comercial nao aprovados. Ultimo ajuste so na fixture de erro deprivate_calls teve16HTTP afetados ecaso diagnostico reexecutados explicitamente. Evidence invalidated because an input changed.

## EV-071 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e ownTEMP; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:3219168585ef1e2d72aec4066e03e12f8f6b536fbb75b2848b09cb2e845ae418`
- Timestamp: `2026-10-10T20:03:32+00:00`
- Observations: 504workspace verdes,34 ambientais ignoradas;25/25web reais26,17s;Clippy/fmt/diff verdes. Escopo original integracao/9paresprotocolo-modo/cache/fontes/permissoes preservado no codigo pos-fmt. Revisao atual emreview.md;gate sucessor TK007 separado emEV068.
- Evidence refs: none
- Limitations: Renovacao somente do escopo original do predecessor. UI/live/sintese/Windows/DPI e inferencia comercial nao aprovados. Ultimo ajuste so na fixture de erro deprivate_calls teve16HTTP afetados ecaso diagnostico reexecutados explicitamente. Evidence invalidated because an input changed.

## EV-072 — stale

- Ticket: `TK-004`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`
- Procedure: `pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e ownTEMP; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check; gate deentrega transitoria EV068`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:5571f4bdc7e091d083c264d405f9fc82e32d3f8cef388d2c494f4a57371c915d`
- Timestamp: `2026-10-10T20:08:35+00:00`
- Observations: 253/253UI,33arquivos,37,05s;typecheckverde. Fontes/atividade/citacoes pt-BR/en/teclado ehistorico semnova rede;togglepersistido;falsoW999/HTML/URLfile/foreignsource rejeitados. Backend504workspace e25fixados:cancelamento/TTL/limites/registry/metadataSQLite/isolation/delete/identity/toggle/efemeridade. Entrega transitoria atualEV068,FD001resolved. Revisao Standards/Spec sembloqueante;nenhumprodutivo mudou apos provas dafat ia.
- Evidence refs: `EV-068`
- Limitations: Aprova escopo TK004 pela seam renderizada UI ebackend/pin controlado. Nao afirmajanela Windows/DPI/login real,inferencia comercial,sintese/qualidade live ouE2E nativo deTK005. 34ambientais ignoradas na suiteworkspace nao contam comoaprovacao;25fixados econtrole diagnostico foram executados separadamente. Ultimo ajuste dafixture erroGateway teve16HTTP/casoisolado reexecutados. Evidence invalidated because an input changed.

## EV-073 — stale

- Ticket: `TK-006`
- Acceptance: `AC-001`, `AC-013`
- Procedure: `cargo run -p aura-web --example quality -- 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-r9-1791663097282/report.json' 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-r9-1791663097282/targets.json'; doze buscas originais; inspecao independente; manifesto de dez HTML publicado antes de extrair; handle22833 aguardado ate exit0; git diff --check; hybrid validate`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:b057afbcf051bbdf41b5340c395b438bc07c1c7354da2d9e59132ff80d83d318`
- Timestamp: `2026-10-10T20:20:58+00:00`
- Observations: 10/12 fontes relevantes pelo conteudo;8/10 literais/origens extraidos;58HTTP/422s,exit0/phasecomplete. IBGE nao verificado403;Tauri tema errado eunsupported_content;SearXNG SearchAPI ausente mantidos. Uma sessao,semconta/chave/pago;limiares originais. Standards/Spec revisados sem bloqueante;produto/harness inalterados.
- Evidence refs: `EV-068`
- Limitations: Aprova apenas backendTK006. Requisicoes de inspecao independente nao sao contadas pelo transporteAura. Amostra nao garante SLA/disponibilidade continua/equivalencia proprietaria. Sintese3jornadas/E2EWindows/SIWC real/DPI nao aprovados;FD002/003 menores abertos. Evidence invalidated because an input changed.

## EV-074 — stale

- Ticket: `TK-005`
- Acceptance: `AC-001`, `AC-013`, `AC-014`
- Procedure: `AURA_CODEX_BIN=TEMP/aura-038-model-picker-production/profile/bin/codex/rust-v0.159.0/bin/codex-app-server.exe;AURA_E2E_DIR=TEMP;cada caso novo executado isoladamente;rustfmt --edition2024 crates/aura-app/tests/web_research.rs;cargo test -p aura-app --test web_research -- --ignored --test-threads=1;cargo clippy -p aura-app --test web_research -- -D warnings;cargo fmt --all -- --check;git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e477963154be379540c114e9c3cb08d391907e5090a252285335e6a337d62796`
- Timestamp: `2026-10-10T20:30:11+00:00`
- Observations: 5/5cadeiadeterministica realHost/pin/gateway/MCP/proveniencia/transcript em3,57s;HTML42/12/30 e2dominios/dedup;fallback429,blocked403,interruptstream,sétimaleitura limitada mesmocache. Clippy/fmt/diffcheck verdes. Nenhumcodigo produtivo mudado;casosja verdes,semredartificial.
- Evidence refs: `EV-073`
- Limitations: Providerinferenciascripted eSO/DNS/HTTP controlados,sem conta/internet. Nao aprova3jornadasdesinteseao vivo oujanelaWindows. Builddemo23928 aindaemexecucao;E2E nativo naoexecutado. AC009 semnovaprova nativa. MatrizpreexistenteEV068 ebackendEV073 conservados semsubstituir gatespendentes. Evidence invalidated because an input changed.

## EV-075 — stale

- Ticket: `TK-005`
- Acceptance: `AC-009`, `AC-014`
- Procedure: `pnpm -C apps/desktop tauri build --no-bundle --features demo; sessao23928 observada ate exit0`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:199d30da93b59d40708e8cbd4b44a788b5a967768824591543cafedb2f00bc43`
- Timestamp: `2026-10-10T20:34:13+00:00`
- Observations: Builddemo exit0,release terminou em3m30s;typescript eVite passaram;target/release/aura.exe gerado. Nao executadajanelaWebresearch nem sinteseaovivo. Handle23928 terminal,naoreiniciado.
- Evidence refs: `EV-074`
- Limitations: Somente compilacao nativa efrontend. E2Eresearch,composicao fixturecomsidecarreal e3jornadassintese ainda pendentes. OSdemo eapp-serverFake neste binario,sem inferirlogin real. WarningVite ineffective_dynamic_import preservado;semfalha. Evidence invalidated because an input changed.

## EV-076 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e TEMP; check_encrypted_reasoning.ps1 -CodexBin pin -Output encrypted-reasoning-compaction-plan10-result.json; python check_storage_auditor.py; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:d4d089c7d02f101b3cbb7916529c73450905d8b252c32d1880433d481422a3fa`
- Timestamp: `2026-10-10T20:37:41+00:00`
- Observations: Renovacao real plan10:workspaceexit0;25fixados/25,48s;UI253/23,14s etypecheck;clippyfmt verdes. AESGCM externo preservado/decrypt/tamper,diagpublicopositivo,auditorgzip/zstd/SQLite passaram;90arquivos/139linhasSQLite sem4marcadores. Revisao Standards/Spec atual sembloqueante no escopo TK-001;semalteracao produtiva antesdestasprovas.
- Evidence refs: none
- Limitations: Aprova somente o escopo de TK-001;neonativeE2E/sintese3jornadas/DPI/contaSIWC/comercial naoexercitados. Gatequalidadebackend precisa renovarTK006. Perfilcrypto semframescomprimidos;controlesauditorcomprimidos separados. Testesambientaisignorados nao saoaprovados;matrix25 executada explicitamente. Modelo/rede/OS controlados. Evidence invalidated because an input changed.

## EV-077 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e TEMP; check_encrypted_reasoning.ps1 -CodexBin pin -Output encrypted-reasoning-compaction-plan10-result.json; python check_storage_auditor.py; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:6e0b4bcd5445969ce023d611e23029ac34f5762fc6dbe4a7dff543df887f438c`
- Timestamp: `2026-10-10T20:37:42+00:00`
- Observations: Renovacao real plan10:workspaceexit0;25fixados/25,48s;UI253/23,14s etypecheck;clippyfmt verdes. AESGCM externo preservado/decrypt/tamper,diagpublicopositivo,auditorgzip/zstd/SQLite passaram;90arquivos/139linhasSQLite sem4marcadores. Revisao Standards/Spec atual sembloqueante no escopo TK-002;semalteracao produtiva antesdestasprovas.
- Evidence refs: none
- Limitations: Aprova somente o escopo de TK-002;neonativeE2E/sintese3jornadas/DPI/contaSIWC/comercial naoexercitados. Gatequalidadebackend precisa renovarTK006. Perfilcrypto semframescomprimidos;controlesauditorcomprimidos separados. Testesambientaisignorados nao saoaprovados;matrix25 executada explicitamente. Modelo/rede/OS controlados. Evidence invalidated because an input changed.

## EV-078 — stale

- Ticket: `TK-007`
- Acceptance: `AC-008`, `AC-009`, `AC-011`, `AC-012`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e TEMP; check_encrypted_reasoning.ps1 -CodexBin pin -Output encrypted-reasoning-compaction-plan10-result.json; python check_storage_auditor.py; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e0d85d6dc94d1d6cc742ea56b23682b05c8ac9f55cc64c804ace0908aed1c0f6`
- Timestamp: `2026-10-10T20:37:45+00:00`
- Observations: Renovacao real plan10:workspaceexit0;25fixados/25,48s;UI253/23,14s etypecheck;clippyfmt verdes. AESGCM externo preservado/decrypt/tamper,diagpublicopositivo,auditorgzip/zstd/SQLite passaram;90arquivos/139linhasSQLite sem4marcadores. Revisao Standards/Spec atual sembloqueante no escopo TK-007;semalteracao produtiva antesdestasprovas.
- Evidence refs: none
- Limitations: Aprova somente o escopo de TK-007;neonativeE2E/sintese3jornadas/DPI/contaSIWC/comercial naoexercitados. Gatequalidadebackend precisa renovarTK006. Perfilcrypto semframescomprimidos;controlesauditorcomprimidos separados. Testesambientaisignorados nao saoaprovados;matrix25 executada explicitamente. Modelo/rede/OS controlados. Evidence invalidated because an input changed.

## EV-079 — stale

- Ticket: `TK-004`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e TEMP; check_encrypted_reasoning.ps1 -CodexBin pin -Output encrypted-reasoning-compaction-plan10-result.json; python check_storage_auditor.py; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:1f3e0789081a47195598dbaccd9b68e801dac44fa2c313583c55b0ce816d990a`
- Timestamp: `2026-10-10T20:37:47+00:00`
- Observations: Renovacao real plan10:workspaceexit0;25fixados/25,48s;UI253/23,14s etypecheck;clippyfmt verdes. AESGCM externo preservado/decrypt/tamper,diagpublicopositivo,auditorgzip/zstd/SQLite passaram;90arquivos/139linhasSQLite sem4marcadores. Revisao Standards/Spec atual sembloqueante no escopo TK-004;semalteracao produtiva antesdestasprovas.
- Evidence refs: none
- Limitations: Aprova somente o escopo de TK-004;neonativeE2E/sintese3jornadas/DPI/contaSIWC/comercial naoexercitados. Gatequalidadebackend precisa renovarTK006. Perfilcrypto semframescomprimidos;controlesauditorcomprimidos separados. Testesambientaisignorados nao saoaprovados;matrix25 executada explicitamente. Modelo/rede/OS controlados. Evidence invalidated because an input changed.

## EV-080 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e TEMP; check_encrypted_reasoning.ps1 -CodexBin pin -Output encrypted-reasoning-compaction-plan10-result.json; python check_storage_auditor.py; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:9152334dbd37d3f30078e882ad8f96fd4d85b1af1f811a6d0d6b8e8020e53182`
- Timestamp: `2026-10-10T20:38:50+00:00`
- Observations: Provas plan10 executadas:workspaceexit0;25fixados/25,48s;UI253/23,14s/typecheck;clippyfmt verdes. AESGCM preservado/decrypt/tamper/diagpublicopositivo,auditorgzip/zstd/SQLite passaram;90arquivos/139linhasSQLite sem4marcadores. Revisao atual sembloqueante neste escopo. ACs correspondem exatamente ao ticket TK-003; corrigidas referencias insuficientes do registro anterior.
- Evidence refs: none
- Limitations: Scope TK-003 apenas;E2Enativo/sintese/DPI/SIWCcomercial naoaprovados. Probecompr imidos:controlesseparados,zero framesnoperfil. Gate liveTK006aindapendente. Runtimeprodutivointacto. Evidence invalidated because an input changed.

## EV-081 — stale

- Ticket: `TK-007`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`, `AC-014`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1 com pin rust-v0.159.0 e TEMP; check_encrypted_reasoning.ps1 -CodexBin pin -Output encrypted-reasoning-compaction-plan10-result.json; python check_storage_auditor.py; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e0d85d6dc94d1d6cc742ea56b23682b05c8ac9f55cc64c804ace0908aed1c0f6`
- Timestamp: `2026-10-10T20:38:51+00:00`
- Observations: Provas plan10 executadas:workspaceexit0;25fixados/25,48s;UI253/23,14s/typecheck;clippyfmt verdes. AESGCM preservado/decrypt/tamper/diagpublicopositivo,auditorgzip/zstd/SQLite passaram;90arquivos/139linhasSQLite sem4marcadores. Revisao atual sembloqueante neste escopo. ACs correspondem exatamente ao ticket TK-007; corrigidas referencias insuficientes do registro anterior.
- Evidence refs: none
- Limitations: Scope TK-007 apenas;E2Enativo/sintese/DPI/SIWCcomercial naoaprovados. Probecompr imidos:controlesseparados,zero framesnoperfil. Gate liveTK006aindapendente. Runtimeprodutivointacto. Evidence invalidated because an input changed.

## EV-082 — stale

- Ticket: `TK-006`
- Acceptance: `AC-001`, `AC-013`
- Procedure: `cargo run -p aura-web --example quality -- C:/Users/gabri/AppData/Local/Temp/aura-039-quality-plan10-1791664733872/report.json C:/Users/gabri/AppData/Local/Temp/aura-039-quality-plan10-1791664733872/targets.json; process7974 observed terminal exit0; independent official sources inspected; immutable ten-target manifest; metadata sanitization check; fixed-baseline Standards/Spec review`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:46942c35bd8932752209b23719dd03176b12b6b21533a9028ddc06ba341d01fb`
- Timestamp: `2026-10-10T20:46:18+00:00`
- Observations: 10/12 relevant,8/10 extraction literals and origins,58AuraHTTP/193s,phasecomplete/exit0. Museum body independently confirmed20:44:59UTC within30min; extraction targets unchanged. IBGE unverified,Tauri wrongtopic/unsupported_content,SearXNG literal absent retained. Standards/Spec no blocker.
- Evidence refs: `EV-080`
- Limitations: Backend only; independent inspection HTTP outside Aura counter. Not continuous availability,SLA,live synthesis,SIWC,nativeWindows or DPI proof. FD002/003 minor open. Evidence invalidated because an input changed.

## EV-083 — stale

- Ticket: `TK-005`
- Acceptance: `AC-001`, `AC-009`, `AC-013`, `AC-014`
- Procedure: `cargo check -p aura-desktop --features demo,e2e; pnpm -C apps/desktop tauri build --no-bundle --features demo,e2e; pnpm -C apps/desktop/e2e test -- --spec ./specs/web-research.e2e.ts using isolated TEMP/aura-039-native-web-r10-20261010, cached rust-v0.159.0, compatible WebDriver`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:72972a0faf5c7b6fceee964821bd999f14506ea191dc02a236502e0097b39a42`
- Timestamp: `2026-10-10T20:52:31+00:00`
- Observations: Check/build passed release2m54s; native test reached real citations/activities/two read sources and failed at provider payload assertion line87. Diagnose test output decoding; not valid product red. Session99948 terminalexit1.
- Evidence refs: none
- Limitations: No native complete approval; external opener not reached; cancel/toggle/live synthesis not executed. Global E2E tsc has preexisting errors; cargo deny unavailable. Evidence invalidated because an input changed.

## EV-084 — stale

- Ticket: `TK-005`
- Acceptance: `AC-001`, `AC-009`, `AC-013`, `AC-014`
- Procedure: `pnpm -C apps/desktop tauri build --no-bundle --features demo,e2e; cargo check/clippy -p aura-desktop --features demo,e2e -- -D warnings; scoped E2E tsc; pnpm -C apps/desktop/e2e test -- --spec ./specs/web-research.e2e.ts in TEMP/aura-039-native-web-r10-run12-20261010 using rust-v0.159.0 and WebView2 154.0.4258.62; cargo test -p aura-app --test web_research -- --ignored --test-threads=1 with TEMP/aura-039-plan10-integrated-final`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:fcda78e561e3fccc94cb80381ceb4b4cdb0edf5ac731c8cb8d56288ee5396060`
- Timestamp: `2026-10-10T21:11:24+00:00`
- Observations: Native3/3passed4.2s,integrated5/5passed3.43s,releasebuild2m54s,featurecheck/clippy and scopedTS passed. Real UI/model input/history/keyboard/no-refetch/pendingcancel/toggle/off attempted read with noHTTP/reenable covered; only external network/model fixtures, actual pinned chain. Failed harness runs preserved; selectors/timing/DTO corrected without product change.
- Evidence refs: none
- Limitations: Scripted model does not prove3liveLLM synthesis journeys. Browserclick executed but destination not independently confirmed. SIWC/DPI remain pending. GlobalE2E tsc preexisting errors; cargo-deny ownQA installation session31736 still running. Native web-disabled proposal suppression differs from directRPC error JSON; no error code fabricated. Evidence invalidated because an input changed.

## EV-085 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch; cargo check -p aura-desktop; cargo clippy -p aura-web --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check; cargo deny check licenses with target/qa-tools/cargo-deny/bin on process-localPATH`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:3e2d7897dd62e1c387e75a55fa611b145df6b6c5ea311624406ff632d15a85fc`
- Timestamp: `2026-10-10T21:14:26+00:00`
- Observations: 54publicsearch/fetch cases passed(29+25);2liveignored. Production desktopcheck,weball-targetsclippy/fmt/diff passed; licensesok withcargo-deny0.20.2. Lock change onlyoptionaldesktop e2e deps alreadyinworkspace; backend unchanged. Standards/Spec current no blocker.
- Evidence refs: `EV-082`
- Limitations: Scope TK001 only; not liveLLMsynthesis,nativebrowserdestination,DPI or SIWC. Current livebackend gateEV082 separate. No app-server upgrade,globalinstall or newlibrary. Evidence invalidated because an input changed.

## EV-086 — stale

- Ticket: `TK-005`
- Acceptance: `AC-001`, `AC-013`, `AC-014`
- Procedure: `Public auth_status/settings_get/models_list/diagnostics probe; native Windows build e2e no demo exit0 2m57s. pnpm -C apps/desktop/e2e exec node TEMP/aura-039-live-chatgpt-20261010-182133/synthesis.mjs; actual SIWC gpt-6-luna/default installed0.159.0, no AURA_CODEX_BIN override, three fixed ephemeral questions; messageCompleted/webSource public events; close own ephemeral conversations.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:164ff6be97180fd7df7ac7591fff973593a788ec47d2df286ab5aeb421ebca5c`
- Timestamp: `2026-10-10T21:34:30+00:00`
- Observations: Actual auth/inference/tools worked;3completed in40.2/27.9/22.9s. Museum readings unsupported; explicit caveat. Python/Rust2official pageContent and WCAG1pageContent succeeded, but model reported opaque identifiers without text/sourceId and no verified answer/citation. Synthesis gatefailed; terminalcompleted is not testpass. No settings/provider/account mutation or credential export.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-039-live-chatgpt-20261010-182133/synthesis-result.json`
- Limitations: Code-mode output rehydration is a hypothesis; exact real provider output shape not captured. Initial incomplete-runtime/harness attempts preserved and excluded from quality approval. No HTTP-counter claims. UI external-browser destination, fullresize and real150/200DPI remain pending. Evidence invalidated because an input changed.

## EV-087 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch; cargo deny check licenses (process-local PATH target/qa-tools/cargo-deny/bin)`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:9ab875a89e139806f0fe726eacdfec1611f5b11b78d4ba6839ae783bae9cbb27`
- Timestamp: `2026-10-10T21:55:02+00:00`
- Observations: 54 cases passed (25 search,29 fetch),2 live ignored; licenses ok. Backend source unchanged; review against spec r1/plan11: anonymous search/fallback/session/budget and errors preserved.
- Evidence refs: none
- Limitations: Backend scope only; live quality/synthesis and native UI are separate gates. Evidence invalidated because an input changed.

## EV-088 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `cargo test -p aura-web --test search --test fetch`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:958f0bbf2960426151e102121569f6cb0989ed6710f7fb0207f8f64c2c8d44c7`
- Timestamp: `2026-10-10T21:55:02+00:00`
- Observations: 29 fetch and25 search cases passed; review current spec/plan: safe DNS/IPpin/redirect, bounded stream/extraction, versioned Unicode pagination, cancellation/limits unchanged.
- Evidence refs: none
- Limitations: Two live tests ignored. Backend fetch scope; synthesis/native gates separate. Evidence invalidated because an input changed.

## EV-089 — stale

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `Two public CodexService red/green cases (empty caller namespace then idle resume); cargo test --workspace --exclude aura-desktop; real_app_server web:: matrix plus one handshake-only case retry; cargo test -p aura-app --test web_research -- --ignored --test-threads=1; UI tests/typecheck; workspace clippy; fmt/diff; native e2e build; normal installed pin and existing ChatGPT three unchanged questions without overrides.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:88a6f5c7aa5ca8f503519f43620b7a93da71cdcdd2b0c4f955f9cf852a5444e0`
- Timestamp: `2026-10-10T22:04:16+00:00`
- Observations: Both genuine missing-directive reds then green;19service/505workspace/253UI.24/25matrix first run; remaining case passed on isolated retry after initial handshake closed (before consumer behavior).5/5 integrated. SIWCgpt-6-luna3/3: comparison reads two official domains,WCAG correct W1,Museum explicit unsupported-content caveat with valid snippet citation. Fixed-baseline Standards/Spec review no blocker for this correction; direct tools preserve gateway privacy seam.
- Evidence refs: none
- Limitations: Initial matrix attempt lacked QA parent directory and ran no behavior; later one handshake failure retained. Other real models not synthesized. Native source destination and038DPI/resize separate; three journeys do not approve backend12/10gate or general absence of all derived private information. Evidence invalidated because an input changed.

## EV-090 — stale

- Ticket: `TK-007`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`, `AC-014`
- Procedure: `Current workspace505, fixed-pin matrix24+one handshake-only retry, public gateway tests; check_encrypted_reasoning.ps1 -CodexBin normal rust-v0.159.0 -Output encrypted-reasoning-compaction-plan11-result.json; python check_storage_auditor.py; fixed-baseline Standards/Spec review.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:6bacc394edaf496731cafeb4a75aca0650a87967e2b1e4f4c3ccfa445f8d1d6e`
- Timestamp: `2026-10-10T22:05:20+00:00`
- Observations: Current privacy gates preserved with direct Aura MCP: output/arguments/reasoning authorized transient delivery, scope/budget/cancel/expiry, normal history/restart/compaction. AESGCM external decrypt passed,tamper rejected;88files139SQLite rows0markers;public diagnostic positive/private absent;gzip/zstd auditor positive control passed. No raw fallback or broader resolver added.
- Evidence refs: `EV-089`
- Limitations: Synthetic representative ciphertext, no vendor cryptographic format claimed; no compressed frames produced by Codex. SIWC3journeys ephemeral delivery proof only; native UI/browser gates separate. Initial handshake failure preserved in EV089. Evidence invalidated because an input changed.

## EV-091 — passed

- Ticket: `TK-004`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`
- Procedure: `Current workspace505 and UI253/typecheck; fixed-pin history/privacy matrix24+one handshake retry; normal native e2e build; web-research.e2e.ts on isolated profile with real fixed-pin/gateway/MCP/WebService and external scripted adapters; inspect native-research.png.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:ddce689eefb6b7fd1ffda3899c980cbad13da9c63564c1dd8f8cd5756ed7027c`
- Timestamp: `2026-10-10T22:07:21+00:00`
- Observations: Native3/3 passed4.5s: two read sources/inline citations and history restore without requests, pending stop, global Settings toggle cancellation/no further reads/reenable. Screenshot inspected: model picker,42/12/30 answer and W1/W2 links/read metadata fit. Current reducer/i18n/IPC/parity/unsafeURL regressions passed; privacy persistent source metadata gates retained. Standards/Spec review no blocker within TK004 scope.
- Evidence refs: `EV-089`, `EV-090`
- Limitations: Native opener action executed; destination in external browser still unverified (TK005). Scripted provider/web adapters do not substitute SIWC3journeys or live search12/10; no DPI150/200 approval.

## EV-092 — stale

- Ticket: `TK-006`
- Acceptance: `AC-013`
- Procedure: `Explicit semantic revalidation of executed EV082/quality-results-2026-10-10-r10.json: compare exact aura-web and report input fingerprints, recount12searches/10reads and original10/12+8/10 thresholds, verify58HTTP/193s caps; classify consumer-only plan11 impact and fixed-baseline review. No new live network run.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:0686bcf2f44c3d37b53ce95fe780d6102c3aaa3ccbd765e5e8bfc578be9b3a8f`
- Timestamp: `2026-10-10T22:07:55+00:00`
- Observations: Original same-day backend run actual; current backend/harness identical SHA.12queries10HTML,10relevant8matched,58HTTP193s. Accepted cases/targets unchanged; no backend behavior/dependency change. EV082 remains historical stale; new classification explicit, current synthesis separateEV089.
- Evidence refs: `EV-082`
- Limitations: Executed revalidation, not fresh live queries. Original measurement2026-10-10T20:46:18Z; continuous service availability not guaranteed. Does not approve native browser/DPI/resize. Evidence invalidated because an input changed.

## EV-093 — stale

- Ticket: `TK-006`
- Acceptance: `AC-001`
- Procedure: `Current aura-web search/fetch54 cases, native SIWC3journeys with hosted web disabled; explicit same-backend anonymous quality evaluation revalidation in quality-revalidation-plan11.json.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:4b5b7580e7dd980f4713cfc8c7fed3cdcbd4fa2495b711f2642fae3e9bc62b2f`
- Timestamp: `2026-10-10T22:08:17+00:00`
- Observations: Anonymous free search sends no provider credentials or hosted search. Current search cancellation/fallback/session/quota behaviors passed; product real ChatGPT reads usable pages through free Aura tools. Original actual backend quality run same SHA and unchanged gates explicitly revalidated, not newly executed.
- Evidence refs: `EV-087`, `EV-089`, `EV-092`
- Limitations: Free service availability/quota not guaranteed; no new quality queries. Existing ChatGPT subscription supplies inference; no new search credential or paid search route. Native destination/DPI separate. Evidence invalidated because an input changed.

## EV-094 — passed

- Ticket: `TK-005`
- Acceptance: `AC-001`, `AC-013`, `AC-014`
- Procedure: `Current product native e2e build; unchanged three SIWC questions on existing ChatGPT/gpt-6-luna without prototype overrides; public Host events/source metadata evaluated against independent official pages; deterministic web_research5/5 and native web3/3; backend quality same-code semantic revalidation EV092/093.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:af6887fd680e33846353a9ca992547597f24c6c7b5c8d4ce4a3eb44e82989005`
- Timestamp: `2026-10-10T22:09:23+00:00`
- Observations: SIWC3/3 real:15.185/33.661/13.022sec completed and evaluated. WCAG fourprinciples W1 read/cited; Python/Rust separate official pages W6/W1 read/cited; Museu official snippet W3 with unsupported-content caveat, not page read. UI sources/history/stop/toggle3/3; deterministic5/5; privacy real pin current. FD004 resolved; native gate external destination remains separate.
- Evidence refs: `EV-089`, `EV-090`, `EV-091`, `EV-092`, `EV-093`
- Limitations: Other commercial models not synthesized; source registry events not guaranteed to produce every answer correct. Backend12/10 data explicitly revalidated from actual same-code same-day run, not new queries. AC009 opener destination pending;038DPI/resize separate.

## EV-095 — partial

- Ticket: `TK-005`
- Acceptance: `AC-009`
- Procedure: `Native web-research.e2e.ts3/3 and screenshot inspection after native e2e build; native opener invoked by source link; external-browser destination could not be observed under available Windows browser tooling.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:fe9d53d2cb1615ee4bb093bfc35225a37ba84562d31b64303b675a05c7b96d06`
- Timestamp: `2026-10-10T22:09:23+00:00`
- Observations: Real UI read sources,citations/history,interrupt,Settings off/reenable passed. Native opener received source action; browser final address unavailable to permitted tools.
- Evidence refs: `EV-091`
- Limitations: Requires manual confirmation or supported external-browser inspection to approve destination; do not infer it from link href or opener invocation.

## EV-096 — stale

- Ticket: `TK-005`
- Acceptance: `AC-014`
- Procedure: `cargo check -p aura-desktop; cargo clippy --workspace --all-targets -- -D warnings (including desktop, no demo/e2e flags)`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e42924d881ab476fa15dffb17cfc0fe82adb17b9690048bd88f9b883fef6bb9a`
- Timestamp: `2026-10-10T22:16:32+00:00`
- Observations: Default production desktop typecheck and fullworkspace alltargets Clippy exit0. Complements prior fixed-pin25 and native integration; optional e2e web adapters are not enabled in this production check. No source change or additional dependency.
- Evidence refs: `EV-094`
- Limitations: Compile/lint evidence complements runtime EV094, does not itself prove SIWC synthesis or external browser destination. AC009 still partialEV095. Evidence invalidated because an input changed.

## EV-097 — stale

- Ticket: `TK-005`
- Acceptance: `AC-014`
- Procedure: `pnpm -C apps/desktop tauri build --no-bundle; session77227 observed terminal exit0; hash/size of actual target/release/aura.exe recorded after completion`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:5f47c6e4db36334d418806395b6f7853d6a1a71828ef6cece6d7cc7c1da418e0`
- Timestamp: `2026-10-10T22:22:08+00:00`
- Observations: Normal default-feature native executable built3m08; no demo/e2e/bundle. Frontend typecheck/Vite and releaseRust succeeded. Complements runtime/fixed-pin/model/UI evidence, current artifact available for review.
- Evidence refs: `EV-096`
- Limitations: No installer/publication/installation; native150/200DPI and external-browser destination remain unverified. Compile alone is not broad runtime approval. Evidence invalidated because an input changed.

## EV-098 — passed

- Ticket: `TK-005`
- Acceptance: `AC-009`
- Procedure: `Teste manual orientado: usuário pediu leitura WCAG no Aura, clicou na fonte W1 e confirmou que o navegador abriu https://www.w3.org/WAI/standards-guidelines/wcag/; endereço copiado da barra após o clique.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e096b5182e99195f0f0bd57c0650495fd400b0cc15bc6f9c078cb43e6f053721`
- Timestamp: `2026-10-11T01:45:51+00:00`
- Observations: Usuário respondeu explicitamente: Sim, cliquei na fonte e abriu esse endereço. URL HTTP(S) da fonte e destino coincidem. Complementa EV091/095 sem inferir destino apenas do href.
- Evidence refs: `EV-095`
- Limitations: Atestado manual do usuário; automação do navegador continuou impedida pela ferramenta. Não prova DPI150/200.

## EV-099 — passed

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-012`
- Procedure: `Executed explicit version-0.3 input revalidation: exact directory fingerprint after reversing only HTTP/MCP client metadata in memory; compare all other prior inputs; execute workspace508/UI253/typecheck/fmt/Clippy/licenses-sources and signed production build.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e474025742c098899782d5c42c1654277850a3ac90cd30e6f352fb0727dfc279`
- Timestamp: `2026-10-11T04:24:58+00:00`
- Observations: Only workspace/package metadata and two client identifiers changed. Every backend algorithm, consumer, privacy policy, oracle and historical external result matches its previously tested fingerprint. Fresh local suites passed; historical external/native results are explicitly revalidated, not claimed as rerun.
- Evidence refs: `EV-087`
- Limitations: No fresh live-provider quality/synthesis/native run. Existing external measurements remain historical; free-service availability is not guaranteed. Does not approve 150/200 DPI or clean Windows installation.

## EV-100 — passed

- Ticket: `TK-002`
- Acceptance: `AC-004`, `AC-005`, `AC-006`, `AC-012`
- Procedure: `Executed explicit version-0.3 input revalidation: exact directory fingerprint after reversing only HTTP/MCP client metadata in memory; compare all other prior inputs; execute workspace508/UI253/typecheck/fmt/Clippy/licenses-sources and signed production build.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:57e08e762e3197ca6d32f281180cc87ab0b1e528dca7f9c7bfaea65e568538fa`
- Timestamp: `2026-10-11T04:24:58+00:00`
- Observations: Only workspace/package metadata and two client identifiers changed. Every backend algorithm, consumer, privacy policy, oracle and historical external result matches its previously tested fingerprint. Fresh local suites passed; historical external/native results are explicitly revalidated, not claimed as rerun.
- Evidence refs: `EV-088`
- Limitations: No fresh live-provider quality/synthesis/native run. Existing external measurements remain historical; free-service availability is not guaranteed. Does not approve 150/200 DPI or clean Windows installation.

## EV-101 — passed

- Ticket: `TK-003`
- Acceptance: `AC-001`, `AC-007`, `AC-008`, `AC-011`, `AC-014`
- Procedure: `Executed explicit version-0.3 input revalidation: exact directory fingerprint after reversing only HTTP/MCP client metadata in memory; compare all other prior inputs; execute workspace508/UI253/typecheck/fmt/Clippy/licenses-sources and signed production build.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:0b05f42b4167b038edc04aa2ad9b5e885ee486d79b7d5ff15ad5a92d96a204c3`
- Timestamp: `2026-10-11T04:24:59+00:00`
- Observations: Only workspace/package metadata and two client identifiers changed. Every backend algorithm, consumer, privacy policy, oracle and historical external result matches its previously tested fingerprint. Fresh local suites passed; historical external/native results are explicitly revalidated, not claimed as rerun.
- Evidence refs: `EV-089`
- Limitations: No fresh live-provider quality/synthesis/native run. Existing external measurements remain historical; free-service availability is not guaranteed. Does not approve 150/200 DPI or clean Windows installation.

## EV-102 — passed

- Ticket: `TK-007`
- Acceptance: `AC-008`, `AC-009`, `AC-010`, `AC-011`, `AC-012`, `AC-014`
- Procedure: `Executed explicit version-0.3 input revalidation: exact directory fingerprint after reversing only HTTP/MCP client metadata in memory; compare all other prior inputs; execute workspace508/UI253/typecheck/fmt/Clippy/licenses-sources and signed production build.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:9cc42d24fb345d1a5b3cc7fa818dbe6946b9e7a53660412e1041b0def425ef28`
- Timestamp: `2026-10-11T04:24:59+00:00`
- Observations: Only workspace/package metadata and two client identifiers changed. Every backend algorithm, consumer, privacy policy, oracle and historical external result matches its previously tested fingerprint. Fresh local suites passed; historical external/native results are explicitly revalidated, not claimed as rerun.
- Evidence refs: `EV-090`
- Limitations: No fresh live-provider quality/synthesis/native run. Existing external measurements remain historical; free-service availability is not guaranteed. Does not approve 150/200 DPI or clean Windows installation.

## EV-103 — passed

- Ticket: `TK-006`
- Acceptance: `AC-013`
- Procedure: `Executed explicit version-0.3 input revalidation: exact directory fingerprint after reversing only HTTP/MCP client metadata in memory; compare all other prior inputs; execute workspace508/UI253/typecheck/fmt/Clippy/licenses-sources and signed production build.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:92edc2f83a7f2a6e0a937673b1b1fa4da9bc11e8f175ff40312bb524a59c4c0c`
- Timestamp: `2026-10-11T04:24:59+00:00`
- Observations: Only workspace/package metadata and two client identifiers changed. Every backend algorithm, consumer, privacy policy, oracle and historical external result matches its previously tested fingerprint. Fresh local suites passed; historical external/native results are explicitly revalidated, not claimed as rerun.
- Evidence refs: `EV-092`
- Limitations: No fresh live-provider quality/synthesis/native run. Existing external measurements remain historical; free-service availability is not guaranteed. Does not approve 150/200 DPI or clean Windows installation.

## EV-104 — passed

- Ticket: `TK-006`
- Acceptance: `AC-001`
- Procedure: `Executed explicit version-0.3 input revalidation: exact directory fingerprint after reversing only HTTP/MCP client metadata in memory; compare all other prior inputs; execute workspace508/UI253/typecheck/fmt/Clippy/licenses-sources and signed production build.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:cd880b8eafbe70755a3e6511ff80cbbf4d54d174496ad8f0d0d017b8bf513664`
- Timestamp: `2026-10-11T04:24:59+00:00`
- Observations: Only workspace/package metadata and two client identifiers changed. Every backend algorithm, consumer, privacy policy, oracle and historical external result matches its previously tested fingerprint. Fresh local suites passed; historical external/native results are explicitly revalidated, not claimed as rerun.
- Evidence refs: `EV-093`
- Limitations: No fresh live-provider quality/synthesis/native run. Existing external measurements remain historical; free-service availability is not guaranteed. Does not approve 150/200 DPI or clean Windows installation.

## EV-105 — passed

- Ticket: `TK-005`
- Acceptance: `AC-014`
- Procedure: `Executed explicit version-0.3 input revalidation: exact directory fingerprint after reversing only HTTP/MCP client metadata in memory; compare all other prior inputs; execute workspace508/UI253/typecheck/fmt/Clippy/licenses-sources and signed production build.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:6059a1d1a40df0556640b61a9fc695cc60d5879413bf74d5d0cacfe0b1f7ac26`
- Timestamp: `2026-10-11T04:25:00+00:00`
- Observations: Only workspace/package metadata and two client identifiers changed. Every backend algorithm, consumer, privacy policy, oracle and historical external result matches its previously tested fingerprint. Fresh local suites passed; historical external/native results are explicitly revalidated, not claimed as rerun.
- Evidence refs: `EV-096`
- Limitations: No fresh live-provider quality/synthesis/native run. Existing external measurements remain historical; free-service availability is not guaranteed. Does not approve 150/200 DPI or clean Windows installation.

## EV-106 — passed

- Ticket: `TK-005`
- Acceptance: `AC-014`
- Procedure: `pnpm -C apps/desktop tauri build --ci --bundles 'nsis,msi'; terminal exit0; independent permanent-key signature verification and PE/MSI literal version 0.3.0 inspection.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:ab9956335db4afbe8bef54d280d66427afc69aec1fa08cf23120404afd84ad5c`
- Timestamp: `2026-10-11T04:25:00+00:00`
- Observations: Normal production app and both signed installers built without demo/e2e. NSIS/MSI signatures verify, corrupted bytes reject, resources report 0.3.0; permanent key/endpoint identical to shipped0.2.
- Evidence refs: none
- Limitations: No new native runtime or clean-machine installer test; original runtime evidence remains historical.
