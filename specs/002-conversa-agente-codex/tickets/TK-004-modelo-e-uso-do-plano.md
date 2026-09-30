---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 002-conversa-agente-codex
type: delivery
status: implemented
ticket_revision: 4
requires: ["TK-003"]
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-011", "AC-012"]
spec_revision: 2
plan_revision: 2
owned_areas: ["apps/desktop/src/conversation/ModelPicker.tsx", "apps/desktop/src/conversation/PlanUsageIndicator.tsx", "crates/aura-codex/src/models.rs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-004 — Seletor de modelo/esforço e indicador "Usando plano ChatGPT"

## Objetivo e limites

Entrega `CodexService::models(provider)` (para o plano ChatGPT: `GET /v1/models` pelo Gateway com a conta ativa, filtrando `visibility:"list"` e mantendo a ordem do servidor), esforços de raciocínio por modelo, persistência da escolha, e o indicador "Usando plano ChatGPT" + "Gerenciar uso" junto ao seletor.

Não inclui: modelos de Provedores BYOK (003 TK-007 estende o mesmo seletor).

## Leitura em ordem

1. `https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference.md` → formato de `/v1/models` (`models[]`, `slug`, `display_name`, `visibility`).
2. `https://developers.openai.com/siwc/ui-ux-guidelines.md` → "Show when the ChatGPT plan is in use".
3. `crates/aura-gateway/src/upstream/chatgpt_plan.rs` (TK-002) → rota `/models`.
4. `crates/aura-codex/src/service.rs` (TK-003) → `start` recebe `model`/`effort`.
5. `crates/aura-core/src/settings.rs` (001 TK-004).

## Decisões já resolvidas

- Cache de modelos por conta por 10 min; invalidar ao trocar de conta (`account_changed`).
- Esforços: usar metadados de `/v1/models` quando presentes; senão consultar `model/list` do app-server apenas como catálogo de capacidades (não como autorização); se ausentes, oferecer `low/medium/high` e deixar o turno validar.
- Esforço salvo não suportado → padrão do modelo.
- Indicador: texto "Usando plano ChatGPT" + botão "Gerenciar uso" (abre `https://chatgpt.com/settings/usage` no navegador).
- Liberdade local: visual.

## Mapa de alterações

- Novo: `crates/aura-codex/src/models.rs` → `ModelInfo{id, display_name, efforts, default_effort, input_modalities}`, `models(provider)`.
- Existente: `aura-core::settings` → `default_model`, `default_effort` por Provedor.
- Novo: `apps/desktop/src/conversation/{ModelPicker.tsx,PlanUsageIndicator.tsx}`.

## Contrato técnico

- Entradas: Provedor ativo.
- Saídas: lista de modelos; seleção persistida.
- Invariantes: nunca listar modelos de outra conta; ordem do servidor preservada.
- Erros: falha de listagem → último cache + aviso.

## Exemplos de aceite

- **AC-011**: `wiremock` `/v1/models` → `{models:[{slug:"m-a",display_name:"Modelo A",visibility:"list"},{slug:"m-x",visibility:"hidden"},{slug:"m-b",display_name:"Modelo B",visibility:"list"}]}` → seletor mostra "Modelo A", "Modelo B" nessa ordem; escolher B → próximo `thread/start{model:"m-b"}`; reiniciar → B mantido; trocar de conta → lista recarregada.
- **AC-012**: Conversa no plano ChatGPT → indicador visível com link para `https://chatgpt.com/settings/usage`; Conversa BYOK (003) → indicador mostra o nome do Provedor.

## Dependências e sequência de execução

Depende de: TK-003.

- [ ] TK-004.1 `models(chatgpt-plan)` com `wiremock` red→green.
- [ ] TK-004.2 Persistência e regra de esforço.
- [ ] TK-004.3 UI + Vitest; manual com conta real; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-codex models`; `pnpm -C apps/desktop test -- ModelPicker PlanUsageIndicator`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: conta de teste com poucos modelos.

## Condição de retorno à planejadora

Retornar se `/v1/models` não trouxer informação suficiente para esforços e o app-server rejeitar esforços não suportados de forma que prejudique a UX.

## Relatório de saída

Relatar resultados e EV refs.
