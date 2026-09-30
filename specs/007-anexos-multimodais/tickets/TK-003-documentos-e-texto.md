---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 007-anexos-multimodais
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-006"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-ingest/src/office.rs", "crates/aura-ingest/src/text.rs", "crates/aura-ingest/tests/corpus/docs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-003 — Documentos, apresentações, texto e código

## Objetivo e limites

Entrega `OfficeIngestor` (DOCX, PPTX, ODT/ODP por XML; RTF; HTML → Markdown) e `TextIngestor` (txt, md, json, xml, yaml, código com detecção de linguagem por extensão e encoding).

Não inclui: `.doc`/`.ppt` legados (tentativa via IFilter fica como melhoria; hoje "não suportado").

## Leitura em ordem

1. `crates/aura-ingest/src/{model.rs,ingestor.rs}` (TK-001).
2. Especificação OOXML básica (`word/document.xml` estilos de título, `w:tbl`; `ppt/slides/slideN.xml`, `ppt/notesSlides/`).

## Decisões já resolvidas

- DOCX: estilos `Heading1..6`/`Title` → `#`…; listas numeradas/marcadores → `1.`/`-`; tabelas → Markdown; imagens → ignoradas com marcador `[imagem]`.
- PPTX: `## Slide N — <título>` + texto + `> Notas: …`.
- Texto: `chardetng` para encoding; arquivos > 1 MB de texto são truncados no resumo com aviso e disponíveis por `read`.
- Liberdade local: organização do parser.

## Mapa de alterações

- Novo: `crates/aura-ingest/src/{office.rs,text.rs}`.
- Novo: fixtures `tests/corpus/docs/{estrutura.docx,apresentacao.pptx,documento.odt,nota.rtf,pagina.html,codigo.rs,dados.json,latin1.txt}`.

## Contrato técnico

- Entradas: arquivo; `Selector::Section|Slide` para `read`.
- Saídas: `IngestedDoc` com texto Markdown e localizadores.
- Erros: XML inválido → `Corrupt`.

## Exemplos de aceite

- **AC-006**: `estrutura.docx` (Título "Relatório", H2 "Resumo", lista de 3 itens, tabela 2×2) → Markdown `# Relatório`, `## Resumo`, `- item…`×3, tabela com cabeçalho; `apresentacao.pptx` (5 slides, notas no 2) → 5 seções `## Slide N` e `> Notas:` só no 2; `codigo.rs` → bloco ```rust; `latin1.txt` → acentos corretos.

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-003.1 `TextIngestor` (encoding, código) red→green.
- [ ] TK-003.2 DOCX, depois PPTX, depois ODT/RTF/HTML — um por vez.
- [ ] TK-003.3 Evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-ingest office text`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se documentos reais do corpus falharem em > 5% (H-015).

## Relatório de saída

Relatar resultados e EV refs.
