---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 047-acervo
type: delivery
status: ready
ticket_revision: 1
requires: ["TK-004"]
requirement_refs: ["FR-006"]
acceptance_refs: ["AC-010", "AC-011"]
spec_revision: 1
plan_revision: 1
owned_areas: ["tools/aura-bench", "bench/acervo", "bench/README.md"]
verification_status: not_run
---

# TK-005 — `aura-bench memory`: conjunto dourado e escala

## Objetivo e limites

Medir o Acervo real, com o mesmo indexador e a mesma busca do produto:
- a qualidade no conjunto dourado do S1;
- a latência da busca completa com 500 mil trechos.

Não inclui: perguntas reais do usuário, que entram quando houver; notebook real.

## Leitura em ordem

1. `specs/047-acervo/plan.md` → `aura-bench memory`.
2. `tools/aura-bench/src/main.rs` (subcomandos, saída JSON) e `bench/README.md`.
3. `tools/spikes/s1-embeddings/golden/README.md` e `src/main.rs` (métricas e conversão do corpus).

## Decisões já resolvidas

- **Fixture:** `bench/acervo/golden/{corpus.json, queries.json, README.md}`, cópia do S1 com o README de origem e de viés.
- **Conversão das fontes** do corpus:
  - `reuniao` → Reunião com uma fala por documento (`them`), título = título do documento;
  - `nota` → Nota;
  - as demais fontes (conversa, página, anexo, tela) → Nota com o título no texto, porque a Fase 1 ainda não as tem.
  - A diferença fica registrada no relatório.
- **Métricas** como no S1: acerto@1, acerto@3, acerto@10, MRR, posição média e p50/p95 do `memory_search` sem a ferramenta MCP.
- **`--escala N`:**
  - texto sintético de um vocabulário fixo de 5.000 palavras com semente;
  - vetores aleatórios normalizados com semente;
  - mede p50/p95 da busca completa (lexical + vetorial + fusão) sem filtro e com 30 dias;
  - 100 consultas.
- **Saída:** JSON em `--out` e linha legível; nada no git além da fixture.
- Liberdade local: organização do código do bench.

## Mapa de alterações

- Novo `tools/aura-bench/src/memory.rs`. Existente `tools/aura-bench/src/main.rs` e `Cargo.toml`.
- Novo `bench/acervo/golden/*`. Existente `bench/README.md`.

## Contrato técnico

- `aura-bench memory --model-dir <dir> [--out <json>]` → `{acerto1, acerto3, acerto10, mrr, p50_ms, p95_ms, n}`.
- `aura-bench memory --escala <N> [--out <json>]` → `{N, p50_ms, p95_ms, p50_30d_ms, p95_30d_ms}`.

## Exemplos de aceite

- **AC-010:** com o bekko-a8m, o MRR fica em 0,892 ± 0,02. O valor medido e a diferença para o S1 entram no relatório; a diferença esperada vem da conversão das fontes e do corte de 1.000 caracteres.
- **AC-011:** com 500 mil, p95 ≤ 300 ms no desktop, sem filtro e com 30 dias.
- Teste de unidade das métricas com literais: posições [1, 3, sem resposta, 2] → acerto@1 0,25, acerto@3 0,75, MRR (1 + 1/3 + 0 + 1/2)/4.

## Dependências e sequência de execução

Depende de: TK-004.

- [ ] TK-005.1 Métricas (literais), red → green.
- [ ] TK-005.2 Conversão do corpus e execução com o modelo (AC-010).
- [ ] TK-005.3 Escala (AC-011), com prioridade baixa.
- [ ] TK-005.4 README, regressão, clippy, fmt e evidência.

## Validação

- Comandos:
  - `cargo test -p aura-bench`;
  - `cargo run -p aura-bench --release -- memory --model-dir <dir>`;
  - `cargo run -p aura-bench --release -- memory --escala 500000`;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo fmt --all -- --check`.
- Estado esperado: passam; AC-010 e AC-011 dentro dos limites.

## Condição de retorno à planejadora

Pare e devolva se o MRR ficar mais de 0,02 abaixo do S1 por causa do desenho (por exemplo, o corte de trecho ou o cabeçalho), e não da conversão das fontes. Isso exige decisão sobre o indexador.

## Relatório de saída

Arquivos e símbolos, ACs, comandos, números, `EV`, limitações e pendências.
