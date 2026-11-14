---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 046-spikes-segundo-cerebro
type: spike
status: done
ticket_revision: 5
requires: ["TK-001"]
requirement_refs: ["FR-001", "FR-003"]
acceptance_refs: ["AC-006", "AC-007", "AC-008"]
spec_revision: 1
plan_revision: 1
owned_areas: ["tools/spikes/s1-embeddings", "specs/046-spikes-segundo-cerebro/s1.md", "docs/adr/0014-embeddings-locais-no-worker.md"]
verification_status: passed
---





# TK-002 — S1 — finalistas em Rust e fusão

## Objetivo e limites

Medir os finalistas e a base em Rust (`ort` + tokenizador), com qualidade, custo e fusões no conjunto dourado. Escolher o modelo padrão e a fusão, e atualizar o ADR 0014.

Não inclui: integração no worker do Aura (Fase 1).

## Leitura em ordem

1. `specs/046-spikes-segundo-cerebro/plan.md` → S1.
2. `docs/adr/0014-embeddings-locais-no-worker.md`.
3. `crates/aura-worker/Cargo.toml` e `Cargo.lock` → versão do `ort` (2.0.0-rc.12).

## Decisões já resolvidas

- 1 thread, lote 1; prefixos e pooling dos cartões; normalização L2.
- Critério: maior MRR; empate (< 0,02) → menor latência por documento.
- Fusões: RRF (1, 1), RRF (1, 0,5), lexical condicionado a termo raro.

## Mapa de alterações

- Novo: projeto `tools/spikes/s1-embeddings` (Cargo, `[workspace]` próprio), relatório `s1.md`, atualização do ADR 0014.

## Contrato técnico

- `s1 eval --modelo <id> --corpus <json> --perguntas <json> --saida <json>`
- Saída: métricas por modelo e por fusão; custo (p50/p95, RAM de pico, carga).

## Exemplos de aceite

- **AC-006**: quatro modelos com acerto@1/@10, MRR, posição média, latência e RAM.
- **AC-007**: três fusões contra o vetor puro, por estilo de pergunta.
- **AC-008**: recomendação com o critério aplicado.

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-002.1 Testes de unidade das fusões e métricas (literais calculados à parte).
- [ ] TK-002.2 Binário de avaliação; rodar os 4 modelos.
- [ ] TK-002.3 Relatório `s1.md` e ADR 0014.
- [ ] TK-002.4 Evidência.

## Validação

- `cargo test --manifest-path tools/spikes/s1-embeddings/Cargo.toml`; execução de `s1 eval` por modelo.

## Condição de retorno à planejadora

Pare e devolva se uma medição exigir mudar código de produto, se um modelo ou biblioteca não tiver licença permissiva, ou se o resultado contradisser um ADR de forma que exija decisão do dono do produto (registre a proposta).

## Relatório de saída

Arquivos, comandos, números, `EV`, limitações, decisão proposta e próxima ação.
