---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 008-extensoes-do-agente
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-005", "AC-006", "AC-007", "AC-008", "AC-009"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-extensions/src/mcp_config.rs", "crates/aura-extensions/src/import.rs", "apps/desktop/src/extensions/mcp", "crates/aura-store/src/migrations/0007_extensions.sql"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-002 — Gerenciador de Servidores MCP

## Objetivo e limites

Entrega adicionar/editar/remover servidores stdio e HTTP, segredos no cofre com injeção por variável de ambiente, status e ferramentas (`mcpServerStatus/list`), liga/desliga por ferramenta, modo de aprovação por servidor, OAuth (`mcpServer/oauth/login`) e importação de configurações externas.

Não inclui: o servidor MCP do próprio Aura (004), marketplace de conectores.

## Leitura em ordem

1. Docs atuais: `https://developers.openai.com/codex/mcp` e config reference `mcp_servers.<id>.*` (command, args, env, env_vars, url, bearer_token_env_var, enabled_tools, disabled_tools, default_tools_approval_mode, required); app-server `mcpServerStatus/list`, `config/mcpServer/reload`, `mcpServer/oauth/login`, `externalAgentConfig/detect|import`.
2. `specs/008-extensoes-do-agente/plan.md` → mcp_config, import, OT-001.
3. `crates/aura-codex/src/home.rs` (002) → `ConfigContributor`; `crates/aura-gateway/src/credentials.rs` (003 TK-001/002 TK-002) → `CredentialStore`.

## Decisões já resolvidas

- Mapeamento de aprovação: "sempre perguntar" → `approve`, "perguntar para escrita" → `writes` (padrão), "automático" → `auto` (com aviso).
- Após salvar: reescrever `config.toml` e chamar `config/mcpServer/reload`.
- Importação: preferir `externalAgentConfig/detect|import` do app-server para MCP quando cobrir a fonte; complementar com importadores próprios para Claude Desktop/Cursor/VS Code; valores com cara de segredo (`*_KEY`, `*_TOKEN`, `Authorization`) vão ao cofre.
- Liberdade local: UI.

## Mapa de alterações

- Novo: `crates/aura-extensions/src/{mcp_config.rs,import.rs}`.
- Novo: `crates/aura-store/src/migrations/0007_extensions.sql`.
- Existente: `crates/aura-codex/src/extensions.rs` → `mcp_status`, `mcp_oauth_login`, `reload_mcp`; `supervisor.rs` → injeta variáveis `AURA_MCP_*`.
- Novo: `apps/desktop/src/extensions/mcp/{McpView.tsx,McpServerForm.tsx,ToolsList.tsx,ImportDialog.tsx}`.

## Contrato técnico

- Entradas: `McpServerSpec` da UI.
- Saídas: TOML contribuído; status/ferramentas.
- Invariantes: OT-001.
- Erros: `McpError::{InvalidSpec, RuntimeMissing(npx|uvx|python), StartFailed(log_tail), OAuthFailed}`.

## Exemplos de aceite

- **AC-005**: servidor de referência `npx -y @modelcontextprotocol/server-everything` (CI Windows com Node) → status conectado e ferramentas listadas (`echo`, `add`, …); comando inexistente → erro com últimas 20 linhas de stderr.
- **AC-006**: `wiremock` como servidor OAuth + `mcpServer/oauth/login` roteirizado no falso → URL aberta; `mcpServer/oauthLogin/completed{success:true}` → conectado.
- **AC-007**: desligar `add` → TOML `disabled_tools = ["add"]` (snapshot) e reload chamado.
- **AC-008**: modo "perguntar para escrita" → TOML `default_tools_approval_mode = "writes"`; manual: ferramenta de escrita gera cartão de Aprovação.
- **AC-009**: fixtures `claude_desktop_config.json` (2 servidores, um com `env.API_KEY`), `.cursor/mcp.json` (1) → 3 detectados; importar → TOML sem `API_KEY` literal e cofre com `Aura/mcp/<nome>/API_KEY`.

## Dependências e sequência de execução

Depende de: TK-001 (compartilha `aura-extensions` e raiz de configuração).

- [ ] TK-002.1 Unit `mcp_config` → TOML (snapshots) red→green.
- [ ] TK-002.2 Integração com servidor de referência (AC-005/AC-007).
- [ ] TK-002.3 OAuth (AC-006) e aprovação (AC-008).
- [ ] TK-002.4 Importadores (AC-009); UI; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-extensions mcp_config import`; `cargo nextest run -p aura-codex --features win-integration mcp_reference` (Windows com Node); `pnpm -C apps/desktop test -- mcp`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: Node ausente no runner.

## Condição de retorno à planejadora

Retornar se a injeção de segredos por ambiente não for suportada para servidores stdio na versão fixada.

## Relatório de saída

Relatar resultados e EV refs.
