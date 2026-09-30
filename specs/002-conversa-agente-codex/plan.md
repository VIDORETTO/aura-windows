---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 002-conversa-agente-codex
revision: 2
spec_revision: 2
status: ready
---

# Plan: Conversa agêntica via Codex app-server

## Summary

Um crate `aura-auth` implementa o "Continue with ChatGPT" (Sign in with ChatGPT com uso do plano, ADR 0007): PKCE, registro dinâmico, validação do ID token e escopos, cofre, renovação e revogação; o servidor loopback do `aura-gateway` nasce aqui com o upstream `chatgpt-plan`, que injeta o access token vigente. Um crate `aura-codex` encapsula tudo o que é Codex atrás de uma Interface de domínio (`CodexService` + fluxo de `ConversationEvent`), com um cliente JSON-RPC sobre um `Transport` substituível, um supervisor de processo e um mapeador puro de itens do protocolo para eventos do Aura. A UI nunca vê tipos do protocolo do Codex: recebe eventos do Aura por `Channel` Tauri, com deltas agrupados por quadro. A persona do Aura entra por `thread/start.baseInstructions`; os modos Chat/Tarefa mapeiam para `sandbox`/`approvalPolicy`/`cwd` do Codex.

## Technical context

- Language/runtime: Rust (tokio), React/TS.
- Dependencies: `codex-app-server` **fixado** — candidato inicial `rust-v0.159.0`, asset `codex-app-server-package-x86_64-pc-windows-msvc.tar.zst` (91 519 982 bytes, `sha256:2b563acd95fd3cfc4660c6686fdf963687b0c9bd488d6f4af4a75918ddc5458c`, digest lido da API do GitHub em 2026-09-29; o pacote inclui os binários auxiliares de sandbox; confirmar conteúdo no TK-001). Alternativa só-exe: `codex-app-server-x86_64-pc-windows-msvc.exe.zst` (60 718 516 bytes, `sha256:6afe4ff2bf248f6c991d8e219f7ec83e6ecb41181344fa8e9cc458f70a0c6e21`). Crates: `tokio`, `serde_json`, `zstd`, `tar`, `sha2`, `reqwest`, `jsonschema` (testes), `insta` (snapshots). UI: renderizador markdown incremental (avaliar `streamdown`; fallback `react-markdown` + `remark-gfm` com memo por bloco), Shiki carregado sob demanda.
- Sign in with ChatGPT (docs em `developers.openai.com/siwc`, lidas em 2026-09-29): authorize `https://auth.openai.com/api/accounts/authorize`; token `https://auth.openai.com/api/accounts/oauth/token`; discovery/JWKS/revogação via `https://auth.openai.com/.well-known/openid-configuration`; escopos `openid profile email offline_access resource.invoke chatgpt.tokens.use.direct`; `resource=https://api.openai.com/v1`; callback `http://127.0.0.1:<porta>/auth/callback` (não `localhost`); primeira vez `client_id=dynamic_agent_client` + `agent_name_hint=Aura` + `ext_agent_host_id`; access token 1 h; refresh rotativo 30 dias (serializar refresh). Crates: `oauth2` ou implementação direta com `reqwest`, `jsonwebtoken` (validação do ID token por JWKS), `p256`/`jose` para o host id por thumbprint JWK (RFC 9278).
- Storage/data: credenciais SIWC por conta no Credential Manager (`Aura/chatgpt/<client_id>`), metadados não secretos (e-mail, `sub`, client_id, escopos, expiração, host id) em `chatgpt_accounts` no `aura.db`; `%LOCALAPPDATA%\Aura\bin\codex\<versão>\`, `%LOCALAPPDATA%\Aura\codex-home\`, `%LOCALAPPDATA%\Aura\workspaces\<id>\`; tabela `conversations_meta(thread_id, workspace_path, mode, provider_id, ephemeral, created_at)` no `aura.db`.
- Test command: `cargo nextest run -p aura-codex`; `pnpm -C apps/desktop test -- conversation`; `pnpm -C apps/desktop e2e -- --spec e2e/conversa.spec.ts` (Windows, com app-server falso); roteiros manuais com conta real.
- Target/platform: Windows; `aura-codex` compila e testa em Linux com o app-server falso.

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-009; AC-001–AC-025.

## Modules, interfaces, consumers, and seams

| Module | Interface (resumo) | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `aura-codex::transport` | `trait Transport { async fn send(&self, line: String); fn incoming(&self) -> impl Stream<Item=String>; }` | `RpcClient` | Dois adapters reais: `StdioProcessTransport` (produção) e `InMemoryTransport` ligado ao app-server falso (testes) |
| `aura-codex::rpc` | `RpcClient::{request<P,R>(method, params) -> R, notifications() -> Stream<Notification>, server_requests() -> Stream<(ServerRequest, Responder)>}` | `CodexService` | Via `InMemoryTransport` |
| `aura-codex::supervisor` | `AppServerSupervisor::{ensure_running() -> RpcClient, touch(), shutdown()}`; políticas `IdlePolicy{timeout}`, `RestartPolicy{backoff, max_in_window}` | `CodexService` | Binário de teste `fake-app-server` (processo real que fala o protocolo mínimo e pode "crashar" sob comando) + relógio controlado (`tokio::time::pause`) |
| `aura-codex::binary` | `CodexBinary::ensure(version, progress) -> PathBuf` (download, zstd/tar, SHA-256, instalação atômica) | supervisor | `wiremock` servindo o artefato; SHA errado → erro |
| `aura-auth::siwc` | `SiwcClient::{begin_login(account: New|Existing(id)) -> LoginAttempt{authorize_url}, complete(callback) -> ChatGptAccount, refresh(account), revoke(account)}`; `HostId::load_or_create()` | host, UI | `wiremock` simulando authorize/token/JWKS/revogação; ID tokens assinados por chave de teste |
| `aura-auth::store` | `ChatGptAccounts::{list, active, set_active, save(tokens), clear_tokens}` sobre `CredentialStore` (003 TK-001 usa a mesma abstração; criada aqui se ainda não existir) | host, gateway | Store em memória + Credential Manager real no Windows |
| `aura-auth::refresher` | `TokenRefresher` renova quando faltam ≤ 5 min ou após 401; serializa por conta; publica `AccessToken` vigente | gateway | Relógio pausado |
| `aura-gateway::server` + `upstream::chatgpt_plan` | Servidor loopback (`127.0.0.1`, porta efêmera, token por execução) com rota `/p/chatgpt-plan/v1/responses` repassando a `https://api.openai.com/v1/responses` com o Bearer vigente; `GET /p/chatgpt-plan/v1/models` | app-server | `wiremock` como upstream; 003 estende com provedores BYOK |
| `aura-codex::home` | `CodexHome::prepare(root, settings) -> PathBuf` escreve `config.toml` (provedor `aura-chatgpt-plan`, web_search, mcp/provedores registrados por outros esforços via `ConfigContributor`) | supervisor, 003, 004, 008 | Snapshot do TOML gerado |
| `aura-codex::mapping` | `fn map_notification(n: &Notification, state: &mut TurnState) -> Vec<ConversationEvent>` (puro) | `CodexService` | Transcripts JSONL gravados → snapshots `insta` de eventos |
| `aura-codex::service` | `CodexService` (ver "Interface pública") | host Tauri (`conversation.rs`) | App-server falso roteirizado + transcripts |
| `apps/desktop/src-tauri/src/conversation.rs` | Comandos specta e `Channel<ConversationEvent>` | UI | E2E com app-server falso |
| `apps/desktop/src/conversation/` | `ConversationView`, `Message`, `ItemCard`, `ApprovalCard`, `LoginCard`, `ModelPicker`, `RateLimitMeter`, `HistoryPanel`, `ChipsBar`; store `useConversationStore` | usuário | Vitest + `mockIPC` |

