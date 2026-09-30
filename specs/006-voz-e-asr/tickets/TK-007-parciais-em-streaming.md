---
schema: hybrid/ticket
schema_version: 1.0
id: TK-007
effort: 006-voz-e-asr
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-004"]
requirement_refs: ["FR-008"]
acceptance_refs: ["AC-012"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-worker/src/asr_stream.rs", "crates/aura-asr/src/streaming.rs", "apps/desktop/src/overlay/PartialTranscript.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-007 — Parciais em streaming

## Objetivo e limites

Entrega `asr.stream.start/push/finish` no worker para modelos com `streaming = true` (Moonshine V2 streaming) e janela deslizante com reprocessamento para modelos sem streaming nativo (Parakeet, a cada 700 ms sobre os últimos 8 s), exibindo o parcial em cinza no input e substituindo pelo final.

Não inclui: streaming em nuvem.

## Leitura em ordem

1. `crates/aura-worker/src/asr.rs` e `crates/aura-asr/src/transcriber.rs` (TK-003) → `Transcriber::stream`.
2. `apps/desktop/src-tauri/src/voice.rs` (TK-004).
3. Docs atuais: `transcribe-rs` Moonshine streaming.

## Decisões já resolvidas

- Parcial é só visual (não entra no texto até o final).
- Pseudo-streaming (janela deslizante) só é ativado se o hardware tiver ≥ 6 núcleos ou GPU (senão desliga para não competir com a fala final).
- Liberdade local: estilo do parcial.

## Mapa de alterações

- Novo: `crates/aura-worker/src/asr_stream.rs`, `crates/aura-asr/src/streaming.rs` (`StreamingSession`).
- Novo: `apps/desktop/src/overlay/PartialTranscript.tsx`.

## Contrato técnico

- Entradas: chunks PCM de 20 ms durante Listening.
- Saídas: eventos `Partial{text}` e `Final{text}`.
- Invariantes: final é sempre o resultado da transcrição completa (não a concatenação de parciais).

## Exemplos de aceite

- **AC-012**: `real-models` com Moonshine Tiny streaming alimentado em tempo real por `jfk.wav` → primeiro `Partial` ≤ 500 ms após o início da fala e atraso médio parcial ≤ 500 ms; `Final` igual ao `transcribe` completo do mesmo áudio.

## Dependências e sequência de execução

Depende de: TK-004.

- [ ] TK-007.1 Streaming Moonshine no worker (red→green).
- [ ] TK-007.2 Pseudo-streaming com janela (medir custo).
- [ ] TK-007.3 UI; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-asr --features real-models streaming`; `pnpm -C apps/desktop test -- PartialTranscript`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: CI lento distorce latência — medir na máquina de referência.

## Condição de retorno à planejadora

Retornar se o pseudo-streaming degradar a latência do final além de 1,5 s (AC-007).

## Relatório de saída

Relatar latências e EV refs.
