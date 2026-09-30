---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 004-contexto-de-tela
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-001"]
acceptance_refs: ["AC-001", "AC-002"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-capture/src/source.rs", "crates/aura-capture/src/wgc.rs", "apps/desktop/src-tauri/src/capture.rs", "apps/desktop/src/conversation/chips/ScreenChip.tsx"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-001 — Captura de tela sob demanda como Chip (Marco 1)

## Objetivo e limites

Entrega `FrameSource` com `WgcSource` e `SyntheticSource`, o comando `capture_screen_chip{target: Monitor|Window}` que captura o monitor ou a janela do Aplicativo anterior, gera miniatura, salva PNG no workspace pendente e adiciona um Chip via `ContextTray`; Comando rápido `/screen`, botão e `Ctrl+Shift+S`.

Não inclui: Política/Janelas excluídas (TK-003 — até lá a captura sob demanda do usuário é permitida sem redação, apenas com o Overlay excluído por affinity), região (TK-002), ferramentas do agente (TK-004).

## Leitura em ordem

1. `specs/004-contexto-de-tela/spec.md` → US-001, limites.
2. `specs/004-contexto-de-tela/plan.md` → `aura-capture::source`, OT-001 (a partir do TK-003).
3. `apps/desktop/src-tauri/src/overlay/focus.rs` (001 TK-003) → `PreviousApp`.
4. `crates/aura-core/src/context.rs` (002 TK-005) → `ContextTray`, `ChipKind::Screen`.
5. Docs atuais: `windows-capture` (captura única de monitor/janela), `GraphicsCaptureItem`.

## Decisões já resolvidas

- Captura única via WGC (sessão aberta, 1 frame, fechada); monitor = `PreviousApp.monitor_id`; janela = `PreviousApp.hwnd`.
- Arquivo: PNG em resolução nativa no workspace; ao enviar, 002 reduz para ≤ 2048 px no maior lado (acrescentar redução em `drain_for_turn` se ainda não existir).
- Rótulo do Chip: "Tela · <processo sem .exe> — <título truncado 40>"; alternância Monitor/Janela no próprio Chip.
- Liberdade local: codificação da miniatura (WebP 256 px).

## Mapa de alterações

- Novo: `crates/aura-capture/Cargo.toml`, `src/lib.rs`, `src/source.rs` (`FrameSource`, `Frame{width, height, bgra: Vec<u8>, captured_at}`, `Target`), `src/wgc.rs` (`WgcSource`), `src/synthetic.rs`.
- Novo: `apps/desktop/src-tauri/src/capture.rs` → `capture_screen_chip`, `chip_toggle_screen_target`.
- Novo: `apps/desktop/src/conversation/chips/ScreenChip.tsx`; `InputBar` trata `/screen` e `Ctrl+Shift+S`.

## Contrato técnico

- Entradas: `Target::Monitor(id) | Target::Window(hwnd)`.
- Saídas: `ContextChip{kind: Screen, preview_path, payload: File(png)}`.
- Invariantes: janelas do Aura nunca aparecem (affinity de 001 OT-002).
- Erros: `CaptureError::{WindowMinimized, WindowGone, Unsupported, Os(code)}` → Chip não criado e toast com motivo.
- Efeitos: arquivo no workspace pendente/conversa.

## Exemplos de aceite

- **AC-001**: Overlay visível pintado de magenta (modo debug 001) + `capture_screen_chip(Monitor)` → PNG sem pixel `#FF00FF`; 20 execuções → p95 do comando ao evento `chips_changed` ≤ 300 ms; rótulo "Tela · notepad — Sem título - Bloco de Notas".
- **AC-002**: janela de teste A (verde `#00FF00`, 400×300) parcialmente coberta por janela B (azul) + `Target::Window(A)` → imagem 400×300 (± borda DWM) só com verde.
- Unit (Linux): `SyntheticSource` → Chip com dimensões corretas.

## Dependências e sequência de execução

Depende de: 001-fundacao-overlay/TK-003 e 002-conversa-agente-codex/TK-005 (outros esforços). Nenhum ticket deste esforço.

- [ ] TK-001.1 Unit com `SyntheticSource` → Chip (red→green).
- [ ] TK-001.2 Integração Windows AC-001 (magenta) red→green com `WgcSource`.
- [ ] TK-001.3 AC-002 janela.
- [ ] TK-001.4 UI `/screen`, botão, atalho; Vitest.
- [ ] TK-001.5 Demonstração do Marco 1 (vídeo: atalho → login → `/screen` → pergunta → resposta); evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-capture`; `cargo nextest run -p aura-capture --features win-integration capture_once` (Windows interativo); `pnpm -C apps/desktop test -- ScreenChip`.
- Estado esperado: verdes; vídeo do Marco 1.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner sem desktop → `not_run` e executar na máquina de referência.

## Condição de retorno à planejadora

Retornar se a WGC não capturar janelas de certos apps (UWP/elevados) de forma relevante para o caso de uso principal.

## Relatório de saída

Relatar símbolos, latências, EV refs, vídeo do Marco 1.
