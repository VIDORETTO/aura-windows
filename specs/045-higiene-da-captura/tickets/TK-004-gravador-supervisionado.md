---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 045-higiene-da-captura
type: delivery
status: done
ticket_revision: 8
requires: ["TK-003"]
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-005", "AC-006"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-capture/src/segments.rs", "crates/aura-capture/tests/pipeline.rs", "crates/aura-app/src/recorder.rs", "crates/aura-app/src/host.rs"]
verification_status: passed
last_update: Evidence invalidated after an input changed.
---








# TK-004 — Gravador de tela supervisionado (A-02)

## Objetivo e limites

Erro por Segmento não derruba o gravador. Com 5 falhas seguidas, ele encerra; o estado informado reflete o gravador vivo; a supervisão do host reinicia com recuo exponencial.

Não inclui: áudio (já registra e segue), A-07 (monitor), mudança de IPC.

## Leitura em ordem

1. `crates/aura-capture/src/segments.rs` → laço do `SegmentRecorder::start` (`?` em `push`, `flush` e `store_seg`), `stop`.
2. `crates/aura-app/src/recorder.rs` → `ScreenRun`, `active`, `apply`.
3. `crates/aura-app/src/host.rs` → laço de 15 s (`tick_reminders`/`meeting_poll`) e `reconcile_capture` (`HostEvent::Privacy`).
4. `crates/aura-capture/tests/pipeline.rs` → testes com relógio pausado do tokio.

## Decisões já resolvidas

- Falha = erro de codificação ou de armazenamento ao gravar um Segmento; captura de quadro sem sucesso continua sendo ignorada.
- Limite de 5 falhas seguidas; o contador zera ao gravar.
- `next_backoff(tentativa)`: 15 s × 2^n, teto de 900 s; zera depois de um reinício que grava.
- A supervisão roda no laço de 15 s existente e emite `HostEvent::Privacy` só quando a lista de Fontes ativas muda.
- Liberdade local: onde guardar o estado do recuo; nomes privados.

## Mapa de alterações

- Existente: `segments.rs` → laço do gravador, novo `is_finished()`.
- Existente: `recorder.rs` → `active`, novo `supervise`, novo `next_backoff`.
- Existente: `host.rs` → chamada no laço de 15 s.
- Novo: testes em `pipeline.rs` (`Sealer` que falha) e testes puros de `next_backoff`.

## Contrato técnico

- `SegmentRecorder::is_finished(&self) -> bool`.
- `CaptureService::supervise(&self, now_ms: i64) -> Option<Vec<String>>`: devolve `Some(lista)` quando a lista de Fontes ativas mudou.
- `next_backoff(attempt: u32) -> Duration`.
- Erros: o último erro do gravador fica no log em `warn`.
- Efeitos: um reinício por janela de recuo; nenhum reinício com a tela fora do modo, ou com Pausa.

## Exemplos de aceite

- **AC-005**: `Sealer` falha na 1ª chamada; depois de 30 s do relógio pausado com Segmentos de 10 quadros a 1 s → há Segmentos gravados e `is_finished()` é falso.
- **AC-006**: `Sealer` sempre falha → depois de 5 Segmentos, `is_finished()` é verdadeiro. `next_backoff(0..=7)` = 15, 30, 60, 120, 240, 480, 900, 900 s. No `aura-app`, `active()` sem `screen` quando o gravador encerrou e `supervise` devolve a lista nova, se a plataforma de teste permitir montar o `CaptureService`. Se não permitir, registrar a limitação e cobrir pela compilação e pelo teste de host.

## Dependências e sequência de execução

Depende de: TK-003 (mesmo arquivo `segments.rs`).

- [ ] TK-004.1 AC-005 vermelho (a tarefa encerra no 1º erro).
- [ ] TK-004.2 Green: aviso + contador; `is_finished`.
- [ ] TK-004.3 AC-006: `next_backoff` puro; `active`/`supervise`; chamada no host.
- [ ] TK-004.4 Regressão completa, clippy, fmt, evidência.

## Validação

- Comandos: `cargo test -p aura-capture`; `cargo test -p aura-app`; `cargo test --workspace --exclude aura-desktop`; `cargo clippy -p aura-capture -p aura-audio -p aura-app --all-targets -- -D warnings`; `cargo fmt --all -- --check`.
- Estado esperado: passam.

## Condição de retorno à planejadora

Se a supervisão exigir um novo evento IPC ou mudar o formato de `PrivacyState`, parar e devolver (ADR 0009).

## Relatório de saída

Arquivos e símbolos, ACs, comandos, `EV`, limitações, pendências.
