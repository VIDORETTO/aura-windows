---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 003-byok-gateway
revision: 1
spec_revision: 1
status: ready
---

# Plan: BYOK com endpoint customizado

## Summary

Um crate `aura-gateway` com três partes: (1) `ProviderRegistry` + `CredentialStore` (Windows Credential Manager via `keyring`), (2) servidor HTTP loopback (`axum`) que recebe requisições Responses API do Codex em `/p/<id>/v1/responses` e as envia ao provedor por um `UpstreamAdapter` (passagem, Chat Completions ou Anthropic), (3) tradutores puros request/stream testados por fixtures reais. O host registra cada Provedor no `config.toml` do `CODEX_HOME` pelo `ConfigContributor` do esforço 002.

## Technical context

- Language/runtime: Rust, `tokio`, `axum`, `reqwest` (stream, rustls + certificados do sistema via `rustls-platform-verifier`), `eventsource-stream`/parser SSE próprio, `serde_json`, `keyring` (backend Windows Credential Manager), `wiremock` e `insta` nos testes.
- Dependencies externas (consultar docs atuais no TK de cada adaptador): OpenAI Responses API (streaming events `response.*`), OpenAI Chat Completions streaming (`chat.completion.chunk`, `tool_calls[].index`), Anthropic Messages streaming (`message_start`, `content_block_start/delta/stop`, `input_json_delta`, `thinking_delta`), Gemini endpoint OpenAI-compatível (`https://generativelanguage.googleapis.com/v1beta/openai/`).
- Storage/data: tabela `providers(id, name, preset, wire, base_url, extra_headers_json, models_json, status, last_error, updated_at)`; segredo em Credential Manager: alvo `Aura/provider/<id>`.
- Test command: `cargo nextest run -p aura-gateway`; integração com app-server real atrás de `AURA_CODEX_BIN` (Linux/Windows): `cargo nextest run -p aura-gateway --features codex-e2e`.
- Target/platform: Windows (cofre), lógica de tradução em qualquer SO.

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-006; AC-001–AC-014.

## Modules, interfaces, consumers, and seams

| Module | Interface | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `aura-gateway::registry` | `ProviderRegistry::{list, get, upsert(ProviderDraft) -> Provider, remove(id), set_status}`; `Provider{id, name, preset, wire: Wire{Responses, ChatCompletions, Anthropic}, base_url, extra_headers, models: Vec<ModelSpec>}` | UI de Provedores, `ConfigContributor`, servidor | SQLite real em tempdir |
| `aura-gateway::credentials` | `trait CredentialStore { put(id, Secret), get(id) -> Option<Secret>, delete(id) }` | registry, servidor | Adapters reais: `WindowsCredentialStore` (produção, via `keyring`) e `MemoryCredentialStore` (testes/Linux) |
| `aura-gateway::server` (existente, criado em 002 TK-002) | `Gateway::start(...) -> GatewayHandle{port, token}`; este esforço acrescenta rotas `POST /p/{id}/v1/responses` e `GET /p/{id}/v1/models` para Provedores BYOK | supervisor do app-server (002) | Requisições HTTP reais no loopback |
| `aura-gateway::upstream` | `trait UpstreamAdapter { async fn stream(&self, req: ResponsesRequest, ctx) -> Result<impl Stream<Item = ResponsesEvent>, UpstreamError> }`; `Passthrough`, `ChatCompletionsAdapter`, `AnthropicAdapter` | servidor | `wiremock` servindo fixtures SSE reais |
| `aura-gateway::translate::chat` | `to_chat_request(&ResponsesRequest, &ModelSpec) -> ChatRequest`; `ChatStreamTranslator::push(chunk) -> Vec<ResponsesEvent>`; `finish()` | `ChatCompletionsAdapter` | Puro: fixtures de requisição do Codex (gravadas) e SSE de provedores (gravadas) |
| `aura-gateway::translate::anthropic` | `to_messages_request`, `AnthropicStreamTranslator` | `AnthropicAdapter` | Idem |
| `aura-gateway::errors` | `UpstreamError → (http status, Responses error body)` compatível com o mapeamento de `codexErrorInfo` | servidor | Unit |
| `aura-gateway::discovery` | `discover_models(provider) -> Vec<ModelSpec>` + heurísticas de capacidade | UI | `wiremock` |
| `aura-gateway::codex_config` | `impl ConfigContributor` → `[model_providers.aura-<id>] name, base_url, wire_api="responses", env_key="AURA_GATEWAY_TOKEN", request_max_retries, stream_idle_timeout_ms` | `CodexHome::prepare` (002) | Snapshot TOML |
| `apps/desktop/src/settings/providers/` | `ProvidersSection`, `ProviderForm`, `TestConnectionButton`; `ModelPicker` agrupado | usuário | Vitest + mockIPC |

