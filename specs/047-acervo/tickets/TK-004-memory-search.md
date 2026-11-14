---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 047-acervo
type: delivery
status: ready
ticket_revision: 1
requires: ["TK-003"]
requirement_refs: ["FR-004", "FR-005"]
acceptance_refs: ["AC-001", "AC-002", "AC-004", "AC-009"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-memory/src/fusion.rs", "crates/aura-mcp/src/tools.rs", "crates/aura-app/src/tools.rs", "crates/aura-app/src/acervo.rs", "crates/aura-app/src/host.rs", "crates/aura-policy/src/lib.rs", "crates/aura-app/tests", "apps/desktop/src/ipc/types.ts"]
verification_status: not_run
---

# TK-004 — Ferramenta `memory_search`, Política e Registro de acesso

## Objetivo e limites

O agente chama `memory_search` e recebe trechos com origem. O resultado vem da busca híbrida (RRF 1 : 1, ADR 0014) com filtros de período e fonte. O acesso obedece à Permissão do agente para o Acervo e fica no Registro de acesso.

Não inclui: controle na UI (049); Conversas como fonte (048); mudanças em `note_search`/`meeting_search`.

## Leitura em ordem

1. `specs/047-acervo/plan.md` → Ferramenta `memory_search`, `fusion`.
2. `crates/aura-mcp/src/tools.rs` (definições, `read_only`) e `crates/aura-app/src/tools.rs` (despacho, `decide`, `ConsentBroker`, `PrivacyRepo`).
3. `crates/aura-policy/src/lib.rs` (`Source`, `AgentPermission`, `Policy`, `decide`).
4. `crates/aura-app/tests/host.rs` (`call(...)`) e `crates/aura-app/tests/ipc_contract.rs` (contrato dourado).

## Decisões já resolvidas

- **RRF:** k = 60, pesos iguais. O empate desfaz pelo `id` maior (mais novo). Cada lista entra com até 50 itens.
- **Entrada:** `query` (obrigatório); `de`/`ate` como data ISO `AAAA-MM-DD`, ou data e hora; `fontes` ⊆ {`nota`, `salvo`, `reuniao`}; `limite` com padrão 8 e teto 20.
- **Saída por trecho:**
  - `id`, `fonte`, `origem`, `titulo`, `data` (ISO local);
  - `instante` (`mm:ss`) e `oradores` quando for Reunião;
  - `texto`, cortado em 1.000 caracteres;
  - `pontuacao`.
- **Topo da saída:** `semantica` ∈ `completa` | `incompleta`, e `motivo` quando incompleta.
- **Política:**
  - `Source::Memory`;
  - `Policy.memory_agent: AgentPermission`, com `#[serde(default)]` = `Ask`;
  - `Never` → recusa sem ler;
  - `Ask` → `ConsentBroker`, como as Fontes de captura, com concessão por Conversa;
  - `Always` → direto.
- **Registro de acesso:** `source = "memory"`, `tool = "memory_search"`, decisão. O `reason` leva a consulta e o número de trechos, como pede o AC-009. O `access_log` fica no `aura.db`, em claro como o resto das fontes até a CAND-086, e não sai do PC.
- Descrição da ferramenta em inglês, como as demais, pedindo para citar a origem.
- Liberdade local: organização do código do despacho.

## Mapa de alterações

- Novo `crates/aura-memory/src/fusion.rs`.
- Existente `crates/aura-mcp/src/tools.rs`: `MEMORY_SEARCH` e a definição.
- Existente `crates/aura-app/src/tools.rs`: despacho, Política e registro.
- Existente `crates/aura-app/src/acervo.rs`: `search`, que faz a fusão e monta a saída.
- Existente `crates/aura-policy/src/lib.rs`: `Source::Memory` e `memory_agent`.
- Existente `crates/aura-app/tests/{host.rs ou acervo.rs, ipc_contract.rs}` e os arquivos dourados.
- Existente `apps/desktop/src/ipc/types.ts`.

## Contrato técnico

- `aura_memory::fusion::rrf(listas: &[&[i64]], k: f64) -> Vec<(i64, f64)>`.
- `Acervo::search(&self, embedder: Option<&dyn Embedder>, q: &MemoryQuery) -> Result<MemoryResult, …>`.
- Ferramenta MCP `memory_search` com `input_schema` fechado (`additionalProperties: false`) e anotação somente leitura.
- `Policy` serializada com `memoryAgent` (camelCase). A mudança do contrato dourado é regenerada com `UPDATE_GOLDEN=1`.

## Exemplos de aceite

- **`rrf` com literais:** listas `[1, 2, 3]` e `[3, 1, 4]`, k = 60.
  - Pontuações: 1 → 1/61 + 1/62; 3 → 1/63 + 1/61; 2 → 1/62; 4 → 1/63.
  - Ordem: 1, 3, 2, 4.
- **AC-001:**
  - Reunião de 15/10/2026 com a fala "fechamos o orçamento de nuvem em R$ 15 mil por mês", do orador `them`, aos 12:30;
  - Nota "orçamento de marketing: R$ 8 mil";
  - `memory_search {query: "quanto ficou o orçamento de nuvem?"}` → 1º trecho com `fonte: reuniao`, `origem` = id da Reunião, `oradores` contém `them`, `instante: "12:30"` e `data` 2026-10-15.
  - Roda com o `Embedder` de teste e, como evidência manual, com o motor real.
- **AC-002:** `memory_search {query: "quanto vamos gastar por mês com servidores?"}` → o trecho da Reunião entre os 3 primeiros. Exige o motor real: teste `#[ignore]` com `AURA_EMBED_MODEL_DIR`, como evidência manual. Com o `Embedder` de teste, só o caminho é verificado, não a qualidade.
- **AC-004:**
  - com `de: "2026-11-01"`, a Reunião de outubro não aparece;
  - com `fontes: ["nota"]`, só Notas.
- **AC-009:**
  - `memory_agent = never` → resposta de recusa, e nenhuma leitura no `memory.db` (Acervo contado por um contador de teste);
  - `ask` → passa pelo `ConsentBroker` (aprovar ou negar);
  - `always` → direto;
  - em cada caso, uma linha no `access_log` com `source = memory`, a decisão e, quando houve leitura, o número de trechos.
- **Contrato IPC:** o dourado regenerado e o `types.ts` com `memoryAgent`; `pnpm -C apps/desktop typecheck` passa.

## Dependências e sequência de execução

Depende de: TK-003.

- [ ] TK-004.1 `rrf` (literais), red → green.
- [ ] TK-004.2 `Acervo::search` com fusão, filtros e saída (AC-001 com o `Embedder` de teste, AC-004).
- [ ] TK-004.3 Política, `memory_search` no despacho e Registro de acesso (AC-009); contrato IPC.
- [ ] TK-004.4 Teste com o motor real (AC-001, AC-002); regressão, clippy, fmt, typecheck e evidência.

## Validação

- Comandos:
  - `cargo test -p aura-memory`;
  - `cargo test -p aura-policy`;
  - `cargo test -p aura-app`;
  - `UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract`, depois sem a variável;
  - `pnpm -C apps/desktop typecheck`;
  - `pnpm -C apps/desktop test`;
  - `AURA_WORKER_BIN=<aura-worker.exe com engines> AURA_EMBED_MODEL_DIR=<dir> cargo test -p aura-app --test acervo -- --ignored acervo_real`;
  - `cargo test --workspace --exclude aura-desktop`, com as exclusões do TK-001;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo fmt --all -- --check`.
- Estado esperado: passam.

## Condição de retorno à planejadora

Pare e devolva se a Política exigir mudar o modo de captura (`SourcePolicy.mode`) para o Acervo, ou se o consentimento por Conversa não servir para uma Fonte sem captura.

## Relatório de saída

Arquivos e símbolos, ACs, comandos, `EV`, limitações e pendências.
