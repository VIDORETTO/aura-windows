# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [ ] TK-001 — TK-001 — Catálogo de modelos, detecção de hardware e recomendação
Status: `implemented` | Bloqueado por: nenhum

- [ ] TK-001.1 Verificar licenças e montar `models.toml` (registrar decisão por modelo).
- [ ] TK-001.2 Teste de parse do catálogo real (red→green).
- [ ] TK-001.3 Unit `recommend` caso a caso.
- [ ] TK-001.4 `hardware::probe` Windows (integração: valores plausíveis > 0).
- [ ] TK-001.5 UI + Vitest; evidências.

## [ ] TK-002 — TK-002 — Gerenciador de downloads de modelos
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-002.1 Download simples + SHA (red→green).
- [ ] TK-002.2 Retomada após queda (red→green); reinício do app (estado persistido).
- [ ] TK-002.3 Cancelar, espaço, remoção.
- [ ] TK-002.4 UI; evidências.

## [ ] TK-003 — TK-003 — `aura-worker` e motor de transcrição local
Status: `implemented` | Bloqueado por: TK-002

- [ ] TK-003.1 Worker com `health` + `WorkerClient` idle/restart (red→green com relógio pausado).
- [ ] TK-003.2 `asr.transcribe` com Moonshine Tiny (`real-models`) red→green.
- [ ] TK-003.3 Whisper GGUF via `transcribe-cpp` e Parakeet via ONNX; DirectML com fallback CPU.
- [ ] TK-003.4 Medições H-006/H-014; H-013 (005) se pendente; evidências.

## [ ] TK-004 — TK-004 — Push-to-talk no Overlay
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-004.1 Unit `trim_silence` red→green.
- [ ] TK-004.2 Unit máquina de estados (casos acima) red→green.
- [ ] TK-004.3 UI + Vitest; onboarding.
- [ ] TK-004.4 Manual com Parakeet V3 e medição; evidências.

## [ ] TK-005 — TK-005 — Atalho global de voz, idioma e vocabulário personalizado
Status: `implemented` | Bloqueado por: TK-004

- [ ] TK-005.1 Unit `vocabulary` red→green.
- [ ] TK-005.2 Idioma no `AsrOptions` (contrato worker).
- [ ] TK-005.3 Atalho global de voz (integração Windows).
- [ ] TK-005.4 UI; evidências.

## [ ] TK-006 — TK-006 — ASR em nuvem via Provedor BYOK com fallback local
Status: `implemented` | Bloqueado por: TK-004

- [ ] TK-006.1 `CloudTranscriber` com `wiremock` red→green.
- [ ] TK-006.2 `FallbackTranscriber` red→green.
- [ ] TK-006.3 UI; manual com Groq; evidências.

## [ ] TK-007 — TK-007 — Parciais em streaming
Status: `implemented` | Bloqueado por: TK-004

- [ ] TK-007.1 Streaming Moonshine no worker (red→green).
- [ ] TK-007.2 Pseudo-streaming com janela (medir custo).
- [ ] TK-007.3 UI; evidências.
