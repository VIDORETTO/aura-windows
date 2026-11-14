---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 047-acervo
type: delivery
status: ready
ticket_revision: 1
requires: ["TK-002"]
requirement_refs: ["FR-003", "FR-004"]
acceptance_refs: ["AC-005", "AC-008"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-memory/src/vector.rs", "crates/aura-memory/tests/vector.rs", "crates/aura-worker", "crates/aura-asr/embeddings.toml", "crates/aura-asr/src/catalog.rs", "crates/aura-app/src/acervo.rs", "crates/aura-app/src/platform.rs", "crates/aura-app/src/host.rs", "crates/aura-app/tests/acervo.rs", "crates/aura-win", "apps/desktop/src-tauri/src"]
verification_status: not_run
---

# TK-003 — Vetores, embeddings no worker e lote ocioso

## Objetivo e limites

Toda Lembrança ganha vetor binário + int8, calculado no worker com o bekko-a8m (ADR 0014) em lotes, com o usuário ocioso. A varredura vetorial do ADR 0012 roda no worker. Sem worker ou sem modelo, nada quebra: a busca fica só lexical e diz isso.

Não inclui: a ferramenta `memory_search` e a fusão (TK-004); o bench (TK-005).

## Leitura em ordem

1. `specs/047-acervo/plan.md` → `vector`, Embeddings em lote, Processos.
2. `specs/046-spikes-segundo-cerebro/s2.md` (desenho medido) e `tools/spikes/s2-vetores/src/{lib.rs, main.rs}`.
3. `tools/spikes/s1-embeddings/src/main.rs` → `embed` (tokenização, pooling por média, L2, truncamento de 256).
4. `crates/aura-worker/src/{main.rs, engine.rs}` e `crates/aura-asr/src/worker_client.rs` (protocolo, `ensure_model`, ociosidade).
5. `crates/aura-asr/src/{catalog.rs, download.rs}` e `crates/aura-asr/models.toml`.
6. `crates/aura-app/src/platform.rs` → `Foreground`; `crates/aura-win` (onde mora a API do Windows).

## Decisões já resolvidas

- **Quantização e varredura** como no S2:
  - `to_binary`: bit 1 para componente > 0;
  - `to_int8`: ×254, arredondado, saturado em ±127;
  - Hamming por `u64::count_ones`;
  - 200 candidatos → int8 → k.
- **Faixa de `rowid` do período:** `min(id)`/`max(id)` das Lembranças com `data_ms` no intervalo. A data e a fonte de cada linha de `vetores_bin` são conferidas antes de disputar os candidatos.
- **Worker:** abre o `memory.db` com `open_read_only` e cache de N × 72 bytes + 4 MB. O host é o único escritor.
- **Métodos RPC novos do worker:**
  - `embed.load {engine, modelDir}`, com `engine` ∈ `fake`, `onnx-bekko`;
  - `embed.passages {texts}` → `{vectors: [base64 f32 LE]}`;
  - `memory.open {path, key}`, com a chave em hex;
  - `memory.search {query, deMs?, ateMs?, fontes?, k1, k}` → `{hits: [{id, score}]}`.
- **Motor `fake`:** vetor determinístico por texto, para testes. É o hash SHA-256 do texto dobrado, expandido para 384 componentes e normalizado: textos iguais dão vetores iguais.
- **Motor real:** `ort` `=2.0.0-rc.12` com os recursos do `transcribe-rs`, mais `tokenizers` 0.22 sem `onig`. Pooling por média e normalização L2, sem prefixo, truncamento em 256 tokens. Fica atrás do recurso `engines`.
- **Catálogo:** `crates/aura-asr/embeddings.toml`, lido por `Catalog::embeddings()`. O ASR continua lendo só o `models.toml`. Os arquivos `onnx/model.onnx` e `tokenizer.json` do `hotchpotch/bekko-embedding-v1-a8m` vêm de um commit fixo, com SHA-256 e tamanho conferidos nos arquivos baixados.
- **Lote:**
  - até 32 textos;
  - no máximo um por minuto;
  - só com `Foreground::idle_ms() >= 60_000`, ou desconhecido;
  - sem Reunião ativa e sem Push-to-talk.
- **Texto do vetor:** `{cabecalho} {texto}`.
- **Modelo do vetor:** o `modelo` da Lembrança guarda o id do catálogo (`bekko-a8m`). Lembrança com `modelo` diferente do atual volta para a fila.
- **Ociosidade:** `Foreground::idle_ms(&self) -> Option<u64>`, com padrão `None`. A implementação Windows usa `GetLastInputInfo` em `aura-win`, chamada pelo adaptador do src-tauri.
- Liberdade local: organização interna do worker, nomes privados.

## Mapa de alterações

- Novo `crates/aura-memory/src/vector.rs`: quantização, `TopK`, `scan`, `set_vectors` e `pending`. Novo `crates/aura-memory/tests/vector.rs`.
- Existente `crates/aura-worker/src/main.rs`: métodos `embed.*` e `memory.*`.
- Novo `crates/aura-worker/src/embed.rs`. Existente `crates/aura-worker/Cargo.toml`.
- Novo `crates/aura-asr/embeddings.toml`. Existente `crates/aura-asr/src/catalog.rs`.
- Existente `crates/aura-app/src/acervo.rs`: `trait Embedder`, `WorkerEmbedder` e `embed_pending`.
- Existente `crates/aura-app/src/platform.rs`: `idle_ms`.
- Existente `crates/aura-app/src/host.rs`: lote no laço de 15 s.
- Existente `crates/aura-win`: `user_idle_ms`. Existente `apps/desktop/src-tauri/src`: adaptador.

## Contrato técnico

- `aura_memory::vector::{to_binary, to_int8, hamming, dot_i8}`.
- `MemoryDb::set_vectors(&self, id, v: &[f32], modelo: &str) -> Result<(), MemoryError>`.
- `MemoryDb::pending(&self, modelo, limite) -> Result<Vec<(i64, String)>, MemoryError>`.
- `MemoryDb::search_vector(&self, q: &[f32], filtro: &Filtro, k1, k) -> Result<Vec<(i64, f32)>, MemoryError>`.
- `aura_app::acervo::Embedder` (`async`):
  - `passages(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String>`;
  - `search(&self, q: &str, filtro: &Filtro, k1, k) -> Result<Vec<(i64, f32)>, String>`.
- `Acervo::embed_pending(&self, embedder: &dyn Embedder, limite) -> Result<usize, …>`.
- Erros: worker ausente, modelo ausente e falha no motor viram `Err(String)` com motivo legível, nunca pânico.

## Exemplos de aceite

- **Equivalência:** 50 vetores literais, normalizados, e 5 perguntas, montados de modo que o top-10 exato tenha margem > 0,05 entre o 10º e o 11º. `search_vector` com k1 = 50 devolve o mesmo top-10 da busca exata em f32 calculada à parte no teste.
- **AC-005 (vetor):**
  - Nota nova, conciliada → `pending` a contém;
  - `embed_pending` com o `Embedder` de teste → `pending` vazio, e `search_vector` com o vetor do mesmo texto devolve a Nota em 1º;
  - rodar `embed_pending` de novo → 0.
- **AC-008:** com o `Embedder` que falha ("modelo não baixado"), `embed_pending` devolve erro legível, e as Lembranças continuam na fila e no lexical.
- **Worker** (protocolo, motor `fake`):
  - `embed.load {engine: "fake"}` + `embed.passages ["a", "a", "b"]` → 3 vetores de 384, o 1º igual ao 2º;
  - `memory.open` com um banco criado pelo host + `memory.search` devolve o id esperado.
- **Motor real:** teste `#[ignore]` com `AURA_EMBED_MODEL_DIR`. O cosseno entre o vetor do Rust e o vetor de referência do S1 para 3 frases é ≥ 0,98. É evidência manual.

## Dependências e sequência de execução

Depende de: TK-002.

- [ ] TK-003.1 Quantização e `search_vector` (equivalência), red → green.
- [ ] TK-003.2 `set_vectors`, `pending`, `trait Embedder` e `embed_pending` (AC-005 vetor, AC-008).
- [ ] TK-003.3 Worker: motor `fake`, `memory.*` e protocolo; motor real atrás de `engines`; catálogo.
- [ ] TK-003.4 Ociosidade (`aura-win`) e lote no host; regressão, clippy, fmt e evidência (com o motor real, manual).

## Validação

- Comandos:
  - `cargo test -p aura-memory`;
  - `cargo test -p aura-worker`;
  - `cargo test -p aura-app --test acervo`;
  - `cargo test -p aura-asr`;
  - `cargo check -p aura-worker --features engines`;
  - `AURA_EMBED_MODEL_DIR=<dir> cargo test -p aura-worker --features engines -- --ignored embed_real`;
  - `cargo test --workspace --exclude aura-desktop`, com as exclusões do TK-001;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo fmt --all -- --check`.
- Estado esperado: passam.

## Condição de retorno à planejadora

Pare e devolva nestes casos:
- o `ort` direto conflita com o do `transcribe-rs`, gerando duas versões ou dois runtimes;
- o modelo exige outro formato de entrada;
- o worker precisa ficar residente.

## Relatório de saída

Arquivos e símbolos, ACs, comandos, `EV`, limitações e pendências.
