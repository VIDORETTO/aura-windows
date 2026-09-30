---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 008-extensoes-do-agente
revision: 1
spec_revision: 1
status: ready
---

# Plan: Extensões do agente

## Summary

Quase tudo aqui é orquestração de capacidades que o Codex app-server já tem (skills, MCP, planos, diffs, memórias). O Aura acrescenta: gerência de configuração (`ConfigContributor` para `mcp_servers.*`, raízes de skills), cofre para segredos de MCP, importadores, o registro de Comandos rápidos (expansão local de modelos) e a UI dos painéis. Um crate fino `aura-extensions` concentra regras testáveis (validação de Skill, expansão de Comandos rápidos, mapeamento de config MCP, importadores).

## Technical context

- Language/runtime: Rust + React.
- Dependencies: app-server `skills/list`, `skills/extraRoots/set`, `skills/config/write`, `skills/changed`, `config/batchWrite`, `config/mcpServer/reload`, `mcpServerStatus/list`, `mcpServer/oauth/login`, `mcpServer/oauthLogin/completed`, `externalAgentConfig/detect`/`import`, `collaborationMode/list` (experimental), `turn/plan/updated`, `turn/diff/updated`, `features.memories` e `memory/status` (experimental); `zip` crate; `gray_matter`/`serde_yaml` para frontmatter.
- Storage/data: `%LOCALAPPDATA%\Aura\skills\`, tabela `quick_commands(id, name, template, builtin)`, segredos MCP no Credential Manager (`Aura/mcp/<servidor>`), `settings.personal_instructions`.
- Test command: `cargo nextest run -p aura-extensions -p aura-codex`; `pnpm -C apps/desktop test -- extensions`.
- Target/platform: Windows (qualquer SO para lógica).

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-005; AC-001–AC-016.

## Modules, interfaces, consumers, and seams

| Module | Interface | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `aura-extensions::skills` | `validate_skill_dir(path) -> Result<SkillManifest>`, `install(src, dest)`, `create(name, description, body)` | host/UI | Diretórios temporários com fixtures |
| `aura-extensions::quick` | `QuickCommand{name, template, builtin}`, `expand(cmd, ctx: QuickContext{selection?, screen_chip?, typed?, args}) -> Expansion{prompt_text, chips}` (puro) | InputBar | Casos literais |
| `aura-extensions::mcp_config` | `McpServerSpec{name, transport: Stdio{cmd,args,env_refs}|Http{url, auth}, enabled_tools, disabled_tools, approval_mode}` → contribuição TOML; `env_refs` resolvidos do cofre no spawn | `CodexHome` | Snapshot TOML |
| `aura-extensions::import` | `detect_external() -> Vec<DetectedServer>` (Claude Desktop `%APPDATA%\Claude\claude_desktop_config.json`, Cursor `~/.cursor/mcp.json`, VS Code `mcp.json`, Codex `~/.codex/config.toml`) | UI | Fixtures de arquivos |
| `aura-codex` (existente) | `skills()`, `set_skill_enabled`, `mcp_status()`, `mcp_oauth_login(name)`, `reload_mcp()`, `start(opts{collaboration_mode?})`, eventos `PlanUpdated`, `DiffUpdated` | host | App-server falso + transcripts |
| UI | `SkillsView`, `SkillEditor`, `ImportReview`, `McpView`, `McpServerForm`, `SlashMenu`, `QuickCommandsEditor`, `ProgressPanel`, `ChangesPanel`, `FilesPanel`, `MemoriesView`, `InstructionsSection` | usuário | Vitest + mockIPC |

Segredos de servidores stdio: variáveis marcadas como segredo não vão ao `config.toml`; o host injeta-as no ambiente do app-server com nomes `AURA_MCP_<SERVIDOR>_<VAR>` e o TOML usa `env_vars` whitelisting (confirmar semântica de `env`/`env_vars` na versão fixada). HTTP: `bearer_token_env_var` apontando para variável injetada pelo host.

## Chosen approach and alternatives

- **Delegar ao Codex** o que ele já faz; o Aura só configura e apresenta.
- **Comandos rápidos expandidos localmente** (não são Skills): rápidos, determinísticos, sem custo de contexto.
- Modo plano: preferir `collaborationMode` (experimental) atrás de flag; fallback: `sandbox: readOnly` + instrução de planejamento + botões que reenviam "Execute o plano aprovado" em Modo Tarefa.
- Alternativa descartada: implementar gerenciador de MCP próprio fora do Codex.

## Data, compatibility, and external dependencies

Migração `0007_extensions.sql` (`quick_commands`, `mcp_servers_meta`). Comandos embutidos semeados na migração (não editáveis, desativáveis).

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001, AC-003 | Contrato (falso `skills/list`, `skills/changed`) + Vitest | Spec | Lista com origem; criar → aparece |
| AC-002 | Unit `validate_skill_dir` + Vitest | Padrão agentskills | Fixture sem `description` → erro; zip com scripts → revisão lista `scripts/*` |
| AC-004 | Contrato | Docs app-server (item `skill`) | `turn/start.input` inclui `{type:"skill", name, path}` e texto `$nome …` |
| AC-005, AC-007, AC-008 | Integração com servidor MCP de referência (Node) em CI Windows + snapshot TOML | Spec | Status/ferramentas; `disabled_tools`; aprovação conforme modo |
| AC-006 | `wiremock` OAuth + contrato `mcpServer/oauth/login` | Docs | Fluxo completo simulado |
| AC-009 | Unit importadores | Fixtures | Detecta 3 servidores; segredo movido ao cofre |
| AC-010, AC-011 | Unit `expand` + Vitest | Casos literais | `/traduzir inglês` com seleção "olá" → "Traduza para inglês:\n\nolá" |
| AC-012 | Contrato + manual | Spec | `collaborationMode` ou fallback; nenhum comando executado |
| AC-013, AC-014 | Contrato (`turn/plan/updated`, `turn/diff/updated`) + Vitest | Transcript | Painéis refletem eventos; prévia HTML sandbox |
| AC-015 | Contrato + manual | Spec | `features.memories` toggled; tela lista/edita arquivos de memória |
| AC-016 | Contrato (payload) | Spec | `developerInstructions` contém instruções pessoais |

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `crates/aura-extensions/src/{skills.rs,quick.rs,mcp_config.rs,import.rs}` | new | ver tabela | Regras | 2026-09-29 |
| `crates/aura-codex/src/{extensions.rs,plan_mode.rs}` | new | chamadas ao app-server | Integração | 2026-09-29 |
| `apps/desktop/src/extensions/**`, `src/conversation/{SlashMenu.tsx,ProgressPanel.tsx,ChangesPanel.tsx,FilesPanel.tsx}` | new | UI | Interface | 2026-09-29 |
| `crates/aura-store/src/migrations/0007_extensions.sql` | new | tabelas | Dados | 2026-09-29 |

## Derived technical obligations

- **OT-001** → FR-002: nenhum segredo de MCP em `config.toml` ou logs.
- **OT-002** → FR-001: importar Skill nunca executa scripts; só copia após confirmação.
- **OT-003** → FR-004: prévia HTML sempre em iframe `sandbox` sem `allow-same-origin` e com CSP sem rede.

## Risks and gates

- APIs experimentais (collaborationMode, memory/status) podem mudar entre versões — flag + contrato por versão.
- Servidores MCP de terceiros podem ser maliciosos — aviso na adição e modo de aprovação padrão "perguntar para escrita".
- G2: satisfeito. G3: TK-001 depende de 002 TK-003 concluído.
