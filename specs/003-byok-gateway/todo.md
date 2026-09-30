# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [ ] TK-001 — TK-001 — Cadastro de Provedores, Credenciais no cofre e teste de conexão
Status: `implemented` | Bloqueado por: nenhum

- [ ] TK-001.1 Integração registry + MemoryCredentialStore (AC-001/AC-002) red→green.
- [ ] TK-001.2 `test_connection` com `wiremock`, uma categoria por vez (AC-003).
- [ ] TK-001.3 `WindowsCredentialStore` + AC-004/AC-005 no Windows.
- [ ] TK-001.4 UI + Vitest.
- [ ] TK-001.5 Evidências.

## [ ] TK-002 — TK-002 — Provedores BYOK no Gateway (modo passagem) e registro no Codex
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-002.1 Rota BYOK registrada por provedor; 404 para desconhecido (red→green).
- [ ] TK-002.2 Passagem com `wiremock` (red→green).
- [ ] TK-002.3 `ConfigContributor` snapshot + env do supervisor.
- [ ] TK-002.4 `codex-e2e` AC-006; gravar fixtures de requisições do Codex; confirmar HTTP SSE sem estado (risco do plano).
- [ ] TK-002.5 Manual OpenRouter/Ollama; evidências.

## [ ] TK-003 — TK-003 — Tradução Chat Completions: texto em streaming e erros
Status: `implemented` | Bloqueado por: TK-002

- [ ] TK-003.1 Validador de eventos Responses (unit com fixture OpenAI Responses real).
- [ ] TK-003.2 `to_chat_request` com requisição real do Codex (snapshot revisado) red→green.
- [ ] TK-003.3 `ChatStreamTranslator` texto (AC-007) um fixture por vez.
- [ ] TK-003.4 Erros (AC-011) um caso por vez; `codex-e2e` confirma mapeamento.
- [ ] TK-003.5 Evidências.

## [ ] TK-007 — TK-007 — Descoberta de modelos, capacidades e troca de Provedor
Status: `implemented` | Bloqueado por: TK-002

- [ ] TK-007.1 `discover_models` por preset (um por vez) red→green.
- [ ] TK-007.2 Seletor agrupado + Vitest.
- [ ] TK-007.3 `continue_with_provider` contrato + UI.
- [ ] TK-007.4 Evidências.

## [ ] TK-004 — TK-004 — Tradução Chat Completions: tool calls em streaming
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-004.1 `map_tools` (function/custom/hospedada) red→green.
- [ ] TK-004.2 Histórico com tools (snapshot) red→green.
- [ ] TK-004.3 Stream com uma tool, depois paralelas, depois custom.
- [ ] TK-004.4 `codex-e2e` multi-turno com tool; evidências.

## [ ] TK-005 — TK-005 — Tradução Chat Completions: imagens e uso de tokens
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-005.1 `map_image_part` red→green.
- [ ] TK-005.2 `map_usage` red→green.
- [ ] TK-005.3 `codex-e2e` com imagem; evidências.

## [ ] TK-006 — TK-006 — Adaptador Anthropic Messages
Status: `implemented` | Bloqueado por: TK-004, TK-005

- [ ] TK-006.1 `to_messages_request` (sistema, mesclagem, tools, imagem) um caso por vez.
- [ ] TK-006.2 Stream texto → thinking → tool_use.
- [ ] TK-006.3 Erros; `codex-e2e`; evidências.
