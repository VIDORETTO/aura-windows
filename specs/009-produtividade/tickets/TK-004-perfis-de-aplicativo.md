---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 009-produtividade
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-007", "AC-008"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/productivity/profiles.rs", "crates/aura-store/src/migrations/0008_productivity.sql", "apps/desktop/src/settings/profiles"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-004 — Perfis de aplicativo

## Objetivo e limites

Entrega `AppProfile`, `match_profile`, a aplicação do perfil em Conversas novas (instruções, anexar tela ao abrir, modo e modelo padrão), o selo "Perfil: <nome>" e o editor com "Criar perfil para este app".

Não inclui: Memórias por perfil.

## Leitura em ordem

1. `specs/009-produtividade/plan.md` → profiles.
2. `crates/aura-codex/src/modes.rs` e `memories.rs` (002 TK-009, 008 TK-006) → composição de instruções.
3. `apps/desktop/src-tauri/src/overlay/focus.rs` (001) → `PreviousApp`.

## Decisões já resolvidas

- Casamento: `process_pattern` (glob, sem diferenciar maiúsculas) e `title_glob` opcional; mais específico vence (processo+título > só processo); empate → mais recente.
- Instruções do perfil entram após as pessoais em `developerInstructions` ("Perfil <nome>: …").
- Perfil só afeta Conversas novas abertas com aquele Aplicativo anterior.
- Liberdade local: UI.

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/productivity/profiles.rs`.
- Novo: `crates/aura-store/src/migrations/0008_productivity.sql`.
- Novo: `apps/desktop/src/settings/profiles/{ProfilesView.tsx,ProfileForm.tsx}`; selo no cabeçalho do Overlay.

## Contrato técnico

- Entradas: `PreviousApp`, perfis.
- Saídas: `Option<AppProfile>` aplicado.
- Invariantes: sem perfil → comportamento padrão inalterado.

## Exemplos de aceite

- **AC-007** (unit): perfis `[{code.exe}, {chrome.exe, title:"*Jira*"}]`; `PreviousApp{code.exe}` → perfil VS Code; `{chrome.exe, "PROJ-1 - Jira"}` → perfil Jira; `{chrome.exe, "YouTube"}` → nenhum. Contrato: Conversa nova com perfil VS Code → `developerInstructions` contém "Perfil VS Code: Responda com código TypeScript" e Chip de tela presente.
- **AC-008**: "Criar perfil para este app" com Aplicativo anterior `EXCEL.EXE` → formulário com `process_pattern = "excel.exe"`.

## Dependências e sequência de execução

Depende de: 008-extensoes-do-agente/TK-006 e 004-contexto-de-tela/TK-001 (outros esforços).

- [ ] TK-004.1 Unit `match_profile` red→green.
- [ ] TK-004.2 Aplicação na Conversa (contrato).
- [ ] TK-004.3 UI; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-desktop profiles`; `pnpm -C apps/desktop test -- ProfilesView`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Nenhuma prevista.

## Relatório de saída

Relatar resultados e EV refs.
