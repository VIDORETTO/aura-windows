---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 005-captura-de-audio
revision: 1
spec_revision: 1
status: ready
---

# Plan: Captura de áudio

## Summary

Crate `aura-audio` com `AudioSource` (adapters WASAPI mic/loopback e sintético), reamostragem para 16 kHz mono, fan-out para consumidores (medidor, push-to-talk do 006, gravador de segmentos) e um `AudioSegmentRecorder` que codifica Opus em segmentos de 10 s cifrados, reaproveitando `SegmentStore`/`retention`/Política do 004 (OT-005 do 004). Recortes de áudio são transcritos pela Interface `Transcriber` do 006.

## Technical context

- Language/runtime: Rust.
- Dependencies (fixar/consultar no ticket): `wasapi` (loopback e captura em modo evento), `cpal` (enumeração/alternativa para mic), `rubato` (reamostragem), `audiopus`/`opus` (codificação), `ogg` (contêiner Ogg Opus por segmento), `rtrb` (ring buffer lock-free entre thread de áudio e consumidores).
- Storage/data: `captures\audio\<source>\<yyyy-mm-dd>\<segment>.seg`; mesmas tabelas do 004 com `source ∈ {Mic, SystemAudio}`.
- Test command: `cargo nextest run -p aura-audio`; Windows `--features win-integration`.
- Target/platform: Windows (WASAPI); lógica de pipeline testável em Linux com fonte sintética.

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-004; AC-001–AC-009.

## Modules, interfaces, consumers, and seams

| Module | Interface | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `aura-audio::source` | `trait AudioSource { fn start(&self, device: DeviceSel) -> Result<AudioStream>; fn devices(&self) -> Vec<DeviceInfo>; }`; `AudioStream` entrega `Chunk{samples_f32_16k_mono, at}` | pipeline | Adapters reais: `WasapiMic`, `WasapiLoopback` (produção) e `SyntheticAudio` (tom/arquivo WAV, testes) |
| `aura-audio::pipeline` | `AudioHub::{subscribe(Consumer) -> Subscription, level() -> Level}` | medidor, 006 PTT, recorder | Com `SyntheticAudio` |
| `aura-audio::recorder` | `AudioSegmentRecorder::{start(kind), stop}` → Ogg Opus 10 s cifrado via `Vault::seal_bytes` | buffer/gravação | `SyntheticAudio` + decodificar e comparar tom |
| `aura-audio::clips` | `AudioClips::from_range(source|Both, from, to) -> AudioClip{wav_path, segments}` | host, MCP | Integração |
| `aura-mcp::tools::audio` | `audio_recent{minutes≤30, source:"mic"|"system"|"both"}` → texto transcrito com tempos | Codex | Cliente MCP de teste com `Transcriber` falso |
| UI | `AudioSection`, `LevelMeter`, `AudioChip` | usuário | Vitest |

Transcrição de Recortes usa `aura-asr::Transcriber` (006 TK-003). Para "Ambos", cada Fonte é transcrita separadamente e as falas são intercaladas por tempo com rótulos "Você"/"Sistema".

## Chosen approach and alternatives

- **WASAPI direto** para loopback (o `cpal` não expõe loopback de forma confiável no Windows); `cpal` só para enumeração se simplificar.
- **Opus em Ogg por segmento**: ~180 KB/min, ótimo para fala, cifrado como os de tela.
- **Um único hub por Fonte** com fan-out evita abrir o dispositivo várias vezes (PTT + buffer simultâneos).
- Alternativa descartada: guardar PCM (10× maior).

## Data, compatibility, and external dependencies

Sem migração nova (usa `0004_capture.sql`). Configurações novas em `Settings`: `audio.mic_device`, `audio.system_device`, modos e N por Fonte.

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001 | Integração sintética + manual | Spec (≥ 20 Hz) | Tom de −12 dBFS → nível reportado −12 ± 1 dB, eventos a ≥ 20 Hz |
| AC-002 | Integração Windows (dispositivo virtual) + manual | Spec | Desabilitar dispositivo → fallback padrão + aviso |
| AC-003, AC-006 | Reuso de `retention::plan` + integração | Spec | 20 min simulados → só últimos 5 min |
| AC-004 | Integração Windows | Tom 1 kHz tocado → segmento decifrado/decodificado contém pico em 1 kHz (FFT) | 60 s → duração 60 ± 1 s |
| AC-005 | Vitest + unit `decide` | Spec | Pausa → nenhum segmento novo |
| AC-007, AC-009 | Integração com `Transcriber` falso + Vitest | Transcrição do falso | Chip e payload com texto rotulado e tempos |
| AC-008 | Cliente MCP de teste | Spec | Permissões Nunca/Perguntar/Sempre |

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `crates/aura-audio/src/{source.rs,wasapi.rs,synthetic.rs,pipeline.rs,recorder.rs,clips.rs,level.rs}` | new | ver tabela | Áudio | 2026-09-29 |
| `crates/aura-mcp/src/tools/audio.rs` | new | `audio_recent` | Agente | 2026-09-29 |
| `apps/desktop/src-tauri/src/audio.rs` | new | comandos | Host | 2026-09-29 |
| `apps/desktop/src/settings/audio/**`, `src/conversation/chips/AudioChip.tsx` | new | UI | Interface | 2026-09-29 |
| `crates/aura-capture/src/{segments.rs,retention.rs}` | existing (004 TK-005/006) | genéricos por `Source` | Reuso | 004 |

## Derived technical obligations

- **OT-001** → FR-002: nenhum PCM de áudio vai a disco em texto claro; só Segmentos cifrados e arquivos de Recorte dentro do Workspace da conversa quando o usuário anexa.
- **OT-002** → 006: `AudioHub::subscribe` entrega 16 kHz mono f32 com latência ≤ 30 ms para o push-to-talk.
- **OT-003** → FR-004: `audio_recent` passa por `decide` com `source` correspondente e registra acesso.

## Risks and gates

- Drivers com formatos exóticos (multicanal 48 kHz float) — reamostrar e mixar para mono sempre.
- Captura do próprio TTS do Aura no loopback — marcar sessão de áudio do Aura e, se possível, excluir via `AUDCLNT_STREAMOPTIONS`; senão documentar.
- G2: satisfeito. G3: TK-001 sem blockers internos; TK-002 depende de 004 TK-005 (segmentos) e TK-004 de 006 TK-003 (`Transcriber`).
