---
schema: hybrid/spec
schema_version: 1.0
effort_id: 048-memoria-de-conversas
revision: 1
status: draft
profile: standard
---

# Memória de Conversas

## Problem and desired result

Não há busca entre Conversas (auditoria A-10): o histórico fica nos rollouts JSONL do Codex, sem cifra, e o agente não lembra de uma Conversa anterior. Resultado desejado: cada Turno concluído vira Lembrança no Acervo, e cada Conversa encerrada ganha um resumo indexado (plano §5.5, CAND-074). Assim, `memory_search` também responde "o que eu pedi ao Aura sobre X semana passada?".

## Consumers and actors

Usuário; agente; `aura-app` (eventos de Conversa); Acervo (047).

## Scope

### Included

- Turnos (pedido + resposta final, sem o ruído de ferramentas) como Lembranças, com id da Conversa e do Turno.
- Resumo de cinco linhas por Conversa encerrada, indexado.
- Conversa efêmera nunca entra.
- Apagar uma Conversa apaga as Lembranças dela.

### Excluded

- Propostas de Memória e perfil (Fase 3); página Memória (049).

## User journeys and scenarios

### US-001 — Lembrar de uma Conversa anterior (P1)

#### Acceptance scenarios

- **AC-001** — Depois de uma Conversa sobre "migrar o legado em Delphi", uma nova Conversa que pergunta "o que eu tinha pedido sobre a migração?" recebe, via `memory_search`, o Turno de origem, com o id da Conversa e a data.
- **AC-002** — Conversa efêmera não gera Lembrança; apagar uma Conversa apaga suas Lembranças.

## Requirements

- **FR-001** — Turnos e resumos no Acervo com origem.
- **FR-002** — Remoção pela linhagem; nada de Conversa efêmera.

## Limits, errors, and compatibility

O formato dos rollouts do Codex é do app-server fixado (ADR 0001): ler pelos eventos que o host já recebe, não pelo arquivo.

## Hypotheses and dependencies

Depende do 047 (Acervo). O resumo usa o modelo da Conversa só com consentimento (Q-013); sem ele, indexa só os Turnos.

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001 e AC-002 com evidência.

### Post-delivery observation

- **SC-002** — O usuário recupera pedidos antigos sem rolar o histórico.

## Decisions and open questions

- Em discovery: o resumo por Conversa depende da Q-013 (dados fora de um Turno) e fica atrás de opção.
