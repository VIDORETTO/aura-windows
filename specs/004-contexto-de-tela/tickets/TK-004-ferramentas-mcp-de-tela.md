---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 004-contexto-de-tela
type: delivery
status: implemented
ticket_revision: 4
requires: ["TK-003"]
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-007", "AC-010", "AC-011"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-mcp", "crates/aura-capture/src/text.rs", "apps/desktop/src/conversation/PermissionCard.tsx", "apps/desktop/src-tauri/src/mcp_host.rs"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---




# TK-004 — Servidor MCP do Aura com ferramentas de tela e Permissão do agente

## Objetivo e limites

Entrega o crate `aura-mcp` (servidor streamable HTTP em `/mcp` no loopback), as tools `screen_capture`, `active_window_info`, `screen_text`, o `ToolHost` do host com cartão de Permissão no Overlay, o registro `mcp_servers.aura` no `config.toml` e o header `X-Aura-Conversation` por thread.

Não inclui: `screen_recent` (TK-007), tools de áudio (005).

## Leitura em ordem

1. `docs/adr/0006-ferramentas-do-aura-via-mcp-local.md`.
2. Resultado do spike H-003 em `docs/research/harness-evaluation.md` (002 TK-001). Se H-003 falhou, ler a decisão registrada (fallback `dynamicTools`).
3. `specs/004-contexto-de-tela/plan.md` → `aura-mcp`, identificação da Conversa, OT-004.
4. `crates/aura-gateway/src/server.rs` (002 TK-002) → montar rota e auth.
5. Docs atuais: `rmcp` (servidor streamable HTTP, tool com conteúdo `image`), Codex `mcp_servers.<id>.url/bearer_token_env_var/default_tools_approval_mode`.

## Decisões já resolvidas

- Token MCP separado (`AURA_MCP_TOKEN`), injetado no ambiente do app-server.
- `default_tools_approval_mode = "auto"` para `aura` (Política do Aura decide; ADR 0006/plano).
- `screen_capture{target:"monitor"|"window", reason:string}` → conteúdo `image/png` (≤ 2048 px) + texto com app/título; `active_window_info{}` → JSON `{process, title, url?}` (URL só de navegadores via UIA quando disponível); `screen_text{source:"auto"|"uia"|"ocr", max_chars≤20000}` → texto + origem.
- Permissão "Perguntar": o chamado MCP aguarda até 60 s a resposta do cartão; sem resposta → `denied` com motivo `timeout`.
- Concessão "nesta Conversa" grava `agent_grants(conversation, Screen, conversation-scope)`.
- Liberdade local: textos das descrições das tools (em inglês, concisos, orientados ao modelo).

## Mapa de alterações

- Novo: `crates/aura-mcp/{Cargo.toml,src/lib.rs,src/host.rs,src/tools/screen.rs}` → `McpServer::router(host)`, `trait ToolHost`.
- Novo: `crates/aura-capture/src/text.rs` → `ScreenText`, `UiaText`, `WinOcr`.
- Novo: `apps/desktop/src-tauri/src/mcp_host.rs` → `impl ToolHost` (liga política, captura, UI).
- Novo: `apps/desktop/src/conversation/PermissionCard.tsx`.
- Existente: `crates/aura-codex/src/home.rs`/`service.rs` → `mcp_servers.aura` e override de header por thread.

## Contrato técnico

- Entradas: chamadas MCP `tools/call`.
- Saídas: `CallToolResult{content:[image|text], isError}`.
- Invariantes: OT-004; toda chamada gera `access_log`.
- Erros: `{code:"denied"|"paused"|"excluded"|"unavailable"|"timeout"}` com `isError:true`.
- Efeitos: cartão de permissão no Overlay (mostra o Overlay se oculto? Não: badge na bandeja + notificação discreta; o cartão aparece ao abrir).

## Exemplos de aceite

- **AC-007**: cliente MCP de teste + `ToolHost` falso: Permissão Nunca → `isError`, `code:"denied"`; Perguntar → `request_permission` chamado com `{tool:"screen_capture", reason:"…"}`, resposta "nesta Conversa" → imagem, e 2ª chamada na mesma Conversa não pergunta; Sempre → imagem direto.
- **AC-010**: manual/`codex-e2e` com conta real: Permissão Sempre, "o que está aberto na minha tela?" → Item `mcpToolCall aura.screen_capture` completed e resposta mencionando o app visível.
- **AC-011**: integração Windows: janela WinForms/Win32 de teste com label "Texto de teste Aura 123" → `screen_text{source:"uia"}` contém o texto; janela que desenha texto em bitmap (sem UIA) → `source:"auto"` retorna via OCR com `origin:"ocr"` e contém "Aura 123".

## Dependências e sequência de execução

Depende de: TK-003; 002-conversa-agente-codex/TK-001 e TK-002 (outro esforço).

- [ ] TK-004.1 Servidor MCP + tool `active_window_info` com cliente de teste (red→green).
- [ ] TK-004.2 `screen_capture` com Permissões (AC-007) um modo por vez.
- [ ] TK-004.3 `screen_text` UIA/OCR (AC-011).
- [ ] TK-004.4 Registro no Codex + header por thread; confirmar override por thread.
- [ ] TK-004.5 Cartão de Permissão; AC-010 manual; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-mcp`; `cargo nextest run -p aura-capture --features win-integration screen_text`; roteiro manual AC-010.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: idioma de OCR não instalado no Windows → registrar e instalar pacote de idioma.

## Condição de retorno à planejadora

Retornar se H-003 falhou e `dynamicTools` também não entregar imagens, ou se overrides por thread não funcionarem para headers MCP e houver Conversas simultâneas.

## Relatório de saída

Relatar tools publicadas (schemas), resultado H-003 aplicado, EV refs.
