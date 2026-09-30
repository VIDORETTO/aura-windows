---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 001-fundacao-overlay
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-008", "AC-009"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/overlay", "apps/desktop/src/overlay", "apps/desktop/src/design"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-002 — Janela do Overlay translúcida, sempre no topo e invisível em capturas

## Objetivo e limites

Entrega a janela `overlay` pré-criada e oculta, com Acrylic (Win11) / blur ou sólido (Win10), cantos arredondados, sombra, sempre no topo, fora da taskbar e do Alt+Tab, excluída de capturas, e o `OverlayShell` React com a barra de entrada visual (sem envio). Mostrar/esconder por comando de debug e pelo item "Abrir" da bandeja.

Não inclui: atalho global (TK-003), foco no Aplicativo anterior (TK-003), estados compacto/expandido e posição por monitor (TK-005), opacidade configurável (TK-004).

## Leitura em ordem

1. `specs/001-fundacao-overlay/plan.md` → "Janela do Overlay" e OT-001/OT-002.
2. `docs/design/ui-ux.md` → tokens, layout "Compacto", acessibilidade.
3. `apps/desktop/src-tauri/src/lib.rs` → `run()` (criado no TK-001) para registrar a janela no `setup`.
4. Docs atuais: `window-vibrancy` (apply_acrylic/apply_blur/apply_mica), Win32 `SetWindowDisplayAffinity`, `WS_EX_TOOLWINDOW`.

## Decisões já resolvidas

- Janela criada no `setup` com `visible:false, transparent:true, decorations:false, alwaysOnTop:true, skipTaskbar:true, resizable:true, shadow:true`, tamanho inicial 640×64 lógico.
- Helper único `overlay::window::exclude_from_capture(hwnd)` (OT-002) reutilizado por outras janelas futuras.
- Detecção de SO: build ≥ 22000 → Acrylic; senão blur; se falhar → sólido `--surface` com alfa.
- Liberdade local: estrutura dos componentes React, nomes de classes.
- Alternativa descartada: Mica (não é translúcida sobre o conteúdo atrás, apenas sobre o wallpaper).

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/overlay/window.rs` → `create_overlay_window(app)`, `show()`, `hide()`, `exclude_from_capture(hwnd)`, `apply_backdrop(window, os_build)`.
- Novo: `apps/desktop/src-tauri/src/overlay/mod.rs` → comandos `overlay_show`, `overlay_hide`, `overlay_toggle` (specta).
- Novo: `apps/desktop/src/overlay/OverlayShell.tsx`, `InputBar.tsx`; `apps/desktop/src/design/tokens.css` com tokens de `docs/design/ui-ux.md`.
- Existente: `apps/desktop/src-tauri/src/tray.rs` → item "Abrir" chama `overlay_show`.
- Fora da fatia: hotkeys, settings.

## Contrato técnico

- Entradas: comandos `overlay_show/hide/toggle`; evento `AppEvent::OpenOverlay`.
- Saídas: janela visível/oculta; evento `overlay_state { visible: bool }` para a UI.
- Invariantes: a janela nunca é destruída enquanto o app roda (OT-001); `exclude_from_capture` aplicado antes da primeira exibição.
- Erros: falha em `SetWindowDisplayAffinity` → log `warn` e marca `capture_exclusion=false` exposta em diagnóstico (não bloqueia exibição).
- Efeitos: nenhum ícone na taskbar/Alt+Tab.
- Compatibilidade: Windows 10 2004+ e 11.

## Exemplos de aceite

- **AC-008**: Win11, wallpaper colorido + `overlay_show` → print mostra fundo translúcido desfocado, cantos arredondados, sombra; `Alt+Tab` não lista "Aura"; taskbar sem ícone; com um vídeo em tela cheia sem bordas no navegador, o Overlay aparece por cima. Win10 22H2 → fundo desfocado ou sólido translúcido com os mesmos comportamentos.
- **AC-009**: Overlay visível pintado com cor de teste `#FF00FF` (modo debug `AURA_DEBUG_OVERLAY_COLOR`) + captura WGC do monitor por um teste automatizado → nenhum pixel `#FF00FF` na imagem; roteiro manual: Ferramenta de Captura, compartilhamento de tela no Teams e OBS não mostram o Overlay. Efeito proibido: Overlay visível em qualquer captura.

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-002.1 Teste automatizado Windows `overlay_is_absent_from_wgc_capture` (red: sem affinity o magenta aparece).
- [ ] TK-002.2 Criar janela oculta com affinity → green.
- [ ] TK-002.3 Backdrop Acrylic/blur/sólido e estilo `WS_EX_TOOLWINDOW`; roteiro manual AC-008 em Win11 e Win10.
- [ ] TK-002.4 `OverlayShell`/`InputBar` com tokens (teste Vitest de render com rótulo acessível "Pergunte algo").
- [ ] TK-002.5 Registrar evidências (prints + saída do teste).

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-desktop --test overlay_capture` (Windows, sessão interativa); `pnpm -C apps/desktop test -- overlay`; roteiro manual AC-008/AC-009 (Win11 + Win10 22H2).
- Estado esperado: teste de captura verde; prints conforme exemplos.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner sem sessão de desktop interativa não captura tela — registrar `not_run` com limitação e executar na máquina de referência.

## Condição de retorno à planejadora

Retornar se `transparent:true` + Acrylic for inviável no Windows 10 (decidir aparência alternativa), ou se `WDA_EXCLUDEFROMCAPTURE` não ocultar a janela para alguma ferramenta listada.

## Relatório de saída

Relatar símbolos criados, prints por SO, resultado do teste de captura com EV refs, ferramentas de captura testadas e limitações.
