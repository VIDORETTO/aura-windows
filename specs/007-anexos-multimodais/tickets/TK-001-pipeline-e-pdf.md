---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 007-anexos-multimodais
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-001", "FR-002", "FR-004"]
acceptance_refs: ["AC-001", "AC-002", "AC-003", "AC-004"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-ingest/src/model.rs", "crates/aura-ingest/src/ingestor.rs", "crates/aura-ingest/src/pipeline.rs", "crates/aura-ingest/src/budget.rs", "crates/aura-ingest/src/to_turn.rs", "crates/aura-worker/src/ingest.rs", "apps/desktop/src/conversation/chips/FileChip.tsx", "apps/desktop/src/conversation/chips/ReduceDialog.tsx", "crates/aura-ingest/tests/corpus/pdf"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-001 — Pipeline de Ingestão, orçamento e PDF

## Objetivo e limites

Entrega o modelo `IngestedDoc`, a Interface `Ingestor`, o `Pipeline` (detecção por conteúdo, hash, progresso, erros), o `budget::plan`, `to_turn_inputs`, o Chip de arquivo com resumo/estimativa/progresso, o diálogo de redução, e o extrator de PDF no worker (texto por página, imagens de páginas, OCR para escaneados).

Não inclui: outros formatos (TK-002+), cache e `attachment_read` (TK-006).

## Leitura em ordem

1. `specs/007-anexos-multimodais/plan.md` → Modules, OT-001, OT-002.
2. `crates/aura-core/src/context.rs` (002 TK-005) → `ContextTray`, `TurnInput`, `ChipKind`.
3. `crates/aura-worker/src/{main.rs,rpc.rs}` (006 TK-003) → registrar métodos `ingest.*`.
4. `crates/aura-capture/src/text.rs` (004 TK-004) → OCR do Windows reutilizável (`WinOcr`) — mover para `aura-core`/crate comum se necessário.
5. Docs atuais: `pdfium-render` (texto por página, render para bitmap), limites de PDFium.

## Decisões já resolvidas

- Detecção: magic bytes via `infer` + fallback por extensão para texto.
- Página "pouco texto" = < 200 caracteres → incluir imagem (1024 px no maior lado); escaneado = < 50 caracteres em ≥ 80% das páginas → modo imagem (máx. 20 páginas) + OCR.
- Tokens: 4 chars/token; imagem = 765 tokens (padrão; configurável por modelo).
- Limites: 200 MB, 500 páginas.
- Liberdade local: layout do Chip e do diálogo de redução.

## Mapa de alterações

- Novo: `crates/aura-ingest/{Cargo.toml,src/lib.rs,src/model.rs,src/ingestor.rs,src/pipeline.rs,src/budget.rs,src/to_turn.rs}`.
- Novo: `crates/aura-worker/src/ingest.rs` → `ingest.pdf`.
- Novo: `crates/aura-ingest/src/pdf.rs` → `PdfIngestor` (cliente do worker).
- Novo: `apps/desktop/src/conversation/chips/{FileChip.tsx,ReduceDialog.tsx}`; `chip_add_files` (002) passa a usar o `Pipeline`.
- Novo: fixtures `tests/corpus/pdf/{relatorio-12p.pdf,escaneado-3p.pdf,senha.pdf,corrompido.pdf}` (geradas por script com texto conhecido, licença livre).

## Contrato técnico

- Entradas: caminhos de arquivos.
- Saídas: `ContextChip{kind: File, summary, token_estimate, status}`; no envio, `TurnInput`s por `to_turn_inputs`.
- Invariantes: OT-001, OT-002.
- Erros: `IngestError::{Unsupported, Encrypted, Corrupt, TooLarge{limit}, TooManyPages{limit}, Timeout}`.

## Exemplos de aceite

- **AC-001**: anexar `relatorio-12p.pdf` → Chip "relatorio-12p.pdf · 12 páginas · ~9 k tokens", status Processing → Ready.
- **AC-002**: `senha.pdf` → "Protegido por senha"; `corrompido.pdf` → "Arquivo corrompido"; arquivo esparso de 250 MB → "Acima de 200 MB".
- **AC-003** (unit): docs `[40k, 40k, 40k]` tokens, orçamento 60k → `reductions_needed = true` com sugestão por doc; `[20k, 10k]` → tudo incluído.
- **AC-004**: `[Página 3]` do relatório contém "Receita líquida cresceu 12%"; página 7 (só gráfico) → bloco imagem; `escaneado-3p.pdf` → 3 imagens + texto OCR contendo "Aura teste OCR".

## Dependências e sequência de execução

Depende de: 002-conversa-agente-codex/TK-005 e 006-voz-e-asr/TK-003 (outros esforços).

- [ ] TK-001.1 Unit `budget::plan` e `to_turn_inputs` (red→green).
- [ ] TK-001.2 `Pipeline` com ingestor falso (detecção, erros, progresso).
- [ ] TK-001.3 `ingest.pdf` no worker (AC-004) um caso por vez.
- [ ] TK-001.4 Chips e redução (Vitest); evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-ingest budget to_turn pipeline`; `cargo nextest run -p aura-worker --features win-integration pdf`; `pnpm -C apps/desktop test -- FileChip ReduceDialog`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: binário PDFium ausente → erro de ambiente, não red.

## Condição de retorno à planejadora

Retornar se PDFium não puder ser distribuído junto ao worker ou se H-015 falhar para PDFs.

## Relatório de saída

Relatar resultados, corpus, EV refs.
