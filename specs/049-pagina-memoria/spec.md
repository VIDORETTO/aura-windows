---
schema: hybrid/spec
schema_version: 1.0
effort_id: 049-pagina-memoria
revision: 1
status: draft
profile: standard
---

# Página Memória

## Problem and desired result

O usuário precisa ver, buscar e controlar o que o Aura lembra (plano §5, CAND-076): buscar com origem, ver a linha do tempo, apagar por período, fonte ou item, ver o uso de disco e ligar ou desligar o Acervo.

## Consumers and actors

Usuário; UI (React, ADR 0009); `aura-app` (comandos).

## Scope

### Included

- Página "Memória" nas Configurações:
  - busca com os mesmos resultados do `memory_search`, com o chip de origem;
  - "Esquecer" por item, por período e por fonte;
  - uso de disco do `memory.db`;
  - interruptor do Acervo e da permissão do agente.

### Excluded

- Momentos de tela (Fase 2) e Fichas (Fase 3).

## User journeys and scenarios

### US-001 — Ver e esquecer (P1)

#### Acceptance scenarios

- **AC-001** — Buscar "orçamento" mostra os trechos com fonte, data e link para a Reunião, a Nota ou a Conversa.
- **AC-002** — "Esquecer outubro" apaga as Lembranças do período e o uso de disco cai; a busca não as devolve mais.

## Requirements

- **FR-001** — Busca, origem, Esquecer e uso de disco na UI, com textos em pt-BR e en (teste de paridade).

## Limits, errors, and compatibility

Contrato IPC dourado (ADR 0009); tokens de design de `docs/design/ui-ux.md`.

## Hypotheses and dependencies

Depende do 047 (e do 048 para as Conversas).

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001 e AC-002 com evidência na UI real.

### Post-delivery observation

- **SC-002** — O usuário entende o que o Aura lembra e apaga o que quer.

## Decisions and open questions

- Em discovery: layout a definir com o design system.