### Interface pública de `CodexService`

```rust
pub struct CodexService { /* supervisor, estado */ }
impl CodexService {
  pub async fn models(&self, provider: &ProviderRef) -> Result<Vec<ModelInfo>>; // AC-011 (chatgpt-plan: GET /v1/models via gateway)
  pub async fn start(&self, opts: ConversationOptions) -> Result<ConversationId>; // AC-007, AC-017, AC-024, AC-025
  pub async fn resume(&self, id: &ConversationId) -> Result<ConversationSnapshot>; // AC-015
  pub async fn send(&self, id: &ConversationId, input: Vec<TurnInput>) -> Result<TurnId>; // AC-005, AC-013
  pub async fn steer(&self, id: &ConversationId, turn: &TurnId, input: Vec<TurnInput>) -> Result<()>; // AC-022
  pub async fn interrupt(&self, id: &ConversationId) -> Result<()>;   // AC-006
  pub async fn compact(&self, id: &ConversationId) -> Result<()>;     // AC-023
  pub async fn list(&self, q: HistoryQuery) -> Result<Page<ConversationSummary>>; // AC-015
  pub async fn rename/pin/archive/delete(...) -> Result<()>;          // AC-016
  pub async fn respond(&self, req: PendingRequestId, decision: Decision) -> Result<()>; // AC-018..AC-021
  pub fn events(&self) -> broadcast::Receiver<ConversationEvent>;
}
```

