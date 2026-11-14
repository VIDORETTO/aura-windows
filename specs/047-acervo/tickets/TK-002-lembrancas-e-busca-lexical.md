---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 047-acervo
type: delivery
status: ready
ticket_revision: 1
requires: ["TK-001"]
requirement_refs: ["FR-002", "FR-003"]
acceptance_refs: ["AC-003", "AC-004", "AC-005", "AC-006"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-memory/src/chunk.rs", "crates/aura-memory/src/lexical.rs", "crates/aura-memory/src/lembrancas.rs", "crates/aura-memory/tests/lembrancas.rs", "crates/aura-app/src/acervo.rs", "crates/aura-app/src/host.rs", "crates/aura-app/tests/acervo.rs"]
verification_status: not_run
---

# TK-002 — Lembranças, conciliação e busca lexical

## Objetivo e limites

Notas, Salvos e falas de Reunião viram Lembranças com origem, data e cabeçalho por regra. O índice acompanha sozinho o que é criado, mudado e apagado. A busca lexical com filtros de período e fonte responde na hora.

Não inclui: vetores e worker (TK-003); a ferramenta do agente e a fusão (TK-004).

## Leitura em ordem

1. `specs/047-acervo/plan.md` → `chunk`, `lexical`, Fontes e conciliação.
2. `crates/aura-store/src/migrations/0010_reminders_notes.sql`, `0011_meetings.sql` e `0014_projects.sql` (tabelas de origem).
3. `crates/aura-app/src/notes.rs` (`add`, `delete`) e `crates/aura-app/src/meeting.rs` (`end`/status, `delete`, `set_project`, `recover`, `utterances`).
4. `crates/aura-app/src/host.rs` → laço de 15 s e os comandos que chamam Notas e Reuniões.
5. `tools/spikes/s1-embeddings/src/lib.rs` (`fold`, `tokens`, `STOP`) e `main.rs` (consulta FTS5).

## Decisões já resolvidas

- **Linhagem:** `(fonte, origem, parte)`.
  - `fonte` ∈ `nota`, `salvo`, `reuniao`;
  - `origem` = id da Nota ou da Reunião;
  - `parte` = `""` na Nota e no Salvo, e o `t0` (ms) da primeira fala do grupo na Reunião.
- **Reunião:**
  - só entra com status ≠ `active`;
  - as falas são agrupadas em ordem de `t0` até o grupo durar ≥ 30 s, com teto de 60 s ou 1.500 caracteres;
  - cada linha é `{orador}: {texto}`;
  - o orador vem gravado como `you`/`them` ou como nome, e sai como está.
- **Cabeçalhos:**
  - `Nota — {AAAA-MM-DD}.`;
  - `Salvo — {AAAA-MM-DD}.`;
  - `Reunião — {título} ({tipo}), {AAAA-MM-DD}.`, mais ` Projeto: {nome}.` quando houver Projeto.
  - A data vem do `created_at` da Nota, ou do `started_at` da Reunião somado ao `t0`.
  - A data sai no fuso local, pelo `Clock::utc_offset_secs` do host.
- **`data_ms`:**
  - Nota: `created_at` × 1000, porque o `created_at` está em segundos (`now_secs`);
  - Reunião: `started_at` + `t0`, porque `started_at` já está em ms (`MeetingService::start` grava `now_ms`).
- **`hash`:** SHA-256 de `cabecalho + "\n" + texto`. Hash igual não muda nada; hash diferente troca o texto, zera `modelo` e apaga os vetores.
- **Consulta lexical:** OR dos tokens dobrados fora da lista vazia, cada um entre aspas. Ordem pelo `bm25(lembrancas_fts)`, com filtros `data_ms BETWEEN` e `fonte IN (…)` dentro do SQL.
- **Ganchos no host:**
  - depois de `notes.add` e `notes.delete`;
  - ao encerrar, apagar ou mudar o Projeto de uma Reunião;
  - na partida;
  - a cada 10 min no laço de 15 s.
- Liberdade local: nomes privados, onde guardar o instante da última conciliação.

## Mapa de alterações

- Novo `crates/aura-memory/src/chunk.rs`: `Lembranca` e `chunk_meeting`, puras.
- Novo `crates/aura-memory/src/lexical.rs`: `fold`, `tokens` e `fts_query`, puras.
- Novo `crates/aura-memory/src/lembrancas.rs`: `upsert`, `delete_origin`, `lineage` e `search_lexical`.
- Novo `crates/aura-memory/tests/lembrancas.rs`.
- Existente `crates/aura-app/src/acervo.rs`: `Acervo` com `reconcile` e `search_lexical`.
- Existente `crates/aura-app/src/host.rs`: ganchos e cadência.
- Novo `crates/aura-app/tests/acervo.rs`.

## Contrato técnico

- `aura_memory::Lembranca { fonte, origem, parte, titulo, cabecalho, texto, data_ms }`.
- `aura_memory::chunk::chunk_meeting(meeting: &MeetingSource, utterances: &[UtteranceSource], utc_offset_secs: i32) -> Vec<Lembranca>` e `chunk_note(note: &NoteSource, utc_offset_secs: i32) -> Lembranca`.
- `aura_memory::lexical::fts_query(q: &str) -> Option<String>`: devolve `None` quando não sobra termo.
- `MemoryDb::upsert(&self, l: &Lembranca) -> Result<Upsert, MemoryError>`, com `Upsert::{Inserted(id), Unchanged(id), Replaced(id)}`.
- `MemoryDb::delete_origin(&self, fonte, origem) -> Result<usize, MemoryError>`: apaga as Lembranças e os vetores.
- `MemoryDb::search_lexical(&self, q, filtro: &Filtro, limite) -> Result<Vec<(i64, f64)>, MemoryError>`.
- `Filtro { de_ms: Option<i64>, ate_ms: Option<i64>, fontes: Vec<Fonte> }`.
- `aura_app::acervo::Acervo::{new(store, db), reconcile(&self) -> Result<Reconcile, …>, search_lexical(…)}`, com `Reconcile { inseridas, trocadas, apagadas }`.

## Exemplos de aceite

- **AC-005 (lexical):**
  - criar a Nota "comprar o adaptador PAG-1187" e conciliar → `search_lexical("PAG-1187")` devolve a Lembrança;
  - conciliar de novo → `Reconcile { 0, 0, 0 }`;
  - mudar o texto da Nota no `aura.db` e conciliar → `trocadas = 1`, e a busca acha só o texto novo.
- **AC-006:** apagar a Nota (ou a Reunião) e conciliar → `apagadas` ≥ 1 e a busca não a devolve.
- **AC-003 (lexical):** com a Nota "PAG-1187" e outras 3 Notas e 1 Reunião sem o código, `search_lexical("PAG-1187")` devolve a Nota em 1º.
- **AC-004 (lexical):**
  - Reunião de 15/10/2026 com "orçamento de nuvem" e Nota de 05/11/2026 com "orçamento de marketing";
  - com o filtro de 01/11 a 30/11, `search_lexical("orçamento")` devolve só a Nota;
  - com `fontes = [reuniao]`, só a Reunião.
- `chunk_meeting` com literais calculados à parte:
  - falas em 0, 10, 20, 35, 70, 80 e 130 s, com 5 s cada, viram grupos com `parte` = 0, 70 000 e 130 000;
  - o primeiro grupo vai de 0 a 40 s, porque ≥ 30 s fecha o grupo depois da fala de 35 s.

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-002.1 `chunk_meeting` e `fts_query` (puras), red → green.
- [ ] TK-002.2 `upsert`, `delete_origin` e `search_lexical` com filtros (AC-003, AC-004 lexical).
- [ ] TK-002.3 `Acervo::reconcile` (AC-005 lexical, AC-006).
- [ ] TK-002.4 Ganchos e cadência no host; regressão, clippy, fmt e evidência.

## Validação

- Comandos:
  - `cargo test -p aura-memory`;
  - `cargo test -p aura-app --test acervo`;
  - `cargo test --workspace --exclude aura-desktop`, com as mesmas exclusões do TK-001;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo fmt --all -- --check`.
- Estado esperado: passam.

## Condição de retorno à planejadora

Pare e devolva se a origem não tiver id estável (Nota ou Reunião sem chave), ou se o gancho no host exigir mudar um comando IPC.

## Relatório de saída

Arquivos e símbolos, ACs, comandos, `EV`, limitações e pendências.
