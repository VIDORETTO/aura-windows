---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 001-fundacao-overlay
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-002"]
requirement_refs: ["FR-005"]
acceptance_refs: ["AC-010", "AC-014", "AC-015"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-core/src/settings.rs", "crates/aura-store", "apps/desktop/src/settings", "apps/desktop/src-tauri/src/settings.rs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-004 — Configurações persistentes e janela de Configurações

## Objetivo e limites

Entrega o tipo `Settings` com validação, o store SQLite com migrações e `SettingsRepo`, os comandos `settings_get`/`settings_update` com evento `settings_changed`, a janela de Configurações (seções Geral e Aparência) e a aplicação imediata de tema, opacidade, atalho, duplo toque, "manter aberto" e "Iniciar com o Windows".

Não inclui: cofre de segredos (TK-006), seções de provedores/captura/voz (esforços seguintes adicionam seções).

## Leitura em ordem

1. `specs/001-fundacao-overlay/plan.md` → tabela de Modules (`aura-core::settings`, `aura-store`) e "Data".
2. `docs/design/ui-ux.md` → tokens e opacidade.
3. `apps/desktop/src-tauri/src/overlay/hotkey.rs` → `HotkeyService::register` (TK-003, se já concluído; caso contrário apenas o contrato do plano).
4. Docs atuais: `rusqlite`, `rusqlite_migration`, `tauri-plugin-autostart`, `tauri-specta`.

## Decisões já resolvidas

- `Settings` serializado por chave em `settings(key, value_json)`; defaults definidos no código; leitura tolera chaves desconhecidas (compatibilidade futura).
- Janela de Configurações criada sob demanda (não pré-aquecida) e destruída ao fechar, para não ocupar memória.
- Atalho só é persistido após `HotkeyService::register` ter sucesso; em falha, o valor anterior permanece (AC-007 já coberto no TK-003).
- Autostart por `tauri-plugin-autostart` (chave `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`) com argumento `--background`.
- Liberdade local: layout interno das seções, componentes de formulário.

## Mapa de alterações

- Novo: `crates/aura-core/src/settings.rs` → `Settings`, `SettingsPatch`, `Theme`, `FocusLossBehavior`, `Settings::apply`.
- Novo: `crates/aura-store/src/{lib.rs,migrations/0001_init.sql,settings_repo.rs}` → `Store::open`, `SettingsRepo::{load, save}`.
- Novo: `apps/desktop/src-tauri/src/settings.rs` → comandos `settings_get`, `settings_update`, evento `settings_changed`, `open_settings_window`.
- Novo: `apps/desktop/src/settings/{SettingsWindow.tsx,GeneralSection.tsx,AppearanceSection.tsx}`; `apps/desktop/src/state/settingsStore.ts`.
- Existente: `apps/desktop/src/overlay/OverlayShell.tsx` → aplica `--overlay-alpha` e tema a partir do store.
- Fora da fatia: segredos, provedores.

## Contrato técnico

- Entradas: `SettingsPatch` parcial vindo da UI.
- Saídas: `Settings` completo validado; evento `settings_changed(Settings)`.
- Invariantes: `opacity ∈ [0.70, 1.00]` (arredondado a 0.01); `theme ∈ {System, Light, Dark}`; atalho parseável pelo formato do plugin.
- Erros: `SettingsError::OutOfRange{field}`, `SettingsError::InvalidShortcut`, `SettingsError::ShortcutInUse`, `SettingsError::Storage`.
- Efeitos: persistência atômica (transação); aplicação imediata no Overlay e no serviço de atalho; autostart registrado/removido.
- Compatibilidade: chaves desconhecidas no banco são preservadas.

## Exemplos de aceite

- **AC-010**: `settings_update({opacity: 0.8})` → Overlay com `--overlay-alpha: 0.8` imediatamente (Vitest com mockIPC verifica a CSS var); reiniciar app → continua 0.8. `settings_update({opacity: 0.5})` → erro `OutOfRange{opacity}` e valor anterior mantido. Tema Escuro → classe `theme-dark` no root.
- **AC-014**: salvar `{keep_open_on_blur: true, theme: Dark}` → `Store::open` novamente no mesmo arquivo → `SettingsRepo::load` retorna esses valores (teste de integração com SQLite real em tempdir); E2E: abrir Configurações por `Ctrl+,` e pela bandeja.
- **AC-015**: `settings_update({start_with_windows: true})` → chave Run `Aura` com `"<exe>" --background`; `false` → chave ausente (teste Windows); roteiro manual: logoff/logon mostra só a bandeja.

## Dependências e sequência de execução

Depende de: TK-002.

- [ ] TK-004.1 Unit `Settings::apply` para limites (red→green por caso).
- [ ] TK-004.2 Integração `SettingsRepo` com SQLite real (AC-014) red→green.
- [ ] TK-004.3 Comandos specta + UI; Vitest AC-010 red→green.
- [ ] TK-004.4 Autostart (AC-015) teste Windows + roteiro manual.
- [ ] TK-004.5 Regressão e evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-core settings -p aura-store`; `pnpm -C apps/desktop test -- settings`; `cargo nextest run -p aura-desktop --test autostart` (Windows); roteiro manual AC-015.
- Estado esperado: todos verdes; roteiro com prints.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: política corporativa que bloqueia chave Run é ambiente; registrar.

## Condição de retorno à planejadora

Retornar se o formato de atalho do plugin não suportar o gesto/teclas previstas no design ou se o autostart exigir privilégios de administrador.

## Relatório de saída

Relatar esquema criado, chaves de configuração, resultados, EV refs e limitações.
