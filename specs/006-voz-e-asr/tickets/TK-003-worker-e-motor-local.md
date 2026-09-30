---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 006-voz-e-asr
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-002"]
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-006"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-worker", "crates/aura-asr/src/transcriber.rs", "crates/aura-asr/src/worker_client.rs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-003 — `aura-worker` e motor de transcrição local

## Objetivo e limites

Entrega o binário `aura-worker` (JSON-RPC stdio + pipe de dados), o `WorkerClient` (spawn sob demanda via `ChildRegistry`, idle 2 min, reinício), a Interface `Transcriber` com `WorkerTranscriber` e `FakeTranscriber`, os métodos `asr.load`/`asr.transcribe` com `transcribe-rs`/`transcribe-cpp`, e as medições H-006/H-014.

Não inclui: PTT (TK-004), streaming (TK-007), métodos de mídia/ingestão (004 TK-007 e 007 acrescentam no mesmo worker).

## Leitura em ordem

1. `docs/adr/0005-worker-sob-demanda-para-cargas-nativas.md`.
2. `specs/006-voz-e-asr/plan.md` → transcriber, worker, OT-001, passagem de áudio.
3. `apps/desktop/src-tauri/src/children.rs` (001 TK-001) e `crates/aura-codex/src/rpc.rs` (002 TK-001) → padrão JSON-RPC a reutilizar (extrair para `aura-core::jsonrpc` se conveniente).
4. Docs atuais: `transcribe-rs` (engines ONNX, opções de idioma), `transcribe-cpp` (Whisper GGUF, GPU), `ort` DirectML.

## Decisões já resolvidas

- Worker é um binário Tauri sidecar (`bundle.externalBin`) sem janela (`windows_subsystem = "windows"`).
- `asr.load` mantém um modelo carregado por vez; trocar de modelo descarrega o anterior.
- `Transcript{language, segments:[{start_ms, end_ms, text}], text}`.
- Idle: `WorkerClient` encerra o processo 2 min após a última requisição, exceto se `keep_warm` (Overlay em escuta) estiver ativo.
- Liberdade local: organização interna do worker.

## Mapa de alterações

- Novo: `crates/aura-worker/{Cargo.toml,src/main.rs,src/rpc.rs,src/asr.rs}`.
- Novo: `crates/aura-asr/src/{transcriber.rs,worker_client.rs}` → `Transcriber`, `WorkerTranscriber`, `FakeTranscriber`, `WorkerClient`.
- Existente: `apps/desktop/src-tauri/tauri.conf.json` → `externalBin` do worker.

## Contrato técnico

- Entradas: `AudioInput::{Pcm16kMono(Vec<f32>), WavFile(PathBuf)}`, `AsrOptions{model, language: Auto|Code, prompt?}`.
- Saídas: `Transcript`.
- Invariantes: OT-001; OT-003 (PCM por pipe, não disco).
- Erros: `AsrError::{ModelMissing, LoadFailed, WorkerCrashed, Timeout}`; crash → 1 reinício automático e nova tentativa.
- Efeitos: processo filho sob demanda.

## Exemplos de aceite

- **AC-006**: `real-models` (CI Linux e Windows): Moonshine Tiny + `jfk.wav` → texto contém "ask not what your country can do for you" (normalizado); `WorkerClient` com `tokio::time::pause`: após 2 min sem requisição → processo encerrado; próxima requisição → novo processo e evento `AsrState::Loading` antes do resultado. Máquina de referência: Parakeet V3 com frase pt de 10 s "O Aura transcreve minha voz rapidamente no Windows" → WER ≤ 10%, tempo de transcrição < 1 s (H-006) e carga < 2 s (H-014) registrados.

## Dependências e sequência de execução

Depende de: TK-002.

- [ ] TK-003.1 Worker com `health` + `WorkerClient` idle/restart (red→green com relógio pausado).
- [ ] TK-003.2 `asr.transcribe` com Moonshine Tiny (`real-models`) red→green.
- [ ] TK-003.3 Whisper GGUF via `transcribe-cpp` e Parakeet via ONNX; DirectML com fallback CPU.
- [ ] TK-003.4 Medições H-006/H-014; H-013 (005) se pendente; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-asr worker_client -p aura-worker`; `cargo nextest run -p aura-asr --features real-models transcribe`; roteiro de medição na máquina de referência.
- Estado esperado: verdes; medições registradas.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: download do modelo de teste bloqueado → cache de CI.

## Condição de retorno à planejadora

Retornar se `transcribe-rs`/`transcribe-cpp` não compilarem para `x86_64-pc-windows-msvc` com as features necessárias, ou se H-006 falhar por margem > 2×.

## Relatório de saída

Relatar versões, medições, resultados, EV refs.
