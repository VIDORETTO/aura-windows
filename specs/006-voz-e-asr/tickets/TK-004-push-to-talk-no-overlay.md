---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 006-voz-e-asr
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-003"]
requirement_refs: ["FR-005"]
acceptance_refs: ["AC-007", "AC-008", "AC-013"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/voice.rs", "apps/desktop/src/overlay/MicButton.tsx", "apps/desktop/src/overlay/ListeningIndicator.tsx", "crates/aura-asr/src/vad.rs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-004 — Push-to-talk no Overlay

## Objetivo e limites

Entrega a máquina de estados `PushToTalk` (Idle → Listening → Transcribing → Done | Cancelled | Empty), segurar `Ctrl+Space`/clique para alternar, VAD Silero para cortar silêncio e detectar "só silêncio", inserção no cursor, "Enviar ao terminar de falar", e o cartão de onboarding quando não há modelo.

Não inclui: atalho global (TK-005), parciais (TK-007), nuvem (TK-006).

## Leitura em ordem

1. `specs/006-voz-e-asr/spec.md` → US-003.
2. `crates/aura-audio/src/pipeline.rs` (005 TK-001) → `AudioHub::subscribe`.
3. `crates/aura-asr/src/transcriber.rs` (TK-003).
4. `apps/desktop/src/overlay/InputBar.tsx` (001/002) → inserção no cursor.
5. `docs/design/ui-ux.md` → mapa de teclado (segurar `Ctrl+Space`).

## Decisões já resolvidas

- Ao entrar em Listening, `WorkerClient.keep_warm = true` e `asr.load` em paralelo (esconde a latência de carga).
- VAD: descartar silêncio inicial/final; se a fala detectada < 300 ms → estado `Empty` com dica "Não ouvi nada".
- Limite de 5 min → transição automática para Transcribing.
- Durante Pausa de privacidade → botão desabilitado com "Pausa ativa".
- Liberdade local: animação do indicador (onda de nível).

## Mapa de alterações

- Existente: `apps/desktop/src-tauri/src/voice.rs` → `PushToTalk`, comandos `ptt_start`, `ptt_stop`, `ptt_cancel`, evento `ptt_state`.
- Novo: `crates/aura-asr/src/vad.rs` → `trim_silence(pcm) -> Option<Vec<f32>>`.
- Novo: `apps/desktop/src/overlay/{MicButton.tsx,ListeningIndicator.tsx}`, `settings/voice/NoModelCard.tsx`.

## Contrato técnico

- Entradas: pressionar/soltar; clique; `Esc`.
- Saídas: texto inserido; envio automático opcional.
- Invariantes: OT-003 (sem áudio em disco); uma sessão por vez.
- Erros: `AsrError` → toast com ação; microfone negado → link para configurações do Windows.

## Exemplos de aceite

- **AC-007** (unit da máquina de estados com `FakeTranscriber` e áudio sintético): press → Listening; release → Transcribing → Done("olá mundo") e texto inserido em "Pergunta: |" → "Pergunta: olá mundo|"; `Esc` em Listening → Cancelled sem inserir; áudio só com silêncio → Empty. Manual (Parakeet V3): 10 s de fala → texto ≤ 1,5 s após soltar (medir 10 vezes, p95).
- **AC-008**: opção ligada + Done → `conversation_send` chamado com o texto.
- **AC-013**: nenhum modelo e nenhuma nuvem → cartão "Baixar Parakeet V3 (456 MB)" (recomendação real do TK-001); após instalar, `ptt_start` funciona.

## Dependências e sequência de execução

Depende de: TK-003; 005-captura-de-audio/TK-001 (outro esforço).

- [ ] TK-004.1 Unit `trim_silence` red→green.
- [ ] TK-004.2 Unit máquina de estados (casos acima) red→green.
- [ ] TK-004.3 UI + Vitest; onboarding.
- [ ] TK-004.4 Manual com Parakeet V3 e medição; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-asr vad -p aura-desktop voice`; `pnpm -C apps/desktop test -- MicButton`; roteiro manual.
- Estado esperado: verdes; p95 ≤ 1,5 s.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: sem microfone.

## Condição de retorno à planejadora

Retornar se `Ctrl+Space` colidir com IMEs comuns de forma a exigir outro padrão.

## Relatório de saída

Relatar resultados, medições, EV refs.