Pontos específicos da tradução (fixados após gravar requisições reais do Codex no TK-002):

- `input[]` do Responses → `messages[]`: `message{role:user|developer|assistant, content:[input_text|input_image|output_text]}` → mensagens Chat (`developer`/`instructions` → `system`); `function_call` → `assistant.tool_calls[]`; `function_call_output` → `tool` message; `reasoning` → descartado (ou `reasoning_content` se o provedor aceitar de volta).
- `tools[]`: `function` → `function`; tipo `custom` (ex.: `apply_patch` freeform) → função com parâmetro único `{"input": string}` e reconversão para `custom_tool_call` na volta; tools hospedadas (`web_search`) → removidas e sinalizadas em `x-aura-dropped-tools`.
- Streaming: `delta.content` → `response.output_text.delta`; `delta.tool_calls[i]` → `response.output_item.added(function_call)` + `response.function_call_arguments.delta` por índice; `finish_reason` → `response.output_item.done` + `response.completed{usage}`; `reasoning_content` → `response.reasoning_summary_text.delta`.
- Metadados de modelo desconhecido para o Codex (janela de contexto, saída máxima) vão por `thread/start.config` (`model_context_window`, `model_max_output_tokens`) a partir do `ModelSpec` — confirmar chaves na versão fixada.

## Chosen approach and alternatives

- **Gateway para todos os Provedores BYOK** (passagem ou tradução) — ADR 0003 revisado: chaves nunca saem do host.
- **Tradução orientada a fixtures gravadas** em vez de só ler documentação: grava-se o que o Codex realmente envia e o que cada provedor realmente devolve.
- **Token de loopback por execução** passado ao app-server por `AURA_GATEWAY_TOKEN`; requisições sem ele ou com `Origin` → 403.
- Alternativa descartada: LiteLLM (Python) e `auth.command` para provedores Responses.

## Data, compatibility, and external dependencies

