---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 006-voz-e-asr
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-003", "AC-004", "AC-005"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-asr/src/download.rs", "crates/aura-store/src/migrations/0005_asr.sql", "apps/desktop/src/settings/voice/DownloadProgress.tsx"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-002 — Gerenciador de downloads de modelos

## Objetivo e limites

Entrega `Downloader` com progresso, pausa/retomada por `Range`, retomada após queda de rede e reinício do app, verificação SHA-256, extração atômica (`tar.gz`/`tar.zst`/arquivo único), cancelamento, verificação de espaço e remoção de modelos.

Não inclui: transcrição (TK-003).

## Leitura em ordem

1. `specs/006-voz-e-asr/plan.md` → download, OT-002.
2. `crates/aura-asr/src/catalog.rs` (TK-001).
3. Referência: Handy `managers/model/download.rs`.
4. Docs atuais: `reqwest` streaming + `Range`, `ETag`/`If-Range`.

## Decisões já resolvidas

- Arquivo parcial `models/asr/.downloads/<id>.part` + registro em `downloads`; retomada usa `If-Range` com ETag salvo; servidor sem suporte a Range → reinicia do zero.
- Até 3 tentativas automáticas com backoff (2, 8, 30 s) em falhas de rede.
- Espaço exigido = 1,2 × `size_bytes` (arquivo + extração).
- Extração em `models/asr/.staging/<id>` e `rename` para `models/asr/<id>`.
- Liberdade local: frequência de eventos de progresso (≥ 4/s).

## Mapa de alterações

- Novo: `crates/aura-asr/src/download.rs` → `Downloader`, `DownloadHandle`, `DownloadEvent`, `DownloadError::{Network, Checksum, InsufficientSpace{needed}, Cancelled, Extract}`.
- Novo: `crates/aura-store/src/migrations/0005_asr.sql`.
- Existente: `voice.rs` → comandos `asr_download_start/pause/resume/cancel`, `asr_model_remove`.
- Novo: `apps/desktop/src/settings/voice/DownloadProgress.tsx`.

## Contrato técnico

- Entradas: `ModelId`.
- Saídas: eventos de progresso; modelo instalado.
- Invariantes: OT-002; um download por modelo por vez.
- Efeitos: arquivos em `models/asr`.

## Exemplos de aceite

- **AC-003**: `wiremock` servindo 10 MB com suporte a Range; conexão derrubada em 4 MB → retoma com `Range: bytes=4194304-` e conclui; SHA confere → instalado. Cancelar em 50% → `.part` inexistente. Fixture com SHA diferente → `Checksum` e nada instalado.
- **AC-004**: espaço livre simulado 500 MB para modelo de 456 MB (precisa 547 MB) → `InsufficientSpace{needed: 547 MB}`.
- **AC-005**: remover modelo instalado e selecionado com outro instalado → pasta ausente e seleção = outro; sem outro → "nenhum".

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-002.1 Download simples + SHA (red→green).
- [ ] TK-002.2 Retomada após queda (red→green); reinício do app (estado persistido).
- [ ] TK-002.3 Cancelar, espaço, remoção.
- [ ] TK-002.4 UI; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-asr download`; `pnpm -C apps/desktop test -- DownloadProgress`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum (tudo local com `wiremock`).

## Condição de retorno à planejadora

Retornar se o espelho escolhido não suportar Range/ETag.

## Relatório de saída

Relatar resultados e EV refs.
