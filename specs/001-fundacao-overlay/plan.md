---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 001-fundacao-overlay
revision: 1
spec_revision: 2
status: ready
---

# Plan: Fundação do app residente e Overlay

## Summary

Criar o monorepo (workspace Cargo + app Tauri 2 com React), o host residente com bandeja e instância única, a janela do Overlay pré-criada e oculta com efeitos nativos do Windows, o detector de atalho (RegisterHotKey + gesto de duplo toque por hook de baixo nível), o cálculo de posicionamento por monitor, o store SQLite de configurações com cofre cifrado e o `aura-bench` para medir abertura e memória. Todo o comportamento testável sem Windows vive em módulos puros com adapters finos para Win32.

## Technical context

- Language/runtime: Rust stable (fixar em `rust-toolchain.toml` no TK-001), edition 2024; Node 22 LTS + pnpm; TypeScript 5 estrito.
- Dependencies (fixar versões no TK-001, consultar docs atuais via Context7): `tauri` 2.x (Handy usa 2.11.5), `tauri-plugin-single-instance`, `tauri-plugin-global-shortcut`, `tauri-plugin-autostart`, `tauri-plugin-log`, `tauri-specta`/`specta`, `window-vibrancy`, `windows` (Win32: `SetWindowDisplayAffinity`, `SetWindowsHookExW`, `MonitorFromWindow`, `GetForegroundWindow`), `rusqlite` (bundled), `rusqlite_migration`, `aes-gcm`, `rand`, `secrecy`, `tracing`, `tracing-appender`; UI: React 19, Vite, Tailwind 4, Zustand, Vitest, Testing Library.
- Storage/data: `%LOCALAPPDATA%\Aura\aura.db` (SQLite, WAL); logs em `%LOCALAPPDATA%\Aura\logs\`.
- Test command: `cargo nextest run --workspace`; `pnpm -C apps/desktop test`; `pnpm -C apps/desktop e2e` (Windows) — identificados, ainda não executados (repositório vazio).
- Target/platform: Windows 10 2004+ x64 / Windows 11. Crates puros também compilam em Linux para CI rápido.

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-007; AC-001–AC-017.

## Modules, interfaces, consumers, and seams

| Module (novo) | Interface | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `crates/aura-core::settings` | `Settings` (struct tipada com defaults), `SettingsPatch`, validação (`opacity ∈ [0.70, 1.00]`, atalho parseável) | host, UI via specta | Função pura `Settings::apply(patch) -> Result<Settings, SettingsError>` |
| `crates/aura-core::gesture` | `DoubleTapDetector::on_key(KeyEvent{key, down, at}) -> Option<Gesture>` | adapter de hook no host | Sequências sintéticas de eventos com timestamps (oráculo: 400/600 ms da spec) |
| `crates/aura-core::placement` | `place_overlay(monitors, target_monitor, saved: Option<SavedPlacement>, size) -> Rect` | host ao mostrar o Overlay | Exemplos literais de monitores/DPI |
| `crates/aura-store` | `Store::open(path, protector)`, `SettingsRepo::{load, save}`, `Vault::{seal, open}` | host; esforços 002+ | SQLite real em diretório temporário; `SecretProtector` com adapter DPAPI (Windows) e adapter de teste (chave fixa) |
| `crates/aura-store::protect` | `trait SecretProtector { protect(&[u8]) -> Vec<u8>; unprotect(&[u8]) -> Result<Vec<u8>> }` | `Vault` | Dois adapters reais: DPAPI (produção) e em memória (teste) — a seam existe porque DPAPI só existe no Windows |
| `apps/desktop/src-tauri` (`aura-desktop`) | Comandos `overlay_show/hide/toggle`, `settings_get/update`, eventos `settings_changed`, `overlay_state` | UI React | E2E `tauri-driver` + roteiros manuais |
| `apps/desktop/src/overlay` | Componentes `OverlayShell`, `InputBar`; store Zustand `useOverlayStore` | usuário | Vitest + `mockIPC` |
| `tools/aura-bench` | CLI `aura-bench open-latency --runs 50`, `aura-bench idle-memory --minutes 5` | CI Windows, 010 | Execução real no Windows |

Janela do Overlay: criada no `setup` com `visible: false`, `transparent: true`, `decorations: false`, `alwaysOnTop: true`, `skipTaskbar: true`, `shadow: true`; após criação aplicar `WS_EX_TOOLWINDOW` (fora do Alt+Tab), `window-vibrancy::apply_acrylic` (Win11; Win10 com fallback `apply_blur`), e `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)`.

Foco: `RegisterHotKey` entrega `WM_HOTKEY` ao processo, o que concede direito de primeiro plano; para o duplo toque (hook `WH_KEYBOARD_LL`), usar `AllowSetForegroundWindow`/técnica de `AttachThreadInput` com a thread do Aplicativo anterior. O `HWND` do Aplicativo anterior é capturado antes de mostrar e restaurado com `SetForegroundWindow` ao esconder.

## Chosen approach and alternatives

- **Janela pré-criada e oculta** (show/hide) em vez de criar sob demanda: criar WebView2 custa 200–600 ms. Alternativa descartada: criar a cada abertura.
- **`tauri-plugin-global-shortcut` para combinações** e **hook LL próprio só para o duplo toque**: o hook só é instalado se o gesto estiver ativo (custo zero caso contrário).
- **SQLite para configurações** em vez de JSON (tauri-plugin-store): o mesmo banco servirá histórico, auditoria e índices; migrações versionadas desde o início.
- **Cofre com DPAPI + AES-GCM**: chave de dados aleatória de 256 bits, guardada protegida por DPAPI (escopo usuário) na tabela `vault_keys`; valores cifrados com nonce aleatório por valor. Alternativa descartada: DPAPI direto em cada valor (mais lento e sem rotação de chave).
- **Segredos com `secrecy::SecretString`** e camada `tracing` que substitui campos marcados por `[REDACTED]`.

## Data, compatibility, and external dependencies

Migração inicial `0001_init.sql`: tabelas `settings(key TEXT PRIMARY KEY, value_json TEXT, updated_at)`, `vault_keys(id, protected_key BLOB, created_at)`, `secrets(id TEXT PRIMARY KEY, nonce BLOB, ciphertext BLOB, key_id)`, `overlay_placements(monitor_id TEXT PRIMARY KEY, x, y, w, h, dpi)`. `monitor_id` = nome do dispositivo + resolução; mudança de DPI escala o retângulo salvo.

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001 | E2E Windows + manual | Spec | `tauri-driver`: janela `overlay` existe e `isVisible=false`; roteiro manual confere menu da bandeja |
| AC-002 | E2E Windows | Spec | Iniciar segundo processo; contar processos `aura.exe` = 1 após 2 s; Overlay visível |
| AC-003 | Teste Rust de integração (supervisor de filhos) + manual | Spec | `ChildRegistry` encerra filhos registrados; manual: Sair e `Get-Process` sem `aura*` após 3 s |
| AC-004 | `aura-bench open-latency` | 100 ms p95 da spec | 50 invocações via `SendInput` do atalho; timestamp de `EVENT_OBJECT_SHOW` + primeiro frame (`DwmFlush`) |
| AC-005 | E2E Windows | Spec | Foco no Bloco de Notas → abrir → Esc → `GetForegroundWindow` = Bloco de Notas |
| AC-006 | Unit `DoubleTapDetector` | Literais 400/600 ms | Sequências: (Ctrl↓↑, 150 ms, Ctrl↓↑) → toggle; (Ctrl↓, C↓↑, Ctrl↑ …) → nada; (Ctrl↓↑, 450 ms, Ctrl↓↑) → nada; ativação + toque em 300 ms → ignorado |
| AC-007 | Teste de integração Windows | Spec | Registrar atalho com outro processo de teste, tentar salvar → erro `ShortcutInUse`, atalho antigo ainda ativo |
| AC-008 | Manual com prints (Win11 e Win10) | Spec | Roteiro no TK-002 |
| AC-009 | Manual + teste automatizado WGC | Spec | Captura WGC do monitor com Overlay visível; pixel da área do Overlay ≠ cor de marcação do Overlay (Overlay pintado com cor-teste) |
| AC-010 | Vitest (UI aplica CSS var) + manual | Spec | Patch `opacity=0.8` → `--overlay-alpha: 0.8`; reinício mantém |
| AC-011 | Vitest + manual | Spec (200 ms) | Estado `expanded` após `Ctrl+↓`; `prefers-reduced-motion` → `transition: none` |
| AC-012 | Unit `place_overlay` | Exemplos literais | Salvo no monitor presente → mesmo rect; monitor ausente → centro do terço superior do alvo; rect sempre dentro da área de trabalho |
| AC-013 | E2E Windows | Spec | Clicar em outra janela → Overlay invisível; com opção → visível |
| AC-014 | Integração `SettingsRepo` (SQLite real) + E2E | Spec | save → reopen → valores iguais |
| AC-015 | Integração Windows | Spec | Ativar → chave `HKCU\...\Run\Aura` existe com `--background`; desativar → removida |
| AC-016 | Integração `Vault` + teste Windows DPAPI | Spec | Bytes do arquivo não contêm o literal `sk-test-SEGREDO-123`; reabrir → lê; `CryptUnprotectData` com outro usuário (runner com usuário secundário) falha — se o runner não permitir, registrar limitação e executar manualmente |
| AC-017 | Unit da camada de redação + integração do rotator | Spec | Evento com campo secreto → linha contém `[REDACTED]`; escrever 55 MB → 5 arquivos ≤ 10 MB |

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `Cargo.toml`, `rust-toolchain.toml`, `.github/workflows/ci.yml` | new | workspace, CI Linux+Windows | Base do monorepo | 2026-09-29 |
| `apps/desktop/package.json`, `vite.config.ts`, `src/main.tsx` | new | app React | UI | 2026-09-29 |
| `apps/desktop/src-tauri/src/{main.rs,lib.rs,tray.rs,children.rs}` | new | `run`, `build_tray`, `ChildRegistry` | Host residente | 2026-09-29 |
| `apps/desktop/src-tauri/src/overlay/{window.rs,focus.rs,hotkey.rs,ll_hook.rs}` | new | `OverlayWindow`, `ForegroundTracker`, `HotkeyService` | Janela e atalho | 2026-09-29 |
| `crates/aura-core/src/{settings.rs,gesture.rs,placement.rs}` | new | `Settings`, `DoubleTapDetector`, `place_overlay` | Regras puras | 2026-09-29 |
| `crates/aura-store/src/{lib.rs,migrations/,settings_repo.rs,vault.rs,protect.rs}` | new | `Store`, `SettingsRepo`, `Vault`, `SecretProtector`, `DpapiProtector` | Persistência | 2026-09-29 |
| `crates/aura-core/src/logging.rs` | new | `init_logging`, `RedactionLayer` | Logs | 2026-09-29 |
| `apps/desktop/src/overlay/`, `apps/desktop/src/settings/`, `apps/desktop/src/design/tokens.css` | new | `OverlayShell`, `InputBar`, `SettingsWindow` | UI | 2026-09-29 |
| `tools/aura-bench/` | new | `open-latency`, `idle-memory` | Medição | 2026-09-29 |

## Derived technical obligations

- **OT-001** → FR-007/AC-004: a janela do Overlay MUST ser criada na inicialização e nunca destruída enquanto o app roda.
- **OT-002** → FR-003/AC-009: toda janela do Aura que possa ficar visível durante captura (Overlay, seletor de região, Minibar) MUST aplicar `WDA_EXCLUDEFROMCAPTURE` via helper único `exclude_from_capture(hwnd)`.
- **OT-003** → FR-006: nenhum tipo que carregue segredo implementa `Debug`/`Display` sem redação (`SecretString`).
- **OT-004** → FR-001/AC-003: todo processo filho criado por qualquer esforço MUST ser registrado no `ChildRegistry` (Job Object do Windows com `KILL_ON_JOB_CLOSE`).
- **OT-005** → FR-002: o `ForegroundTracker` MUST publicar o Aplicativo anterior (HWND, PID, nome do processo, título, monitor) para os esforços 004/009.

## Risks and gates

- Risco: roubo de foco bloqueado pelo Windows com o hook LL. Mitigação: técnicas de `AttachThreadInput`; fallback documentado (Overlay visível sem foco + dica "clique para digitar") só se tudo falhar — exige retorno à planejadora.
- Risco: Acrylic com `transparent: true` causando artefatos no Windows 10. Mitigação: fallback sólido translúcido.
- Risco: runners do GitHub não têm sessão interativa para medir abertura com fidelidade. Mitigação: `aura-bench` roda também na máquina de referência; CI só vigia regressão grosseira.
- G2: satisfeito (sem decisões pendentes). G3: TK-001 sem blockers; demais tickets seguem o grafo.
