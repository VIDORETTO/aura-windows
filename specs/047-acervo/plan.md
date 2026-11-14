---
schema: hybrid/plan
schema_version: 1.0
effort_id: 047-acervo
revision: 1
spec_revision: 1
status: ready
---

# Plano — Acervo

Consome o spec r1, o ADR 0012 (aceito com o S2, `specs/046-spikes-segundo-cerebro/s2.md`) e o ADR 0014 (S1). Baseline: branch `claude/segundo-cerebro-fase0`. Cinco tickets em série; cada um entrega um comportamento testável na seam pública.

## Desenho

### Processos e dono de cada dado

- **Host (`aura-app`)** é o único escritor do `memory.db`. Ele:
  - cria o banco e aplica as migrações;
  - concilia as fontes do `aura.db` com as Lembranças;
  - grava texto, FTS5 e vetores;
  - responde à parte lexical da busca;
  - funde os resultados.
- **Worker (`aura-worker`)** calcula embeddings (bekko-a8m, ADR 0014) e faz a varredura vetorial. Abre o `memory.db` só para leitura, com cache dimensionado (N × 72 bytes + 4 MB). Continua sob demanda (ADR 0005, 2 min de ociosidade). A primeira busca depois de ociosidade paga a carga do modelo (~1 s, S1) e a primeira varredura (46 ms com 100 mil, S2).
- Sem worker, sem modelo baixado ou com estouro de tempo (5 s), a busca devolve só o lexical, com `semantica: "incompleta"` e o motivo (AC-008).

### Crate novo `crates/aura-memory`

Sem API do Windows. Depende de `rusqlite`, `serde`, `thiserror` e `aura-core`. É usado pelo host e pelo worker.

- `db`: `MemoryDb::open(path, key: &[u8; 32])` com `PRAGMA key = "x'…'"`; a chave crua pula o PBKDF2 (0,3 ms contra 126 ms, S2). Também `open_read_only`, migrações próprias (`user_version`) e `PRAGMA cache_size`.
- `schema`:
  - `lembrancas(id INTEGER PRIMARY KEY, fonte TEXT, origem TEXT, parte TEXT, titulo TEXT, cabecalho TEXT, texto TEXT, data_ms INTEGER, hash TEXT, modelo TEXT NULL)`, com índice único em `(fonte, origem, parte)` e índice em `data_ms`;
  - `lembrancas_fts`: FTS5 de conteúdo externo (`content='lembrancas'`) sobre `cabecalho, texto`, com `tokenize='unicode61 remove_diacritics 2'` e gatilhos de sincronia;
  - `vetores_bin(id INTEGER PRIMARY KEY, data_ms INTEGER, fonte INTEGER, v BLOB)` com 48 bytes, e `vetores_i8(id INTEGER PRIMARY KEY, v BLOB)` com 384 bytes. O `id` é o mesmo da Lembrança.
- `chunk`: funções puras que montam as Lembranças.
  - Nota e Salvo: uma Lembrança cada, com cabeçalho `Nota — {data}.` ou `Salvo — {data}.`.
  - Reunião: falas consecutivas agrupadas até ≥ 30 s de duração, com teto de 60 s ou 1.500 caracteres. Linhas no formato `{orador}: {texto}`. Cabeçalho `Reunião — {título} ({tipo}), {data}. Projeto: {nome}.`. A `parte` é o `t0` da primeira fala do grupo.
  - `hash` = SHA-256 de cabeçalho + texto.
- `lexical`:
  - normalização e lista vazia portadas do S1 (`fold`, `tokens`, `STOP`);
  - consulta FTS5 = OR dos termos fora da lista vazia, cada um entre aspas;
  - BM25 com filtros de período e fonte feitos dentro do SQL.
- `vector`: `to_binary`, `to_int8` (×254, saturado em ±127), `hamming`, `dot_i8` e `TopK`, portados do S2.
  - A varredura lê `vetores_bin` só na faixa de `rowid` do período: `min(id)` e `max(id)` das Lembranças com `data_ms` no intervalo, via índice. Dentro da faixa, confere `data_ms` e `fonte` de cada linha antes de disputar os 200 candidatos.
  - Os candidatos são reordenados pelo int8 e saem os k melhores.
  - Assim o filtro é exato mesmo quando o `rowid` sai da ordem do tempo, por reindexação ou carga inicial, que é feita em ordem cronológica.
- `fusion`: RRF k = 60 com pesos iguais (ADR 0014) sobre as listas lexical e vetorial; empate desfeito pelo `id` mais novo.

### Chave e cofre

