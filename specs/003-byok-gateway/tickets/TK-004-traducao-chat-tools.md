---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 003-byok-gateway
type: delivery
status: implemented
ticket_revision: 4
requires: ["TK-003"]
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-008"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-gateway/src/translate/chat.rs", "crates/aura-gateway/tests/fixtures"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---




# TK-004 — Tradução Chat Completions: tool calls em streaming

## Objetivo e limites

Entrega o mapeamento de `tools` (function e custom), `function_call`/`function_call_output` no histórico e deltas `tool_calls[i]` → `function_call` com argumentos em streaming, incluindo chamadas paralelas e reconversão de custom tools.

Não inclui: tools hospedadas (removidas e sinalizadas).

## Leitura em ordem

1. `specs/003-byok-gateway/plan.md` → "Pontos específicos da tradução" (tools, custom).
2. `crates/aura-gateway/tests/fixtures/codex-requests/` (TK-002) → quais tools o Codex envia e em que formato.
3. `crates/aura-gateway/src/translate/chat.rs` (TK-003).
4. Docs atuais: Responses `function_call`, `custom_tool_call`, eventos `response.function_call_arguments.delta/done`; Chat `tool_calls` streaming (index, id só no primeiro chunk).

## Decisões já resolvidas

- `custom` tool → function com schema `{"type":"object","properties":{"input":{"type":"string"}},"required":["input"]}`; na volta, se o nome pertence a uma custom tool, emitir `custom_tool_call{input}` em vez de `function_call`.
- `parallel_tool_calls` repassado; se o provedor não suporta, remover o campo (quirk por preset).
- Argumentos JSON inválidos no fim do stream → emitir o item mesmo assim (o app-server devolve erro de ferramenta ao modelo).
- Liberdade local: estrutura do acumulador por índice.

## Mapa de alterações

- Existente: `crates/aura-gateway/src/translate/chat.rs` → `map_tools`, histórico com tools, `ChatStreamTranslator` com acumulador de tool calls.
- Novo: fixtures `groq/tools-paralelas.sse`, `gemini/tool-simples.sse`, `openai-chat/custom-tool.sse`, `deepseek/tool-com-reasoning.sse`.

## Contrato técnico

- Entradas: requisições com `tools`, histórico com `function_call`/`function_call_output`.
- Saídas: eventos `output_item.added{type:function_call|custom_tool_call, call_id, name}`, `function_call_arguments.delta`, `output_item.done` com argumentos completos.
- Invariantes: `call_id` estável entre ida e volta; ordem dos itens = ordem dos índices.
- Erros: nenhum novo.

## Exemplos de aceite

- **AC-008**: `groq/tools-paralelas.sse` com `tool_calls[0]=get_weather{"city":"SP"}` e `tool_calls[1]=get_time{}` intercalados → 2 itens `function_call` com `call_id` distintos e argumentos completos; próxima requisição do Codex com `function_call_output` → mensagens `tool` com `tool_call_id` corretos (snapshot); `codex-e2e`: turno em que o modelo (fixture) chama a Ferramenta do Aura de teste e depois responde "feito" → turno `completed` com item de ferramenta. Custom tool `apply_patch` → volta como `custom_tool_call{input:"*** Begin Patch…"}`.

## Dependências e sequência de execução

Depende de: TK-003.

- [ ] TK-004.1 `map_tools` (function/custom/hospedada) red→green.
- [ ] TK-004.2 Histórico com tools (snapshot) red→green.
- [ ] TK-004.3 Stream com uma tool, depois paralelas, depois custom.
- [ ] TK-004.4 `codex-e2e` multi-turno com tool; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-gateway translate::chat::tools`; `AURA_CODEX_BIN=… cargo nextest run -p aura-gateway --features codex-e2e chat_tools_e2e`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: idem TK-003.

## Condição de retorno à planejadora

Retornar se o Codex depender de tipos de tool sem representação em Chat Completions (além de custom/hospedadas) para funções essenciais.

## Relatório de saída

Relatar tipos de tool cobertos, quirks, resultados, EV refs.
