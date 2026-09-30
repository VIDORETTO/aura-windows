---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 005-captura-de-audio
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-001"]
acceptance_refs: ["AC-001", "AC-002"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-audio/src/source.rs", "crates/aura-audio/src/wasapi.rs", "crates/aura-audio/src/synthetic.rs", "crates/aura-audio/src/pipeline.rs", "crates/aura-audio/src/level.rs", "apps/desktop/src/settings/audio", "apps/desktop/src-tauri/src/audio.rs"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-001 — Fontes de áudio (microfone e sistema), hub e medidor

## Objetivo e limites

Entrega `AudioSource` com `WasapiMic`, `WasapiLoopback` e `SyntheticAudio`, reamostragem para 16 kHz mono, `AudioHub` com fan-out e medidor de nível, seleção de dispositivo com fallback, e a seção "Áudio" nas Configurações com teste de nível.

Não inclui: segmentos/buffer (TK-002), push-to-talk (006 TK-004 consome o hub).

## Leitura em ordem

1. `specs/005-captura-de-audio/plan.md` → Modules, OT-002, riscos.
2. `crates/aura-core/src/settings.rs` (001 TK-004) → adicionar chaves de áudio.
3. Docs atuais: crate `wasapi` (loopback `AUDCLNT_STREAMFLAGS_LOOPBACK`, modo evento), `rubato`, `rtrb`; `IMMNotificationClient` para troca de dispositivo padrão.

## Decisões já resolvidas

- Thread de captura de alta prioridade (MMCSS "Pro Audio") escreve em `rtrb`; thread de processamento reamostra (rubato `FftFixedIn`) e publica `Chunk` de 20 ms.
- Nível = RMS em dBFS por janela de 50 ms, publicado a 20 Hz.
- Fallback: se o `device_id` salvo não existir, usar padrão e emitir `DeviceFallback{wanted, using}`; `IMMNotificationClient` detecta reconexão.
- Liberdade local: organização de threads.

## Mapa de alterações

- Novo: `crates/aura-audio/{Cargo.toml,src/lib.rs,src/source.rs,src/wasapi.rs,src/synthetic.rs,src/pipeline.rs,src/level.rs}`.
- Novo: `apps/desktop/src-tauri/src/audio.rs` → `audio_devices`, `audio_level_test_start/stop`, evento `audio_level`.
- Novo: `apps/desktop/src/settings/audio/{AudioSection.tsx,LevelMeter.tsx}`.

## Contrato técnico

- Entradas: `DeviceSel::{Default, Id(String)}`, Fonte.
- Saídas: `Chunk{samples: [f32; 320], at}` a 16 kHz; `Level{dbfs}` a 20 Hz.
- Invariantes: um único stream de dispositivo por Fonte, independentemente de assinantes.
- Erros: `AudioError::{NoDevice, AccessDenied, DeviceLost, Os(code)}`; `AccessDenied` → mensagem com `ms-settings:privacy-microphone`.

## Exemplos de aceite

- **AC-001**: `SyntheticAudio` tom 440 Hz a −12 dBFS → `level` −12 ± 1 dB e ≥ 20 eventos/s (unit); manual: falar no microfone e tocar vídeo → medidores reagem.
- **AC-002**: integração Windows com dispositivo virtual (VB-CABLE ou dispositivo de teste do runner) desabilitado → evento `DeviceFallback` e captura continua no padrão; reabilitar → volta ao escolhido.
- Reamostragem (unit): 48 kHz estéreo tom 1 kHz → 16 kHz mono com pico FFT em 1 kHz ± 10 Hz.

## Dependências e sequência de execução

Depende de: 001-fundacao-overlay/TK-004 (outro esforço). Nenhum ticket deste esforço.

- [ ] TK-001.1 Unit reamostragem e nível com `SyntheticAudio` (red→green).
- [ ] TK-001.2 `AudioHub` fan-out (dois assinantes, um stream) red→green.
- [ ] TK-001.3 WASAPI mic/loopback (Windows) + fallback (AC-002).
- [ ] TK-001.4 UI e evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-audio`; `cargo nextest run -p aura-audio --features win-integration`; `pnpm -C apps/desktop test -- audio`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner sem dispositivo de áudio → `not_run` para WASAPI e execução na máquina de referência.

## Condição de retorno à planejadora

Retornar se o loopback WASAPI não funcionar com dispositivos Bluetooth/HDMI comuns sem alternativa.

## Relatório de saída

Relatar resultados, dispositivos testados, EV refs.
