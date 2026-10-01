---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 001-fundacao-overlay
type: delivery
status: implemented
ticket_revision: 5
requires: ["TK-002"]
requirement_refs: ["FR-002", "FR-007"]
acceptance_refs: ["AC-004", "AC-005", "AC-006", "AC-007"]
spec_revision: 2
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/overlay", "crates/aura-core/src/gesture.rs", "tools/aura-bench"]
verification_status: stale
last_update: Evidence invalidated after an input changed.
---




# TK-003 — Atalho de invocação, duplo toque, foco e medição de abertura

## Objetivo e limites

Entrega o Atalho de invocação global (padrão `Ctrl+Shift+Space`), o gesto opcional de duplo toque em Ctrl, o rastreio do Aplicativo anterior com devolução de foco ao esconder, o botão "Minimizar para a bandeja" para esconder (o `Esc` só fecha menus — spec revisão 2) e o `aura-bench open-latency`/`idle-memory` com primeira medição.

Não inclui: UI de configuração do atalho (TK-004 expõe o campo; aqui o valor vem de constante/arquivo de teste), posicionamento por monitor (TK-005; aqui o Overlay aparece no centro do monitor do Aplicativo anterior).

## Leitura em ordem

1. `specs/001-fundacao-overlay/spec.md` → US-002 (AC-004–AC-007) e limites sobre janelas elevadas.
2. `specs/001-fundacao-overlay/plan.md` → "Foco", OT-005, riscos.
3. `apps/desktop/src-tauri/src/overlay/window.rs` → `show()/hide()` (TK-002).
4. Docs atuais: `tauri-plugin-global-shortcut`; Win32 `RegisterHotKey`, `SetWindowsHookExW(WH_KEYBOARD_LL)`, `GetForegroundWindow`, `AttachThreadInput`, `SetForegroundWindow`, `SetWinEventHook(EVENT_OBJECT_SHOW)`.

## Decisões já resolvidas

- `DoubleTapDetector` é puro em `aura-core::gesture`; constantes `WINDOW_MS=400`, `COOLDOWN_MS=600`; qualquer tecla diferente de Ctrl entre os toques cancela.
- Hook LL instalado somente se o gesto estiver ativo; roda em thread dedicada com loop de mensagens; callback retorna rápido (envia evento por canal).
- `ForegroundTracker` guarda `PreviousApp { hwnd, pid, process_name, title, monitor_id }` no instante anterior ao `show()`.
- Conflito de atalho detectado pelo erro de registro (`ERROR_HOTKEY_ALREADY_REGISTERED`) → `SettingsError::ShortcutInUse`.
- Liberdade local: estrutura de threads, nomes internos.

## Mapa de alterações

- Novo: `crates/aura-core/src/gesture.rs` → `DoubleTapDetector`, `KeyEvent`, `Gesture::Toggle`.
- Novo: `apps/desktop/src-tauri/src/overlay/hotkey.rs` → `HotkeyService::{register(accelerator) -> Result<(), ShortcutError>, unregister}`.
- Novo: `apps/desktop/src-tauri/src/overlay/ll_hook.rs` → `LowLevelKeyboardHook::start(tx)`.
- Novo: `apps/desktop/src-tauri/src/overlay/focus.rs` → `ForegroundTracker::{capture_previous, restore_previous}`, `PreviousApp`.
- Novo: `tools/aura-bench/src/{main.rs,open_latency.rs,idle_memory.rs}`.
- Existente: `overlay/window.rs` → `show()` chama `capture_previous` antes; `hide()` chama `restore_previous`.
- Fora da fatia: UI de configurações.

## Contrato técnico

- Entradas: pressionamento do atalho; sequência de teclas Ctrl; botão "Minimizar para a bandeja"; `Esc` fecha menus/histórico/gravação e nunca esconde.
- Saídas: `overlay_toggle`; evento `previous_app_changed(PreviousApp)` publicado (OT-005).
- Invariantes: toggles não reentram (debounce de 150 ms para o atalho); o Aplicativo anterior nunca é o próprio Aura.
- Erros: `ShortcutError::InUse`, `ShortcutError::Invalid(accelerator)`; falha em restaurar foco (janela elevada/fechada) → apenas esconder.
- Efeitos: foco de teclado na barra de entrada ao mostrar.
- Concorrência: eventos do hook chegam por canal; processamento no thread principal do Tauri.

## Exemplos de aceite

- **AC-004**: app ocioso ≥ 1 min, Bloco de Notas em foco + `aura-bench open-latency --runs 50` → relatório com p95 ≤ 100 ms; efeito proibido: criação de nova janela WebView (contagem de processos `msedgewebview2` estável).
- **AC-005**: Bloco de Notas em foco → atalho → digitar "x" no Overlay → apagar → atalho (ou "Minimizar para a bandeja") → `GetForegroundWindow` é o Bloco de Notas e digitar "y" insere no Bloco de Notas; `Esc` com o Overlay focado → continua visível.
- **AC-006** (unit): `[Ctrl↓@0, Ctrl↑@60, Ctrl↓@200, Ctrl↑@260] → Toggle`; `[Ctrl↓@0, C↓@50, C↑@90, Ctrl↑@120, Ctrl↓@250, Ctrl↑@300] → nenhum`; `[Ctrl↓@0, Ctrl↑@50, Ctrl↓@500, Ctrl↑@560] → nenhum`; `Toggle@260` seguido de `[Ctrl↓@400, Ctrl↑@450, Ctrl↓@600, Ctrl↑@650] → nenhum` (cooldown até 860). Oráculo: números da spec.
- **AC-007**: processo auxiliar de teste registra `Ctrl+Alt+K` → `HotkeyService::register("Ctrl+Alt+K")` retorna `InUse` e o atalho anterior continua disparando.

## Dependências e sequência de execução

Depende de: TK-002.

- [ ] TK-003.1 Testes unitários de AC-006 (red) → `DoubleTapDetector` (green), um caso por vez.
- [ ] TK-003.2 Teste de integração Windows AC-007 (red) → `HotkeyService` (green).
- [ ] TK-003.3 E2E AC-005 (red) → `ForegroundTracker` + `Esc` (green).
- [ ] TK-003.4 `aura-bench open-latency` e `idle-memory`; executar na máquina de referência (AC-004) e anexar relatório.
- [ ] TK-003.5 Regressão completa e evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-core gesture`; `cargo nextest run -p aura-desktop --test hotkey_conflict` (Windows); `pnpm -C apps/desktop e2e -- --spec e2e/foco.spec.ts`; `cargo run -p aura-bench --release -- open-latency --runs 50 --out bench/open-latency.json` e `-- idle-memory --minutes 5 --out bench/idle-memory.json` na máquina de referência.
- Estado esperado: testes verdes; p95 ≤ 100 ms; memória registrada (compara com 150 MB, meta final no 010).
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner sem desktop interativo invalida medição de latência — registrar limitação.

## Condição de retorno à planejadora

Retornar se o Windows impedir o foco na barra de entrada ao usar o hook LL mesmo com `AttachThreadInput`, ou se o p95 medido ficar acima de 150 ms com a janela pré-criada (reavaliar H-001 e arquitetura de janela).

## Relatório de saída

Relatar símbolos, resultados dos testes, relatório do bench (p50/p95/máx, memória por processo), EV refs, limitações de ambiente e próxima ação.
