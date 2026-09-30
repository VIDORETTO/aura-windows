---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 007-anexos-multimodais
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-005"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-ingest/src/sheets.rs", "crates/aura-ingest/tests/corpus/sheets"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-002 — Planilhas: esquema, amostra e estatísticas

## Objetivo e limites

Entrega `SheetsIngestor` para xlsx/xls/xlsm/ods (`calamine`) e csv/tsv (`csv` + detecção de delimitador/encoding), com inferência de cabeçalho e tipos, amostra de 50 linhas em Markdown e estatísticas por coluna numérica, e suporte a `read` por aba/intervalo (usado no TK-006).

Não inclui: gráficos embutidos, macros.

## Leitura em ordem

1. `crates/aura-ingest/src/{model.rs,ingestor.rs}` (TK-001).
2. `specs/007-anexos-multimodais/spec.md` → AC-005.
3. Docs atuais: `calamine` (tipos `Data`, datas, fórmulas com valor em cache).

## Decisões já resolvidas

- Cabeçalho = primeira linha não vazia se ≥ 60% das células forem texto e a linha seguinte tiver tipos diferentes; senão colunas `A, B, C…`.
- Tipo da coluna = tipo majoritário (≥ 80%) nas primeiras 1 000 linhas; datas formatadas ISO.
- Números formatados com ponto decimal no conteúdo enviado; locale só na UI.
- Liberdade local: formato da tabela Markdown (alinhamento).

## Mapa de alterações

- Novo: `crates/aura-ingest/src/sheets.rs` → `SheetsIngestor`, `infer_header`, `infer_types`, `column_stats`.
- Novo: fixtures `tests/corpus/sheets/{vendas.xlsx,dados.csv,legado.xls,planilha.ods,latin1.csv}`.

## Contrato técnico

- Entradas: arquivo de planilha; `Selector::Sheet{name, rows}` para `read`.
- Saídas: `IngestedDoc` com um bloco `Table` por aba + bloco de texto com esquema/estatísticas.
- Invariantes: nenhuma aba omitida no resumo (mesmo se a amostra for vazia).
- Erros: arquivo com senha → `Encrypted`.

## Exemplos de aceite

- **AC-005**: `vendas.xlsx` (abas Jan, Fev, Mar; Jan com 400 linhas, colunas Data, Produto, Valor) → esquema "Jan: 400 linhas × 3 colunas — Data (data), Produto (texto), Valor (número)"; estatísticas de Valor: mín 10.00, máx 999.90, média 30.86, soma 12345.67 (valores calculados à parte no script gerador da fixture); célula com fórmula `=B2*2` mostra o valor salvo. `latin1.csv` com `;` → acentos corretos e 3 colunas.

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-002.1 Unit `infer_header`/`infer_types` red→green.
- [ ] TK-002.2 `column_stats` com valores da fixture.
- [ ] TK-002.3 Integração com cada formato do corpus; `read` por intervalo; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-ingest sheets`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se `calamine` não ler `.xls` legado com fidelidade suficiente.

## Relatório de saída

Relatar resultados e EV refs.
