---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 008-extensoes-do-agente
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-001"]
acceptance_refs: ["AC-001", "AC-002", "AC-003", "AC-004"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-extensions/src/skills.rs", "crates/aura-codex/src/extensions.rs", "apps/desktop/src/extensions/skills", "apps/desktop/src/conversation/SlashMenu.tsx"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-001 — Skills: listar, importar com revisão, criar e invocar por `/`

## Objetivo e limites

Entrega a raiz de Skills do Aura (`skills/extraRoots/set` a cada início do app-server), lista com origem e ativação (`skills/config/write`), importação de pasta/zip com revisão, editor de Skill, e o menu `/` com Skills (invocação explícita com item `skill`).

Não inclui: Comandos rápidos (TK-003 usa o mesmo `SlashMenu`), marketplace.

## Leitura em ordem

1. Docs atuais: `https://developers.openai.com/codex/skills` (estrutura, escopos, invocação) e app-server `skills/list`, `skills/extraRoots/set`, `skills/config/write`, `skills/changed`, "Start a turn (invoke a skill)".
2. `specs/008-extensoes-do-agente/plan.md` → skills, OT-002.
3. `crates/aura-codex/src/{service.rs,supervisor.rs}` (002).
4. `apps/desktop/src/overlay/InputBar.tsx` (001/002).

## Decisões já resolvidas

- `validate_skill_dir`: exige `SKILL.md` com frontmatter `name` (kebab-case, ≤ 64) e `description` (≤ 1024); lista `scripts/`, `references/`, `assets/`.
- Import de zip: extrair em pasta temporária, validar, mostrar revisão (conteúdo do `SKILL.md` renderizado + lista de scripts com conteúdo expansível), instalar só após "Instalar".
- Menu `/`: seções "Skills" e "Comandos"; busca fuzzy por nome/descrição.
- Liberdade local: layout do editor.

## Mapa de alterações

- Novo: `crates/aura-extensions/{Cargo.toml,src/lib.rs,src/skills.rs}`.
- Novo: `crates/aura-codex/src/extensions.rs` → `skills`, `set_skill_enabled`, `set_extra_roots`.
- Novo: `apps/desktop/src/extensions/skills/{SkillsView.tsx,SkillEditor.tsx,ImportReview.tsx}`; `apps/desktop/src/conversation/SlashMenu.tsx`.

## Contrato técnico

- Entradas: pasta/zip; formulário do editor; seleção no `/`.
- Saídas: Skill instalada; `turn/start.input` com `{type:"text", text:"$<nome> <texto>"}` + `{type:"skill", name, path}`.
- Invariantes: OT-002.
- Erros: `SkillError::{MissingManifest, InvalidName, MissingDescription, AlreadyExists}`.

## Exemplos de aceite

- **AC-001**: falso `skills/list` com 3 Skills (Aura, Usuário, Sistema) → lista com origens; desativar → `skills/config/write{path, enabled:false}`.
- **AC-002**: fixture sem `description` → `MissingDescription`; zip com `scripts/run.ps1` → revisão mostra o script; cancelar → nada copiado.
- **AC-003**: criar "revisar-contrato" → pasta `skills/revisar-contrato/SKILL.md` com frontmatter correto; evento `skills/changed` → aparece no `/` sem reiniciar.
- **AC-004**: `/revisar` + Enter + "contrato.pdf anexado" → payload `[{type:"text", text:"$revisar-contrato contrato.pdf anexado"}, {type:"skill", name:"revisar-contrato", path:"…\\skills\\revisar-contrato\\SKILL.md"}]`.

## Dependências e sequência de execução

Depende de: 002-conversa-agente-codex/TK-003 (outro esforço). Nenhum ticket deste esforço.

- [ ] TK-001.1 Unit `validate_skill_dir` red→green.
- [ ] TK-001.2 Contrato lista/ativação/raízes.
- [ ] TK-001.3 Import + revisão; editor.
- [ ] TK-001.4 `SlashMenu` + payload AC-004; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-extensions skills -p aura-codex extensions`; `pnpm -C apps/desktop test -- skills SlashMenu`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se `skills/extraRoots/set` não existir na versão fixada (usar raiz `USER` ou `CODEX_HOME`).

## Relatório de saída

Relatar resultados e EV refs.
