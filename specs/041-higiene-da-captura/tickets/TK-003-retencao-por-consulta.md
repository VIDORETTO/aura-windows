---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 041-higiene-da-captura
type: delivery
status: done
ticket_revision: 6
requires: ["TK-002"]
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-007", "AC-008", "AC-009"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-capture/src/segments.rs", "crates/aura-capture/tests/pipeline.rs", "crates/aura-capture/tests/retention_store.rs", "crates/aura-capture/tests/retention_cost.rs", "crates/aura-audio/src/recorder.rs"]
verification_status: passed
---





# TK-003 — Retenção por consulta e órfãos com cadência (A-04)

## Objetivo e limites

Depois de cada Segmento, a retenção consulta e apaga só candidatos; a varredura de órfãos fica separada (no início do gravador e de hora em hora); `range` filtra no SQL; `delete` em lote.

Não inclui: mover a captura e a codificação para outra thread; supervisão (TK-004).

## Leitura em ordem

1. `crates/aura-capture/src/segments.rs` → `SegmentStore::{list, range, delete, apply_retention}`, `SegmentRecorder::start` (`store_seg`).
2. `crates/aura-capture/src/retention.rs` → `plan` (oráculo).
3. Migração do `capture_segments` e do índice `(source, end_ms)` em `crates/aura-store` (só leitura).
4. `crates/aura-capture/tests/pipeline.rs` → uso de `apply_retention` (padrão).

## Decisões já resolvidas

- O resultado é igual ao de `plan` para o mesmo conjunto, com empate pela ordem de inserção (`ORDER BY start_ms, rowid`).
- Algoritmo:
  1. apaga os `buffer` vencidos;
  2. agrega total de bytes e quantidade de vencidos por idade no conjunto sujeito a limite;
  3. se algo precisa sair, percorre por `start_ms` com a regra de `plan` e para quando o total fica ≤ teto e não há vencidos restantes;
  4. apaga em lote.
- `sweep_orphans()` público. O `SegmentRecorder` (tela) e o `AudioSegmentWriter` (áudio) chamam a varredura no início e quando passar 1 h desde a última, pelo relógio de cada um. Hoje os dois dependem da varredura embutida na retenção, e o áudio tem pasta própria.
- Liberdade local: SQL exato, tamanho do lote, estrutura interna.

## Mapa de alterações

- Existente: `segments.rs` → `list` (ordem com `rowid`), `range`, `delete`, `apply_retention`, laço do `SegmentRecorder`.
- Existente: `crates/aura-audio/src/recorder.rs` → `AudioSegmentWriter::{new, flush}` (varredura no início e de hora em hora).
- Novo: `SegmentStore::sweep_orphans`; teste `crates/aura-capture/tests/retention_store.rs` (equivalência, órfãos, `range`); medição `#[ignore]` em `crates/aura-capture/tests/retention_cost.rs`, separada para compilar também no código anterior.

## Contrato técnico

- `apply_retention(&RetentionPolicy, now_ms) -> Result<usize, SegmentError>`: número de Segmentos apagados (sem órfãos).
- `sweep_orphans() -> Result<usize, SegmentError>`: número de órfãos removidos.
- Invariantes: nenhum Segmento fora do resultado de `plan` é apagado; arquivo publicado pela metade (`.tmp`) só sai na varredura, sob o mutex de publicação.
- Concorrência: mantém o mutex `publication`.

## Exemplos de aceite

- **AC-007**: 3 sementes × 2.000 Segmentos (Fontes tela/mic/sistema; kinds `buffer`/`continuous`/`recording`; `manual` alternado) × 4 políticas → ids apagados iguais aos de `plan(list())`.
- **AC-008**: `.seg` sem linha e `.seg.tmp` sobrevivem a `apply_retention` e somem com `sweep_orphans`; linha sem arquivo some com `sweep_orphans`.
- **AC-009**: tela 0–100 s em Segmentos de 10 s; `range("screen", 25_000, 55_000)` → inícios 20, 30, 40 e 50 s.

## Dependências e sequência de execução

Depende de: TK-002.

- [ ] TK-003.1 AC-008 vermelho (hoje `apply_retention` remove órfãos).
- [ ] TK-003.2 Green: `sweep_orphans` separado; o gravador varre no início e de hora em hora.
- [ ] TK-003.3 AC-007 (equivalência) e AC-009, com retenção por consulta e `range` em SQL.
- [ ] TK-003.4 Medição antes e depois (60.000 `continuous`), regressão completa, evidência.

## Validação

- Comandos: `cargo test -p aura-capture`; medição `cargo test -p aura-capture --release --test retention_cost -- --ignored --nocapture`, no commit anterior e no novo.
- Estado esperado: equivalência sem diferenças; tempo menor na medição.
- Distinguir defeito de ambiente: arquivo bloqueado por antivírus não é red; repetir.

## Condição de retorno à planejadora

Se a equivalência exigir mudar a ordem de exclusão de `plan`, parar e devolver.

## Relatório de saída

Arquivos e símbolos, ACs, tempos antes e depois, `EV`, pendências.
