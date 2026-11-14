---
schema: hybrid/ticket
schema_version: "1.0"
id: TK-005
effort: 046-spikes-segundo-cerebro
type: spike
status: blocked
ticket_revision: 1
requires: ["TK-004"]
requirement_refs: ["FR-002", "FR-003"]
acceptance_refs: ["AC-012"]
spec_revision: 1
plan_revision: 1
owned_areas: ["specs/046-spikes-segundo-cerebro/s3-s4.md", "docs/adr/0013-momentos-de-tela-por-evento.md"]
verification_status: not_run
---

# TK-005 — S3/S4 — dia real do usuário

## Objetivo e limites

O usuário roda a ferramenta de campo num dia normal e entrega o JSON de resumo; o agente analisa e atualiza o ADR 0013.

Não inclui: construir ou alterar a ferramenta (TK-004); código de produto.

Bloqueado: aguarda o usuário (delegação de 13/11/2026: "s3 vou usar quando estiver pronto, s4 vou usar").

## Leitura em ordem

1. `specs/046-spikes-segundo-cerebro/s3-s4.md` → roteiro.

## Decisões já resolvidas

- Só números; o usuário pode revisar o JSON antes de entregar.

## Mapa de alterações

- Existente: `s3-s4.md` (resultados), ADR 0013 (status).

## Contrato técnico

- Entrada: `resumo-*.json` do dia.

## Exemplos de aceite

- **AC-012**: resultado registrado com o dia real; ADR 0013 aceito ou revisado.

## Dependências e sequência de execução

Depende de: TK-004 e do usuário.

- [ ] TK-005.1 Receber o JSON.
- [ ] TK-005.2 Analisar contra os orçamentos do plano §8.1.
- [ ] TK-005.3 ADR 0013 e evidência.

## Validação

- Análise reprodutível do JSON por script.

## Condição de retorno à planejadora

Pare e devolva se uma medição exigir mudar código de produto, se um modelo ou biblioteca não tiver licença permissiva, ou se o resultado contradisser um ADR de forma que exija decisão do dono do produto (registre a proposta).

## Relatório de saída

Arquivos, comandos, números, `EV`, limitações, decisão proposta e próxima ação.
