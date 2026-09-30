---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 005-captura-de-audio
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-003", "AC-004", "AC-006"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-audio/src/recorder.rs", "apps/desktop/src/settings/audio/AudioModes.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-002 — Buffer recente, Gravação manual e Contínuo para áudio

## Objetivo e limites

Entrega `AudioSegmentRecorder` (Ogg Opus 10 s cifrado) ligado ao `SegmentStore`/`retention` do 004 com `source ∈ {Mic, SystemAudio}`, os Modos por Fonte, Gravações de áudio na tela "Gravações" e a verificação H-013.

Não inclui: indicadores/pausa (TK-003), anexos (TK-004).

## Leitura em ordem

1. `crates/aura-capture/src/{segments.rs,retention.rs,recordings.rs}` (004 TK-005/TK-006) → Interfaces genéricas por `Source`.
2. `crates/aura-audio/src/pipeline.rs` (TK-001).
3. Docs atuais: `audiopus`/`opus` (24 kbps VBR, `Application::Voip`), contêiner Ogg.

## Decisões já resolvidas

- Frames Opus de 20 ms; segmento fecha a cada 10 s, Ogg em memória, `Vault::seal_bytes` com chave `"capture-segment"`.
- Mesmas regras de retenção/Contínuo do 004; `recordings.source` aceita Mic/SystemAudio.
- H-013: comparar WER de um trecho de referência (LibriSpeech/Common Voice pt) em PCM vs. Opus 24 kbps com o modelo padrão do 006 quando disponível; se 006 ainda não existir, registrar como `not_run` e executar no 006 TK-003.
- Liberdade local: organização interna.

## Mapa de alterações

- Novo: `crates/aura-audio/src/recorder.rs` → `AudioSegmentRecorder`.
- Existente: `crates/aura-capture/src/segments.rs` → aceitar `Source::Mic|SystemAudio` (se ainda não genérico).
- Novo: `apps/desktop/src/settings/audio/AudioModes.tsx`; tela Gravações exibe áudio com player `<audio>` via `aura-media://`.

## Contrato técnico

- Entradas: Modo por Fonte, N.
- Saídas: `.seg` cifrados, `recordings` de áudio.
- Invariantes: OT-001 (005).
- Erros: codificador falha → segmento descartado com log; captura segue.

## Exemplos de aceite

- **AC-003**: `SyntheticAudio` + relógio acelerado 20 min, N=5 → 31 segmentos da Fonte; nenhum com cabeçalho `OggS` legível (cifrado).
- **AC-004**: Windows: tocar tom 1 kHz 60 s com Áudio do sistema em Manual → Gravação 60 ± 1 s; decodificar → pico 1 kHz.
- **AC-006**: reuso de `retention::plan` com `source=SystemAudio` (unit adicional).

## Dependências e sequência de execução

Depende de: TK-001; 004-contexto-de-tela/TK-006 (outro esforço).

- [ ] TK-002.1 Recorder com sintético + cifragem (red→green).
- [ ] TK-002.2 Integração Windows AC-004.
- [ ] TK-002.3 UI de modos; H-013; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-audio recorder`; `cargo nextest run -p aura-audio --features win-integration recorder_loopback`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: sem dispositivo de áudio.

## Condição de retorno à planejadora

Retornar se H-013 mostrar degradação > 1 ponto de WER (aumentar bitrate ou guardar PCM comprimido sem perdas).

## Relatório de saída

Relatar tamanhos por minuto, resultados, EV refs.
