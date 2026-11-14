# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [x] TK-001 — TK-001 — Retenção do modo Contínuo (A-01)
Status: `done` | Bloqueado por: nenhum

- [ ] TK-001.1 Teste AC-001 vermelho (lista vazia hoje).
- [ ] TK-001.2 Green mínimo em `plan`.
- [ ] TK-001.3 AC-002 e AC-003; regressão `cargo test -p aura-capture`.
- [ ] TK-001.4 Evidência e checkpoint.

## [x] TK-002 — TK-002 — Áudio com a política de retenção viva (A-03)
Status: `done` | Bloqueado por: TK-001

- [ ] TK-002.1 Teste vermelho com a assinatura nova (falha porque a política copiada apaga).
- [ ] TK-002.2 Green: campo `Arc`, leitura no momento de aplicar, `start_audio` passa o `Arc`.
- [ ] TK-002.3 Regressão `cargo test -p aura-audio` e `-p aura-app`.
- [ ] TK-002.4 Evidência e checkpoint.

## [x] TK-003 — TK-003 — Retenção por consulta e órfãos com cadência (A-04)
Status: `done` | Bloqueado por: TK-002

- [ ] TK-003.1 AC-008 vermelho (hoje `apply_retention` remove órfãos).
- [ ] TK-003.2 Green: `sweep_orphans` separado; o gravador varre no início e de hora em hora.
- [ ] TK-003.3 AC-007 (equivalência) e AC-009, com retenção por consulta e `range` em SQL.
- [ ] TK-003.4 Medição antes e depois (60.000 `continuous`), regressão completa, evidência.

## [x] TK-004 — TK-004 — Gravador de tela supervisionado (A-02)
Status: `done` | Bloqueado por: TK-003

- [ ] TK-004.1 AC-005 vermelho (a tarefa encerra no 1º erro).
- [ ] TK-004.2 Green: aviso + contador; `is_finished`.
- [ ] TK-004.3 AC-006: `next_backoff` puro; `active`/`supervise`; chamada no host.
- [ ] TK-004.4 Regressão completa, clippy, fmt, evidência.