A conta ChatGPT fica fora do `CodexService`: `aura-auth` expõe `AuthService::{accounts, active, login, cancel, logout, enable_plan_usage}` (AC-001–AC-004, AC-026, AC-027).

`ConversationEvent` (domínio do Aura): `TurnStarted`, `MessageDelta{item, text}`, `MessageCompleted{item, markdown}`, `ReasoningSummaryDelta`, `ToolCall{item, kind, title, status, detail}`, `ApprovalRequested{request, kind, command|diff, cwd, reason, options}`, `UserInputRequested{request, questions, auto_resolve_ms}`, `ApprovalResolved`, `TokenUsage{used, window}`, `Compacted`, `TurnCompleted{status, error: Option<TurnError>}`, `AccountChanged`, `RateLimitsChanged`, `AppServerState{Starting|Ready|Stopped|Restarting|Failed}`.

`TurnError` mapeia `codexErrorInfo` e o corpo do erro upstream preservado pelo gateway: `subscription_sharing_usage_limit_exceeded` (429) → limite do plano ("Gerenciar uso"); `subscription_sharing_user_not_eligible` → não elegível; `subscription_sharing_unsupported_capability` → parâmetro; `UsageLimitExceeded{resets_at}` (BYOK) → limite; `HttpConnectionFailed`/`ResponseStream*` → sem conexão; `Unauthorized` → sessão expirada; `ContextWindowExceeded` → contexto; demais → `Other{code}`.

### Mapeamento de modos (AC-024)

| Modo | `sandbox` | `approvalPolicy` | `cwd` / roots | `developerInstructions` |
| --- | --- | --- | --- | --- |
| Chat | `readOnly` | `onRequest` | Workspace da conversa | "Você está no Modo Chat: responda, use ferramentas de leitura; não altere arquivos." |
| Tarefa | `workspaceWrite` com `writableRoots = [workspace, pastas concedidas]`, `networkAccess=false` (toggle) | `onRequest` | Workspace da conversa | "Modo Tarefa: planeje, peça aprovação para efeitos." |

Sandbox Windows: `[windows] sandbox = "unelevated"` por padrão (não exige admin); oferecer configuração `elevated` via `windowsSandbox/setupStart` quando `windowsSandbox/readiness` indicar (UI no TK-009).

## Chosen approach and alternatives

- **Tipos do protocolo escritos à mão para o subconjunto usado + validação contra o JSON Schema gerado** (`codex app-server generate-json-schema --out crates/aura-codex/schema/<versão>`), em vez de depender do crate `codex-app-server-protocol` via git: evita arrastar dependências internas do Codex e deixa o contrato explícito. Testes validam cada mensagem enviada contra o schema da versão fixada.
- **Persona via `baseInstructions`** por Conversa, arquivo `apps/desktop/src-tauri/resources/persona/{pt-BR,en}.md`.
- **Download no primeiro uso** do pacote do app-server (instalador pequeno); o esforço 010 pode oferecer instalador "completo" com o pacote embutido.
- **Deltas agrupados por `requestAnimationFrame`** na UI e por janela de 16 ms no host para cumprir 50 ms de pintura sem inundar IPC.
- Alternativa descartada: expor tipos do Codex diretamente à UI (acoplaria a UI à versão do app-server).

## Data, compatibility, and external dependencies

