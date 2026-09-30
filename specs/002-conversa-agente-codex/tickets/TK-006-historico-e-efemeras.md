---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 002-conversa-agente-codex
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-003"]
requirement_refs: ["FR-006"]
acceptance_refs: ["AC-015", "AC-016", "AC-017"]
spec_revision: 2
plan_revision: 2
owned_areas: ["crates/aura-codex/src/history.rs", "apps/desktop/src/conversation/history", "crates/aura-store/src/conversations_repo.rs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-006 — Histórico, retomada e Conversas efêmeras

## Objetivo e limites

Entrega listar/buscar/retomar/renomear/fixar/arquivar/excluir Conversas e a Conversa efêmera (`Ctrl+Shift+E`), com limpeza do Workspace da conversa.

Não inclui: busca semântica, exportação de conversas.

## Leitura em ordem

1. Docs atuais do app-server → `thread/list` (filtros `searchTerm`, `isPinned`, `archived`), `thread/read includeTurns`, `thread/resume`, `thread/name/set`, `thread/metadata/update isPinned`, `thread/archive`, `thread/delete`, `ephemeral`.
2. `crates/aura-store/src/migrations/0002_conversations.sql` (TK-003).
3. `docs/design/ui-ux.md` → atalhos `Ctrl+H`, `Ctrl+Shift+E`.

## Decisões já resolvidas

- Fonte de verdade da lista: `thread/list` filtrado por `sourceKinds`/`serviceName` do Aura; `conversations_meta` guarda só workspace, modo, provedor.
- Exclusão: confirmação modal; `thread/delete` → remover pasta do workspace → remover meta. Falha ao remover pasta → registrar e tentar na próxima inicialização (lista `pending_deletions`).
- Efêmera: `thread/start{ephemeral:true}`, workspace em `workspaces/_ephemeral/<uuid>` removido ao fechar a Conversa ou ao sair do app; nenhum registro em `conversations_meta`.
- Liberdade local: layout do painel de histórico.

## Mapa de alterações

- Novo: `crates/aura-codex/src/history.rs` → `list`, `read`, `rename`, `pin`, `archive`, `delete`.
- Novo: `crates/aura-store/src/conversations_repo.rs` → `ConversationsRepo::{upsert, get, remove, pending_deletions}`.
- Existente: `service.rs` → `resume`, `start{ephemeral}`.
- Novo: `apps/desktop/src/conversation/history/{HistoryPanel.tsx,HistoryItem.tsx,ConfirmDelete.tsx}`.

## Contrato técnico

- Entradas: `HistoryQuery{search?, archived: bool, cursor?}`.
- Saídas: `Page<ConversationSummary{id, title, preview, updated_at, pinned, archived}>`; `ConversationSnapshot{turns}`.
- Invariantes: fixadas primeiro, depois por `updated_at` desc; efêmeras nunca aparecem.
- Erros: `HistoryError::{NotFound, Busy(turn ativo), Storage}`.
- Efeitos: remoção de pasta em exclusão/efêmera.

## Exemplos de aceite

- **AC-015**: falso com 3 threads (uma fixada) → lista na ordem fixada, depois por data; busca "fatura" envia `searchTerm:"fatura"`; abrir → `thread/resume` + `thread/read{includeTurns:true}` e turnos exibidos.
- **AC-016**: renomear "Relatório Q3" → `thread/name/set`; fixar → `thread/metadata/update{isPinned:true}`; excluir com confirmação → `thread/delete` e pasta `workspaces/<id>` inexistente depois (teste com tempdir).
- **AC-017**: `Ctrl+Shift+E` → `thread/start{ephemeral:true}` e badge "efêmera"; fechar → pasta `_ephemeral/<uuid>` removida; `thread/list` do Aura não contém o id.

## Dependências e sequência de execução

Depende de: TK-003.

- [ ] TK-006.1 Contrato `list/read/resume` (AC-015) red→green.
- [ ] TK-006.2 Rename/pin/archive/delete + limpeza (AC-016).
- [ ] TK-006.3 Efêmera (AC-017).
- [ ] TK-006.4 UI + Vitest + E2E falso.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-codex history -p aura-store conversations`; `pnpm -C apps/desktop test -- history`; `pnpm -C apps/desktop e2e -- --spec e2e/historico.spec.ts`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: arquivo bloqueado por antivírus na remoção → `pending_deletions`.

## Condição de retorno à planejadora

Retornar se `thread/list` não permitir filtrar apenas threads do Aura dentro do `CODEX_HOME` isolado (se o isolamento já garante isso, registrar).

## Relatório de saída

Relatar símbolos, resultados e EV refs.
