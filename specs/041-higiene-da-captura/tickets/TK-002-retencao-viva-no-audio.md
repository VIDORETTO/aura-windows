---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 041-higiene-da-captura
type: delivery
status: done
ticket_revision: 5
requires: ["TK-001"]
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-004"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-audio/src/recorder.rs", "crates/aura-audio/tests/recording.rs", "crates/aura-app/src/recorder.rs"]
verification_status: passed
---





# TK-002 — Áudio com a política de retenção viva (A-03)

## Objetivo e limites

O `AudioSegmentWriter` lê a política compartilhada a cada aplicação, e o `CaptureService` passa o mesmo `Arc` da tela.

Não inclui: filtrar a retenção por Fonte, supervisão, formato de áudio.

## Leitura em ordem

1. `crates/aura-audio/src/recorder.rs` → `AudioSegmentWriter::{new, push, flush}` e a chamada a `apply_retention`.
2. `crates/aura-app/src/recorder.rs` → `CaptureService::start_audio` (cópia da política) e `apply` (atualização do `Arc`).
3. `crates/aura-audio/tests/recording.rs` → teste existente (padrão de montagem).

## Decisões já resolvidas

- Tipo: `Arc<Mutex<RetentionPolicy>>`, o mesmo do `RecorderConfig` da tela.
- Liberdade local: momento exato da leitura do `Mutex` (antes de aplicar).
- Descartado: reiniciar o áudio quando muda o Buffer.

## Mapa de alterações

- Existente: `AudioSegmentWriter::new` (assinatura) e o campo `retention`; `CaptureService::start_audio`; o teste existente (construção).
- Novo: teste `live_retention_change_reaches_audio` em `crates/aura-audio/tests/recording.rs`.

## Contrato técnico

- Entradas: `Arc<Mutex<RetentionPolicy>>` compartilhado.
- Saídas: a retenção aplicada ao fechar um Segmento usa o valor atual.
- Invariantes: os chamadores sem compartilhamento podem criar um `Arc` próprio.
- Concorrência: o `Mutex` é mantido só durante a leitura (clone).

## Exemplos de aceite

- **AC-004**: tela `buffer` de 1 min, de 0 a 20 min, no mesmo armazenamento; writer com `Arc` de 5 min alterado para 30 min; um Segmento de áudio fecha em ~20 min → nenhuma tela apagada. `Arc` de volta a 5 min, novo Segmento → tela que termina antes de 15 min apagada.

## Dependências e sequência de execução

Depende de: TK-001 (mesma área de retenção; série).

- [ ] TK-002.1 Teste vermelho com a assinatura nova (falha porque a política copiada apaga).
- [ ] TK-002.2 Green: campo `Arc`, leitura no momento de aplicar, `start_audio` passa o `Arc`.
- [ ] TK-002.3 Regressão `cargo test -p aura-audio` e `-p aura-app`.
- [ ] TK-002.4 Evidência e checkpoint.

## Validação

- Comando: `cargo test -p aura-audio`; `cargo test -p aura-app`.
- Estado esperado: passam; o novo teste falha antes da correção pelo motivo esperado.
- Distinguir defeito de ambiente: falha de compilação pela assinatura é passo intermediário, não red comportamental. O red vale com a assinatura nova e o comportamento antigo (clonar no `new`).

## Condição de retorno à planejadora

Se outro chamador de `AudioSegmentWriter::new` não puder receber um `Arc`, parar e devolver.

## Relatório de saída

Arquivos e símbolos, AC, comandos, `EV`, pendências.
