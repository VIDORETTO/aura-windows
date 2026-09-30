---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 003-byok-gateway
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-003", "FR-006"]
acceptance_refs: ["AC-006"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-gateway/src/server.rs", "crates/aura-gateway/src/upstream/passthrough.rs", "crates/aura-gateway/src/codex_config.rs", "crates/aura-gateway/tests/fixtures/codex-requests", "crates/aura-gateway/tests/fixtures/openai-responses"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-002 — Provedores BYOK no Gateway (modo passagem) e registro no Codex

## Objetivo e limites

Estende o servidor loopback criado em 002 TK-002 com a rota `/p/{id}/v1/responses` para Provedores BYOK em modo passagem, o `ConfigContributor` que registra `aura-<id>` no `config.toml`, a injeção de `AURA_GATEWAY_TOKEN` no ambiente do app-server e `thread/start.modelProvider` para Conversas BYOK. Grava as requisições reais do Codex como fixtures para os tradutores.

Não inclui: tradução (TK-003+).

## Leitura em ordem

1. `docs/adr/0003-gateway-responses-local-para-byok.md` (revisado: modo passagem).
2. `specs/003-byok-gateway/plan.md` → server, codex_config, OT-002, riscos (WebSocket/estado).
3. `crates/aura-codex/src/home.rs` e `supervisor.rs` (002 TK-001) → `ConfigContributor`, ambiente do filho.
4. Docs atuais: https://developers.openai.com/codex/config-advanced (model_providers, `env_key`, `stream_idle_timeout_ms`, `request_max_retries`), OpenAI Responses API streaming.

## Decisões já resolvidas

- Reusar auth/origin, porta e token do servidor de 002 TK-002.
- Config por provedor: `[model_providers.aura-<id>] name="<nome>" base_url="http://127.0.0.1:<porta>/p/<id>/v1" wire_api="responses" env_key="AURA_GATEWAY_TOKEN" stream_idle_timeout_ms=120000 request_max_retries=2`.
- Porta muda a cada execução → `config.toml` reescrito no `prepare` antes de cada spawn.
- Passagem: reescreve URL para `<base_url do provedor>/responses`, remove `Authorization` recebido, injeta a Credencial no formato do preset, repassa corpo e stream sem alteração.
- Liberdade local: organização do roteador axum.

## Mapa de alterações

- Existente (002 TK-002): `crates/aura-gateway/src/server.rs` → registrar rotas BYOK por provedor do registry.
- Existente (002 TK-002): `upstream/mod.rs` (`UpstreamAdapter`). Novo: `upstream/passthrough.rs` → `Passthrough` (injeção da Credencial conforme preset).
- Novo: `crates/aura-gateway/src/codex_config.rs` → `GatewayConfigContributor`.
- Existente: `crates/aura-codex/src/supervisor.rs` → env `AURA_GATEWAY_TOKEN`; `service.rs` → `start(opts{provider: ProviderRef})` passa `modelProvider`.
- Novo: fixtures `tests/fixtures/codex-requests/*.json` (gravadas do Codex real apontado para o gateway em modo captura) e `openai-responses/*.sse`.

## Contrato técnico

- Entradas: `POST /p/{id}/v1/responses` com `Authorization: Bearer <token>`.
- Saídas: stream SSE do upstream repassado byte a byte.
- Invariantes: OT-002; provedor desconhecido → 404 JSON no formato de erro Responses.
- Erros: token ausente/errado → 401; `Origin` presente → 403; upstream indisponível → 502 com corpo de erro Responses.
- Efeitos: nenhum log de corpo de requisição (apenas método, id do provedor, status, duração).

## Exemplos de aceite

- **AC-006**: `codex-e2e`: app-server real com `modelProvider:"aura-oa"` → Gateway → `wiremock` servindo `openai-responses/texto-simples.sse` com verificação de header `Authorization: Bearer sk-test-OA` → turno `completed` com o texto da fixture. Sem token → 401; com `Origin: http://evil` → 403. Manual: OpenRouter (Responses) e Ollama `/v1/responses` com modelo local.
- Captura: roteiro `tools/codex-record --gateway-capture` salva as 3 primeiras requisições do Codex (texto, com tools de sandbox, com imagem) redigidas.

## Dependências e sequência de execução

Depende de: TK-001; 002-conversa-agente-codex/TK-002 e TK-003 (outro esforço).

- [ ] TK-002.1 Rota BYOK registrada por provedor; 404 para desconhecido (red→green).
- [ ] TK-002.2 Passagem com `wiremock` (red→green).
- [ ] TK-002.3 `ConfigContributor` snapshot + env do supervisor.
- [ ] TK-002.4 `codex-e2e` AC-006; gravar fixtures de requisições do Codex; confirmar HTTP SSE sem estado (risco do plano).
- [ ] TK-002.5 Manual OpenRouter/Ollama; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-gateway server passthrough`; `AURA_CODEX_BIN=<caminho> cargo nextest run -p aura-gateway --features codex-e2e passthrough_e2e`.
- Estado esperado: verdes; fixtures gravadas e revisadas.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: `AURA_CODEX_BIN` ausente → `codex-e2e` `not_run` com limitação.

## Condição de retorno à planejadora

Retornar se o Codex usar WebSocket, `previous_response_id` ou outro estado de servidor para provedores customizados, inviabilizando tradução sem estado.

## Relatório de saída

Relatar formato real das requisições do Codex (resumo), fixtures gravadas, resultados, EV refs.
