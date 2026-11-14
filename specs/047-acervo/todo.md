# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [ ] TK-001 — TK-001 — Banco cifrado do Acervo
Status: `in_progress` | Bloqueado por: nenhum

- [ ] TK-001.1 Crate e troca de features; build do workspace com Perl nativo.
- [ ] TK-001.2 AC-007 vermelho (crate sem `open`), depois verde: cifra, chave errada, versão, FTS5.
- [ ] TK-001.3 `open_memory_db` pelo cofre (AC-007 no `aura-app`).
- [ ] TK-001.4 CI, vocabulário, arquitetura; regressão, clippy, fmt e evidência.

## [ ] TK-002 — TK-002 — Lembranças, conciliação e busca lexical
Status: `ready` | Bloqueado por: TK-001

- [ ] TK-002.1 `chunk_meeting` e `fts_query` (puras), red → green.
- [ ] TK-002.2 `upsert`, `delete_origin` e `search_lexical` com filtros (AC-003, AC-004 lexical).
- [ ] TK-002.3 `Acervo::reconcile` (AC-005 lexical, AC-006).
- [ ] TK-002.4 Ganchos e cadência no host; regressão, clippy, fmt e evidência.

## [ ] TK-003 — TK-003 — Vetores, embeddings no worker e lote ocioso
Status: `ready` | Bloqueado por: TK-002

- [ ] TK-003.1 Quantização e `search_vector` (equivalência), red → green.
- [ ] TK-003.2 `set_vectors`, `pending`, `trait Embedder` e `embed_pending` (AC-005 vetor, AC-008).
- [ ] TK-003.3 Worker: motor `fake`, `memory.*` e protocolo; motor real atrás de `engines`; catálogo.
- [ ] TK-003.4 Ociosidade (`aura-win`) e lote no host; regressão, clippy, fmt e evidência (com o motor real, manual).

## [ ] TK-004 — TK-004 — Ferramenta `memory_search`, Política e Registro de acesso
Status: `ready` | Bloqueado por: TK-003

- [ ] TK-004.1 `rrf` (literais), red → green.
- [ ] TK-004.2 `Acervo::search` com fusão, filtros e saída (AC-001 com o `Embedder` de teste, AC-004).
- [ ] TK-004.3 Política, `memory_search` no despacho e Registro de acesso (AC-009); contrato IPC.
- [ ] TK-004.4 Teste com o motor real (AC-001, AC-002); regressão, clippy, fmt, typecheck e evidência.

## [ ] TK-005 — TK-005 — `aura-bench memory`: conjunto dourado e escala
Status: `ready` | Bloqueado por: TK-004

- [ ] TK-005.1 Métricas (literais), red → green.
- [ ] TK-005.2 Conversão do corpus e execução com o modelo (AC-010).
- [ ] TK-005.3 Escala (AC-011), com prioridade baixa.
- [ ] TK-005.4 README, regressão, clippy, fmt e evidência.
