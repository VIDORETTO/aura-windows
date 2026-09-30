---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 002-conversa-agente-codex
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-002"]
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-005", "AC-006", "AC-007", "AC-008"]
spec_revision: 2
plan_revision: 2
owned_areas: ["crates/aura-codex/src/mapping.rs", "crates/aura-codex/src/service.rs", "apps/desktop/src-tauri/resources/persona", "apps/desktop/src/conversation", "crates/aura-store/src/migrations/0002_conversations.sql"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-003 — Conversa em streaming com a Persona do Aura

## Objetivo e limites

Entrega `start`/`send`/`interrupt` no `CodexService`, o mapeador puro de notificações para `ConversationEvent`, o Channel de eventos para a UI com agrupamento por quadro, a `ConversationView` com markdown incremental, a Persona e o mapeamento de erros.

Não inclui: imagens/Chips (TK-005), modelo selecionável (TK-004; aqui usa o `isDefault` de `model/list`), histórico (TK-006), aprovações (TK-007), modos (TK-009; aqui sempre Chat).

## Leitura em ordem

1. `specs/002-conversa-agente-codex/plan.md` → `ConversationEvent`, `TurnError`, OT-003.
2. Docs atuais do app-server → "Start a turn", "Turn events", "Items", "Item deltas", erros `codexErrorInfo`.
3. `crates/aura-codex/tests/transcripts/<versão>/` (TK-001) e `tools/codex-record`.
4. `apps/desktop/src/overlay/OverlayShell.tsx` (001 TK-005) → expansão ao enviar.
5. `docs/design/ui-ux.md` → Overlay expandido, "Mensagem do agente", acessibilidade `aria-live`.

## Decisões já resolvidas

- `thread/start {modelProvider: "aura-chatgpt-plan" (ou o Provedor escolhido), model, cwd: workspace, sandbox:"readOnly", approvalPolicy:"onRequest", baseInstructions: persona(idioma), developerInstructions: modo Chat, serviceName:"aura_desktop"}`.
- Workspace criado antes do primeiro turno em `workspaces/<thread_id>/` e registrado em `conversations_meta` (AC-025 é verificado no TK-009; aqui basta criar).
- Deltas: host agrupa em janelas de 16 ms; UI aplica por `requestAnimationFrame`; mensagem finalizada substitui o acumulado pelo texto do `item/completed` (fonte de verdade).
- Markdown: componente `Markdown` com parse por blocos memorizados; Shiki só quando há bloco de código.
- Liberdade local: estrutura de componentes e CSS.

## Mapa de alterações

- Novo: `crates/aura-codex/src/mapping.rs` → `map_notification`, `TurnState`, `TurnError::from_codex`.
- Existente: `crates/aura-codex/src/service.rs` → `start`, `send`, `interrupt`.
- Novo: `crates/aura-store/src/migrations/0002_conversations.sql` → `conversations_meta`.
- Novo: `apps/desktop/src-tauri/resources/persona/{pt-BR,en}.md`.
- Existente: `apps/desktop/src-tauri/src/conversation.rs` → `conversation_start`, `conversation_send`, `conversation_interrupt`, `Channel<ConversationEvent>`.
- Novo: `apps/desktop/src/conversation/{ConversationView.tsx,Message.tsx,Markdown.tsx,ItemCard.tsx,ErrorBanner.tsx}`, `state/conversationStore.ts`.

## Contrato técnico

- Entradas: texto (1–100 000 caracteres).
- Saídas: sequência de `ConversationEvent` por turno terminando em `TurnCompleted`.
- Invariantes: todo turno emite exatamente um `TurnCompleted`; texto final = `item/completed.text`.
- Erros: ver tabela `TurnError` do plano; texto vazio → não envia; > 100 000 → erro de validação com sugestão.
- Efeitos: app-server iniciado se necessário (AC-009 do TK-001); Overlay expande.

## Exemplos de aceite

- **AC-005**: transcript gravado "liste 3 atalhos em tabela" → snapshot de eventos (`TurnStarted`, N×`MessageDelta`, `MessageCompleted`, `TurnCompleted{Completed}`); Vitest: markdown com tabela 3×2 e bloco de código com botão "Copiar"; medição: `p95(paint - received) ≤ 50 ms` em 500 deltas sintéticos no modo diagnóstico.
- **AC-006**: durante deltas, `interrupt` → `turn/interrupt{threadId}` enviado; `TurnCompleted{Interrupted}`; store mantém o texto parcial "Aqui estão"; UI mostra rótulo "interrompido".
- **AC-007**: payload `thread/start` capturado pelo falso contém `baseInstructions` idêntico a `persona/pt-BR.md` quando a UI está em pt-BR; valida contra o schema.
- **AC-008**: erro upstream `429 {error:{code:"subscription_sharing_usage_limit_exceeded"}}` → modal "Limite de uso atingido" com "Gerenciar uso" (principal, abre `https://chatgpt.com/settings/usage`) e "Usar outro provedor" (secundária), sem horário inventado; `403 subscription_sharing_user_not_eligible` → "Seu plano ou workspace não permite uso no Aura"; `400 subscription_sharing_unsupported_capability` com `param` → mensagem citando o parâmetro; `codexErrorInfo: UsageLimitExceeded` com reset (BYOK) → "Reinicia às 21:00"; `Unauthorized` → "Sessão expirada" + "Entrar novamente"; `ContextWindowExceeded` → "Conversa longa demais" + "Compactar"/"Nova conversa"; `HttpConnectionFailed` sem status → "Sem conexão" + "Tentar novamente"; `Other` → "Erro inesperado (código X)".

## Dependências e sequência de execução

Depende de: TK-002.

- [ ] TK-003.1 Unit `mapping` com transcript de streaming (red→green).
- [ ] TK-003.2 Unit `TurnError` um caso por vez.
- [ ] TK-003.3 Contrato AC-007 (payload) e AC-006.
- [ ] TK-003.4 UI: Vitest de markdown e interrupção; medição de pintura.
- [ ] TK-003.5 Roteiro manual real: pergunta, tabela, código, interromper, "quem é você?".

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-codex mapping service`; `pnpm -C apps/desktop test -- conversation`; `pnpm -C apps/desktop e2e -- --spec e2e/conversa.spec.ts`; roteiro manual.
- Estado esperado: verdes; p95 de pintura ≤ 50 ms registrado.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: limites da conta de teste esgotados → usar transcript; registrar.

## Condição de retorno à planejadora

Retornar se o app-server não preservar o corpo do erro upstream (códigos `subscription_sharing_*`) em `additionalDetails`/mensagem, se o resultado do spike (TK-001) indicar que `baseInstructions` não substitui a persona, ou se o volume de deltas exigir mudança de protocolo UI↔host.

## Relatório de saída

Relatar eventos suportados, medições, prints, EV refs e limitações.
