---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 041-higiene-da-captura
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-001"]
acceptance_refs: ["AC-001", "AC-002", "AC-003"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-capture/src/retention.rs"]
verification_status: not_run
last_update: plan trata continuous; testes AC-001..003 verdes
---



# TK-001 — Retenção do modo Contínuo (A-01)

## Objetivo e limites

`retention::plan` aplica idade e tamanho aos Segmentos `continuous`, junto com as gravações sujeitas a limite, num teto global.

Não inclui: armazenamento (`segments.rs`), áudio, supervisão, UI.

## Leitura em ordem

1. `crates/aura-capture/src/retention.rs` → `plan`, testes `buffer_keeps_only_the_last_n_minutes` e `continuous_keeps_seven_days_and_twenty_gigabytes`.
2. `crates/aura-app/src/recorder.rs` → `background_kind` (`continuous`) e o `INSERT INTO recordings … manual` (1).
3. `docs/qa/auditoria-codigo-2026-11-12.md` → A-01.

## Decisões já resolvidas

- Sujeito a limite = `continuous`, ou `recording` com (`!manual` ou `apply_to_manual`).
- Teto de GB global sobre todas as Fontes; ordem por `start_ms` estável (ordem de entrada).
- Liberdade local: nome do predicado auxiliar.
- Descartado: teto por Fonte.

## Mapa de alterações

- Existente: `crates/aura-capture/src/retention.rs` → `plan`, doc de `SegmentMeta::kind`, testes novos.
- Fora da fatia: `segments.rs`, `aura-app`, `aura-audio`.

## Contrato técnico

- Entradas: `&[SegmentMeta]`, `&RetentionPolicy`, `now_ms`.
- Saídas: ids a apagar, do início mais antigo para o mais novo.
- Invariantes: `buffer` só pelo Buffer recente; `manual` sem `apply_to_manual` nunca sai por idade ou tamanho.
- Erros: nenhum (função pura).
- Efeitos: nenhum.

## Exemplos de aceite

- **AC-001**: 10 × `continuous` (1 dia; 2 GB) → `[d0, d1, d2]`; com 3 GB → `[d0, d1, d2, d3]` (literais do spec).
- **AC-002**: 10 × `recording`, `manual = true` → `[]`; com `apply_to_manual` → 4 ids. `buffer` 120 × 10 s → 89 (caso existente).
- **AC-003**: tela `continuous` 3 GB/dia d0–d9 + microfone `continuous` 1 GB/dia d5–d9 → tela d0–d4 (literal no `tdd.md`).

## Dependências e sequência de execução

Depende de: nenhum.

- [ ] TK-001.1 Teste AC-001 vermelho (lista vazia hoje).
- [ ] TK-001.2 Green mínimo em `plan`.
- [ ] TK-001.3 AC-002 e AC-003; regressão `cargo test -p aura-capture`.
- [ ] TK-001.4 Evidência e checkpoint.

## Validação

- Diretório: raiz do repositório.
- Comando: `cargo test -p aura-capture retention`; regressão `cargo test -p aura-capture`.
- Estado esperado: todos passam; antes da correção, o caso `continuous` falha com lista vazia.
- Distinguir defeito de ambiente: erro de compilação ou de linker não é red.

## Condição de retorno à planejadora

Se o contrato de ordem ou de teto global entrar em conflito com outro consumidor de `plan`, parar e devolver.

## Relatório de saída

Arquivos e símbolos, ACs, comandos e resultado, `EV`, pendências.
