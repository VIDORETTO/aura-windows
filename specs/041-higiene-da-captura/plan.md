---
schema: hybrid/plan
schema_version: 1.0
effort_id: 041-higiene-da-captura
revision: 1
spec_revision: 1
status: ready
---

# Plano — higiene da captura

Consome o spec r1; baseline git `c4b34f2` (branch `claude/segundo-cerebro-fase0`). Quatro tickets em série, porque tocam os mesmos arquivos. Sem dependência nova, migração, IPC ou API do Windows.

## Módulos e abordagem

**TK-001 (A-01), `crates/aura-capture/src/retention.rs`.** `plan` passa a considerar "sujeito a limite" o Segmento com `kind == "continuous"`, ou `kind == "recording"` e (`!manual` ou `apply_to_manual`). O laço de idade e tamanho existente fica como está: ordem por `start_ms`, soma global e exclusão do mais antigo primeiro. O comentário de `SegmentMeta::kind` passa a listar `continuous`. O teste antigo com `recording`/`manual = false` (combinação que não existe em produção) continua válido e ganha os casos de produção.

**TK-002 (A-03), `crates/aura-audio/src/recorder.rs` e `crates/aura-app/src/recorder.rs`.** `AudioSegmentWriter::new` recebe `Arc<Mutex<RetentionPolicy>>` no lugar de um valor e lê a política no momento de aplicar. `CaptureService::start_audio` passa `self.retention.clone()`. O teste existente em `crates/aura-audio/tests/recording.rs` muda só a construção.

**TK-003 (A-04), `crates/aura-capture/src/segments.rs`.**

- `apply_retention(policy, now)` deixa de listar a tabela e de varrer pastas. Ela:
  1. apaga os `buffer` com `end_ms < now − buffer_ms`;
  2. calcula, por agregação, o total de bytes e o número de Segmentos vencidos por idade no conjunto sujeito a limite (mesmo `JOIN recordings` para `manual`);
  3. se sobrou algo a apagar, percorre os candidatos por `start_ms` com a mesma regra de `plan` e para quando o total fica ≤ teto e não restam vencidos;
  4. apaga em lote (linha e arquivo).
- A equivalência com `plan` é o oráculo.
- `sweep_orphans()` passa a ser público e separado: remove `.seg` sem linha, `.seg.tmp` e linhas sem arquivo. `SegmentRecorder` o chama ao iniciar e depois a cada 1 h do relógio injetado.
- `range` filtra com `WHERE source = ? AND end_ms > ? AND start_ms < ?`.
- `delete` usa uma consulta por lote.
- Medição: teste `#[ignore]` com 60.000 Segmentos `continuous`, que imprime o tempo da retenção antes e depois. Roda com `--ignored --nocapture` e entra como evidência (SC-002).

**TK-004 (A-02).**

- `crates/aura-capture/src/segments.rs`: no laço do `SegmentRecorder`, erro de `encoder.push`, `encoder.flush` ou `store_seg` vira `tracing::warn!` e incrementa um contador de falhas seguidas, que zera quando um Segmento é gravado. Com 5 falhas seguidas, a tarefa encerra com o último erro. Ganha `SegmentRecorder::is_finished()`.
- `crates/aura-app/src/recorder.rs`:
  - `active()` não conta um gravador de tela encerrado;
  - novo `CaptureService::supervise(now_ms) -> Option<Vec<String>>`: se a política quer tela e o gravador não está vivo, tenta reiniciar quando o recuo venceu (15 s, 30 s, 60 s… teto 15 min, reiniciado ao gravar um Segmento) e devolve a nova lista de Fontes ativas quando ela mudou.
- `crates/aura-app/src/host.rs`: o laço de 15 s chama `h.supervise_capture()`, que envia `HostEvent::Privacy` com a lista nova quando muda.
- O cálculo do recuo é uma função pura (`next_backoff`) com testes de literais.

Alternativas descartadas:
- Teto de GB por Fonte: muda o significado do controle.
- Reiniciar dentro da própria tarefa do gravador: esconderia o estado do host.
- Thread própria para a captura de tela: escopo de outro ticket, registrado como pendência.

## Seams, TDD e validação

Seams públicas:
- `aura_capture::retention::plan`;
- `aura_capture::segments::{SegmentStore, SegmentRecorder}`, com `Store::open_in_memory`, `tempfile`, `NullEncoder`/encoder de teste e um `Sealer` de teste que falha sob comando (o `Sealer` é injetável e varia de verdade: armazenamento e cifra);
- `aura_audio::recorder::AudioSegmentWriter`;
- `CaptureService::{active, supervise}`, se a plataforma de teste do `aura-app` permitir montar o serviço. Se não permitir, o reinício é coberto por `next_backoff` puro + teste do `SegmentRecorder`, e a integração no host é verificada na compilação e no teste de host existente. Isso fica registrado no ticket.

Nada de mock de módulo interno. Um caso por vez, red antes de green; erro de ambiente não é red.

Comandos (raiz do repositório, Windows):
- `cargo test -p aura-capture`
- `cargo test -p aura-audio`
- `cargo test -p aura-app`
- `cargo test --workspace --exclude aura-desktop`
- `cargo clippy -p aura-capture -p aura-audio -p aura-app --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- Medição: `cargo test -p aura-capture --release retention_cost -- --ignored --nocapture`

## Gates

- G1: spec aceito (pedido explícito do usuário de abrir a Fase 0).
- G2: seams e oráculos definidos acima.
- G3: `package` pronto antes de implementar cada ticket.
- Evidência só depois de execução real; revisão Standards e Spec separadas antes de `done`.
- Não instalar nem publicar.
