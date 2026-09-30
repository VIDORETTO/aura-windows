---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 003-byok-gateway
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-001", "FR-002"]
acceptance_refs: ["AC-001", "AC-002", "AC-003", "AC-004", "AC-005"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-gateway/src/registry.rs", "crates/aura-gateway/src/credentials.rs", "crates/aura-gateway/src/discovery.rs", "crates/aura-store/src/migrations/0003_providers.sql", "apps/desktop/src/settings/providers"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-001 — Cadastro de Provedores, Credenciais no cofre e teste de conexão

## Objetivo e limites

Entrega `ProviderRegistry`, `CredentialStore` (Windows + memória), presets, a seção "Provedores" nas Configurações e o teste de conexão com categorias de erro.

Não inclui: servidor Gateway e uso em Conversas (TK-002+), descoberta completa de capacidades (TK-007; aqui o teste de conexão apenas chama `/models` ou uma requisição mínima).

## Leitura em ordem

1. `specs/003-byok-gateway/spec.md` → US-001 e "Decisions" (lista de presets).
2. `specs/003-byok-gateway/plan.md` → registry, credentials, OT-001.
3. `crates/aura-core/src/secret.rs` (001 TK-006) → `Secret`.
4. `apps/desktop/src/settings/SettingsWindow.tsx` (001 TK-004) → registrar nova seção.
5. Docs atuais do crate `keyring` (backend Windows) e endpoints `/models` de cada preset.

## Decisões já resolvidas

- Presets em `crates/aura-gateway/presets.toml`: `id, nome, wire, base_url, credencial_obrigatória, suporta_models_endpoint, cabeçalhos_padrão` (ex.: OpenRouter `HTTP-Referer`/`X-Title` opcionais; Anthropic `anthropic-version: 2023-06-01`; Azure `api-key` em vez de Bearer + `api-version` como query).
- Teste de conexão: `GET <base>/models` quando suportado; senão `POST` mínimo (1 token) ao endpoint do formato; timeout 10 s.
- Categorias: `Unauthorized`, `NotFound`, `Timeout`, `Tls`, `Network`, `RateLimited`, `Unexpected(status)`.
- UI exibe só `••••` + 4 últimos caracteres (calculados ao salvar e guardados como metadado não-secreto).
- Liberdade local: layout do formulário.

## Mapa de alterações

- Novo: `crates/aura-gateway/Cargo.toml`, `src/lib.rs`, `src/registry.rs` (`ProviderRegistry`, `Provider`, `ProviderDraft`, `Wire`, `Preset`), `src/credentials.rs` (`CredentialStore`, `WindowsCredentialStore`, `MemoryCredentialStore`), `src/discovery.rs` (`test_connection`), `presets.toml`.
- Novo: `crates/aura-store/src/migrations/0003_providers.sql`.
- Novo: `apps/desktop/src-tauri/src/providers.rs` → comandos `providers_list/upsert/remove/test`.
- Novo: `apps/desktop/src/settings/providers/{ProvidersSection.tsx,ProviderForm.tsx,TestConnectionButton.tsx}`.

## Contrato técnico

- Entradas: `ProviderDraft{name, preset|custom{base_url, wire, headers}, credential: Option<Secret>}`.
- Saídas: `Provider` (sem segredo), `ConnectionResult{ok, category?, detail}`.
- Invariantes: OT-001; `id` estável (slug + sufixo aleatório).
- Erros: `RegistryError::{Invalid(field), Storage, Credential}`.
- Efeitos: segredo em `Aura/provider/<id>` no Credential Manager; `status` e `last_error` persistidos.

## Exemplos de aceite

- **AC-001**: preset OpenRouter + `sk-or-v1-...abcd` → `Provider{wire:Responses, base_url:"https://openrouter.ai/api/v1"}`; `wiremock` `/models` 200 → status `Verified`; UI mostra `••••abcd`.
- **AC-002**: personalizado `http://localhost:8000/v1`, Chat Completions, header `X-Team: a` → salvo e testado.
- **AC-003**: `wiremock` 401 → `Unauthorized`; 404 → `NotFound`; atraso 11 s → `Timeout`; TLS autoassinado → `Tls`; provedor permanece com `status: Error`.
- **AC-004**: remover → `CredentialStore::get(id) == None` (Windows real e memória); lista sem o provedor.
- **AC-005**: salvar `sk-test-BYOK-987` no Windows → `Select-String -Path "$env:LOCALAPPDATA\Aura\*" -Pattern "sk-test-BYOK-987" -Recurse` → 0 resultados (inclui `aura.db`, `-wal`, logs, `codex-home\config.toml`).

## Dependências e sequência de execução

Depende de: 001-fundacao-overlay/TK-006, 001-fundacao-overlay/TK-004 e 002-conversa-agente-codex/TK-002 (outros esforços; reusa a abstração `CredentialStore`). Nenhum ticket deste esforço.

- [ ] TK-001.1 Integração registry + MemoryCredentialStore (AC-001/AC-002) red→green.
- [ ] TK-001.2 `test_connection` com `wiremock`, uma categoria por vez (AC-003).
- [ ] TK-001.3 `WindowsCredentialStore` + AC-004/AC-005 no Windows.
- [ ] TK-001.4 UI + Vitest.
- [ ] TK-001.5 Evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-gateway registry credentials discovery`; `cargo nextest run -p aura-gateway --test windows_credentials` (Windows); `pnpm -C apps/desktop test -- providers`; varredura AC-005.
- Estado esperado: verdes; varredura sem ocorrências.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: rede bloqueada no runner — testes usam `wiremock` local.

## Condição de retorno à planejadora

Retornar se o Credential Manager não for acessível (ex.: perfil restrito) sem alternativa segura.

## Relatório de saída

Relatar presets incluídos, símbolos, resultados, EV refs.
