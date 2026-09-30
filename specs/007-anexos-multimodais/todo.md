# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [ ] TK-001 — TK-001 — Pipeline de Ingestão, orçamento e PDF
Status: `implemented` | Bloqueado por: nenhum

- [ ] TK-001.1 Unit `budget::plan` e `to_turn_inputs` (red→green).
- [ ] TK-001.2 `Pipeline` com ingestor falso (detecção, erros, progresso).
- [ ] TK-001.3 `ingest.pdf` no worker (AC-004) um caso por vez.
- [ ] TK-001.4 Chips e redução (Vitest); evidências.

## [ ] TK-002 — TK-002 — Planilhas: esquema, amostra e estatísticas
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-002.1 Unit `infer_header`/`infer_types` red→green.
- [ ] TK-002.2 `column_stats` com valores da fixture.
- [ ] TK-002.3 Integração com cada formato do corpus; `read` por intervalo; evidências.

## [ ] TK-003 — TK-003 — Documentos, apresentações, texto e código
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-003.1 `TextIngestor` (encoding, código) red→green.
- [ ] TK-003.2 DOCX, depois PPTX, depois ODT/RTF/HTML — um por vez.
- [ ] TK-003.3 Evidências.

## [ ] TK-004 — TK-004 — Áudio como anexo (Transcrição com tempos)
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-004.1 Decodificação por formato (red→green um por vez).
- [ ] TK-004.2 Janelas + desduplicação (unit com transcripts sintéticos).
- [ ] TK-004.3 Integração real; evidências.

## [ ] TK-006 — TK-006 — Originais no workspace, `attachment_read` e cache por hash
Status: `implemented` | Bloqueado por: TK-002, TK-003

- [ ] TK-006.1 `IngestCache` (red→green, incluindo LRU).
- [ ] TK-006.2 Cópia do original + integração no pipeline.
- [ ] TK-006.3 `attachment_read` com seletores (um por tipo); evidências.

## [ ] TK-005 — TK-005 — Vídeo como anexo (keyframes + Transcrição)
Status: `implemented` | Bloqueado por: TK-004

- [ ] TK-005.1 Keyframes por tempo (red→green) com a fixture.
- [ ] TK-005.2 Mudança de cena.
- [ ] TK-005.3 Intercalação com transcrição; evidências.
