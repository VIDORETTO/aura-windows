# TDD

Primeiro vermelho de cada aceite, na seam pública do `plan.md`. Os oráculos são literais da spec ou calculados à parte.

AC-007 (TK-001):
- `MemoryDb::open` com a chave A grava; os 16 primeiros bytes do arquivo ≠ `SQLite format 3\0`; reabrir com a chave B → `WrongKey`; com A → `schema_version() == 1`; `MATCH 'orcamento'` acha "orçamento".
- No `aura-app`, dois `Vault` sobre o mesmo `Store` abrem o mesmo banco; o `Vault` de outro `Store` recebe `WrongKey`.
- Vermelho atual: o crate não existe.

TK-002, `chunk_meeting`: falas em 0, 10, 20, 35, 70, 80 e 130 s, com 5 s cada → `parte` = 0, 70 000 e 130 000; o primeiro grupo vai de 0 a 40 s.

AC-003 (lexical): Nota "comprar o adaptador PAG-1187" + 3 Notas e 1 Reunião sem o código → `search_lexical("PAG-1187")` devolve a Nota em 1º.

AC-004 (lexical):
- Reunião de 15/10/2026 ("orçamento de nuvem") e Nota de 05/11/2026 ("orçamento de marketing");
- filtro de 01/11 a 30/11 → só a Nota;
- `fontes = [reuniao]` → só a Reunião.

AC-005:
- lexical: conciliar acha na hora; conciliar de novo → (0, 0, 0); texto mudado → `trocadas = 1`, e só o texto novo é achado;
- vetor: `pending` contém a Nota; `embed_pending` com o `Embedder` de teste esvazia a fila; `search_vector` com o vetor do mesmo texto devolve a Nota em 1º; segunda passada → 0.

AC-006: apagar a Nota ou a Reunião e conciliar → as Lembranças e os vetores somem; a busca não as devolve.

AC-008: `Embedder` que falha → erro legível; a fila e o lexical continuam; `memory_search` → `semantica: incompleta` com o motivo.

Equivalência da varredura: 50 vetores literais com margem > 0,05 entre o 10º e o 11º → mesmo top-10 da busca exata calculada à parte.

`rrf([1, 2, 3], [3, 1, 4])` com k = 60 → ordem 1, 3, 2, 4.

AC-001: Reunião de 15/10/2026, fala de `them` aos 12:30 com "fechamos o orçamento de nuvem em R$ 15 mil por mês", e Nota "orçamento de marketing: R$ 8 mil" → `memory_search("quanto ficou o orçamento de nuvem?")` devolve a Reunião em 1º, com origem, orador, instante e data.

AC-002 (motor real, manual): "quanto vamos gastar por mês com servidores?" → o trecho da Reunião entre os 3 primeiros.

AC-009: `never` recusa sem ler; `ask` passa pelo `ConsentBroker`; `always` vai direto; uma linha no `access_log` por chamada.

AC-010: MRR no conjunto dourado com o bekko-a8m em 0,892 ± 0,02.

AC-011: p95 da busca completa ≤ 300 ms com 500 mil trechos no desktop.

Métricas do bench: posições [1, 3, sem resposta, 2] → acerto@1 0,25, acerto@3 0,75, MRR = (1 + 1/3 + 0 + 1/2)/4.
