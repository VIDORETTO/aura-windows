---
schema: hybrid/ticket
schema_version: 1.0
id: TK-007
effort: 003-byok-gateway
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-002"]
requirement_refs: ["FR-005"]
acceptance_refs: ["AC-012", "AC-013", "AC-014"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-gateway/src/discovery.rs", "apps/desktop/src/conversation/ModelPicker.tsx", "apps/desktop/src/settings/providers/ModelsEditor.tsx", "crates/aura-codex/src/switch_provider.rs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-007 — Descoberta de modelos, capacidades e troca de Provedor

## Objetivo e limites

Entrega `discover_models` com heurísticas de capacidade, editor manual de modelos, `ModelPicker` agrupado por Provedor com ícones de capacidade, e o fluxo "Usar outro provedor" a partir do banner de limite.

Não inclui: roteamento automático entre provedores.

## Leitura em ordem

1. `specs/003-byok-gateway/plan.md` → discovery, OT-004.
2. `apps/desktop/src/conversation/ModelPicker.tsx` (002 TK-004).
3. `apps/desktop/src/conversation/ErrorBanner.tsx` (002 TK-003) → ação "Usar outro provedor".
4. Docs atuais: `/models` de OpenAI, OpenRouter (inclui `architecture.input_modalities`, `supported_parameters`), Anthropic `/v1/models`, Gemini OpenAI-compat `/models`, Ollama `/api/show` (capabilities).

## Decisões já resolvidas

- Capacidades: usar metadados do provedor quando existirem (OpenRouter, Ollama); senão heurística por nome (`vision|vl|4o|gemini|claude` → imagem) marcada como "estimado" e editável.
- `ModelSpec` salvo em `providers.models_json`; atualização manual por botão.
- Troca de provedor: nova Conversa com `developerInstructions` extra contendo resumo; resumo = pedido ao modelo atual se ainda houver cota, senão últimas 20 mensagens truncadas a 8k caracteres.
- Liberdade local: ícones e layout.

## Mapa de alterações

- Existente: `crates/aura-gateway/src/discovery.rs` → `discover_models`, `estimate_capabilities`.
- Novo: `apps/desktop/src/settings/providers/ModelsEditor.tsx`.
- Existente: `apps/desktop/src/conversation/ModelPicker.tsx` → grupos e ícones.
- Novo: `crates/aura-codex/src/switch_provider.rs` → `continue_with_provider(conversation, provider, model) -> ConversationId`.

## Contrato técnico

- Entradas: provedor; seleção do usuário.
- Saídas: `Vec<ModelSpec>`; nova Conversa.
- Invariantes: ChatGPT sempre primeiro grupo; modelos sem `supports_tools` exibem aviso "sem ferramentas".
- Erros: `DiscoveryError::{Unsupported, Upstream(category)}` → editor manual.

## Exemplos de aceite

- **AC-012**: registry com ChatGPT + OpenRouter (2 modelos) + Groq (1) → seletor com 3 grupos na ordem ChatGPT, OpenRouter, Groq; ícones conforme `ModelSpec`; escolher `groq/llama-x` → próxima `thread/start{modelProvider:"aura-groq…", model:"llama-x"}`.
- **AC-013**: `wiremock` OpenRouter `/models` com `input_modalities:["text","image"]` → `supports_images=true`; provedor personalizado sem `/models` (404) → editor manual; adicionar `meu-modelo` com imagem desmarcada → salvo.
- **AC-014**: banner `UsageLimitExceeded` (falso 002) → "Usar outro provedor" → seletor filtrado a BYOK → escolher → nova Conversa criada com `developerInstructions` contendo "Resumo da conversa anterior:" e a UI navega para ela.

## Dependências e sequência de execução

Depende de: TK-002.

- [ ] TK-007.1 `discover_models` por preset (um por vez) red→green.
- [ ] TK-007.2 Seletor agrupado + Vitest.
- [ ] TK-007.3 `continue_with_provider` contrato + UI.
- [ ] TK-007.4 Evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-gateway discovery -p aura-codex switch_provider`; `pnpm -C apps/desktop test -- ModelPicker ModelsEditor`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se o Codex rejeitar modelos desconhecidos no provedor customizado sem metadados que não conseguimos fornecer por `config`.

## Relatório de saída

Relatar resultados e EV refs.