- `CODEX_HOME` isolado: `config.toml` gerado com `model_provider = "aura-chatgpt-plan"`, `[model_providers.aura-chatgpt-plan] name="ChatGPT plan" base_url="http://127.0.0.1:<porta>/p/chatgpt-plan/v1" env_key="AURA_GATEWAY_TOKEN" wire_api="responses" requires_openai_auth=false supports_websockets=false`, recursos que emitem `tool_search` desligados, `web_search = "live"`, `[windows] sandbox = "unelevated"`, `analytics.enabled = false` (confirmar chave na versão fixada), seções de `model_providers`/`mcp_servers` contribuídas por 003/004/008 via `ConfigContributor`.
- `clientInfo = { name: "aura_desktop", title: "Aura", version: <app> }`; `capabilities.experimentalApi = false` por padrão (estável); flag interna para testar recursos experimentais.
- Atualizar a versão do app-server = novo ticket: baixar, regenerar schema, regravar transcripts, rodar contrato.
- Retomada após parada por inatividade usa `thread/resume` com o id guardado em `conversations_meta`.

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001, AC-002, AC-003, AC-004 | Integração `aura-auth` com `wiremock` (authorize/token/JWKS/revogação) + manual real | Docs SIWC | Callback com `client_id` emitido → troca de código com PKCE; ID token de teste validado (iss, aud, exp, nonce); escopo sem `chatgpt.tokens.use.direct` → plano desativado; revogação `token_type_hint=refresh_token`. Manual com conta Plus real em Win11; varredura de arquivos sem tokens |
| AC-026, AC-027 | Integração `TokenRefresher` (relógio pausado) + `ChatGptAccounts` | Docs SIWC (1 h, rotação) | Renova a 5 min do fim sem interromper stream simulado; `invalid_grant` → `ReloginRequired`; duas contas isoladas |
| AC-005 | Contrato + Vitest + medição | Spec (50 ms) | Transcript real de streaming → eventos; Vitest renderiza tabela/código; medição de `delta_received → paint` via `performance.mark` no modo diagnóstico |
| AC-006 | Contrato | Spec | `interrupt` → `turn/interrupt` enviado e `TurnCompleted{Interrupted}` com texto parcial preservado no store |
| AC-007 | Contrato (payload) + manual | Arquivo de persona | Mensagem `thread/start` contém `baseInstructions` = conteúdo do arquivo da persona do idioma; validado contra schema. Manual: "quem é você?" → responde como Aura |
| AC-008 | Unit `mapping` | Tabela de `TurnError` acima | Um caso por `codexErrorInfo` com transcript sintético derivado do schema |
| AC-009 | Integração supervisor com `fake-app-server` e tempo pausado | Spec (15 min) | Primeira chamada inicia; avançar 15 min sem turno → processo encerrado; `send` seguinte → `thread/resume` + `turn/start` |
| AC-010 | Integração supervisor | Spec (backoff 1/2/4 s, 3 em 60 s) | `fake-app-server --crash-after-turn-start` → `TurnCompleted{Failed}`, reinícios nos tempos esperados, 4ª falha em 60 s → estado `Failed` |
| AC-011, AC-012 | `wiremock` `/v1/models` + Vitest | Resposta roteirizada | UI lista só `visibility:list` na ordem do servidor; indicador "Usando plano ChatGPT" + link |
| AC-013, AC-014 | Vitest + contrato | Spec | Colar imagem → Chip; `send` inclui `localImage` com caminho no workspace; modelo com `inputModalities:["text"]` → envio bloqueado |
| AC-015, AC-016, AC-017 | Contrato + E2E falso | Spec | `thread/list` com `searchTerm`; `thread/delete` + pasta removida; efêmera → `ephemeral:true`, ausente de `list`, workspace temporário removido ao fechar |
| AC-018–AC-021 | Contrato + Vitest + manual real | Docs do app-server (decisões) | Server requests roteirizados → cartões; respostas `accept`/`acceptForSession`/`decline` enviadas; pendência persiste com Overlay oculto |
| AC-022, AC-023 | Contrato | Spec | `turn/steer` com `expectedTurnId`; `thread/compact/start` → `contextCompaction` |
| AC-024, AC-025 | Contrato (payload) + manual real | Tabela de modos | `thread/start` com `sandbox`/`approvalPolicy`/`cwd` corretos; manual: no Chat, pedir "crie um arquivo" não cria |

