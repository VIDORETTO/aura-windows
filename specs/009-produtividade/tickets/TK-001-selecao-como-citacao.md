---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 009-produtividade
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-001"]
acceptance_refs: ["AC-001", "AC-002"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/productivity/selection.rs", "apps/desktop/src-tauri/src/productivity/clipboard_guard.rs", "tools/test-apps/selection-target", "apps/desktop/src/conversation/chips/SelectionChip.tsx"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-001 — Texto selecionado vira Chip de citação

## Objetivo e limites

Entrega `SelectionReader` (UIA → fallback clipboard), `ClipboardSnapshot`, o app de teste `selection-target`, o Chip de seleção e a matriz de compatibilidade.

Não inclui: inserir no app (TK-005 reutiliza `ClipboardSnapshot`).

## Leitura em ordem

1. `specs/009-produtividade/plan.md` → selection, clipboard_guard, OT-001, OT-002.
2. `apps/desktop/src-tauri/src/overlay/focus.rs` (001 TK-003) → `PreviousApp` capturado antes de mostrar.
3. `crates/aura-policy/src/decide.rs` (004 TK-003).
4. Docs atuais: UI Automation `IUIAutomation::GetFocusedElement`, `TextPattern.GetSelection`; clipboard Win32 (`EnumClipboardFormats`, `GetClipboardSequenceNumber`).

## Decisões já resolvidas

- Ordem: UIA (≤ 80 ms) → se vazio e o app for conhecido por não expor UIA (lista), fallback clipboard: snapshot → `SendInput Ctrl+C` no Aplicativo anterior (antes de o Overlay pegar o foco) → aguardar mudança do `GetClipboardSequenceNumber` até 150 ms → ler texto → restaurar snapshot.
- Limite do Chip: 50 000 caracteres (acima, truncar com aviso).
- Liberdade local: lista de apps para fallback.

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/productivity/{mod.rs,selection.rs,clipboard_guard.rs}`.
- Novo: `tools/test-apps/selection-target/` (Win32 com EDIT/RichEdit e seleção programável por argumento).
- Existente: `overlay/window.rs` → ler seleção entre `capture_previous` e `show`.
- Novo: `apps/desktop/src/conversation/chips/SelectionChip.tsx`.

## Contrato técnico

- Entradas: `PreviousApp`.
- Saídas: `Option<String>` → Chip `Selection`.
- Invariantes: OT-001, OT-002; não atrasar a abertura do Overlay além de +100 ms (selection em paralelo; Chip aparece quando pronto).
- Erros: falhas silenciosas (sem Chip) com log `debug`.

## Exemplos de aceite

- **AC-001**: `selection-target --text "Aura seleção 42" --select-all` em primeiro plano + clipboard com "original" (texto + HTML) → abrir Overlay → Chip "Seleção · 15 caracteres" com o texto; clipboard depois = "original" nos dois formatos. Matriz manual: Chrome, Edge, Word, VS Code, Bloco de Notas, Teams.
- **AC-002**: sem seleção → nenhum Chip e sequência do clipboard inalterada; `selection-target` renomeado como processo excluído (regra de teste) → nenhum Chip e nenhuma leitura.

## Dependências e sequência de execução

Depende de: 001-fundacao-overlay/TK-003, 002-conversa-agente-codex/TK-005 e 004-contexto-de-tela/TK-003 (outros esforços).

- [ ] TK-001.1 `ClipboardSnapshot` integração (red→green).
- [ ] TK-001.2 `UiaSelection` com app de teste (red→green).
- [ ] TK-001.3 Fallback clipboard + política.
- [ ] TK-001.4 Chip + matriz manual; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-desktop --features win-integration selection clipboard_guard`; `pnpm -C apps/desktop test -- SelectionChip`; matriz manual.
- Estado esperado: verdes; matriz registrada (H-018).
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner sem desktop interativo.

## Condição de retorno à planejadora

Retornar se o fallback por `Ctrl+C` causar efeitos colaterais visíveis em apps comuns (ex.: terminal enviando interrupção) — nesse caso, lista de apps onde o fallback é proibido.

## Relatório de saída

Relatar matriz de apps, resultados, EV refs.
