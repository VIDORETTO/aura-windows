---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 046-spikes-segundo-cerebro
type: spike
status: blocked
ticket_revision: 2
requires: []
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-013"]
spec_revision: 1
plan_revision: 1
owned_areas: ["specs/046-spikes-segundo-cerebro/s6.md"]
verification_status: not_run
last_update: "TK-006.1 feito: 16 perguntas em s6.md (uso pessoal x profissional, base legal e aviso, tela e conversas escritas, direitos e nuvem, o que o produto pode dizer). Segue bloqueado aguardando o parecer."
---


# TK-006 — S6 — parecer jurídico sobre conversas com terceiros

## Objetivo e limites

Registrar as perguntas para o advogado e, quando houver parecer, a decisão sobre consentimento e uso com clientes.

Não inclui: decidir questão jurídica sem parecer; texto de interface.

Bloqueado: aguarda parecer jurídico ("s6 depois de finalizar").

## Leitura em ordem

1. `docs/research/segundo-cerebro-complemento-2026-11.md` §9; `docs/project/plano-segundo-cerebro.md` §4.5 e I-17.

## Decisões já resolvidas

- Nada é prometido na interface sobre conversas com terceiros antes do parecer.

## Mapa de alterações

- Novo: `specs/046-spikes-segundo-cerebro/s6.md` (perguntas).

## Contrato técnico

- Perguntas objetivas sobre o art. 4º, I da LGPD, base legal para uso com clientes, aviso aos participantes e dados sensíveis em telas.

## Exemplos de aceite

- **AC-013**: perguntas registradas; ticket bloqueado até o parecer.

## Dependências e sequência de execução

Depende de: parecer externo.

- [ ] TK-006.1 Perguntas registradas.
- [ ] TK-006.2 Parecer recebido e decisão registrada.

## Validação

- Leitura do parecer.

## Condição de retorno à planejadora

Pare e devolva se uma medição exigir mudar código de produto, se um modelo ou biblioteca não tiver licença permissiva, ou se o resultado contradisser um ADR de forma que exija decisão do dono do produto (registre a proposta).

## Relatório de saída

Arquivos, comandos, números, `EV`, limitações, decisão proposta e próxima ação.
