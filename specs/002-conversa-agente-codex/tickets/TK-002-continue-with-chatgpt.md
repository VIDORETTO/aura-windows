---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 002-conversa-agente-codex
type: delivery
status: implemented
ticket_revision: 4
requires: ["TK-001"]
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-001", "AC-002", "AC-003", "AC-004", "AC-026", "AC-027"]
spec_revision: 2
plan_revision: 2
owned_areas: ["crates/aura-auth", "crates/aura-gateway/src/server.rs", "crates/aura-gateway/src/upstream/mod.rs", "crates/aura-gateway/src/upstream/chatgpt_plan.rs", "apps/desktop/src/account", "apps/desktop/src-tauri/src/account.rs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-002 — "Continue with ChatGPT" (Sign in with ChatGPT com uso do plano) e Gateway do plano

## Objetivo e limites

Entrega o login oficial SIWC com uso do plano (PKCE, registro dinâmico, validação do ID token e do escopo `chatgpt.tokens.use.direct`), host id estável, várias contas, renovação serializada, saída com revogação, o servidor loopback do Gateway com o upstream `chatgpt-plan` (injeção do Bearer vigente) e o provedor `aura-chatgpt-plan` no `config.toml`, mais a UI de conta (botão com marca aprovada, modal de primeiro uso, menu de contas).

Não inclui: Provedores BYOK (003 estende o mesmo servidor), seletor de modelo (TK-004).

## Leitura em ordem

1. `docs/adr/0007-login-chatgpt-via-sign-in-with-chatgpt.md`.
2. Docs oficiais (ler inteiras): `https://developers.openai.com/siwc/token-sharing-open-source/sign-in.md`, `.../profiles-and-sessions.md`, `.../codex-app-server.md`, `.../errors-and-recovery.md`, `.../preview-limitations.md`, `.../token-reference.md`, `https://developers.openai.com/siwc/ui-ux-guidelines.md` e `https://developers.openai.com/siwc/website.md` (validação do ID token e formatos do botão).
3. `specs/002-conversa-agente-codex/plan.md` (rev. 2) → `aura-auth`, `aura-gateway::server`, OT-006, OT-007.
4. `crates/aura-store/src/vault.rs` e `crates/aura-core/src/secret.rs` (001 TK-006).
5. `crates/aura-codex/src/{home.rs,supervisor.rs}` (TK-001) → `ConfigContributor` e ambiente do filho (`AURA_GATEWAY_TOKEN`).

## Decisões já resolvidas

- Host id: par de chaves P-256 gerado uma vez por instalação; `ext_agent_host_id = urn:ietf:params:oauth:jwk-thumbprint:sha-256:<thumbprint>` (RFC 9278), chave privada protegida pelo `Vault`; persistido antes do primeiro login.
- Primeiro login de uma conta: `client_id=dynamic_agent_client`, `agent_name_hint=Aura` (igual a `clientInfo.name` humano do app-server; o `name` técnico é `aura_desktop`), `ext_agent_host_id`, `response_type=code`, `redirect_uri=http://127.0.0.1:<porta livre>/auth/callback`, `scope=openid profile email offline_access resource.invoke chatgpt.tokens.use.direct`, `resource=https://api.openai.com/v1`, `state`, `nonce`, `code_challenge_method=S256`. Reautorização usa o `client_id` emitido + `id_token_hint`/`login_hint` e sem `agent_name_hint`.
- Callback: validar `state`; `error=access_denied` → AC-002; salvar `client_id` emitido (`oaiapp_...`); rejeitar callback com client_id diferente do pendente.
- Troca de código sem segredo; validar ID token (JWKS da discovery, `iss`, `aud` = client_id emitido, `exp`, `nonce`); exigir `chatgpt.tokens.use.direct` nos escopos concedidos para habilitar o plano.
- Credenciais só no Credential Manager (`Aura/chatgpt/<client_id>`); metadados em `chatgpt_accounts` (migração `0006_chatgpt_accounts.sql`).
- Renovação: `grant_type=refresh_token`, `client_id` emitido, `resource=https://api.openai.com/v1`, sem `scope`; quando faltam ≤ 5 min, ou após 401 do upstream (uma vez); substituir access/refresh/expiração/escopos juntos; um refresh por conta por vez.
- Saída: `POST` form ao `revocation_endpoint` com `token=<refresh>`, `token_type_hint=refresh_token`, `client_id`; 200 vazio = sucesso; falha de rede → backoff e, se não confirmar, limpar local e avisar.
- Gateway: bind `127.0.0.1`, porta efêmera, token por execução (`AURA_GATEWAY_TOKEN`), rejeita `Origin`; `/p/chatgpt-plan/v1/responses` e `/models` repassam para `https://api.openai.com/v1` com o Bearer da conta ativa, preservando corpo, stream, `User-Agent`/originator do Codex e o corpo de erro upstream.
- Liberdade local: layout do menu de contas; implementação OAuth (crate `oauth2` ou `reqwest` direto).

## Mapa de alterações

- Novo: `crates/aura-auth/{Cargo.toml,src/lib.rs,src/siwc.rs,src/host_id.rs,src/store.rs,src/refresher.rs,src/jwks.rs}` → `SiwcClient`, `HostId`, `ChatGptAccounts`, `TokenRefresher`, `AuthService`.
- Novo: `crates/aura-gateway/{Cargo.toml,src/lib.rs,src/server.rs,src/upstream/mod.rs,src/upstream/chatgpt_plan.rs}` → `Gateway::start`, `GatewayHandle{port, token}`, `UpstreamAdapter`, `ChatGptPlanUpstream`.
- Novo: `crates/aura-store/src/migrations/0006_chatgpt_accounts.sql`.
- Existente: `crates/aura-codex/src/home.rs` → provedor `aura-chatgpt-plan`; `supervisor.rs` → inicia depois do Gateway e injeta `AURA_GATEWAY_TOKEN`.
- Novo: `apps/desktop/src-tauri/src/account.rs` → comandos `accounts_list`, `login_start`, `login_cancel`, `account_switch`, `logout`, `enable_plan_usage`; eventos `account_changed`, `login_progress`.
- Novo: `apps/desktop/src/account/{ContinueWithChatGptButton.tsx,FirstUseModal.tsx,AccountMenu.tsx,PlanUsageDisabledCard.tsx}` com os assets de marca aprovados pela OpenAI.

## Contrato técnico

- Entradas: ações da UI; callback HTTP no loopback.
- Saídas: `ChatGptAccount{id, email, sub, client_id, plan_usage_enabled, active}`; `LoginProgress{Started|WaitingBrowser|Completed|Failed(reason)|Cancelled}`; access token vigente disponível ao Gateway.
- Invariantes: OT-006; nunca misturar client_id de uma conta com tokens de outra; um login pendente por vez (novo cancela o anterior e fecha o listener).
- Erros: `AuthError::{Denied, MissingPlanScope, StateMismatch, ClientIdMismatch, InvalidIdToken(reason), InvalidGrant, InvalidClient, Network, RevocationUnconfirmed}`.
- Efeitos: navegador aberto; credenciais no cofre; app-server usa o provedor `aura-chatgpt-plan`.

## Exemplos de aceite

- **AC-001**: `wiremock` de authorize/token/JWKS: callback `?code=c1&state=S&client_id=oaiapp_123&scope=chatgpt.tokens.use.direct+email+offline_access+openid+profile+resource.invoke` → troca com `code_verifier` correto e `redirect_uri` idêntico → ID token de teste válido → conta `teste@exemplo.com` ativa com `plan_usage_enabled=true`; UI mostra modal "Você está usando seu plano ChatGPT" só na primeira vez (segundo login da mesma conta → sem modal). Manual real com conta Plus: login em Win11, ≤ 5 s após consentir.
- **AC-002**: callback `error=access_denied` → sem troca de código, `Denied`; escopos sem `chatgpt.tokens.use.direct` → conta salva com `plan_usage_enabled=false` e cartão com "Ativar uso do plano ChatGPT" (nova autorização com `prompt=consent` e escopo completo) e "Configurar outro provedor".
- **AC-003**: `login_start` + `login_cancel` → listener fechado (porta liberada) e `Cancelled`; novo `login_start` aceito.
- **AC-004**: `logout` → `POST` de revogação com `token_type_hint=refresh_token` e `client_id=oaiapp_123`; 200 → tokens removidos do cofre, mapeamento conta↔client_id e host id mantidos; 503 três vezes → limpa local + aviso "revogação não confirmada"; varredura `Select-String -Recurse "$env:LOCALAPPDATA\Aura" -Pattern "eyJ"` sem ocorrências.
- **AC-026**: relógio pausado, token com `expires_in=3600` → refresh disparado em 55 min; durante um stream simulado no Gateway, a requisição seguinte usa o novo Bearer; resposta de refresh `invalid_grant` → `ReloginRequired` e Conversa preservada.
- **AC-027**: duas contas (`oaiapp_1`/A, `oaiapp_2`/B) → alternar ativa muda o Bearer injetado no Gateway; credenciais de A intactas após login de B.
- Gateway: `codex-e2e` com app-server real → `thread/start{modelProvider:"aura-chatgpt-plan"}` + turno → `wiremock` recebe `POST /v1/responses` com `Authorization: Bearer <access de teste>`, `store:false`, `stream:true`.

## Dependências e sequência de execução

Depende de: TK-001; 001-fundacao-overlay/TK-006 (outro esforço).

- [ ] TK-002.1 `HostId` (thumbprint estável entre reinícios) red→green.
- [ ] TK-002.2 `SiwcClient` begin/complete com `wiremock` (AC-001, AC-002, AC-003) um caso por vez.
- [ ] TK-002.3 Validação do ID token (casos inválidos: iss, aud, exp, nonce, assinatura).
- [ ] TK-002.4 `TokenRefresher` (AC-026) e `ChatGptAccounts` (AC-027); revogação (AC-004).
- [ ] TK-002.5 Gateway + `ChatGptPlanUpstream` + `config.toml`; `codex-e2e`.
- [ ] TK-002.6 UI (marca aprovada, modal, menu de contas); Vitest.
- [ ] TK-002.7 Roteiro manual com conta Plus real; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-auth -p aura-gateway server chatgpt_plan`; `AURA_CODEX_BIN=<caminho> cargo nextest run -p aura-gateway --features codex-e2e chatgpt_plan_e2e`; `pnpm -C apps/desktop test -- account`; roteiro manual Win11.
- Estado esperado: verdes; roteiro com prints (sem tokens visíveis).
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: conta sem Plus/Pro retorna `subscription_sharing_user_not_eligible` no primeiro turno — é comportamento esperado (AC-008), não defeito.

## Condição de retorno à planejadora

Retornar se o registro dinâmico não estiver disponível para o Aura, se o contrato SIWC mudar (endpoints/escopos), ou se o Codex enviar à rota `/v1/responses` algo rejeitado pelo preview (ex.: `tool_search`) que não possa ser desligado por configuração.

## Relatório de saída

Relatar fluxo implementado, campos persistidos (sem segredos), resultados, EV refs, limitações.
