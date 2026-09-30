---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 007-anexos-multimodais
revision: 1
spec_revision: 1
status: ready
---

# Plan: Anexos multimodais

## Summary

Crate `aura-ingest` define a Interface `Ingestor` e o formato de saída `IngestedDoc` (blocos de texto/tabela/imagem com localizadores), com extratores por tipo. Extratores leves (texto, código, CSV, planilhas, DOCX/PPTX) rodam no host; pesados (PDF com PDFium, vídeo, áudio) rodam no `aura-worker`. Um `Budgeter` puro decide o que entra no turno dado o orçamento; o resultado vira `TurnInput`s (texto + `localImage`). A tool MCP `attachment_read` permite leitura seletiva posterior.

## Technical context

- Language/runtime: Rust.
- Dependencies (fixar/consultar): `pdfium-render` (+ binário PDFium x64 no pacote do worker), `calamine`, `csv`, `quick-xml` + `zip` (DOCX/PPTX/ODT), `rtf-parser`, `html2md`/`htmd`, `encoding_rs` + `chardetng`, `sha2`, Media Foundation (worker) para demux/decode de áudio e vídeo, `symphonia` para decodificar formatos de áudio sem MF (ogg/opus/flac).
- Storage/data: `workspaces/<id>/attachments/<hash>.<ext>` (original), cache `%LOCALAPPDATA%\Aura\cache\ingest\<hash>.json` + imagens (limpeza LRU 2 GB).
- Test command: `cargo nextest run -p aura-ingest`; worker `--features win-integration` para vídeo/MF; corpus em `crates/aura-ingest/tests/corpus/`.
- Target/platform: Windows; extratores host testáveis em Linux.

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-006; AC-001–AC-010.

## Modules, interfaces, consumers, and seams

| Module | Interface | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `aura-ingest::model` | `IngestedDoc{kind, summary, blocks: Vec<Block{locator, content: Text|Table|Image(path)}>, token_estimate}`; `Locator::{Page(n), Sheet{name, rows}, Slide(n), Time{from,to}, Section(title)}` | todos | — |
| `aura-ingest::ingestor` | `trait Ingestor { fn supports(&self, mime, ext) -> bool; async fn ingest(&self, path, opts) -> Result<IngestedDoc>; async fn read(&self, path, selector) -> Result<IngestedDoc>; }` | `Pipeline` | Um adapter por formato; corpus real |
| `aura-ingest::pipeline` | `Pipeline::{attach(path) -> AttachmentId, status(id), result(id)}` (detecção por magic bytes, hash, cache, progresso) | host (`ContextTray`) | Integração com corpus |
| `aura-ingest::budget` | `fn plan(docs, budget_tokens, per_image_tokens) -> BudgetPlan{included_blocks, reductions_needed}` (pura) | host | Casos literais |
| `aura-ingest::to_turn` | `fn to_turn_inputs(doc, plan) -> Vec<TurnInput>` | host | Snapshot |
| `aura-worker::ingest` | `ingest.pdf{path, pages?}`, `ingest.audio{path}` (usa `Transcriber` local no mesmo worker), `ingest.video{path, max_frames}` | `Pipeline` | Integração Windows |
| `aura-mcp::tools::attachments` | `attachment_read{attachment_id, selector}` | Codex | Cliente MCP de teste |

## Chosen approach and alternatives

- **Conversão local para texto+imagens** (os modelos da assinatura aceitam só texto/imagem): máximo de compatibilidade entre ChatGPT e BYOK.
- **PDFium** (qualidade de texto e renderização) em vez de `lopdf`/`pdf-extract` (falham em PDFs complexos).
- **Planilhas como esquema + amostra + estatísticas**: dá ao modelo visão global sem estourar contexto; detalhe via `attachment_read`.
- Alternativa descartada: enviar `input_file` da Responses API (não suportado por provedores traduzidos e dependente de upload).

## Data, compatibility, and external dependencies

- Licença PDFium (BSD-3) compatível; binário baixado junto ao worker.
- Transcrição de arquivos usa o mesmo `Transcriber` do 006; sem modelo → Chip com "Baixar modelo de voz" ou ASR em nuvem.

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001 | Integração pipeline + Vitest | Resumos esperados por fixture | `relatorio-12p.pdf` → "12 páginas"; `vendas.xlsx` → "3 abas · 1 240 linhas" |
| AC-002 | Integração | Spec | PDF com senha, zip corrompido, arquivo de 250 MB (esparso) → motivos |
| AC-003 | Unit `budget::plan` | Casos literais | 3 docs de 40k tokens com orçamento 60k → `reductions_needed` |
| AC-004 | Integração corpus | Texto conhecido das fixtures | `[Página 3]` contém frase conhecida; página com figura → imagem; escaneado → imagens + OCR |
| AC-005 | Integração corpus | Valores da fixture | `vendas.xlsx` aba "Jan": colunas `Data(date)`, `Valor(number)`; soma = 12 345,67 |
| AC-006 | Integração corpus | Estrutura esperada | DOCX com H1/H2/lista/tabela → Markdown equivalente; PPTX 5 slides com notas |
| AC-007 | Integração com `FakeTranscriber` + `real-models` | Texto conhecido | mp3 de `jfk` → transcrição com tempos |
| AC-008 | Integração Windows | Vídeo sintético com frames numerados + áudio conhecido | 12 keyframes com números esperados + transcrição |
| AC-009 | Cliente MCP de teste | Fixture | `attachment_read{sheet:"Fev", rows:100..150}` → linhas corretas |
| AC-010 | Integração | Spec | Segunda anexação → sem chamada ao extrator (contador) |

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `crates/aura-ingest/src/{model.rs,ingestor.rs,pipeline.rs,budget.rs,to_turn.rs,text.rs,sheets.rs,office.rs,cache.rs}` | new | ver tabela | Ingestão | 2026-09-29 |
| `crates/aura-ingest/tests/corpus/**` | new | fixtures | Testes | 2026-09-29 |
| `crates/aura-worker/src/ingest.rs` | new | PDF/áudio/vídeo | Worker | 2026-09-29 |
| `crates/aura-mcp/src/tools/attachments.rs` | new | `attachment_read` | Agente | 2026-09-29 |
| `apps/desktop/src/conversation/chips/FileChip.tsx`, `ReduceDialog.tsx` | new | UI | Interface | 2026-09-29 |
| `crates/aura-core/src/context.rs` | existing (002 TK-005) | `ChipKind::File`, `TurnInput` | Integração | 002 TK-005 |

## Derived technical obligations

- **OT-001** → FR-001: detecção por conteúdo (magic bytes), nunca só por extensão.
- **OT-002** → FR-004: `to_turn_inputs` nunca excede o orçamento aprovado pelo `budget::plan`.
- **OT-003** → FR-005: `attachment_read` só lê anexos da Conversa do header `X-Aura-Conversation` (004).

## Risks and gates

- PDFs gigantes ou malformados podem travar o worker — timeout por arquivo (60 s + 1 s/página) e isolamento no processo.
- G2: satisfeito. G3: TK-001 depende de 002 TK-005 e 006 TK-003.
