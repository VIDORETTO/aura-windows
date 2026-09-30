---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 003-byok-gateway
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-004", "TK-005"]
requirement_refs: ["FR-003", "FR-004"]
acceptance_refs: ["AC-010"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-gateway/src/translate/anthropic.rs", "crates/aura-gateway/src/upstream/anthropic.rs", "crates/aura-gateway/tests/fixtures/anthropic"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-006 — Adaptador Anthropic Messages

## Objetivo e limites

Entrega `to_messages_request` e `AnthropicStreamTranslator` cobrindo texto, `thinking` (→ resumo de raciocínio), `tool_use`/`tool_result` (inclusive paralelos), imagens (`source.base64`) e usage, mais erros específicos (`overloaded_error`, `rate_limit_error`).

Não inclui: prompt caching explícito da Anthropic (otimização futura).

## Leitura em ordem

1. `crates/aura-gateway/src/translate/chat.rs` e `chat_media.rs` (TK-003–TK-005) → padrões de acumulador e validador.
2. `crates/aura-gateway/tests/fixtures/codex-requests/` (TK-002).
3. Docs atuais (Context7/claude-api): Messages API streaming (`message_start`, `content_block_start{type:text|tool_use|thinking}`, `content_block_delta{text_delta|input_json_delta|thinking_delta}`, `message_delta{stop_reason, usage}`), `system` top-level, `max_tokens` obrigatório, header `anthropic-version`.

## Decisões já resolvidas

- `system` = instruções concatenadas; `max_tokens` = `ModelSpec.max_output` ou 8192.
- Mensagens consecutivas do mesmo papel são mescladas (exigência da API).
- `function_call_output` → bloco `tool_result` em mensagem `user`.
- `thinking` habilitado só se o `ModelSpec.supports_reasoning` e o esforço ≠ none; `budget_tokens` derivado do esforço (low 2k, medium 8k, high 24k).
- Liberdade local: organização do tradutor.

## Mapa de alterações

- Novo: `crates/aura-gateway/src/translate/anthropic.rs`, `upstream/anthropic.rs`.
- Novo: fixtures `anthropic/{texto,thinking,tool-use-paralelo,imagem,overloaded}.sse`.
- Existente: `errors.rs` → casos Anthropic.

## Contrato técnico

- Entradas: `ResponsesRequest`.
- Saídas: SSE Responses válido (OT-003).
- Invariantes: alternância user/assistant válida; `tool_use.id` ↔ `call_id`.
- Erros: `overloaded_error` → 503/`HttpConnectionFailed`; `rate_limit_error` → 429.

## Exemplos de aceite

- **AC-010**: `anthropic/tool-use-paralelo.sse` com dois `tool_use` e `input_json_delta` fragmentado → 2 `function_call` com JSON completo; `anthropic/thinking.sse` → item `reasoning` com resumo e depois texto; `anthropic/imagem` requisição com PNG → bloco `{"type":"image","source":{"type":"base64","media_type":"image/png","data":…}}`; `codex-e2e` com tool + resposta final → `completed`.

## Dependências e sequência de execução

Depende de: TK-004, TK-005.

- [ ] TK-006.1 `to_messages_request` (sistema, mesclagem, tools, imagem) um caso por vez.
- [ ] TK-006.2 Stream texto → thinking → tool_use.
- [ ] TK-006.3 Erros; `codex-e2e`; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-gateway anthropic`; `AURA_CODEX_BIN=… cargo nextest run -p aura-gateway --features codex-e2e anthropic_e2e`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: idem.

## Condição de retorno à planejadora

Retornar se o histórico do Codex não puder ser convertido numa alternância válida de papéis sem perder informação.

## Relatório de saída

Relatar resultados, quirks, EV refs.
