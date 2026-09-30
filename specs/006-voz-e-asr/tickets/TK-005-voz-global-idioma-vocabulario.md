---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 006-voz-e-asr
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-004"]
requirement_refs: ["FR-005", "FR-007"]
acceptance_refs: ["AC-009", "AC-011"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-asr/src/vocabulary.rs", "apps/desktop/src/settings/voice/LanguageAndVocabulary.tsx", "apps/desktop/src-tauri/src/voice_hotkey.rs"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-005 — Atalho global de voz, idioma e vocabulário personalizado

## Objetivo e limites

Entrega o atalho global "segurar para falar" (padrão `Ctrl+Alt+Space`) que abre o Overlay escutando, a escolha de idioma (automático/fixo), o vocabulário personalizado (prompt para Whisper + substituições pós-transcrição para todos os modelos).

Não inclui: ditado em outros apps (CAND-011).

## Leitura em ordem

1. `specs/006-voz-e-asr/spec.md` → AC-009, AC-011.
2. `apps/desktop/src-tauri/src/overlay/{hotkey.rs,ll_hook.rs}` (001 TK-003) → detectar key-up para "segurar".
3. `apps/desktop/src-tauri/src/voice.rs` (TK-004).

## Decisões já resolvidas

- "Segurar" exige saber quando a tecla é solta: usar o hook LL (instalado só se o atalho de voz estiver ativo) para detectar key-down/up da combinação.
- Idioma fixo passa `language` ao modelo quando suportado; modelos sem seleção de idioma ignoram e a UI indica.
- Vocabulário: até 200 termos; `apply` substitui ocorrências case-insensitive por fronteira de palavra pela grafia cadastrada; Whisper recebe `initial_prompt` com os termos.
- Liberdade local: UI de edição de termos.

## Mapa de alterações

- Novo: `crates/aura-asr/src/vocabulary.rs` → `apply`, `prompt_for_whisper`.
- Novo: `apps/desktop/src-tauri/src/voice_hotkey.rs`.
- Novo: `apps/desktop/src/settings/voice/LanguageAndVocabulary.tsx`.

## Contrato técnico

- Entradas: atalho segurado; configurações de idioma e termos.
- Saídas: Overlay em Listening; transcrição ajustada.
- Invariantes: substituição não altera partes de palavras ("aurora" não vira "Auraora").

## Exemplos de aceite

- **AC-009**: integração Windows: `SendInput` segura `Ctrl+Alt+Space` 2 s com `FakeTranscriber` → Overlay visível em Listening; soltar → texto na barra de entrada.
- **AC-011** (unit): termos `["Aura","Codex","Parakeet"]`, entrada "a aura usa o codex e o parakeet, não aurora" → "a Aura usa o Codex e o Parakeet, não aurora"; `prompt_for_whisper` → "Aura, Codex, Parakeet"; idioma fixo `pt` → `AsrOptions.language = pt` enviado ao worker.

## Dependências e sequência de execução

Depende de: TK-004.

- [ ] TK-005.1 Unit `vocabulary` red→green.
- [ ] TK-005.2 Idioma no `AsrOptions` (contrato worker).
- [ ] TK-005.3 Atalho global de voz (integração Windows).
- [ ] TK-005.4 UI; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-asr vocabulary`; `cargo nextest run -p aura-desktop --test voice_hotkey` (Windows); `pnpm -C apps/desktop test -- LanguageAndVocabulary`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner sem desktop interativo para o hook.

## Condição de retorno à planejadora

Retornar se o hook LL simultâneo (duplo toque + voz) causar latência perceptível de digitação.

## Relatório de saída

Relatar resultados e EV refs.
