---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 009-produtividade
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-003", "AC-004"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/productivity/minibar.rs", "apps/desktop/src-tauri/src/productivity/notify.rs", "apps/desktop/src/minibar"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-002 — Minibar e notificações

## Objetivo e limites

Entrega a janela Minibar pré-criada (40 px, topo do monitor do Overlay, sempre no topo, excluída de captura), a transição Overlay→Minibar ao perder foco com turno ativo (liga o `turn_in_progress` previsto no 001 TK-005), e toasts nativos para "resposta pronta" e "Aprovação pendente" com clique que abre a Conversa.

Não inclui: sons.

## Leitura em ordem

1. `apps/desktop/src-tauri/src/overlay/{window.rs,placement.rs}` (001 TK-002/TK-005) → foco e `turn_in_progress`.
2. `crates/aura-codex/src/mapping.rs` (002 TK-003) → `TurnStarted/TurnCompleted`; `approvals.rs` (002 TK-007).
3. Docs atuais: `tauri-plugin-notification` (ações e clique no Windows) ou WinRT `ToastNotificationManager` com `AppUserModelID`.

## Decisões já resolvidas

- Minibar: largura 480 px centralizada no topo da área de trabalho; conteúdo: ponto de status (âmbar trabalhando, verde pronto, vermelho erro, azul Aprovação), prévia de 1 linha, botão parar.
- Toast só se Overlay oculto/Minibar; respeita "Não perturbe" (o Windows suprime; nada a fazer).
- Liberdade local: animação.

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/productivity/{minibar.rs,notify.rs}`.
- Novo: `apps/desktop/src/minibar/{Minibar.tsx,main.tsx}` (entrada Vite separada).
- Existente: `overlay/window.rs` → evento de foco considera turno ativo.

## Contrato técnico

- Entradas: foco perdido; eventos de turno/aprovação.
- Saídas: Minibar/Overlay; toasts.
- Invariantes: OT-003; nunca Minibar e Overlay visíveis ao mesmo tempo.

## Exemplos de aceite

- **AC-003**: E2E com app-server falso emitindo deltas lentos: clicar no Bloco de Notas → `overlay.isVisible=false`, `minibar.isVisible=true` com prévia da última linha; clicar na Minibar → Overlay visível e Minibar oculta; turno termina com Minibar visível → ponto verde.
- **AC-004**: Overlay oculto + `TurnCompleted` → `notify(TurnDone)` chamado (unit com `FakeNotifier`); manual: toast aparece e o clique abre a Conversa; `ApprovalRequested` → toast "O Aura precisa da sua aprovação".

## Dependências e sequência de execução

Depende de: 001-fundacao-overlay/TK-005 e 002-conversa-agente-codex/TK-007 (outros esforços).

- [ ] TK-002.1 Regra de transição (unit sobre estado de foco/turno) red→green.
- [ ] TK-002.2 Janela Minibar + E2E AC-003.
- [ ] TK-002.3 Notificações + manual; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-desktop minibar notify`; `pnpm -C apps/desktop e2e -- --spec e2e/minibar.spec.ts`; roteiro manual de toasts.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: toasts exigem app registrado com AUMID (instalado) — em dev, registrar atalho de teste.

## Condição de retorno à planejadora

Retornar se toasts não puderem abrir o app em modo dev/instalado por usuário sem MSIX.

## Relatório de saída

Relatar resultados e EV refs.
