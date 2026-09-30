---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 008-extensoes-do-agente
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-010", "AC-011"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-extensions/src/quick.rs", "apps/desktop/src/extensions/quick"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-003 — Comandos rápidos embutidos e personalizados

## Objetivo e limites

Entrega o registro de Comandos rápidos (embutidos + do usuário), a expansão pura com variáveis `{selecao}`, `{tela}`, `{texto}`, `{args}`, `{idioma}`, o editor e a integração no `SlashMenu`.

Não inclui: comandos que executam ações do sistema (só prompts e ações já existentes como `/screen`, `/compactar`).

## Leitura em ordem

1. `specs/008-extensoes-do-agente/spec.md` → AC-010, AC-011.
2. `apps/desktop/src/conversation/SlashMenu.tsx` (TK-001).
3. `crates/aura-core/src/context.rs` (002 TK-005) → Chips (seleção de 009 TK-001, tela de 004 TK-001).

## Decisões já resolvidas

- Embutidos (pt-BR/en): `/tldr` "Resuma em até 5 tópicos:", `/traduzir <idioma>` "Traduza para {idioma}:", `/reescrever` "Reescreva com clareza, mantendo o sentido:", `/explicar` "Explique de forma simples:", `/corrigir` "Corrija gramática e ortografia, devolvendo só o texto corrigido:", `/resumir-tela` (adiciona Chip de tela e "Resuma o que está na tela").
- Ordem de alvo: `{selecao}` se houver Chip de seleção; senão Chip de tela quando o comando pede; senão texto digitado após o comando.
- Texto exibido na conversa: rótulo compacto do comando + alvo; o prompt expandido vai ao modelo.
- Liberdade local: textos finais via i18n.

## Mapa de alterações

- Novo: `crates/aura-extensions/src/quick.rs` → `QuickCommand`, `expand`, `builtins(lang)`.
- Existente: `0007_extensions.sql` → semear embutidos.
- Novo: `apps/desktop/src/extensions/quick/QuickCommandsEditor.tsx`; `SlashMenu` usa comandos.

## Contrato técnico

- Entradas: comando, argumentos, contexto.
- Saídas: `Expansion{prompt_text, display_text, add_chips}`.
- Invariantes: variável ausente → string vazia e aviso no editor (nunca `{selecao}` literal enviado).
- Erros: nome duplicado ou inválido.

## Exemplos de aceite

- **AC-010** (unit): `/traduzir inglês` + seleção "olá, tudo bem?" → prompt "Traduza para inglês:\n\nolá, tudo bem?", display "/traduzir inglês · seleção"; `/tldr` sem seleção com texto digitado "texto longo…" → "Resuma em até 5 tópicos:\n\ntexto longo…"; `/resumir-tela` → `add_chips=[Screen]`.
- **AC-011** (unit): usuário cria `/email-formal` com "Reescreva de forma formal: {selecao}" → com seleção "oi chefe" → "Reescreva de forma formal: oi chefe"; sem seleção e texto "preciso de folga" → "Reescreva de forma formal: preciso de folga".

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-003.1 Unit `expand` caso a caso red→green.
- [ ] TK-003.2 Embutidos + persistência.
- [ ] TK-003.3 Editor e menu; Vitest; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-extensions quick`; `pnpm -C apps/desktop test -- QuickCommandsEditor SlashMenu`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Nenhuma prevista além de conflito de nomes com Skills (decidir precedência) — padrão: Comando rápido embutido > Skill > comando do usuário, com sufixo de origem no menu.

## Relatório de saída

Relatar resultados e EV refs.
