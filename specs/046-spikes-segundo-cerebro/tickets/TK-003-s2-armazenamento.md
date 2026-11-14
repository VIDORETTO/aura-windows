---
schema: hybrid/ticket
schema_version: "1.0"
id: TK-003
effort: 046-spikes-segundo-cerebro
type: spike
status: ready
ticket_revision: 1
requires: ["TK-002"]
requirement_refs: ["FR-001", "FR-003"]
acceptance_refs: ["AC-001", "AC-002", "AC-003", "AC-004"]
spec_revision: 1
plan_revision: 1
owned_areas: ["tools/spikes/s2-vetores", "specs/046-spikes-segundo-cerebro/s2.md", "docs/adr/0012-acervo-cifrado-com-vetores-em-blob.md"]
verification_status: not_run
---

# TK-003 — S2 — armazenamento vetorial do ADR 0012

## Objetivo e limites

Medir o desenho do ADR 0012 com SQLCipher: escala, recall com vetores reais do modelo do S1 em português, frio e quente, e uma máquina modesta aproximada. Atualizar o ADR 0012.

Não inclui: `aura-store`/worker do produto (Fase 1); CI com Perl.

## Leitura em ordem

1. `specs/046-spikes-segundo-cerebro/plan.md` → S2.
2. `docs/adr/0012-acervo-cifrado-com-vetores-em-blob.md`.
3. `docs/research/segundo-cerebro-medicoes-2026-11.md` §3 (números anteriores e ressalvas).

## Decisões já resolvidas

- BLOB binário (48 B) + int8 (384 B) por trecho, ordem de inserção = tempo, filtro por faixa de `rowid`, varredura em Rust (POPCNT) e reordenação em int8.
- Recall só sobre os 100 mil vetores reais; escala com vetores derivados com ruído, só para tempo.
- Máquina modesta aproximada: 1 núcleo + `PROCTHROTTLEMAX` 50%, restaurado no fim.

## Mapa de alterações

- Novo: projeto `tools/spikes/s2-vetores`, relatório `s2.md`, atualização do ADR 0012.

## Contrato técnico

- `s2 gerar` (codifica trechos com o modelo do S1), `s2 bench <N>`, `s2 recall`.
- Saída: JSON com p50/p95 por caso, recall por número de candidatos, frio e quente, disco.

## Exemplos de aceite

- **AC-001** a **AC-004**, como no spec.

## Dependências e sequência de execução

Depende de: TK-002 (modelo escolhido).

- [ ] TK-003.1 Testes de unidade (quantização, hamming, reordenação, recall).
- [ ] TK-003.2 Gerar vetores reais (prioridade ociosa).
- [ ] TK-003.3 Escala, frio/quente, máquina aproximada; restaurar o plano de energia.
- [ ] TK-003.4 Relatório, ADR 0012, evidência.

## Validação

- `cargo test --manifest-path tools/spikes/s2-vetores/Cargo.toml`; execuções registradas.

## Condição de retorno à planejadora

Pare e devolva se uma medição exigir mudar código de produto, se um modelo ou biblioteca não tiver licença permissiva, ou se o resultado contradisser um ADR de forma que exija decisão do dono do produto (registre a proposta).

## Relatório de saída

Arquivos, comandos, números, `EV`, limitações, decisão proposta e próxima ação.
