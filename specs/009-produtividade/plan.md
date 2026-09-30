---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 009-produtividade
revision: 1
spec_revision: 1
status: ready
---

# Plan: Produtividade no desktop

## Summary

Módulos pequenos no host, cada um com uma seam de SO isolada: `selection` (UIA + fallback de clipboard), `clipboard_guard` (salvar/restaurar todos os formatos), `inject` (colar/digitar no Aplicativo anterior), `minibar` (janela extra pré-criada), `notify` (toasts nativos), `tts` (WinRT SpeechSynthesis + `rodio`), e `profiles` (regras puras de casamento por processo). A UI ganha Minibar, botões de TTS e "Inserir no app", e o editor de perfis.

## Technical context

- Language/runtime: Rust + React.
- Dependencies: crate `windows` (UI Automation `IUIAutomationTextPattern`, `SendInput`, clipboard Win32 `OpenClipboard/EnumClipboardFormats/GetClipboardData`, `Windows.Media.SpeechSynthesis`), `tauri-plugin-notification` (ou WinRT `ToastNotificationManager` direto para ações), `rodio` (reprodução).
- Storage/data: tabela `app_profiles(id, process_pattern, title_glob?, instructions, attach_screen, default_mode, default_model?)`; `settings.tts.*`.
- Test command: `cargo nextest run -p aura-desktop productivity`; integração Windows `--features win-integration`; `pnpm -C apps/desktop test -- productivity`.
- Target/platform: Windows.

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-005; AC-001–AC-010.

## Modules, interfaces, consumers, and seams

| Module | Interface | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `selection` | `trait SelectionReader { fn read(&PreviousApp) -> Option<String> }`; `UiaSelection`, `ClipboardSelection` (fallback), `ChainedSelection` | Overlay show | App de teste Win32/WinForms com seleção conhecida; `FakeSelection` |
| `clipboard_guard` | `ClipboardSnapshot::take() -> Snapshot; restore(snapshot)` | selection, inject | Integração Windows com formatos texto+HTML+imagem |
| `inject` | `fn insert(previous: &PreviousApp, text) -> InsertOutcome{Pasted, Typed, CopiedOnly(reason)}` | UI | App de teste com campo de texto |
| `minibar` | janela Tauri pré-criada; `show_minibar(status, preview)`, `restore_overlay()` | Overlay | E2E |
| `notify` | `notify(kind: TurnDone|ApprovalNeeded, conversation)` | host | Integração manual |
| `tts` | `trait Speaker { speak(text, voice), stop() }`; `WindowsSpeaker`, `CloudSpeaker` (BYOK); `fn speakable(markdown) -> String` (puro) | UI | Unit `speakable`; manual de áudio |
| `profiles` | `fn match_profile(&[AppProfile], &PreviousApp) -> Option<&AppProfile>` (puro) | `start` da Conversa | Casos literais |

## Chosen approach and alternatives

- **UIA primeiro** para seleção (sem tocar no clipboard); fallback de clipboard só quando necessário e sempre com restauração completa.
- **Minibar como janela separada pré-criada** (não redimensionar o Overlay) para transição instantânea e sem artefatos de Acrylic.
- **Vozes do Windows offline** como padrão (privacidade); nuvem opcional com consentimento.
- Alternativa descartada: Edge TTS padrão (ThukiWin) — envia texto a serviço externo.

## Data, compatibility, and external dependencies

Migração `0008_productivity.sql` (`app_profiles`). O `PreviousApp` do 001 já traz processo/título.

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001, AC-002 | Integração Windows (app de teste) + matriz manual | Texto conhecido | Seleção "Aura seleção 42" → Chip com o texto; clipboard antes = "original" e depois = "original" (formatos texto e HTML); sem seleção → nada; app excluído → nada |
| AC-003 | E2E | Spec | Turno em andamento (falso) + foco em outra janela → Minibar visível; clique → Overlay |
| AC-004 | Manual + unit de disparo | Spec | Evento `TurnCompleted` com Overlay oculto → toast; clique abre Conversa |
| AC-005, AC-006 | Unit `speakable` + manual | Casos literais | Markdown com código e URL → texto sem código, URL reduzida ao domínio |
| AC-007, AC-008 | Unit `match_profile` + contrato (payload) | Casos literais | `code.exe` → perfil; `chrome.exe` → nenhum; instruções no `developerInstructions` |
| AC-009, AC-010 | Integração Windows (app de teste) | Spec | Texto inserido no campo; clipboard restaurado; app fechado → `CopiedOnly(AppGone)` |

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `apps/desktop/src-tauri/src/productivity/{selection.rs,clipboard_guard.rs,inject.rs,minibar.rs,notify.rs,tts.rs,profiles.rs}` | new | ver tabela | Host | 2026-09-29 |
| `apps/desktop/src/{minibar/**,conversation/SpeakButton.tsx,conversation/InsertButton.tsx,settings/profiles/**}` | new | UI | Interface | 2026-09-29 |
| `crates/aura-store/src/migrations/0008_productivity.sql` | new | `app_profiles` | Dados | 2026-09-29 |
| `tools/test-apps/selection-target/` | new | app Win32 de teste | Testes | 2026-09-29 |

## Derived technical obligations

- **OT-001** → FR-001/FR-005: toda operação que usa a área de transferência usa `ClipboardSnapshot` e restaura em `finally`, inclusive em erro.
- **OT-002** → FR-001: seleção passa por `aura-policy::decide` (Fonte "Seleção" tratada como Tela para exclusões).
- **OT-003** → FR-002: Minibar aplica `exclude_from_capture` (001 OT-002).

## Risks and gates

- Apps que usam clipboard "delay rendering" podem perder formatos na restauração — restaurar ao menos texto/HTML/imagem e registrar limitação.
- G2: satisfeito. G3: dependências cruzadas listadas em cada ticket.