- `Vault::derive_key("memory-db-v1")` dá a chave de 32 bytes (HKDF a partir da DEK já protegida por DPAPI).
- O host passa a chave ao worker pelo pipe privado, como já faz com as chaves derivadas (`seal_with_key`).
- A chave nunca vai para log nem arquivo.

### SQLite do workspace

`rusqlite` do workspace passa de `bundled` para `bundled-sqlcipher-vendored-openssl`.

- O `libsqlite3-sys` é um só por binário, então o `aura.db` também passa a usar o SQLite do SQLCipher (3.51.3, antes 3.53.2). Ele continua sem chave e com o mesmo formato. A suíte inteira roda como oráculo de compatibilidade.
- O build exige Perl nativo no Windows. O CI do Windows ganha um passo que confere `perl -e "print $^O"` = `MSWin32`; se não for, instala o Strawberry Perl. A imagem Windows Server 2025 lista Perl 5.42.0, mas não diz qual distribuição nem se está no PATH ([runner-images](https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-Readme.md)).
- O CI do Linux já tem `perl` e `make`.

### Fontes e conciliação (host, `crates/aura-app/src/acervo.rs`)

- `Acervo::reconcile(now)` compara as fontes do `aura.db` com as Lembranças pela linhagem `(fonte, origem, parte)` e pelo `hash`:
  - insere o que é novo;
  - troca o que mudou;
  - apaga o que sumiu, junto com os vetores.
- É idempotente: rodar duas vezes não muda nada (AC-005).
- Fontes:
  - `notes` com `kind` `note` ou `saved`;
  - `meetings` com status diferente de `active`, mais as `meeting_utterances` delas e o Projeto.
- Chamado:
  - ao salvar ou apagar uma Nota;
  - ao encerrar, apagar ou mudar o Projeto de uma Reunião;
  - na partida;
  - a cada 10 min no laço de 15 s do host.
- Carga inicial em ordem cronológica.
- Lembrança nova entra sem vetor (`modelo` nulo) e já aparece no lexical (AC-005).

### Embeddings em lote

- Lembranças sem vetor, ou com `modelo` diferente do atual, vão ao worker em lotes de até 32. O host grava os vetores. O cálculo usa a mesma montagem de documento do S1 (`{cabeçalho} {texto}`) e truncamento de 256 tokens.
- Cadência: um lote por minuto no máximo, só com o usuário ocioso há ≥ 60 s, sem Reunião ativa e sem Push-to-talk.
  - Novo `Foreground::idle_ms()`, com padrão `None`. A implementação Windows fica em `aura-win`, com `GetLastInputInfo`.
  - Sem a informação de ociosidade (Linux, testes), o lote roda com a mesma cadência e as mesmas exclusões.
- Worker, com o recurso `engines`:
  - `embed.load {modelDir}`;
  - `embed.passages {texts}` → vetores em base64 f32;
  - `memory.open {path, key}`;
  - `memory.search {query, de?, ate?, fontes?, k1, k}` → `[{id, score}]`, que calcula o vetor da pergunta e faz a varredura.
- Dependências novas do worker: `ort` fixado em `=2.0.0-rc.12`, a mesma do lockfile, e `tokenizers` 0.22 (Apache-2.0) sem `onig`.
- Modelo no catálogo:
  - arquivo novo `crates/aura-asr/embeddings.toml`, lido pelo mesmo `Catalog`, para não aparecer na lista de Modelos ASR;
  - arquivos `onnx/model.onnx` e `tokenizer.json` do `hotchpotch/bekko-embedding-v1-a8m` num commit fixo, com SHA-256 e tamanho;
  - download pelo `aura-asr::download` existente, com verificação.

### Ferramenta `memory_search` (ADR 0006)

- Entrada: `query`; `de` e `ate` (ISO 8601, opcionais); `fontes` (subconjunto de `nota`, `salvo`, `reuniao`); `limite` (padrão 8, teto 20).
- Saída por trecho:
  - `id`, `fonte`, `origem` (id da Nota ou da Reunião), `titulo`, `data`;
  - em Reunião, `instante` (mm:ss) e `oradores`;
  - `texto`, cortado em 1.000 caracteres;
  - `pontuacao`.
- No topo da saída, `semantica: "completa" | "incompleta"`.
- Política:
  - nova `Source::Memory` e campo `Policy.memory_agent: AgentPermission`, com `#[serde(default)]` = `Ask`;
  - `Never` recusa sem ler;
  - `Ask` passa pelo `ConsentBroker` e pelas concessões por Conversa (`agent_grants`), como as outras Fontes;
  - cada acesso vai ao Registro de acesso com a consulta e o número de trechos (AC-009).
