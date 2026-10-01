---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 001-fundacao-overlay
type: delivery
status: implemented
ticket_revision: 5
requires: ["TK-003", "TK-004"]
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-011", "AC-012", "AC-013"]
spec_revision: 2
plan_revision: 1
owned_areas: ["crates/aura-core/src/placement.rs", "apps/desktop/src/overlay", "apps/desktop/src-tauri/src/overlay/placement.rs", "crates/aura-store/src/placement_repo.rs"]
verification_status: stale
last_update: Evidence invalidated after an input changed.
---




# TK-005 — Estados compacto/expandido, posição por monitor e perda de foco

## Objetivo e limites

Entrega a alternância compacto ↔ expandido com animação, arrastar/redimensionar, persistência de posição e tamanho por monitor (com fallback quando o monitor some), e o comportamento ao perder foco (esconder ou manter aberto).

Não inclui: Minibar durante resposta (009), conteúdo da conversa (002).

## Leitura em ordem

1. `specs/001-fundacao-overlay/spec.md` → US-004.
2. `docs/design/ui-ux.md` → "Overlay" (tamanhos, posição padrão) e mapa de teclado.
3. `apps/desktop/src-tauri/src/overlay/focus.rs` → `PreviousApp.monitor_id` (TK-003).
4. `crates/aura-store/src/migrations/0001_init.sql` → tabela `overlay_placements` (TK-004).

## Decisões já resolvidas

- `place_overlay` é pura: entradas `monitors: Vec<Monitor{id, work_area, dpi}>`, `target: MonitorId`, `saved: Option<SavedPlacement>`, `state: Compact|Expanded`; saída `Rect` físico.
- Posição padrão: centralizado horizontalmente, topo em 1/3 − metade da altura compacta da área de trabalho; expandido cresce para baixo até 70% da altura.
- Resultado sempre recortado para ficar 100% dentro da `work_area`.
- Arrasto pela área vazia do cabeçalho (`data-tauri-drag-region`); redimensionar só no expandido.
- Perda de foco detectada por evento `Focused(false)` do Tauri; ignorada enquanto existe resposta em andamento (flag `turn_in_progress` que o esforço 002 alimenta; aqui sempre `false`).
- Liberdade local: curva/duração exatas ≤ 200 ms.

## Mapa de alterações

- Novo: `crates/aura-core/src/placement.rs` → `place_overlay`, `Monitor`, `SavedPlacement`, `OverlayMode`.
- Novo: `crates/aura-store/src/placement_repo.rs` → `PlacementRepo::{get(monitor_id), save}`.
- Novo: `apps/desktop/src-tauri/src/overlay/placement.rs` → enumeração de monitores (Win32 `EnumDisplayMonitors`, `GetDpiForMonitor`) e aplicação do `Rect`.
- Existente: `apps/desktop/src/overlay/OverlayShell.tsx` → estados, atalhos `Ctrl+↑/↓`, `prefers-reduced-motion`.
- Existente: `overlay/window.rs` → `show()` usa `place_overlay`; handler de `Moved/Resized` salva com debounce de 300 ms.

## Contrato técnico

- Entradas: `OverlayMode` desejado; eventos de mover/redimensionar; evento de foco.
- Saídas: janela posicionada; `overlay_state { visible, mode }`.
- Invariantes: janela nunca fora da área de trabalho; tamanho mínimo expandido 480×320.
- Erros: monitor salvo inexistente → posição padrão (não é erro visível).
- Efeitos: persistência em `overlay_placements` por `monitor_id` e modo.

## Exemplos de aceite

- **AC-011**: Vitest: estado `compact` + `Ctrl+↓` → `expanded`; `Ctrl+↑` → `compact`; com `matchMedia('(prefers-reduced-motion: reduce)')` → classe `motion-none`. Manual: gravação de tela mostra transição ≤ 200 ms.
- **AC-012** (unit): monitores `[A{work 0,0,1920,1040, dpi 96}, B{1920,0,2560,1400, dpi 144}]`, alvo B, salvo `{B, x:2100,y:100,w:700,h:500}` → `Rect(2100,100,700,500)`; alvo A sem salvo, compacto 640×64 → `Rect(640, 315, 640, 64)` (x=(1920−640)/2, y=round(1040/3−32)=round(314,67)=315); salvo em `C` inexistente → padrão do alvo; salvo parcialmente fora (`x:1800,w:400` em A) → recortado para `x:1520`.
- **AC-013**: E2E: Overlay visível → clicar no Bloco de Notas → Overlay continua visível (padrão); com `hide_on_blur=true` → invisível. Valor antigo `focusLoss: "hide"` gravado não é migrado (spec revisão 2).

## Dependências e sequência de execução

Depende de: TK-003, TK-004.

- [ ] TK-005.1 Unit AC-012, um caso por vez (red→green).
- [ ] TK-005.2 `PlacementRepo` + aplicação no `show()`; manual com 2 monitores.
- [ ] TK-005.3 Vitest AC-011 → UI de estados.
- [ ] TK-005.4 E2E AC-013.
- [ ] TK-005.5 Regressão, evidências (incluindo vídeo curto de demonstração do Marco 1 parcial).

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-core placement -p aura-store placement`; `pnpm -C apps/desktop test -- overlay`; `pnpm -C apps/desktop e2e -- --spec e2e/foco.spec.ts`; roteiro manual com 2 monitores e DPI diferentes.
- Estado esperado: verdes; roteiro sem janela fora da tela.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner com um único monitor não cobre AC-012 multi-monitor — unit + manual cobrem.

## Condição de retorno à planejadora

Retornar se o Tauri reportar coordenadas lógicas/físicas de forma inconsistente entre monitores com DPI diferente a ponto de exigir mudança do contrato de `place_overlay`.

## Relatório de saída

Relatar símbolos, casos de placement cobertos, resultados, EV refs, limitações.
