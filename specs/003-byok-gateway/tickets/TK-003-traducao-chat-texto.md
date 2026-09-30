---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 003-byok-gateway
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-002"]
requirement_refs: ["FR-003", "FR-004"]
acceptance_refs: ["AC-007", "AC-011"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-gateway/src/translate/chat.rs", "crates/aura-gateway/src/upstream/chat.rs", "crates/aura-gateway/src/errors.rs", "crates/aura-gateway/tests/fixtures/openai-chat", "crates/aura-gateway/tests/fixtures/groq", "crates/aura-gateway/tests/fixtures/gemini", "crates/aura-gateway/tests/fixtures/deepseek"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-003 — Tradução Chat Completions: texto em streaming e erros

## Objetivo e limites

Entrega `to_chat_request` (mensagens, instruções, parâmetros), `ChatStreamTranslator` para texto e raciocínio visível, o `ChatCompletionsAdapter` e o mapeamento de erros.

Não inclui: tool calls (TK-004), imagens e usage detalhado (TK-005).

## Leitura em ordem

1. `specs/003-byok-gateway/plan.md` → "Pontos específicos da tradução", OT-003.
2. `crates/aura-gateway/tests/fixtures/codex-requests/` (TK-002) → formato real enviado pelo Codex.
3. Docs atuais: OpenAI Chat Completions streaming; Responses streaming events (`response.created`, `response.output_item.added`, `response.content_part.added`, `response.output_text.delta`, `response.output_item.done`, `response.completed`, `response.failed`); Gemini OpenAI-compat; Groq; DeepSeek (`reasoning_content`).

## Decisões já resolvidas

- `instructions` + mensagens `developer` → uma mensagem `system` no início (concatenadas com `\n\n`).
- Ids de itens gerados (`msg_<uuid>`), `response.id` gerado por requisição.
- `reasoning_content` → `response.reasoning_summary_text.delta` em item `reasoning`.
- Mapeamento de erros conforme tabela do plano (AC-011); corte do stream sem `finish_reason` → `response.failed{code:"stream_disconnected"}` (OT-003).
- Gravar fixtures reais com chaves de teste (Groq, Gemini, DeepSeek, OpenAI Chat) e redigir.
- Liberdade local: estrutura interna do tradutor.

## Mapa de alterações

- Novo: `crates/aura-gateway/src/translate/{mod.rs,chat.rs,responses_events.rs}` → `ResponsesRequest` (subset), `ResponsesEvent`, `to_chat_request`, `ChatStreamTranslator`.
- Novo: `crates/aura-gateway/src/upstream/chat.rs` → `ChatCompletionsAdapter`.
- Novo: `crates/aura-gateway/src/errors.rs` → `UpstreamError`, `to_responses_error`.
- Novo: fixtures `openai-chat/`, `groq/`, `gemini/`, `deepseek/` (texto, reasoning, erro 429, corte).
- Novo: `crates/aura-gateway/tests/support/responses_validator.rs` → valida ordem/forma dos eventos.

## Contrato técnico

- Entradas: `ResponsesRequest` do Codex; SSE `chat.completion.chunk`.
- Saídas: SSE Responses válido.
- Invariantes: OT-003; ordem de deltas preservada.
- Erros: ver mapeamento; `retry-after` repassado em `error.message` estruturado e header.
- Efeitos: nenhum.

## Exemplos de aceite

- **AC-007**: fixture `groq/texto.sse` ("Olá" + " mundo", `finish_reason:"stop"`) → eventos `response.created`, `output_item.added(message)`, `output_text.delta("Olá")`, `output_text.delta(" mundo")`, `output_item.done`, `response.completed`; validador aprova; `codex-e2e` com Groq/Gemini/DeepSeek fixtures → turno com texto "Olá mundo".
- **AC-011**: upstream 429 com `retry-after: 20` → resposta HTTP 429 ao Codex com erro que o app-server mapeia para `UsageLimitExceeded` (confirmar no `codex-e2e`) e UI mostra "tente em 20 s"; 401 → `Unauthorized`; 503 → `HttpConnectionFailed`; corpo `{"error":{"code":"context_length_exceeded"}}` → `ContextWindowExceeded`; corte sem `finish_reason` → `response.failed`.

## Dependências e sequência de execução

Depende de: TK-002.

- [ ] TK-003.1 Validador de eventos Responses (unit com fixture OpenAI Responses real).
- [ ] TK-003.2 `to_chat_request` com requisição real do Codex (snapshot revisado) red→green.
- [ ] TK-003.3 `ChatStreamTranslator` texto (AC-007) um fixture por vez.
- [ ] TK-003.4 Erros (AC-011) um caso por vez; `codex-e2e` confirma mapeamento.
- [ ] TK-003.5 Evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-gateway translate::chat upstream::chat errors`; `AURA_CODEX_BIN=… cargo nextest run -p aura-gateway --features codex-e2e chat_text_e2e`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: ausência de chaves de teste impede gravar fixtures novas — usar as existentes e registrar.

## Condição de retorno à planejadora

Retornar se o app-server não mapear o erro 429 traduzido para uma categoria reconhecível (exigiria outro formato de erro).

## Relatório de saída

Relatar fixtures, resultados, EV refs, quirks por provedor.
