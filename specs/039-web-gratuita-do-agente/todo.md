# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [x] TK-001 — TK-001 — Buscar fontes gratuitamente com alternativa limitada
Status: `done` | Bloqueado por: nenhum

- [x] TK-001.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-001 --json`; exigir ready:true e insumos atuais antes de implementar.
- [x] TK-001.2 Confirmar seams; primeiro caso do tdd.md → red comportamental, não falta de ambiente.
- [x] TK-001.3 Implementar mínimo para green, um caso por vez. Caso já verde registra cobertura sem fabricar red.
- [x] TK-001.4 Executar regressões/integração abaixo, resultados e limitações.
- [ ] TK-001.5 `evidence add` após execução real, `ticket update` para estado, checkpoint, revisão antes de done; projeções por render.

## [x] TK-002 — TK-002 — Ler conteúdo principal de páginas públicas com continuidade
Status: `done` | Bloqueado por: TK-001

- [x] TK-002.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-002 --json`; ready:true e inputs atuais antes de implementar.
- [x] TK-002.2 Confirmar seams, primeiro caso → red comportamental, não ambiente.
- [x] TK-002.3 Green mínimo, um caso por vez; caso já verde é cobertura, não red artificial.
- [x] TK-002.4 Regressões/integração e resultado com limitações.
- [ ] TK-002.5 Evidence add só após execução real, ticket update/checkpoint, revisão antes de done, projeções por render.

## [x] TK-003 — TK-003 — Integrar pesquisa e leitura ao agente nos modos existentes
Status: `done` | Bloqueado por: TK-002

- [x] TK-003.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-003 --json`; ready:true e inputs atuais antes de implementar.
- [x] TK-003.2 Confirmar seams, primeiro caso → red comportamental, não ambiente.
- [x] TK-003.3 Green mínimo, um caso por vez; caso já verde é cobertura, não red artificial.
- [x] TK-003.4 Regressões/integração e resultado com limitações.
- [x] TK-003.5 Evidence add só após execução real, ticket update/checkpoint, revisão antes de done, projeções por render.

## [x] TK-006 — TK-006 — Avaliar relevância e extração na internet real
Status: `done` | Bloqueado por: TK-003

- [x] TK-006.1 Package ready:true e graph antes de criar o harness.
- [x] TK-006.2 Inspecionar seams e oráculos; execução de qualidade não usa internet para fabricar red unitário.
- [x] TK-006.3 Executar 12 buscas; classificar top5 por domínio e conteúdo que responde ao tema.
- [x] TK-006.4 Inspecionar/fixar dez alvos HTML estáticos antes de extrair; executar até dez leituras e registrar os resultados.
- [x] TK-006.5 Evidence add após execução, revisão Standards/Spec, estados pelo runner e projeções por render.

## [x] TK-007 — TK-007 — Entregar resultados web em memória pelo gateway
Status: `done` | Bloqueado por: TK-003

- [x] TK-007.1 Package ready:true, graph e inputs atuais; primeiro teste na fronteira HTTP com red comportamental.
- [x] TK-007.2 Implementar resolução exata e escopo; casos de mensagem/terceiro/fake nonce/cross-thread um por vez.
- [x] TK-007.3 Retenção em orçamento real e ciclo de vida pelo Host/WebService; red → green por caso.
- [x] TK-007.4 Teste normal de retenção, reinício/novo turno e compactação reais; nove combinações de protocolo/modo e regressões existentes.
- [x] TK-007.5 Evidências reais, revisão Standards/Spec, atualização do ADR e checkpoint; retorno ao TK-004 apenas após done.

## [x] TK-004 — TK-004 — Exibir fontes e dar controle da web ao usuário
Status: `done` | Bloqueado por: TK-003, TK-007

- [x] TK-004.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-004 --json`; ready:true e inputs atuais antes de implementar.
- [x] TK-004.2 Confirmar seams, primeiro caso → red comportamental, não ambiente.
- [x] TK-004.3 Green mínimo, um caso por vez; caso já verde é cobertura, não red artificial.
- [x] TK-004.4 Regressões/integração e resultado com limitações.
- [x] TK-004.5 Evidence add só após execução real, ticket update/checkpoint, revisão antes de done, projeções por render.

## [x] TK-005 — TK-005 — Provar pesquisa integrada e qualidade da rota gratuita
Status: `done` | Bloqueado por: TK-004, TK-006

- [ ] TK-005.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-005 --json`; ready:true e inputs atuais antes de implementar.
- [ ] TK-005.2 Confirmar seams, primeiro caso → red comportamental, não ambiente.
- [ ] TK-005.3 Green mínimo, um caso por vez; caso já verde é cobertura, não red artificial.
- [ ] TK-005.4 Regressões/integração e resultado com limitações.
- [ ] TK-005.5 Evidence add só após execução real, ticket update/checkpoint, revisão antes de done, projeções por render.
