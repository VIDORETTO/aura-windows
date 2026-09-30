---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 007-anexos-multimodais
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-004"]
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-008"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-ingest/src/video.rs", "crates/aura-worker/src/ingest_video.rs", "crates/aura-ingest/tests/corpus/video"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-005 — Vídeo como anexo (keyframes + Transcrição)

## Objetivo e limites

Entrega `ingest.video` no worker: demux/decode com Media Foundation, até 12 keyframes distribuídos (com detecção simples de mudança de cena para evitar quadros quase iguais), extração da trilha de áudio e transcrição (reuso do TK-004), intercalação por tempo.

Não inclui: legendas embutidas (melhoria futura), vídeos com DRM.

## Leitura em ordem

1. `crates/aura-worker/src/{media.rs,ingest_audio.rs}` (004 TK-007, TK-004).
2. `crates/aura-capture/src/clips.rs` → `sample_indices` (004 TK-007).

## Decisões já resolvidas

- Amostragem: `sample_indices(duração_em_s, 12)` sobre segundos; se dois keyframes consecutivos tiverem diferença média de pixels < 2%, substituir pelo próximo candidato distinto.
- Keyframes ≤ 1280 px; legenda `[mm:ss]`.
- Saída intercalada: `[00:30] (quadro)` seguido das falas até o próximo quadro.
- Liberdade local: limiar exato de mudança de cena.

## Mapa de alterações

- Novo: `crates/aura-worker/src/ingest_video.rs`, `crates/aura-ingest/src/video.rs`.
- Novo: fixture gerada `tests/corpus/video/numerado-60s.mp4` (quadros com número do segundo + narração conhecida) por script.

## Contrato técnico

- Entradas: arquivo de vídeo.
- Saídas: `IngestedDoc` com blocos Imagem+Texto ordenados por tempo.
- Erros: sem trilha de áudio → só keyframes; codec não suportado → `Unsupported`.

## Exemplos de aceite

- **AC-008**: `numerado-60s.mp4` → 12 imagens cujos números desenhados correspondem a `sample_indices(60,12)` = `[0,5,11,16,21,27,32,38,43,48,54,59]` (± cena), cada uma com legenda de tempo, e texto da narração intercalado.

## Dependências e sequência de execução

Depende de: TK-004; 004-contexto-de-tela/TK-007 (outro esforço; `media.keyframes`).

- [ ] TK-005.1 Keyframes por tempo (red→green) com a fixture.
- [ ] TK-005.2 Mudança de cena.
- [ ] TK-005.3 Intercalação com transcrição; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-worker --features win-integration ingest_video`; `cargo nextest run -p aura-ingest video`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: codecs ausentes (Windows N).

## Condição de retorno à planejadora

Retornar se MF não decodificar mkv/webm comuns, exigindo FFmpeg opcional (decisão de distribuição).

## Relatório de saída

Relatar formatos cobertos, tempos, EV refs.
