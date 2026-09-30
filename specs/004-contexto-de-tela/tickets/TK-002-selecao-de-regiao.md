---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 004-contexto-de-tela
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-004"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/region_window.rs", "apps/desktop/src/region", "crates/aura-capture/src/crop.rs"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-002 — Seleção de região com tela congelada

## Objetivo e limites

Entrega a janela seletora (uma por monitor ou uma que cobre o monitor do cursor), com a captura congelada de fundo, retângulo com dimensões, confirmação ao soltar, cancelamento por `Esc`, e o Chip de região.

Não inclui: anotações na imagem, OCR da região.

## Leitura em ordem

1. `specs/004-contexto-de-tela/spec.md` → AC-004.
2. `crates/aura-capture/src/source.rs` (TK-001).
3. `apps/desktop/src-tauri/src/overlay/window.rs` (001) → `exclude_from_capture`.
4. `docs/design/ui-ux.md` → tokens.

## Decisões já resolvidas

- Congelar = capturar o monitor do cursor antes de mostrar a seletora; a seletora exibe a imagem em tela cheia sem bordas, sempre no topo, excluída de captura.
- Recorte feito no host sobre o `Frame` em pixels físicos (`crop(frame, rect)`); a UI trabalha em coordenadas lógicas e envia o fator de escala.
- Tamanho mínimo 8×8 px; clique sem arrastar cancela.
- Liberdade local: visual do retângulo e rótulo de dimensões.

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/region_window.rs` → `open_region_selector() -> Option<Rect>`.
- Novo: `apps/desktop/src/region/{RegionSelector.tsx,useDragRect.ts}`.
- Novo: `crates/aura-capture/src/crop.rs` → `crop(&Frame, Rect) -> Frame`.
- Existente: `capture.rs` → `capture_region_chip`.

## Contrato técnico

- Entradas: arraste do usuário.
- Saídas: `ContextChip{kind: Region}` com PNG recortado.
- Invariantes: `rect` sempre dentro do monitor; seletora excluída de captura.
- Erros: cancelamento → `None` sem Chip.

## Exemplos de aceite

- **AC-004** (unit): `crop` de frame 100×100 com quadrante superior esquerdo vermelho, rect `(0,0,50,50)` → 50×50 todo vermelho; rect fora dos limites → recortado à interseção. Vitest: arrastar de (10,10) a (110,60) com escala 1.5 → rect físico (15,15,150,75) enviado. Manual: 2 monitores, DPI 100%/150%, `Esc` cancela.

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-002.1 Unit `crop` red→green.
- [ ] TK-002.2 Vitest do seletor (conversão de escala).
- [ ] TK-002.3 Janela seletora e Chip; roteiro manual; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-capture crop`; `pnpm -C apps/desktop test -- RegionSelector`; roteiro manual multi-monitor.
- Estado esperado: verdes; prints.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner com um monitor.

## Condição de retorno à planejadora

Retornar se janelas Tauri transparentes em tela cheia tiverem atraso perceptível (> 150 ms) para aparecer.

## Relatório de saída

Relatar resultados, prints, EV refs.
