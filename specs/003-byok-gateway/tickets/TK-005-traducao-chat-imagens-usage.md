---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 003-byok-gateway
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-003"]
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-009"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-gateway/src/translate/chat_media.rs", "crates/aura-gateway/tests/fixtures/images"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-005 — Tradução Chat Completions: imagens e uso de tokens

## Objetivo e limites

Entrega `input_image` → `image_url` (data URL ou URL), respeito a `detail`, `stream_options.include_usage` e mapeamento de `usage` para `response.completed.usage`.

Não inclui: áudio de entrada (futuro), arquivos (`input_file`) — 007 converte anexos em texto/imagens antes.

## Leitura em ordem

1. `crates/aura-gateway/tests/fixtures/codex-requests/*imagem*.json` (TK-002) → como o Codex envia `localImage` (data URL base64).
2. `crates/aura-gateway/src/translate/chat.rs` (TK-003).
3. Docs atuais: Chat `image_url` com `detail`; `stream_options: {include_usage: true}`.

## Decisões já resolvidas

- Imagens já chegam como data URL do Codex; repassar sem recodificar; se > limite do preset (ex.: 20 MB), responder erro claro.
- `include_usage` ligado quando o preset suporta; `usage.prompt_tokens/completion_tokens/reasoning_tokens` → `input_tokens/output_tokens/output_tokens_details.reasoning_tokens`.
- Liberdade local: organização em `chat_media.rs`.

## Mapa de alterações

- Novo: `crates/aura-gateway/src/translate/chat_media.rs` → `map_image_part`, `map_usage`.
- Existente: `translate/chat.rs` usa as funções.
- Novo: fixtures `images/requisicao-com-imagem.json`, `openai-chat/usage.sse`, `gemini/visao.sse`.

## Contrato técnico

- Entradas: partes `input_image{image_url|file_id, detail}`.
- Saídas: `{"type":"image_url","image_url":{"url":…, "detail":…}}`; `usage` no `response.completed`.
- Invariantes: bytes da imagem não são alterados.
- Erros: `file_id` (não suportado fora da OpenAI) → erro 400 explicativo.

## Exemplos de aceite

- **AC-009**: requisição gravada com `input_image` data URL PNG → corpo Chat contém a mesma data URL e `detail:"auto"` (snapshot); `gemini/visao.sse` → texto "quadrado vermelho"; `openai-chat/usage.sse` com `usage{prompt:120, completion:30}` → `response.completed.usage{input_tokens:120, output_tokens:30}`. Modelo sem visão → bloqueio já tratado em 002 TK-005 com `ModelSpec.supports_images=false` (verificar integração).

## Dependências e sequência de execução

Depende de: TK-003.

- [ ] TK-005.1 `map_image_part` red→green.
- [ ] TK-005.2 `map_usage` red→green.
- [ ] TK-005.3 `codex-e2e` com imagem; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-gateway chat_media`; `AURA_CODEX_BIN=… cargo nextest run -p aura-gateway --features codex-e2e chat_image_e2e`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: idem.

## Condição de retorno à planejadora

Retornar se o Codex enviar imagens como `file_id` (upload prévio) para provedores customizados.

## Relatório de saída

Relatar resultados e EV refs.
