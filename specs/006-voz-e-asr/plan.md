---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 006-voz-e-asr
revision: 1
spec_revision: 1
status: ready
---

# Plan: Voz e ASR

## Summary

Crate `aura-asr` com catálogo (`models.toml` versionado), detecção de hardware, recomendação pura, gerenciador de downloads e a Interface `Transcriber` com dois adapters de produção (local via `aura-worker`; nuvem via Provedor BYOK) e um de teste. O `aura-worker` (novo binário, ADR 0005) hospeda `transcribe-rs` (ONNX: Parakeet, Moonshine, SenseVoice, Canary) e `transcribe-cpp` (Whisper GGUF). O Push-to-talk assina o `AudioHub` do 005, aplica VAD Silero e entrega áudio ao `Transcriber`.

## Technical context

- Language/runtime: Rust.
- Dependencies (fixar/consultar no ticket): `transcribe-rs` (feature `onnx`), `transcribe-cpp`, `ort` (DirectML), `vad-rs`/Silero ONNX, `reqwest` (Range), `sha2`, `tar`/`flate2`/`zstd`, `sysinfo` + DXGI (`IDXGIFactory6::EnumAdapterByGpuPreference`) para hardware.
- Referência de implementação: Handy (`cjpais/Handy`, MIT) — `managers/model.rs` (catálogo) e `managers/model/download.rs` (downloads). Reaproveitar ideias; manter nosso próprio espelho.
- Storage/data: `%LOCALAPPDATA%\Aura\models\asr\<id>\`; `downloads(id, model_id, bytes_done, total, etag, state)`.
- Test command: `cargo nextest run -p aura-asr -p aura-worker`; testes lentos com modelo real atrás de `--features real-models` (baixa Moonshine Tiny ~31 MB e usa `jfk.wav`/frases pt de referência).
- Target/platform: Windows (DirectML/Vulkan); CPU em qualquer SO.

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-008; AC-001–AC-013.

## Modules, interfaces, consumers, and seams

| Module | Interface | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `aura-asr::catalog` | `Catalog::load(embedded)`, `ModelEntry{id, family, engine: Onnx|Ggml, size_bytes, sha256, url, languages, speed, accuracy, streaming, min_ram_mb, gpu_recommended, license}` | UI, downloads, recomendação | Parse do `models.toml` real |
| `aura-asr::hardware` | `fn probe() -> Hardware{ram_mb, cpu_cores, gpus: Vec<Gpu{vendor, vram_mb, dedicated}>, npu: bool}` | recomendação | Adapter Windows real; valores literais nos testes |
| `aura-asr::recommend` | `fn recommend(&Catalog, &Hardware, ui_lang) -> ModelId` (pura) | UI | Tabela de casos |
| `aura-asr::download` | `Downloader::{start(id) -> DownloadHandle, pause, resume, cancel}`, progresso; instalação atômica | UI | `wiremock` com Range e falhas simuladas |
| `aura-asr::transcriber` | `trait Transcriber { async fn transcribe(&self, audio: AudioInput, opts: AsrOptions) -> Result<Transcript>; fn stream(&self, opts) -> Option<StreamingSession>; }` | PTT, 005, 007 | Adapters reais: `WorkerTranscriber` (local) e `CloudTranscriber` (BYOK); `FakeTranscriber` (testes) |
| `aura-worker` (bin) | JSON-RPC stdio: `asr.load{model_path, engine}`, `asr.transcribe{pcm_path|pcm_shm, lang, prompt}`, `asr.stream.*`, `media.keyframes` (004), `ingest.*` (007); `health` | host via `WorkerClient` | Processo real em testes de integração |
| `aura-asr::worker_client` | `WorkerClient` (spawn sob demanda via `ChildRegistry`, idle 2 min, restart) | `WorkerTranscriber`, 004, 007 | Worker real com modelo de teste; worker falso para falhas |
| `aura-asr::vocabulary` | `fn apply(text, &[Term]) -> String` (substituições case-insensitive por fronteira de palavra) + `prompt_for_whisper(&[Term])` | transcriber | Unit |
| `apps/desktop/src-tauri/src/voice.rs` | `PushToTalk` (estado: Idle→Listening→Transcribing→Done/Cancelled), comandos e atalhos | UI | Unit da máquina de estados + Vitest |
| UI `settings/voice/`, `overlay/MicButton.tsx`, `ModelCatalog.tsx` | — | usuário | Vitest |

Passagem de áudio ao worker: PCM f32 16 kHz escrito num arquivo temporário cifrado? Não — o worker é processo local confiável: usar memória compartilhada nomeada (`CreateFileMapping` com DACL do usuário) ou pipe binário; decisão: **pipe binário anônimo adicional** (stdio JSON-RPC para controle + pipe de dados), evitando arquivos.

## Chosen approach and alternatives

- **`transcribe-rs`/`transcribe-cpp`** (MIT, usados pelo Handy com 32k★) em vez de integrar whisper.cpp/sherpa-onnx diretamente — já cobrem Parakeet/Moonshine/SenseVoice/Canary/Whisper com uma API.
- **Worker separado** (ADR 0005) para devolver memória e isolar falhas.
- **Espelho próprio** (Cloudflare R2 ou GitHub Releases de `aura-models`) com SHA-256 no catálogo, populado a partir das fontes originais após verificação de licença.
- Alternativas descartadas: faster-whisper (Python), Whisper via DirectML próprio (esforço alto).

## Data, compatibility, and external dependencies

- Migração `0005_asr.sql` (`downloads`, `asr_settings` em `settings`).
- Catálogo embutido no binário + atualização opcional assinada (010).
- Provedores de ASR em nuvem: presets com `/audio/transcriptions` (OpenAI `gpt-4o-transcribe`/`whisper-1`, Groq `whisper-large-v3`), marcados no registry do 003 (`capabilities.transcription = true`).

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001 | Unit catálogo + Vitest | `models.toml` | Todos os campos presentes; UI renderiza |
| AC-002 | Unit `recommend` | Regras da spec | Casos: (16 GB, sem GPU, pt-BR)→Parakeet V3; (16 GB, RTX 8 GB, pt-BR)→Whisper Turbo; (6 GB, sem GPU, en)→Moonshine Small; (8 GB, ja)→SenseVoice |
| AC-003 | `wiremock` | Spec | Queda após 40% → retoma com `Range: bytes=<n>-`; cancelar → arquivo `.part` removido; SHA errado → removido + erro |
| AC-004 | Unit | 1,2 × tamanho | Espaço simulado insuficiente → `InsufficientSpace{needed}` |
| AC-005 | Integração | Spec | Remover → pasta ausente; seleção muda |
| AC-006 | Integração `real-models` + medição | Texto conhecido de `jfk.wav` ("And so my fellow Americans ask not…") e frase pt gravada | Moonshine Tiny em CI; Parakeet V3 na máquina de referência (H-006/H-014); worker encerra após 2 min (relógio pausado no teste do `WorkerClient`) |
| AC-007, AC-008, AC-013 | Unit máquina de estados + Vitest + manual | Spec (1,5 s) | `FakeTranscriber` com latência controlada; manual com Parakeet V3 |
| AC-009 | Integração Windows + manual | Spec | Atalho global segurado → Overlay escutando |
| AC-010 | `wiremock` `/audio/transcriptions` | Spec | Multipart com WAV; erro 500 → fallback local |
| AC-011 | Unit `vocabulary` + integração Whisper prompt | Termos | "aura e codex" → "Aura e Codex" |
| AC-012 | Integração com Moonshine streaming | Spec (≤ 500 ms) | Parciais emitidos durante áudio sintético falado (arquivo) |

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `crates/aura-asr/src/{catalog.rs,hardware.rs,recommend.rs,download.rs,transcriber.rs,worker_client.rs,cloud.rs,vocabulary.rs}` | new | ver tabela | ASR | 2026-09-29 |
| `crates/aura-asr/models.toml` | new | catálogo | Dados | 2026-09-29 |
| `crates/aura-worker/src/{main.rs,rpc.rs,asr.rs,media.rs}` | new | worker | Processo | 2026-09-29 |
| `apps/desktop/src-tauri/src/voice.rs` | new | `PushToTalk` | Host | 2026-09-29 |
| `apps/desktop/src/settings/voice/**`, `src/overlay/MicButton.tsx` | new | UI | Interface | 2026-09-29 |
| `crates/aura-store/src/migrations/0005_asr.sql` | new | downloads | Dados | 2026-09-29 |

## Derived technical obligations

- **OT-001** → FR-004: o `WorkerClient` é o único dono do processo worker; 004/007 usam o mesmo cliente (um worker por vez).
- **OT-002** → FR-003: instalação só é considerada concluída após SHA-256 conferido e extração atômica.
- **OT-003** → FR-005: nenhum áudio de PTT é gravado em disco (memória → pipe → worker), salvo quando o usuário anexa.
- **OT-004** → FR-001: toda entrada do catálogo tem `license` e `redistribution_ok = true` antes de publicar no espelho.

## Risks and gates

- Licenças de redistribuição (ex.: Cohere, SenseVoice) podem impedir espelhar — retirar do catálogo se necessário (OT-004).
- DirectML com certos drivers pode falhar — fallback CPU automático com aviso.
- G2: satisfeito. G3: TK-001 sem blockers; TK-004 depende de 005 TK-001.
