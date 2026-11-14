---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 047-acervo
type: delivery
status: in_progress
ticket_revision: 2
requires: []
requirement_refs: ["FR-001"]
acceptance_refs: ["AC-007"]
spec_revision: 1
plan_revision: 1
owned_areas: ["Cargo.toml", "Cargo.lock", "crates/aura-memory", "crates/aura-app/src/acervo.rs", "crates/aura-app/src/lib.rs", "crates/aura-app/Cargo.toml", ".github/workflows/ci.yml", "CONTEXT.md", "docs/architecture/overview.md"]
verification_status: not_run
---


# TK-001 — Banco cifrado do Acervo

## Objetivo e limites

Criar o `memory.db` cifrado (SQLCipher, ADR 0012), com migrações próprias e o esquema do Acervo, e abrir esse banco com a chave derivada do cofre. Trocar o `rusqlite` do workspace para SQLCipher sem quebrar o `aura.db`.

Não inclui: Lembranças, conciliação ou busca (TK-002); vetores e worker (TK-003).

## Leitura em ordem

1. `docs/adr/0012-acervo-cifrado-com-vetores-em-blob.md` e `specs/046-spikes-segundo-cerebro/s2.md`.
2. `specs/047-acervo/plan.md` → Desenho (crate `aura-memory`, chave e cofre, SQLite do workspace).
3. `crates/aura-store/src/lib.rs` → `Store::open`, `migrate` (padrão de migração por `user_version`).
4. `crates/aura-store/src/vault.rs` → `Vault::open`, `derive_key`.
5. `tools/spikes/s2-vetores/tests/sqlcipher.rs` (como o SQLCipher se comporta).

## Decisões já resolvidas

- `rusqlite` do workspace com `features = ["bundled-sqlcipher-vendored-openssl"]`. O `libsqlite3-sys` é um só por binário, então o `aura.db` também passa a usar o SQLite do SQLCipher, sem chave.
- Chave crua de 32 bytes: `PRAGMA key = "x'<64 hex>'"`, sem PBKDF2.
- A chave do Acervo é `Vault::derive_key("memory-db-v1")`.
- Esquema da migração 1, como no plano:
  - `lembrancas` + `lembrancas_fts`, com conteúdo externo e gatilhos;
  - `vetores_bin(id, data_ms, fonte, v)`;
  - `vetores_i8(id, v)`.
- Erros com `thiserror`. Chave errada vira `MemoryError::WrongKey`, a partir do `SQLITE_NOTADB` na primeira leitura.
- Liberdade local: nomes privados, organização interna do crate.

## Mapa de alterações

- Novo: `crates/aura-memory/{Cargo.toml, src/lib.rs, src/db.rs, src/migrations/0001_acervo.sql, tests/db.rs}`.
- Existente:
  - `Cargo.toml` da raiz: membro novo, dependência do workspace e features do `rusqlite`;
  - `Cargo.lock`.
- Novo: `crates/aura-app/src/acervo.rs` com `open_memory_db(vault, path)`; existente: `crates/aura-app/src/lib.rs` (`pub mod acervo`) e `crates/aura-app/Cargo.toml`.
- Existente: `.github/workflows/ci.yml`, com um passo no job do Windows que confere o Perl nativo.
- Existente:
  - `CONTEXT.md`: Segundo Cérebro, Acervo e Lembrança;
  - `docs/architecture/overview.md`: `aura-memory` e `memory.db`.

## Contrato técnico

- `aura_memory::MemoryDb::open(path: &Path, key: &[u8; 32]) -> Result<MemoryDb, MemoryError>`: cria o arquivo e a pasta se faltarem, aplica a chave, WAL, `foreign_keys` e as migrações.
- `MemoryDb::open_read_only(path, key, cache_kib: i64) -> Result<MemoryDb, MemoryError>`.
- `MemoryDb::with_conn<T>(&self, f) -> Result<T, MemoryError>`.
- `MemoryDb::schema_version(&self) -> Result<u32, MemoryError>`.
- `MemoryError`: `WrongKey`, `Sqlite(rusqlite::Error)` e `Io(std::io::Error)`.
- `aura_app::acervo::open_memory_db(vault: &Vault, path: &Path) -> Result<MemoryDb, MemoryError>`.
- A chave nunca entra em `Debug`, log ou mensagem de erro.

## Exemplos de aceite

- **AC-007**, em `aura-memory/tests/db.rs`:
  - depois de `open` com a chave A e uma escrita, os 16 primeiros bytes do arquivo ≠ `SQLite format 3\0`;
  - reabrir com a chave B → `Err(WrongKey)`;
  - reabrir com a chave A → `schema_version() == 1`;
  - inserir uma Lembrança com "fechamos o orçamento" e consultar `lembrancas_fts MATCH 'orcamento'` → 1 linha.
- **AC-007**, em `aura-app`, com `Store::open_in_memory` e `StaticKeyProtector`:
  - dois `Vault::open` sobre o mesmo `Store` abrem o mesmo `memory.db`;
  - o `Vault` de outro `Store` recebe `WrongKey`.
- Regressão: a suíte do workspace passa com o SQLite do SQLCipher.

## Dependências e sequência de execução

Depende de: nenhum ticket.

- [ ] TK-001.1 Crate e troca de features; build do workspace com Perl nativo.
- [ ] TK-001.2 AC-007 vermelho (crate sem `open`), depois verde: cifra, chave errada, versão, FTS5.
- [ ] TK-001.3 `open_memory_db` pelo cofre (AC-007 no `aura-app`).
- [ ] TK-001.4 CI, vocabulário, arquitetura; regressão, clippy, fmt e evidência.

## Validação

- Comandos (Windows, Strawberry Perl no PATH):
  - `cargo test -p aura-memory`;
  - `cargo test -p aura-app acervo`;
  - `cargo test --workspace --exclude aura-desktop`, pulando os dois testes de Media Foundation do `aura-win` que travam neste PC;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo fmt --all -- --check`.
- Estado esperado: passam.

## Condição de retorno à planejadora

Pare e devolva se o `aura.db` ou a suíte depender de recurso do SQLite ausente no 3.51.3, ou se o build do workspace não fechar com o SQLCipher.

## Relatório de saída

Arquivos e símbolos, ACs, comandos, `EV`, limitações e pendências.