- O formato da `Policy` muda o contrato IPC dourado (ADR 0009): `UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract` e `src/ipc/types.ts`. O controle na UI fica para o esforço 049; aqui só o tipo.
- `note_search` e `meeting_search` não mudam.

### `aura-bench memory`

- Fixture: `bench/acervo/golden/` (cópia do conjunto do S1, com README de origem).
  - Os documentos viram Notas e Reuniões sintéticas num `aura.db` temporário.
  - O índice passa pelo mesmo `Acervo::reconcile` e pelo mesmo `memory_search`.
  - Relata acerto@1, acerto@10, MRR e p50/p95 (AC-010, com `--model-dir`).
- `--escala 500000`:
  - base sintética com texto gerado de um vocabulário e vetores aleatórios normalizados;
  - mede p50/p95 da busca completa (lexical + vetorial + fusão) sem filtro e com 30 dias (AC-011).
- Os dois rodam fora do CI, porque exigem o modelo e minutos de CPU, e entram como evidência manual.

### Vocabulário

`CONTEXT.md` ganha **Segundo Cérebro**, **Acervo** e **Lembrança**, com as definições do plano §2. `docs/architecture/overview.md` ganha o crate `aura-memory` e o `memory.db`.

## Tickets

| Ticket | Entrega | Aceites |
| --- | --- | --- |
| TK-001 | `aura-memory`: banco cifrado com migrações, FTS5 e troca do `rusqlite` do workspace; passo de Perl no CI; vocabulário | AC-007 |
| TK-002 | Lembranças: `chunk`, conciliação e busca lexical com filtros; ganchos no host | AC-003, AC-004 (lexical), AC-005 (lexical), AC-006 |
| TK-003 | Vetores: quantização, varredura e reordenação; motor de embeddings e métodos `memory.*` no worker; catálogo; lote com ociosidade | AC-005 (vetor), AC-008 |
| TK-004 | `memory_search`: fusão, Política, Registro de acesso e contrato IPC | AC-001, AC-002, AC-004, AC-009 |
| TK-005 | `aura-bench memory`: conjunto dourado e escala | AC-010, AC-011 |

## Seams, TDD e oráculos

- `aura_memory::{chunk, lexical, vector, fusion}`: funções puras, com oráculos literais calculados à parte (exemplos do S1 e do S2).
  - A varredura é testada por equivalência com a busca exata em float32 num conjunto pequeno em que os dois têm de coincidir.
  - O recall vem do bench, não do teste de unidade.
- `aura_memory::db::MemoryDb` com `tempfile`:
  - o arquivo não começa com `SQLite format 3`;
  - abrir com a chave errada falha;
  - FTS5 com `remove_diacritics 2` acha "orcamento" em "orçamento".
- `aura_app::acervo::Acervo` com `Store::open_in_memory` para o `aura.db` e `MemoryDb` em `tempfile`. O worker é um processo externo: entra por um `trait Embedder` com um adaptador real (`WorkerClient`) e um de teste determinístico, porque o processo externo é uma variação real.
- Ferramenta: pelo caminho dos testes de host existentes (`crates/aura-app/tests/host.rs`, `call("memory_search", …)`), com o `Embedder` de teste.
- Worker: teste de protocolo no estilo dos testes existentes do worker, com o motor `fake`. O motor real roda só com `engines` e o modelo baixado: evidência manual.
- Um caso por vez, red antes de green; erro de ambiente não conta como red.

Comandos (Windows, raiz do repositório, Strawberry Perl no PATH):
- `cargo test -p aura-memory`
- `cargo test -p aura-app`
- `cargo test -p aura-worker`
- `cargo test --workspace --exclude aura-desktop`, pulando os dois testes de Media Foundation do `aura-win` que travam neste PC (limitação registrada no 045)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract`
- `pnpm -C apps/desktop typecheck` e `pnpm -C apps/desktop test`
- `cargo run -p aura-bench --release -- memory --model-dir <dir>` e `-- memory --escala 500000`

## Riscos

- A troca do SQLite do `aura.db` muda a versão para trás (3.53.2 → 3.51.3). A suíte inteira roda como oráculo; se algo depender de recurso novo, o ticket para e volta à planejadora.
- O tempo de build a frio sobe ~5 min (OpenSSL). Fica registrado.
- O `ort` direto no worker tem de usar a mesma versão e os mesmos recursos do `transcribe-rs`, para não carregar dois ONNX Runtime.

## Gates

- G1: spec aceito (delegação do usuário).
- G2: seams e oráculos acima.
- G3: `package` com `ready: true` antes de cada ticket.
- Evidência só depois de execução real; revisão antes de `done`.
- Sem publicar nem instalar no sistema; modelos fora do git.
