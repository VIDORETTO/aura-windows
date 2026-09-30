# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [ ] TK-001 — TK-001 — Esqueleto do monorepo e app residente na bandeja
Status: `implemented` | Bloqueado por: nenhum

- [ ] TK-001.1 Criar workspace, toolchain e app Tauri mínimo; confirmar `pnpm tauri dev` abre sem janela visível (red: E2E AC-001 falha antes da config `visible:false`).
- [ ] TK-001.2 Escrever E2E de AC-001 e fazê-lo passar.
- [ ] TK-001.3 Teste de integração Rust `child_registry_kills_adopted_children_on_shutdown` (red) → implementar Job Object (green).
- [ ] TK-001.4 Bandeja + `Quit` → E2E/roteiro AC-003.
- [ ] TK-001.5 Single instance → E2E AC-002.
- [ ] TK-001.6 CI Linux + Windows verde; registrar evidência.

## [ ] TK-002 — TK-002 — Janela do Overlay translúcida, sempre no topo e invisível em capturas
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-002.1 Teste automatizado Windows `overlay_is_absent_from_wgc_capture` (red: sem affinity o magenta aparece).
- [ ] TK-002.2 Criar janela oculta com affinity → green.
- [ ] TK-002.3 Backdrop Acrylic/blur/sólido e estilo `WS_EX_TOOLWINDOW`; roteiro manual AC-008 em Win11 e Win10.
- [ ] TK-002.4 `OverlayShell`/`InputBar` com tokens (teste Vitest de render com rótulo acessível "Pergunte algo").
- [ ] TK-002.5 Registrar evidências (prints + saída do teste).

## [ ] TK-003 — TK-003 — Atalho de invocação, duplo toque, foco e medição de abertura
Status: `implemented` | Bloqueado por: TK-002

- [ ] TK-003.1 Testes unitários de AC-006 (red) → `DoubleTapDetector` (green), um caso por vez.
- [ ] TK-003.2 Teste de integração Windows AC-007 (red) → `HotkeyService` (green).
- [ ] TK-003.3 E2E AC-005 (red) → `ForegroundTracker` + `Esc` (green).
- [ ] TK-003.4 `aura-bench open-latency` e `idle-memory`; executar na máquina de referência (AC-004) e anexar relatório.
- [ ] TK-003.5 Regressão completa e evidências.

## [ ] TK-004 — TK-004 — Configurações persistentes e janela de Configurações
Status: `implemented` | Bloqueado por: TK-002

- [ ] TK-004.1 Unit `Settings::apply` para limites (red→green por caso).
- [ ] TK-004.2 Integração `SettingsRepo` com SQLite real (AC-014) red→green.
- [ ] TK-004.3 Comandos specta + UI; Vitest AC-010 red→green.
- [ ] TK-004.4 Autostart (AC-015) teste Windows + roteiro manual.
- [ ] TK-004.5 Regressão e evidências.

## [ ] TK-005 — TK-005 — Estados compacto/expandido, posição por monitor e perda de foco
Status: `implemented` | Bloqueado por: TK-003, TK-004

- [ ] TK-005.1 Unit AC-012, um caso por vez (red→green).
- [ ] TK-005.2 `PlacementRepo` + aplicação no `show()`; manual com 2 monitores.
- [ ] TK-005.3 Vitest AC-011 → UI de estados.
- [ ] TK-005.4 E2E AC-013.
- [ ] TK-005.5 Regressão, evidências (incluindo vídeo curto de demonstração do Marco 1 parcial).

## [ ] TK-006 — TK-006 — Cofre local cifrado e logs com redação
Status: `implemented` | Bloqueado por: TK-004

- [ ] TK-006.1 Unit `Secret` Debug (red→green).
- [ ] TK-006.2 Integração `Vault` com `StaticKeyProtector` (AC-016 parte 1) red→green.
- [ ] TK-006.3 `DpapiProtector` + teste Windows (AC-016 parte 2 e 3).
- [ ] TK-006.4 `RedactionLayer` e `SizeRotatingWriter` (AC-017) red→green.
- [ ] TK-006.5 Regressão e evidências.
