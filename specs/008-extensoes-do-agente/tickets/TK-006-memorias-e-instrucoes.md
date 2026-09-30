---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 008-extensoes-do-agente
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-005"]
acceptance_refs: ["AC-015", "AC-016"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-codex/src/memories.rs", "apps/desktop/src/extensions/memories", "apps/desktop/src/settings/InstructionsSection.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-006 — Memórias e instruções pessoais

## Objetivo e limites

Entrega a ativação de `features.memories` no `CODEX_HOME` do Aura, a tela Memórias (listar, editar, excluir entradas nos arquivos de memória do Codex), o estado de consolidação (`memory/status` quando disponível), e "Instruções pessoais" globais + instruções extras por Conversa via `developerInstructions`.

Não inclui: memórias por Perfil de aplicativo (009 TK-004).

## Leitura em ordem

1. Docs atuais: `https://developers.openai.com/codex/customization/memories` (formato e local dos arquivos de memória), config `features.memories`, app-server `memory/status` (experimental).
2. `crates/aura-codex/src/home.rs` (002 TK-001).
3. `crates/aura-codex/src/modes.rs` (002 TK-009) → composição de `developerInstructions`.

## Decisões já resolvidas

- Memórias desligadas por padrão; ao ligar, explicar o que é guardado e onde.
- Edição direta dos arquivos de memória no `CODEX_HOME` do Aura com escrita atômica, somente com o app-server ocioso (senão agendar para depois do turno).
- Composição de instruções: Persona (`baseInstructions`) + modo + instruções pessoais + instruções da Conversa (nessa ordem em `developerInstructions`).
- Liberdade local: UI.

## Mapa de alterações

- Novo: `crates/aura-codex/src/memories.rs` → `enable`, `list_entries`, `update_entry`, `delete_entry`, `status`.
- Existente: `modes.rs`/`service.rs` → composição de instruções.
- Novo: `apps/desktop/src/extensions/memories/MemoriesView.tsx`, `apps/desktop/src/settings/InstructionsSection.tsx`.

## Contrato técnico

- Entradas: toggles e textos do usuário.
- Saídas: config e arquivos de memória; `developerInstructions`.
- Invariantes: com memórias desligadas, `features.memories=false` no TOML.
- Erros: formato de arquivo desconhecido na versão fixada → tela somente leitura com aviso.

## Exemplos de aceite

- **AC-015**: ligar → TOML `features.memories = true` (snapshot); fixture de arquivos de memória → lista entradas; excluir uma → arquivo reescrito sem ela; desligar → `false` e nenhuma escrita nova (manual).
- **AC-016**: instruções pessoais "Responda em português, seja direto." + extras da Conversa "Foque em Excel" → `developerInstructions` = "<modo>\n\nInstruções do usuário: Responda em português, seja direto.\n\nNesta conversa: Foque em Excel" (payload do `thread/start`).

## Dependências e sequência de execução

Depende de: TK-001; 002-conversa-agente-codex/TK-009 (outro esforço).

- [ ] TK-006.1 Composição de instruções (unit) red→green.
- [ ] TK-006.2 Toggle de memórias + leitura/escrita de entradas (fixtures da versão fixada).
- [ ] TK-006.3 UI; manual; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-codex memories modes`; `pnpm -C apps/desktop test -- MemoriesView InstructionsSection`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se o formato de memória do Codex não for documentado/estável o bastante para edição segura (limitar a ativar/desativar e apagar tudo).

## Relatório de saída

Relatar resultados e EV refs.
