# Review — 040-overlay-estavel-no-monitor

Baseline fixa: `ed9a9654c6e76e48186fc010a12eba77023b00f5`. Revisão local pelo agente implementador; não se atribui aprovação a outro reviewer.

Escopo observado: `git rev-parse HEAD`, `git diff <baseline>...HEAD`, `git diff --cached`, `git diff`, `git status --short`. Committed/staged vazios nos caminhos da correção; unstaged core/overlay/qa_resize e novos qa_foreground/teste/specs incluídos. Diffs preexistentes de038/039 preservados; sem commit/reset/stash. IDs e oráculos mantidos.

## Standards

Regra pura e pública em aura-core, sem Windows; efeitos de janela continuam no shell Tauri. `set_mode` lê geometria antes de aplicar mínimo e `remember_placement` usa a mesma regra geométrica. Nenhuma nova API de SO fora de aura-win, formato IPC/persistência, dependência/licença ou atualização do pin. show oculto conserva a abertura contextual. Helpers somente em examples, fixture vazia própria com foreground verificado, movimento restrito a basename QA/janela única/client bounds. Nenhuma alteração da configuração pessoal ou encerramento da conversa em produção.

EV-003:56core/1desktop/253UI,typecheck,fmt,clippy e diff passaram. Artefato normal EV-004 separado do QA com identidade própria; hashes/procedimentos/limites em qa.md. Nenhum achado Standards bloqueante no escopo.

## Spec

FR-001/AC-001: red nativo EV-001 reproduziu B→A por Aplicativo anterior; green EV-002 mantém B/âncora nas quatro alternâncias mesmo com posição expandida antiga distinta. Transição real Turno→Minibar→Conversa conserva (2000,80), com aplicativo anterior em A e resposta pelo Codex/Host. Só rede do modelo externa é scripted.

FR-002/AC-002: guardar B não sobrescreve A; reabrir contextual em cada monitor recupera larguras820/700 e posições2000/80 e80/80. IPC público e janelas reais; nenhum oráculo derivado do algoritmo ou banco lateral.

FR-003/AC-003: origem negativa, DPI distintos, fronteira, empate e ausência exercitados na seam pura; regressões de placement cobrem mínimos/DPI/monitor ausente. Inspeção do shell confirma que None preserva fallback existente e que âncora atual só se aplica com monitor geométrico presente. Resize9/9 e seletor2/2 renovados após040; ausência de desconexão física/DPI nativo150/200 declarada, sem simular aprovação desses gates de038.

O preparo inicial de saved com altura722 não cabia em B720; y0 era clamp exigido, não salto de monitor. Novo preparo comprova altura500, preserva as expectativas literais de âncora. Interferências de focus/origin/botão/Snap não foram classificadas como reds. Nenhum aceite enfraquecido. SC-001 entregue; SC-002 é observação cotidiana posterior. Sem achado Spec bloqueante no contrato040.

## Evidence and disposition

EV-001 failed registra versões anteriores; EV-002 passed integra3casos nativos+11regressões; EV-003 passed cobre regra/fallback/regressões; EV-004 passed compila artefato normal, sem substituir prova de comportamento. O teste atual e QA build têm os mesmos fontes produtivos de geometria do executável entregue.

Revisão libera TK-001 de040 para verified/done. 039já fechado por confirmação manual explícita do destino W1; 038TK-003 permanece parcial por150/200. Nenhuma aprovação fictícia de navegador/DPI ou instalação automática.
