---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 002-conversa-agente-codex
type: delivery
status: implemented
ticket_revision: 4
requires: []
requirement_refs: ["FR-001"]
acceptance_refs: ["AC-009", "AC-010"]
spec_revision: 2
plan_revision: 2
owned_areas: ["crates/aura-codex/src/transport.rs", "crates/aura-codex/src/rpc.rs", "crates/aura-codex/src/supervisor.rs", "crates/aura-codex/src/binary.rs", "crates/aura-codex/src/home.rs", "crates/aura-codex/tests", "crates/aura-codex/codex-version.toml", "crates/aura-codex/schema", "tools/codex-record"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---




# TK-001 — Spike de validação + cliente JSON-RPC e supervisor do app-server

## Objetivo e limites

Entrega (1) um spike curto que valida H-016 (turno do app-server com o access token SIWC via provedor Responses, receita oficial com `env_key`), H-008 (persona via `baseInstructions`), H-003 (imagem em resultado MCP chega ao modelo), H-004 parcial (memória ociosa do app-server) e o conteúdo do pacote fixado; (2) `Transport`, `RpcClient`, `CodexBinary`, `CodexHome` e `AppServerSupervisor` com início sob demanda, parada por inatividade e reinício com backoff; (3) app-server falso e gravador de transcripts.

Não inclui: login SIWC no produto (TK-002), conversa/UI (TK-003).

## Leitura em ordem

1. `docs/research/harness-evaluation.md` → "Itens a verificar no spike".
2. `docs/adr/0001-codex-app-server-como-harness.md`.
3. `specs/002-conversa-agente-codex/plan.md` → Modules (transport, rpc, supervisor, binary, home), OT-001/OT-002, "Data".
4. `apps/desktop/src-tauri/src/children.rs` → `ChildRegistry::adopt` (001 TK-001).
5. Docs atuais: https://developers.openai.com/codex/app-server (initialize, transports, erros `-32001`), `codex app-server generate-json-schema`.

## Decisões já resolvidas

- Transporte stdio JSONL; `initialize` com `clientInfo.name = "aura_desktop"`; `initialized` em seguida.
- Pacote `codex-app-server-package-x86_64-pc-windows-msvc.tar.zst` da versão fixada em `codex-version.toml` (candidato `rust-v0.159.0`); instalação atômica em `bin/codex/<versão>/` (extrair para `.tmp`, verificar, renomear).
- Variáveis de ambiente do filho: `CODEX_HOME=<LOCALAPPDATA>\Aura\codex-home`, `RUST_LOG=warn`; `CREATE_NO_WINDOW`; stderr para o log do Aura com redação.
- `IdlePolicy{timeout: 15 min}` contada a partir do último `TurnCompleted` sem turno ativo nem requisição pendente; `RestartPolicy{backoff: [1s,2s,4s], max_in_window: 3, window: 60s}`.
- Spike é time-box de 1 dia; resultados vão para `docs/research/harness-evaluation.md` (seção "Resultados do spike") e, se H-003/H-008 falharem, retorno à planejadora antes de TK-003.
- Liberdade local: estrutura interna do cliente RPC (ids numéricos crescentes recomendados).

## Mapa de alterações

- Novo: `crates/aura-codex/Cargo.toml`, `src/lib.rs`, `src/transport.rs` (`Transport`, `StdioProcessTransport`, `InMemoryTransport`), `src/rpc.rs` (`RpcClient`, `Notification`, `ServerRequest`, `Responder`, `RpcError`), `src/binary.rs` (`CodexBinary::ensure`), `src/home.rs` (`CodexHome::prepare`, `ConfigContributor`), `src/supervisor.rs` (`AppServerSupervisor`, `IdlePolicy`, `RestartPolicy`, `AppServerState`).
- Novo: `crates/aura-codex/codex-version.toml`, `schema/<versão>/*.json` (gerado).
- Novo: `crates/aura-codex/tests/fake_app_server/main.rs` (binário de teste: responde `initialize`, `thread/start`, `turn/start` roteirizado; flags `--crash-after-turn-start`, `--script <jsonl>`).
- Novo: `tools/codex-record/` (roteiro de gravação com redação).
- Fora da fatia: UI, login.

## Contrato técnico

- Entradas: `ensure_running()` chamado por quem precisa do app-server; `touch()` a cada atividade.
- Saídas: `RpcClient` pronto (handshake concluído); `AppServerState` publicado.
- Invariantes: no máximo um processo app-server por instância do Aura; SHA-256 conferido antes de cada spawn (OT-002); processo adotado pelo `ChildRegistry` (OT-001).
- Erros: `SupervisorError::{Download(reason), Checksum{expected, actual}, Spawn(io), Handshake(RpcError), CrashLoop}`; requisições feitas durante reinício aguardam até 10 s e então falham com `Unavailable`.
- Efeitos: download com progresso (`BinaryProgress{bytes, total}`); arquivos em `bin/codex/<versão>/`.
- Concorrência: `ensure_running` idempotente sob chamadas concorrentes (um único spawn).

## Exemplos de aceite

- **AC-009**: `fake-app-server` + `tokio::time::pause()`: `ensure_running()` → 1 spawn e `initialize` enviado; nenhum turno por 15 min simulados → processo terminado e estado `Stopped`; `ensure_running()` depois → novo spawn. Um turno ativo durante 20 min → não encerra. Oráculo: 15 min da spec.
- **AC-010**: `fake-app-server --crash-after-turn-start` → evento `TurnCompleted{Failed(AppServerCrashed)}`; reinícios aos 1 s, 2 s, 4 s; 4º crash dentro de 60 s → `AppServerState::Failed` e sem novos spawns. Após 60 s sem falha, a janela reseta.
- Spike: obter um access token SIWC manualmente (roteiro da doc oficial) e rodar a receita `codex app-server -c model_provider=…` com `ACCESS_TOKEN` → turno `completed` (H-016); registrar p/ versão fixada — `baseInstructions` "Você é Aura…" → resposta a "quem é você?" não menciona ser agente de código (H-008); tool MCP de teste que retorna `image/png` → modelo descreve a cor do quadrado (H-003); working set privado do app-server ocioso após `initialize` (H-004).
- Checksum: servidor `wiremock` com artefato alterado → `Checksum{..}` e nada instalado.

## Dependências e sequência de execução

Depende de: 001-fundacao-overlay/TK-001 (outro esforço; `ChildRegistry` precisa estar `done`). Nenhum ticket deste esforço.

- [ ] TK-001.1 Spike manual no Windows com a versão candidata; registrar resultados e decidir o pin (ou retornar).
- [ ] TK-001.2 Gerar JSON Schema da versão fixada e commitar.
- [ ] TK-001.3 `RpcClient` + `InMemoryTransport`: teste de correlação request/response e server request (red→green).
- [ ] TK-001.4 `CodexBinary::ensure` com `wiremock` (checksum ok/errado).
- [ ] TK-001.5 Supervisor: AC-009 red→green; AC-010 red→green.
- [ ] TK-001.6 `CodexHome::prepare` snapshot do `config.toml`.
- [ ] TK-001.7 `codex-record` gera primeiro transcript (initialize + thread/start + turn simples) com redação verificada.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-codex`; spike: roteiro em `docs/research/harness-evaluation.md` executado no Windows 11 com conta ChatGPT de teste.
- Estado esperado: testes verdes; seção de resultados do spike preenchida com versão, medições e decisões.
- Comando identificado na configuração mas não executado: `codex app-server generate-json-schema` (executar e registrar).
- Distinguir defeito de ambiente: sem rede/sem conta → spike `not_run` com limitação; testes do supervisor não dependem de rede.

## Condição de retorno à planejadora

Retornar se H-008 falhar também com `model_instructions_file`, se H-003 falhar (decidir `dynamicTools` experimental para 004), se o app-server ocioso exceder 150 MB, ou se o pacote exigir privilégios de administrador para rodar.

## Relatório de saída

Relatar versão fixada e SHA, resultados do spike (H-003/H-004/H-008), símbolos criados, resultados de teste, EV refs e próxima ação.
