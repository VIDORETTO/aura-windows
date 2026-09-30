---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 009-produtividade
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-005"]
acceptance_refs: ["AC-009", "AC-010"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/productivity/inject.rs", "apps/desktop/src/conversation/InsertButton.tsx"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-005 — Inserir a resposta no Aplicativo anterior

## Objetivo e limites

Entrega `insert(previous, text)` (colar com restauração da área de transferência; digitação simulada como alternativa; copiar como fallback), o botão "Inserir no app" por mensagem e por bloco de código, e o atalho `Ctrl+Shift+Enter`.

Não inclui: inserção formatada (RTF/HTML) — só texto.

## Leitura em ordem

1. `apps/desktop/src-tauri/src/productivity/clipboard_guard.rs` (TK-001).
2. `apps/desktop/src-tauri/src/overlay/focus.rs` (001 TK-003) → `restore_previous`.
3. `tools/test-apps/selection-target/` (TK-001) → reaproveitar como alvo de inserção.

## Decisões já resolvidas

- Sequência: esconder Overlay → `restore_previous` → aguardar foco (≤ 200 ms) → snapshot clipboard → set texto → `SendInput Ctrl+V` → aguardar 150 ms → restaurar snapshot.
- Se o foco não voltar (janela fechada/elevada) → `CopiedOnly` e toast com motivo (AC-010).
- Texto do bloco de código: sem as cercas ```.
- Liberdade local: tempos exatos dentro dos limites.

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/productivity/inject.rs`.
- Novo: `apps/desktop/src/conversation/InsertButton.tsx`; ação em `Message.tsx` e nos blocos de código.

## Contrato técnico

- Entradas: texto; `PreviousApp`.
- Saídas: `InsertOutcome{Pasted|Typed|CopiedOnly(reason)}`.
- Invariantes: OT-001 (clipboard restaurado sempre, exceto em `CopiedOnly`, onde o texto fica intencionalmente).

## Exemplos de aceite

- **AC-009**: `selection-target --edit-empty` em foco → abrir Overlay → resposta "Olá do Aura" → `Ctrl+Shift+Enter` → campo do app contém "Olá do Aura"; clipboard volta a "original".
- **AC-010**: fechar o `selection-target` antes de inserir → `CopiedOnly(AppGone)`, clipboard contém "Olá do Aura", toast "O app anterior foi fechado; resposta copiada".

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-005.1 Integração AC-009 red→green.
- [ ] TK-005.2 Fallbacks (AC-010) e digitação simulada.
- [ ] TK-005.3 UI e atalho; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-desktop --features win-integration inject`; `pnpm -C apps/desktop test -- InsertButton`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner sem desktop interativo.

## Condição de retorno à planejadora

Retornar se restaurar o foco falhar com frequência em apps comuns (reavaliar técnica de foco do 001).

## Relatório de saída

Relatar matriz de apps, resultados e EV refs.
