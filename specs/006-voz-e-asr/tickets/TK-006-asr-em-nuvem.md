---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 006-voz-e-asr
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-004"]
requirement_refs: ["FR-006"]
acceptance_refs: ["AC-010"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-asr/src/cloud.rs", "apps/desktop/src/settings/voice/CloudAsr.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-006 — ASR em nuvem via Provedor BYOK com fallback local

## Objetivo e limites

Entrega `CloudTranscriber` (multipart `POST <base>/audio/transcriptions` com WAV 16 kHz), seleção de Provedor/modelo de transcrição, selo "nuvem" no microfone e fallback para o modelo local em erro.

Não inclui: streaming em nuvem (Realtime) — CAND-012.

## Leitura em ordem

1. `crates/aura-gateway/src/{registry.rs,credentials.rs}` (003 TK-001) → Provedores e Credenciais; acrescentar `capabilities.transcription`.
2. `crates/aura-asr/src/transcriber.rs` (TK-003).
3. Docs atuais: OpenAI `/v1/audio/transcriptions` (`model`, `language`, `prompt`, `response_format=verbose_json`), Groq equivalente.

## Decisões já resolvidas

- Chamada feita pelo host (não pelo app-server); Credencial lida do `CredentialStore` no momento da chamada.
- `response_format=verbose_json` quando suportado para obter segmentos; senão `json` e um segmento único.
- Fallback: erro de rede/5xx/401 → `WorkerTranscriber` se houver modelo local instalado, com toast "Usando modelo local"; senão erro.
- Liberdade local: UI.

## Mapa de alterações

- Novo: `crates/aura-asr/src/cloud.rs` → `CloudTranscriber`, `FallbackTranscriber{primary, fallback}`.
- Existente: `crates/aura-gateway/presets.toml` → `transcription_models` para OpenAI/Groq.
- Novo: `apps/desktop/src/settings/voice/CloudAsr.tsx`; `MicButton` com selo.

## Contrato técnico

- Entradas: PCM → WAV em memória; `AsrOptions`.
- Saídas: `Transcript`.
- Invariantes: áudio não é gravado em disco para envio.
- Erros: `AsrError::Cloud(category)` mapeado como no 003.

## Exemplos de aceite

- **AC-010**: `wiremock` recebe multipart com `model=whisper-large-v3`, `language=pt`, arquivo `audio.wav` (cabeçalho RIFF, 16 kHz) e header de autenticação do preset → `Transcript` do `verbose_json`; 500 → `FallbackTranscriber` usa `FakeTranscriber` local e emite evento de aviso; sem local → erro exibido.

## Dependências e sequência de execução

Depende de: TK-004; 003-byok-gateway/TK-001 (outro esforço).

- [ ] TK-006.1 `CloudTranscriber` com `wiremock` red→green.
- [ ] TK-006.2 `FallbackTranscriber` red→green.
- [ ] TK-006.3 UI; manual com Groq; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-asr cloud`; `pnpm -C apps/desktop test -- CloudAsr`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: sem chave de teste → só `wiremock`.

## Condição de retorno à planejadora

Retornar se provedores relevantes exigirem formatos incompatíveis com multipart WAV.

## Relatório de saída

Relatar resultados e EV refs.
