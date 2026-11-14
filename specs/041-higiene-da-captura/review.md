# Review — 041-higiene-da-captura

Baseline fixa: `c4b34f24a4bd4748eabbfb14a48160f834046717`. Revisão local feita pelo agente implementador, ticket a ticket; não se atribui aprovação a outro revisor. Escopo de cada rodada: `git rev-parse HEAD`, `git diff <baseline>...HEAD -- <áreas do ticket>`, `git diff --cached`, `git diff` e `git status --short`.

## TK-001 — Retenção do Contínuo (A-01)

HEAD revisado: `67110c3`. Diff: `crates/aura-capture/src/retention.rs` e os artefatos do esforço; nada staged ou unstaged fora disso.

### Standards

- Regra pura em `aura-capture`, sem API do Windows, sem dependência nova, sem mudança de esquema ou IPC.
- `is_limited` é pública para o TK-003 reproduzir a mesma regra no armazenamento; o comentário de `SegmentMeta::kind` foi atualizado.
- O `rustfmt` acusou diferença depois do primeiro commit; corrigido em commit separado (`67110c3`).
- Nenhum achado bloqueante.

### Spec

- **FR-001 / AC-001:** red real (EV-001: `[]` contra `[d0, d1, d2]`) e green (EV-002) com os literais do spec.
- **AC-002:** gravação `manual = true` fica fora do teto e não consome orçamento; com `apply_to_manual`, entra no teto global.
- **AC-003:** teto global sobre tela e microfone, com o literal calculado à parte no `tdd.md`.
- Os oráculos são literais independentes do algoritmo; o caso antigo (`recording`/`manual = false`) continua verde. Nenhum aceite foi enfraquecido.

Disposição: TK-001 liberado para `verified` e `done`.

## TK-002 — Retenção viva no áudio (A-03)

Diff revisado (não commitado na revisão): `crates/aura-audio/src/recorder.rs`, `crates/aura-audio/tests/recording.rs`, `crates/aura-app/src/recorder.rs`.

### Standards

- Mudança de assinatura interna: `AudioSegmentWriter::new` recebe `Arc<Mutex<RetentionPolicy>>`. Há um único chamador de produção (`CaptureService::start_audio`), que passa o mesmo `Arc` da tela.
- O `Mutex` só é mantido durante a clonagem. Sem dependência, IPC ou esquema novos.
- `rustfmt` aplicado antes da evidência.
- Nenhum achado bloqueante.

### Spec

- **FR-002 / AC-004:** red real com a assinatura nova e o comportamento antigo (cópia no `new`), que apagou 14 telas com o Buffer vivo em 30 min. Green: com 30 min nada sai; de volta a 5 min, ficam só as telas que terminam em 16 a 20 min.
- O literal (ficam as telas com fim > 15 min 10 s) foi calculado à parte.
- A regressão do teste do tom continua verde.

Disposição: TK-002 liberado para `verified` e `done`.

## TK-003 — Retenção por consulta e órfãos com cadência (A-04)

Diff revisado: `crates/aura-capture/src/segments.rs`, `crates/aura-audio/src/recorder.rs`, testes `retention_store.rs`, `retention_cost.rs` e `pipeline.rs`. O ticket chegou à revisão 3 para incluir o `AudioSegmentWriter`: sem isso, os órfãos da pasta de áudio deixariam de ser limpos.

### Standards

- SQL montado só com constantes (`SELECT_META`, `LIMITED`) e parâmetros ligados; sem entrada externa no texto do SQL.
- `delete` passou a uma transação; os arquivos saem depois, e uma queda no meio deixa órfão de arquivo, que a varredura limpa.
- A regra "sujeito a limite" existe em dois lugares (`retention::is_limited` e `LIMITED`). O risco de divergência fica coberto pelo teste de equivalência com 24 combinações. Achado de baixa severidade, aceito: documentado no comentário de `LIMITED`.
- Sem dependência nova, esquema, IPC ou API do Windows. `clippy -D warnings` e `rustfmt` limpos.

### Spec

- **FR-004 / AC-007:** equivalência exata com `plan` (3 sementes × 4 políticas × 2.000 Segmentos de 3 Fontes e 3 kinds, `manual` pelo `JOIN`).
- **AC-008:** red real (EV: a retenção removia 3 órfãos); green com a varredura separada; cadência no início e de hora em hora provada pelo relógio injetado.
- **AC-009:** `range` com o literal 20/30/40/50 s.
- **SC-002:** 1.034,5 → 15,5 ms de p50 com 60.000 Segmentos `continuous` em SQLite de arquivo (mesmo PC, mesmo teste).
- A agregação ainda é O(n) sobre o conjunto sujeito a limite, mas sem E/S de arquivos: aceitável para o volume de 7 dias.

Disposição: TK-003 liberado para `verified` e `done`.
