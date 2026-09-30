---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 007-anexos-multimodais
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-002", "TK-003"]
requirement_refs: ["FR-005", "FR-006"]
acceptance_refs: ["AC-009", "AC-010"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-ingest/src/cache.rs", "crates/aura-mcp/src/tools/attachments.rs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-006 — Originais no workspace, `attachment_read` e cache por hash

## Objetivo e limites

Entrega a cópia do original para `workspaces/<id>/attachments/<hash>.<ext>`, o cache de Ingestão por hash com limpeza LRU (2 GB), e a tool MCP `attachment_read{attachment_id, selector}` restrita aos anexos da Conversa.

Não inclui: busca semântica entre anexos.

## Leitura em ordem

1. `crates/aura-ingest/src/{pipeline.rs,sheets.rs,office.rs}` (TK-001–TK-003) → `Ingestor::read`.
2. `crates/aura-mcp/src/tools/screen.rs` (004 TK-004) → padrão de tool e header `X-Aura-Conversation`.
3. `specs/007-anexos-multimodais/plan.md` → OT-003.

## Decisões já resolvidas

- Chave de cache = SHA-256 do conteúdo + versão do extrator.
- `selector` JSON: `{"pages":"3-5"}`, `{"sheet":"Fev","rows":"100-150"}`, `{"slides":"2"}`, `{"time":"01:00-02:30"}`, `{"section":"Resumo"}`.
- Resultado limitado a 20 k tokens estimados por chamada (truncado com aviso e sugestão de seletor menor).
- Liberdade local: parser do seletor.

## Mapa de alterações

- Novo: `crates/aura-ingest/src/cache.rs` → `IngestCache::{get, put, evict_lru}`.
- Novo: `crates/aura-mcp/src/tools/attachments.rs`.
- Existente: `pipeline.rs` → usa cache e copia original.

## Contrato técnico

- Entradas: `attachment_id` (da Conversa), `selector`.
- Saídas: conteúdo (texto/imagens) do trecho.
- Invariantes: OT-003.
- Erros: anexo de outra Conversa → `not_found`; seletor inválido → mensagem com exemplos.

## Exemplos de aceite

- **AC-009**: cliente MCP com header da Conversa A: `attachment_read{vendas.xlsx, {"sheet":"Fev","rows":"100-150"}}` → 51 linhas com os valores da fixture; header da Conversa B → `not_found`.
- **AC-010**: anexar `relatorio-12p.pdf` duas vezes → extrator chamado 1 vez (contador do ingestor falso/instrumentado); cache acima de 2 GB → remove os menos usados.

## Dependências e sequência de execução

Depende de: TK-002, TK-003.

- [ ] TK-006.1 `IngestCache` (red→green, incluindo LRU).
- [ ] TK-006.2 Cópia do original + integração no pipeline.
- [ ] TK-006.3 `attachment_read` com seletores (um por tipo); evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-ingest cache pipeline -p aura-mcp attachments`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se o header de Conversa (004 TK-004) não estiver disponível e houver Conversas simultâneas.

## Relatório de saída

Relatar resultados e EV refs.
