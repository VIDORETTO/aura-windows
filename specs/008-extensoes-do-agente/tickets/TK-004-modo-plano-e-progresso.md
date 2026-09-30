---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 008-extensoes-do-agente
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-012", "AC-013"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-codex/src/plan_mode.rs", "apps/desktop/src/conversation/ProgressPanel.tsx", "apps/desktop/src/conversation/PlanCard.tsx"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-004 — Modo plano e painel de Progresso

## Objetivo e limites

Entrega o Modo plano (via `collaborationMode` com feature flag, ou fallback por sandbox somente leitura + instruções), o cartão de plano com "Executar plano"/"Continuar planejando"/"Sair do modo plano", e o painel Progresso alimentado por `turn/plan/updated`.

Não inclui: Alterações/Arquivos (TK-005).

## Leitura em ordem

1. Docs atuais app-server: `collaborationMode/list`, `turn/start.collaborationMode` (experimental), item `plan`, `item/plan/delta`, `turn/plan/updated`.
2. `crates/aura-codex/src/{modes.rs,mapping.rs}` (002 TK-009/TK-003).
3. Referência de UX: Jan Cowork (plan mode, progress) em `docs/research/competitors.md`.

## Decisões já resolvidas

- Detectar suporte: `collaborationMode/list` contém preset de planejamento → usar; senão fallback.
- "Executar plano": sai do modo plano, muda para Modo Tarefa (se o usuário confirmar) e envia "Execute o plano aprovado." com o plano no contexto.
- Liberdade local: layout do painel.

## Mapa de alterações

- Novo: `crates/aura-codex/src/plan_mode.rs` → `PlanModeSupport`, `start_plan_turn`, `execute_plan`.
- Existente: `mapping.rs` → `PlanUpdated{steps}`, `PlanProposed{text}`.
- Novo: `apps/desktop/src/conversation/{ProgressPanel.tsx,PlanCard.tsx}`.

## Contrato técnico

- Entradas: `/plano` ou botão.
- Saídas: turnos em modo plano; eventos de plano/progresso.
- Invariantes: em modo plano nenhum `workspaceWrite`.
- Erros: preset indisponível → fallback silencioso (log `info`).

## Exemplos de aceite

- **AC-012**: com preset (falso) → `turn/start.collaborationMode` presente; sem preset → `sandboxPolicy: readOnly` + `developerInstructions` de planejamento; item `plan` final → `PlanCard` com os três botões; manual: nenhum comando executado.
- **AC-013**: `turn/plan/updated` com 3 passos `[pending, inProgress, completed]` → painel mostra estados; atualização seguinte muda o 2º para `completed`.

## Dependências e sequência de execução

Depende de: TK-001 (SlashMenu para `/plano`); 002-conversa-agente-codex/TK-009 (outro esforço).

- [ ] TK-004.1 Detecção de suporte + payloads (contrato) red→green.
- [ ] TK-004.2 Mapeamento `PlanUpdated`/`PlanProposed`.
- [ ] TK-004.3 UI; manual real; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-codex plan_mode mapping`; `pnpm -C apps/desktop test -- ProgressPanel PlanCard`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se o fallback não impedir execução de comandos de forma confiável.

## Relatório de saída

Relatar suporte detectado na versão fixada, resultados, EV refs.