Gravação de transcripts: `tools/codex-record` executa um roteiro contra o app-server real com conta de teste e salva JSONL em `crates/aura-codex/tests/transcripts/<versão>/` com tokens, e-mails e ids pessoais substituídos.

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `crates/aura-auth/src/{siwc.rs,host_id.rs,store.rs,refresher.rs,lib.rs}` | new | ver tabela | Login SIWC | 2026-09-29 |
| `crates/aura-gateway/src/{server.rs,upstream/mod.rs,upstream/chatgpt_plan.rs}` | new | servidor + upstream plano ChatGPT | Loopback | 2026-09-29 |
| `crates/aura-codex/src/{transport.rs,rpc.rs,supervisor.rs,binary.rs,home.rs,mapping.rs,service.rs,types.rs}` | new | ver tabela | Núcleo | 2026-09-29 |
| `crates/aura-codex/codex-version.toml` | new | versão, asset, sha256 | Fixação | 2026-09-29 |
| `crates/aura-codex/schema/<versão>/` | new | JSON Schema gerado | Contrato | 2026-09-29 |
| `crates/aura-codex/tests/{fake_app_server/,transcripts/}` | new | falso + gravações | Testes | 2026-09-29 |
| `tools/codex-record/` | new | gravador | Transcripts | 2026-09-29 |
| `apps/desktop/src-tauri/src/conversation.rs` | new | comandos/Channel | Ponte UI | 2026-09-29 |
| `apps/desktop/src-tauri/resources/persona/{pt-BR,en}.md` | new | Persona | AC-007 | 2026-09-29 |
| `apps/desktop/src/conversation/**` | new | componentes/store | UI | 2026-09-29 |
| `crates/aura-store/src/migrations/0002_conversations.sql` | new | `conversations_meta` | Metadados | 2026-09-29 |
| `apps/desktop/src-tauri/src/children.rs` | existing (001 TK-001) | `ChildRegistry::adopt` | Filhos | 001 TK-001 |

## Derived technical obligations

- **OT-001** → FR-001: todo spawn do app-server passa por `ChildRegistry::adopt` (001 OT-004) e usa `CREATE_NO_WINDOW`.
- **OT-002** → FR-001: o binário é verificado por SHA-256 a cada inicialização do supervisor (não só no download).
- **OT-003** → FR-003/AC-005: a UI não re-renderiza a lista inteira a cada delta (memo por mensagem/bloco, virtualização acima de 200 itens).
- **OT-004** → FR-007: decisões de Aprovação só partem de interação explícita na UI ou de regra "nesta conversa" registrada; nenhum código responde `accept` por padrão.
- **OT-006** → FR-002: tokens SIWC nunca saem do host: não vão ao ambiente, config ou linha de comando do app-server, nem a logs; URLs de autorização com `id_token_hint` são redigidas nos logs.
- **OT-007** → FR-002/UX: textos e marca seguem as diretrizes de UI do SIWC (botão "Continue with ChatGPT", modal de primeiro uso, "Usando plano ChatGPT", "Gerenciar uso").
- **OT-005** → FR-005: o tipo `ContextChip { id, kind, label, preview, payload_ref, blocked_reason? }` e `TurnInput` são a Interface pública que 004/005/007 usam para contribuir contexto.

## Risks and gates

- Versões do app-server mudam rápido; se a versão fixada perder compatibilidade com o backend da OpenAI (forçar upgrade), o Aura precisa de release — mitigação: canal de update do 010 e verificação `model/list` com erro `-32600` tratado como "atualize o Aura".
- `baseInstructions` pode não remover todos os comportamentos de programação (H-008) — se falhar, avaliar `model_instructions_file` no `config.toml`; se ambos falharem, retornar à planejadora.
- Custo de download (~92 MB) no primeiro uso — UI com progresso e retomada.
- Programa SIWC está em preview; mudanças de contrato exigem atualizar `aura-auth` — manter a integração isolada nesse crate.
- G2: satisfeito. G3: TK-001 sem blockers internos; depende de 001 TK-001 (processo/ChildRegistry) concluído; TK-002 depende de 001 TK-006 (cofre).
