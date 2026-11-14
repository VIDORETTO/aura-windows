# TDD

AC-001: `plan` com 10 Segmentos `continuous` (dias 0–9, 1 dia cada), `manual = false`, política padrão, `now` = 10 dias. Com 2 GB cada → `[d0, d1, d2]`; com 3 GB cada → `[d0, d1, d2, d3]`. Red atual: lista vazia.

AC-002: os mesmos dias como `recording`/`manual = true` → nada; com `apply_to_manual = true` → 4. `buffer` de 5 min continua com o caso existente (89 de 120).

AC-003: `continuous` de tela (3 GB/dia, dias 0–9) + `continuous` de microfone (1 GB/dia, dias 5–9), nessa ordem de inserção, política padrão, `now` = 10 dias. Literal calculado à parte: total de 35 GB; idade apaga tela d0–d2 (35 → 26 GB); tamanho apaga tela d3 e d4 (26 → 23 → 20 GB); o microfone fica. Resultado: tela d0–d4. Empate de início desempata pela ordem de inserção (`ORDER BY start_ms, rowid` no armazenamento).

AC-004: `SegmentStore` em memória com tela `buffer` de 0 a 20 min (Segmentos de 1 min). Writer de áudio criado com `Arc` de 5 min, que depois muda para 30 min. Um Segmento de áudio que fecha em 20 min não apaga nenhuma tela. Com o `Arc` de volta a 5 min, o Segmento seguinte apaga a tela que termina antes de 15 min. Red atual: o primeiro passo já apaga (política copiada).

AC-005: `Sealer` que falha na 1ª selagem; o gravador, com relógio pausado do tokio, produz Segmentos depois → a lista do armazenamento não fica vazia e a tarefa continua viva. Red atual: a tarefa encerra.

AC-006: `Sealer` que sempre falha → depois de 5 Segmentos, `is_finished()` é verdadeiro. `next_backoff`: 15 s → 30 s → 60 s → … → 900 s (teto); zera após sucesso. `active()`/`supervise` no `aura-app` se a plataforma de teste permitir.

AC-007: conjuntos determinísticos (3 sementes, 2.000 Segmentos de 3 Fontes e 3 kinds, `manual` alternado), 4 políticas. Os ids apagados pelo armazenamento são iguais aos de `plan`.

AC-008: um `.seg` órfão e um `.seg.tmp` criados no diretório continuam depois de `apply_retention` e somem com `sweep_orphans`. Uma linha cujo arquivo foi removido some com `sweep_orphans`.

AC-009: `range` em Segmentos de tela de 0 a 100 s (10 s cada), com `from` = 25 s e `to` = 55 s → os Segmentos que começam em 20, 30, 40 e 50 s.
