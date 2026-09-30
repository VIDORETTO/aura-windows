---
schema: hybrid/ticket
schema_version: 1.0
id: TK-008
effort: 002-conversa-agente-codex
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-003"]
requirement_refs: ["FR-008"]
acceptance_refs: ["AC-022", "AC-023"]
spec_revision: 2
plan_revision: 2
owned_areas: ["crates/aura-codex/src/turn_control.rs", "apps/desktop/src/conversation/ContextUsage.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-008 — Direcionar turno, compactar e uso de contexto

## Objetivo e limites

Entrega `steer` (`Ctrl+Enter` durante resposta), `/compactar`, o evento `TokenUsage` e o indicador de uso de contexto.

Não inclui: fila de mensagens para turnos futuros (enviar com `Enter` durante resposta fica desabilitado com dica de `Ctrl+Enter`).

## Leitura em ordem

1. Docs atuais do app-server → `turn/steer` (`expectedTurnId`), `thread/compact/start`, `thread/tokenUsage/updated`, item `contextCompaction`.
2. `crates/aura-codex/src/mapping.rs` (TK-003).
3. `apps/desktop/src/overlay/InputBar.tsx` (001).

## Decisões já resolvidas

- `steer` usa o `TurnId` ativo do store; se o turno terminou nesse meio tempo (erro "no active turn"), o texto é enviado como novo turno automaticamente e a UI informa.
- `/compactar` é Comando rápido local (não vai ao modelo como texto).
- Indicador: anel com % da janela de contexto; > 85% mostra dica de compactar.
- Liberdade local: visual do anel.

## Mapa de alterações

- Novo: `crates/aura-codex/src/turn_control.rs` → `steer`, `compact`, mapeamento `TokenUsage`.
- Existente: `service.rs` → expõe `steer`/`compact`.
- Novo: `apps/desktop/src/conversation/ContextUsage.tsx`; `InputBar` trata `Ctrl+Enter` durante turno.

## Contrato técnico

- Entradas: texto de steer; comando `/compactar`.
- Saídas: `turn/steer{threadId, input, expectedTurnId}`; `thread/compact/start{threadId}`; eventos `TokenUsage`, `Compacted`.
- Invariantes: steer nunca cria turno novo quando há turno ativo.
- Erros: `SteerError::NoActiveTurn` → fallback para `send`.

## Exemplos de aceite

- **AC-022**: turno `turn_456` ativo + `Ctrl+Enter` "foque nos testes" → `turn/steer{expectedTurnId:"turn_456"}`; nenhum `turn/start`; mensagem aparece na conversa marcada "direcionamento".
- **AC-023**: `/compactar` → `thread/compact/start`; falso emite item `contextCompaction` e `tokenUsage{used: 30000 → 8000, window: 200000}` → indicador de 15% para 4%.

## Dependências e sequência de execução

Depende de: TK-003.

- [ ] TK-008.1 Contrato AC-022 (incluindo fallback) red→green.
- [ ] TK-008.2 Contrato AC-023.
- [ ] TK-008.3 UI + Vitest.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-codex turn_control`; `pnpm -C apps/desktop test -- ContextUsage InputBar`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se `thread/tokenUsage/updated` não trouxer a janela de contexto na versão fixada (indicador precisaria de outra fonte).

## Relatório de saída

Relatar resultados e EV refs.
