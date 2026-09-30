# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [ ] TK-001 — TK-001 — Spike de validação + cliente JSON-RPC e supervisor do app-server
Status: `implemented` | Bloqueado por: nenhum

- [ ] TK-001.1 Spike manual no Windows com a versão candidata; registrar resultados e decidir o pin (ou retornar).
- [ ] TK-001.2 Gerar JSON Schema da versão fixada e commitar.
- [ ] TK-001.3 `RpcClient` + `InMemoryTransport`: teste de correlação request/response e server request (red→green).
- [ ] TK-001.4 `CodexBinary::ensure` com `wiremock` (checksum ok/errado).
- [ ] TK-001.5 Supervisor: AC-009 red→green; AC-010 red→green.
- [ ] TK-001.6 `CodexHome::prepare` snapshot do `config.toml`.
- [ ] TK-001.7 `codex-record` gera primeiro transcript (initialize + thread/start + turn simples) com redação verificada.

## [ ] TK-002 — TK-002 — "Continue with ChatGPT" (Sign in with ChatGPT com uso do plano) e Gateway do plano
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-002.1 `HostId` (thumbprint estável entre reinícios) red→green.
- [ ] TK-002.2 `SiwcClient` begin/complete com `wiremock` (AC-001, AC-002, AC-003) um caso por vez.
- [ ] TK-002.3 Validação do ID token (casos inválidos: iss, aud, exp, nonce, assinatura).
- [ ] TK-002.4 `TokenRefresher` (AC-026) e `ChatGptAccounts` (AC-027); revogação (AC-004).
- [ ] TK-002.5 Gateway + `ChatGptPlanUpstream` + `config.toml`; `codex-e2e`.
- [ ] TK-002.6 UI (marca aprovada, modal, menu de contas); Vitest.
- [ ] TK-002.7 Roteiro manual com conta Plus real; evidências.

## [ ] TK-003 — TK-003 — Conversa em streaming com a Persona do Aura
Status: `implemented` | Bloqueado por: TK-002

- [ ] TK-003.1 Unit `mapping` com transcript de streaming (red→green).
- [ ] TK-003.2 Unit `TurnError` um caso por vez.
- [ ] TK-003.3 Contrato AC-007 (payload) e AC-006.
- [ ] TK-003.4 UI: Vitest de markdown e interrupção; medição de pintura.
- [ ] TK-003.5 Roteiro manual real: pergunta, tabela, código, interromper, "quem é você?".

## [ ] TK-004 — TK-004 — Seletor de modelo/esforço e indicador "Usando plano ChatGPT"
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-004.1 `models(chatgpt-plan)` com `wiremock` red→green.
- [ ] TK-004.2 Persistência e regra de esforço.
- [ ] TK-004.3 UI + Vitest; manual com conta real; evidências.

## [ ] TK-005 — TK-005 — Chips de contexto e envio de imagens
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-005.1 Unit `ContextTray` (adicionar/remover/drenar) red→green.
- [ ] TK-005.2 Contrato do payload AC-013.
- [ ] TK-005.3 UI colar/arrastar + Vitest.
- [ ] TK-005.4 AC-014 + limites.
- [ ] TK-005.5 Manual real com imagem; evidências.

## [ ] TK-006 — TK-006 — Histórico, retomada e Conversas efêmeras
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-006.1 Contrato `list/read/resume` (AC-015) red→green.
- [ ] TK-006.2 Rename/pin/archive/delete + limpeza (AC-016).
- [ ] TK-006.3 Efêmera (AC-017).
- [ ] TK-006.4 UI + Vitest + E2E falso.

## [ ] TK-008 — TK-008 — Direcionar turno, compactar e uso de contexto
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-008.1 Contrato AC-022 (incluindo fallback) red→green.
- [ ] TK-008.2 Contrato AC-023.
- [ ] TK-008.3 UI + Vitest.

## [ ] TK-009 — TK-009 — Modos Chat e Tarefa com Workspace da conversa
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-009.1 Unit `thread_params`/`turn_overrides` (red→green por caso).
- [ ] TK-009.2 Contrato AC-025.
- [ ] TK-009.3 UI de modo e pastas + Vitest.
- [ ] TK-009.4 Sandbox readiness/setup; manual Win11.
- [ ] TK-009.5 Evidências.

## [ ] TK-007 — TK-007 — Aprovações, perguntas do agente e elicitation MCP
Status: `implemented` | Bloqueado por: TK-009

- [ ] TK-007.1 Contrato AC-018 red→green.
- [ ] TK-007.2 AC-019, AC-020 (um por vez).
- [ ] TK-007.3 AC-021 + badge.
- [ ] TK-007.4 UI + Vitest (atalhos `A`/`R`).
- [ ] TK-007.5 Manual real em Modo Tarefa; evidências.
