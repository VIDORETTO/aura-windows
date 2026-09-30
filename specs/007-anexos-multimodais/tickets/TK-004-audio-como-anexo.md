---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 007-anexos-multimodais
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-007"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-ingest/src/audio.rs", "crates/aura-worker/src/ingest_audio.rs", "crates/aura-ingest/tests/corpus/audio"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-004 — Áudio como anexo (Transcrição com tempos)

## Objetivo e limites

Entrega `ingest.audio` no worker (decodificação com Media Foundation/`symphonia`, reamostragem 16 kHz, transcrição em blocos de 30 s com o modelo configurado) e o `AudioIngestor`, com progresso por porcentagem e suporte a `read{Time}`.

Não inclui: diarização.

## Leitura em ordem

1. `crates/aura-worker/src/asr.rs` (006 TK-003) → `asr.transcribe`.
2. `crates/aura-ingest/src/ingestor.rs` (TK-001).
3. Docs atuais: `symphonia` (mp3, flac, ogg/opus, wav), Media Foundation Source Reader (m4a/aac, webm).

## Decisões já resolvidas

- Transcrição em janelas de 30 s com 1 s de sobreposição e desduplicação de fronteira.
- Formato enviado: `[mm:ss] texto` por segmento.
- Sem modelo local e sem nuvem → `IngestError::NeedsAsr` e Chip com "Baixar modelo de voz".
- Limite: 2 h.
- Liberdade local: tamanho do bloco de progresso.

## Mapa de alterações

- Novo: `crates/aura-worker/src/ingest_audio.rs`, `crates/aura-ingest/src/audio.rs`.
- Novo: fixtures `tests/corpus/audio/{jfk.mp3,frase-pt.m4a,curto.ogg,silencio.wav}`.

## Contrato técnico

- Entradas: arquivo de áudio.
- Saídas: `IngestedDoc` com blocos `Time{from,to}`.
- Erros: codec não suportado → `Unsupported`; silêncio total → doc com aviso "sem fala detectada".

## Exemplos de aceite

- **AC-007**: `jfk.mp3` com `FakeTranscriber` determinístico → blocos com tempos crescentes; `real-models` com Moonshine Tiny → texto contém "ask not what your country can do for you"; `silencio.wav` → aviso; progresso reporta 0→100%.

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-004.1 Decodificação por formato (red→green um por vez).
- [ ] TK-004.2 Janelas + desduplicação (unit com transcripts sintéticos).
- [ ] TK-004.3 Integração real; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-ingest audio`; `cargo nextest run -p aura-worker --features win-integration,real-models ingest_audio`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: sem MF (Windows N) para m4a → registrar.

## Condição de retorno à planejadora

Retornar se transcrever 1 h de áudio levar > 10 min com o modelo recomendado (precisaria de fila em background com notificação).

## Relatório de saída

Relatar tempos por minuto de áudio, resultados, EV refs.