- Migração `0003_providers.sql`.
- Remover Provedor não apaga rollouts antigos; `conversations_meta.provider_id` aponta para id inexistente → UI "Provedor removido".
- Trocar de Provedor = nova Conversa com resumo (AC-014): resumo gerado pedindo ao modelo atual (se disponível) ou concatenando as últimas mensagens truncadas (fallback determinístico).
- Respeitar `HTTPS_PROXY`/proxy do sistema (WinHTTP) no `reqwest`.

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001, AC-002 | Integração registry + Vitest | Spec | Upsert com preset/personalizado; UI mostra `••••abcd` |
| AC-003 | `wiremock` | Spec | 401, 404, atraso 11 s, TLS inválido (servidor rustls com cert autoassinado) → categorias |
| AC-004 | Integração | Spec | `remove` → `CredentialStore::get` = None; teste Windows com cofre real |
| AC-005 | Teste Windows + busca em arquivos | Spec | Salvar `sk-test-BYOK-987`; varrer `%LOCALAPPDATA%\Aura` → 0 ocorrências |
| AC-006 | Integração com app-server real (`codex-e2e`) + manual | Spec | Codex → Gateway (passagem) → wiremock Responses com fixture gravada da OpenAI; manual com OpenRouter e Ollama |
| AC-007 | Unit tradutor + `codex-e2e` | Fixtures reais | SSE Groq/Gemini/DeepSeek → eventos Responses válidos (validador de sequência) → turno completo no Codex |
| AC-008 | Unit + `codex-e2e` | Fixtures reais | Fixture com 2 tool calls paralelas → `function_call` × 2; resultado → continuação |
| AC-009 | Unit | Fixture | `input_image` → `image_url` data URL ou URL; modelo sem visão → bloqueio (002) |
| AC-010 | Unit + `codex-e2e` | Fixtures Anthropic | Texto, tool_use, imagem, thinking |
| AC-011 | Unit `errors` | Tabela do plano | 401→Unauthorized; 429+`retry-after: 20`→UsageLimit com 20 s; 5xx→HttpConnectionFailed; `context_length_exceeded`→ContextWindowExceeded |
| AC-012, AC-013 | `wiremock` + Vitest | Spec | `/models` → lista; sem `/models` → entrada manual |
| AC-014 | Contrato (falso 002) + Vitest | Spec | Banner de limite → seletor → nova Conversa com contexto resumido |

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `crates/aura-gateway/src/{registry.rs,credentials.rs,upstream/passthrough.rs,upstream/chat.rs,upstream/anthropic.rs,translate/chat.rs,translate/anthropic.rs,errors.rs,discovery.rs,codex_config.rs}` | new | ver tabela | Gateway BYOK | 2026-09-29 |
| `crates/aura-gateway/src/{server.rs,upstream/mod.rs}` | existing (002 TK-002) | rotas BYOK | Servidor compartilhado com o plano ChatGPT | 002 TK-002 |
| `crates/aura-gateway/tests/fixtures/{codex-requests,openai-chat,groq,gemini,deepseek,anthropic,openai-responses}/` | new | fixtures | Contrato | 2026-09-29 |
| `crates/aura-store/src/migrations/0003_providers.sql` | new | `providers` | Dados | 2026-09-29 |
| `apps/desktop/src/settings/providers/**` | new | UI | Cadastro | 2026-09-29 |
| `apps/desktop/src/conversation/ModelPicker.tsx` | existing (002 TK-004) | agrupamento | Seletor | 002 TK-004 |
| `crates/aura-codex/src/home.rs` | existing (002 TK-001) | `ConfigContributor` | Registro no Codex | 002 TK-001 |

## Derived technical obligations

- **OT-001** → FR-001/AC-005: `Provider` e `ProviderDraft` nunca contêm a Credencial; ela trafega só como `Secret` direto para `CredentialStore`.
- **OT-002** → FR-006: o servidor faz bind só em `127.0.0.1`, porta efêmera, e valida `Authorization: Bearer <token>` com comparação em tempo constante.
- **OT-003** → FR-004: todo tradutor emite sequência Responses válida (`response.created` … `response.completed|failed`) mesmo quando o upstream corta o stream (gera `response.failed`).
- **OT-004** → FR-005: `ModelSpec{id, context_window?, max_output?, supports_images, supports_tools, supports_reasoning}` é a fonte das capacidades que 002 usa para bloquear imagens.

## Risks and gates

- O Codex pode usar recursos da Responses API que não têm equivalente (ex.: `previous_response_id`, `store`, WebSocket). Mitigação: TK-002 grava requisições reais e confirma que o Codex usa HTTP SSE sem estado para provedores customizados; se exigir WebSocket, retornar à planejadora.
- Provedores "OpenAI-compatíveis" divergem em detalhes de streaming de tool calls. Mitigação: fixtures por provedor e quirks isolados por preset.
- G2: satisfeito. G3: TK-001 depende de 001 TK-006 (cofre/secret) e 002 TK-002 (abstração `CredentialStore` e servidor do Gateway) concluídos; TK-007 depende de 002 TK-004 (seletor).
