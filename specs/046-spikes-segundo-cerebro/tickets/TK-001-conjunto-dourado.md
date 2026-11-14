---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 046-spikes-segundo-cerebro
type: spike
status: done
ticket_revision: 5
requires: []
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-005"]
spec_revision: 1
plan_revision: 1
owned_areas: ["tools/spikes/s1-embeddings/golden"]
verification_status: passed
---





# TK-001 — Conjunto dourado sintético do S1

## Objetivo e limites

Criar o conjunto dourado: 240 documentos (reunião, conversa, nota, página, anexo, tela) e 100 perguntas com resposta única. Os autores são diferentes: um subagente escreve os documentos, outro escreve as perguntas vendo só o corpus.

Não inclui: medição (TK-002).

## Leitura em ordem

1. `specs/046-spikes-segundo-cerebro/spec.md` → AC-005.
2. `docs/research/segundo-cerebro-medicoes-2026-11.md` §2 → viés do conjunto anterior (8% de sobreposição nas perguntas por paráfrase).

## Decisões já resolvidas

- Perguntas escritas por modelo (delegação do usuário); o viés fica declarado no relatório.
- Mínimo de 30 perguntas por termo exato e 50 por paráfrase, mais 20 livres, para medir o efeito do lexical sem o viés extremo do conjunto anterior.
- Validação mecânica: ids válidos, perguntas únicas, sobreposição lexical por estilo medida e registrada.

## Mapa de alterações

- Novo: `tools/spikes/s1-embeddings/golden/corpus.json`, `queries.json`, `README.md` (origem, autoria, viés, licença: sintético, sem dado real).

## Contrato técnico

- `corpus.json`: `[{id, fonte, data, titulo, texto}]`.
- `queries.json`: `[{id, pergunta, doc, estilo}]`, com `estilo` ∈ {`exato`, `parafrase`, `livre`}.

## Exemplos de aceite

- **AC-005**: 240 documentos em 6 fontes; 100 perguntas com `doc` válido e único; sobreposição lexical por estilo medida.

## Dependências e sequência de execução

Depende de: nenhum.

- [ ] TK-001.1 Corpus pelo subagente A.
- [ ] TK-001.2 Perguntas pelo subagente B (só vê o corpus).
- [ ] TK-001.3 Validação mecânica e README.
- [ ] TK-001.4 Evidência.

## Validação

- Script de validação em Python sobre os JSON (contagens, ids, sobreposição).

## Condição de retorno à planejadora

Pare e devolva se uma medição exigir mudar código de produto, se um modelo ou biblioteca não tiver licença permissiva, ou se o resultado contradisser um ADR de forma que exija decisão do dono do produto (registre a proposta).

## Relatório de saída

Arquivos, comandos, números, `EV`, limitações, decisão proposta e próxima ação.
