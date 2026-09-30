---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 004-contexto-de-tela
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-003"]
requirement_refs: ["FR-005", "FR-007"]
acceptance_refs: ["AC-009", "AC-013"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-capture/src/segments.rs", "crates/aura-capture/src/retention.rs", "crates/aura-capture/src/encoder.rs", "apps/desktop/src/privacy/CaptureModes.tsx", "apps/desktop/src/overlay/CaptureIndicators.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-005 — Buffer recente de tela (últimos N minutos)

## Objetivo e limites

Entrega o `SegmentRecorder` (WGC stream → H.264 fMP4 em memória → cifrado → `.seg`), o `SegmentStore`, `retention::plan` e o Modo "Buffer recente" com N configurável (1–30 min), fps (0,5–2), indicadores na bandeja/Overlay e o benchmark de CPU/disco.

Não inclui: Gravação manual/Contínuo (TK-006), anexar Recortes (TK-007).

## Leitura em ordem

1. `docs/adr/0004-buffer-de-captura-em-disco-cifrado.md`.
2. `specs/004-contexto-de-tela/plan.md` → "Formato de segmento", OT-003, OT-005, riscos.
3. `crates/aura-store/src/vault.rs` (001 TK-006) → `seal_bytes`, `derive_key`.
4. `crates/aura-policy/src/decide.rs` (TK-003) → Modo/Pausa.
5. Docs atuais: `windows-capture` `VideoEncoder` e Media Foundation (`IMFSinkWriter` com `MFCreateMFByteStreamOnStream` em memória, fMP4, encoder de hardware).

## Decisões já resolvidas

- Segmentos de 10 s; resolução máx. 1920 px no maior lado; bitrate alvo ~300 kbps a 1 fps.
- Frames passam por `decide`+`redact` antes de entrar no encoder (OT-001): janelas excluídas cobertas também no buffer; Pausa interrompe e fecha o segmento corrente.
- `retention::plan` para buffer: manter segmentos cujo `end_ms ≥ now − N`; executado a cada fechamento de segmento e na inicialização.
- Indicador âmbar (buffer) na bandeja e no cabeçalho; clique → menu Parar/Pausar.
- Liberdade local: estrutura do pipeline (threads/canais).

## Mapa de alterações

- Novo: `crates/aura-capture/src/{segments.rs,retention.rs,encoder.rs}` → `SegmentRecorder`, `SegmentStore`, `RetentionPolicy`, `retention::plan`, `trait VideoEncoder` (`MfH264Encoder`, `NullEncoder` para testes).
- Novo: `tools/aura-bench/src/capture_buffer.rs` → `aura-bench capture-buffer --minutes 10`.
- Novo: `apps/desktop/src/privacy/CaptureModes.tsx`, `apps/desktop/src/overlay/CaptureIndicators.tsx`.
- Existente: `tray.rs` → indicadores.

## Contrato técnico

- Entradas: Modo Tela = Buffer recente, N, fps.
- Saídas: arquivos `.seg` cifrados + índice.
- Invariantes: OT-003; nunca mais que N min + 1 segmento no disco para `kind=buffer`.
- Erros: encoder indisponível → fallback software 0,5 fps + aviso; disco < 2 GB → para e notifica.
- Efeitos: indicadores.

## Exemplos de aceite

- **AC-013** (unit `retention::plan`): segmentos de 10 s cobrindo 0–1200 s, `now=1200 s`, N=5 min → excluir todos com `end ≤ 900 s`; manter 30 + corrente. Integração com `SyntheticSource` + `NullEncoder` e relógio acelerado por 20 min → 31 arquivos `.seg`; bytes de cada `.seg` não começam com assinatura MP4 (`ftyp`) (cifrado).
- **AC-009**: ligar buffer → ícone âmbar na bandeja e no cabeçalho; clicar "Parar" → Modo volta a Sob demanda e nenhum segmento novo em 20 s.
- Benchmark (H-005): máquina de referência, 10 min a 1 fps 1080p → CPU média ≤ 3% do processo `aura.exe` e ≤ 50 MB escritos (≈ 300 MB/h).

## Dependências e sequência de execução

Depende de: TK-003.

- [ ] TK-005.1 Unit `retention::plan` red→green (casos: vazio, dentro, fora, fronteira).
- [ ] TK-005.2 Recorder com `SyntheticSource`/`NullEncoder` + cifragem (red→green).
- [ ] TK-005.3 `MfH264Encoder` em memória (Windows) + verificação de reprodução do fMP4 decifrado.
- [ ] TK-005.4 UI de modos e indicadores; Vitest.
- [ ] TK-005.5 Benchmark H-005; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-capture segments retention`; `cargo nextest run -p aura-capture --features win-integration encoder`; `cargo run -p aura-bench --release -- capture-buffer --minutes 10 --out bench/capture-buffer.json`.
- Estado esperado: verdes; benchmark dentro de H-005 ou limitação registrada.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: VM sem encoder de hardware.

## Condição de retorno à planejadora

Retornar se H-005 falhar por margem grande (> 2×), exigindo reduzir fps/resolução padrão ou mudar formato (ADR 0004).

## Relatório de saída

Relatar resultados, benchmark, EV refs.
