---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 038-restaurar-seletores-modelo-raciocinio
type: delivery
status: blocked
ticket_revision: 11
requires: ["TK-002"]
requirement_refs: ["FR-001", "FR-007"]
acceptance_refs: ["AC-008", "AC-009"]
spec_revision: 1
plan_revision: 3
owned_areas: ["apps/desktop/src/overlay/Header.tsx", "apps/desktop/src/ui/Popover.tsx", "apps/desktop/src/ui/floating.tsx", "apps/desktop/src/overlay/OverlayApp.tsx", "apps/desktop/src-tauri/src/overlay.rs", "apps/desktop/e2e/specs/model-picker-recovery.e2e.ts", "apps/desktop/e2e/specs/resize.e2e.ts"]
verification_status: partial
last_update: EV017renova teclado/foco pt-BR/en; EV018renova100%480px e resize9/9 após040. Restam150/200 nativos; nenhum aceite reduzido.
---











# TK-003 — Provar os seletores na janela Windows e impedir cortes

## Objetivo e limites

Entregar controle acionável a 480 px lógicos, menu visível no monitor, teclado/DPI/idiomas; corrigir corte comprovado e validar regressão nativa. Não inclui redesenho de janelas ou publicar instalador.

Autorização: o usuário posteriormente pediu executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Não inclui itens fora da fatia acima.

## Leitura em ordem

1. `AGENTS.md`, `docs/HANDOFF.md`, `CONTEXT.md` e ADRs do plano.
2. `specs/038-restaurar-seletores-modelo-raciocinio/spec.md` r1 e `plan.md` r2.
3. `specs/038-restaurar-seletores-modelo-raciocinio/tdd.md` → casos desta fatia, oráculos e seams propostas.
4. `apps/desktop/src/ui/floating.tsx` → Floating e floatBottom.
5. `apps/desktop/src/ui/Popover.tsx` → usePopover/PopoverPanel.
6. `apps/desktop/src/overlay/OverlayApp.tsx` → useAutoHeight e Header.
7. `apps/desktop/src-tauri/src/overlay.rs` → apply_minimum.
8. `apps/desktop/e2e/wdio.conf.ts` → app/driver/demo.
9. `apps/desktop/e2e/specs/resize.e2e.ts` → padrão de geometria.

## Decisões já resolvidas

Abordagem/contratos e limites do plano r2 são os desta fatia. Seams documentadas adotadas no pedido de execução dos esforços/tickets; aplicar TDD na seam pública. Não alterar comportamento para fazer fixture passar.

Liberdade local: nomes privados, organização interna e mensagens redigidas dentro do contrato. Alternativas descartadas no plano. Dependência/contrato material novo retorna à planejadora.

## Mapa de alterações

Existentes: Header/StatusBadges/Floating/Popover/useAutoHeight/apply_minimum somente onde reprodução exigir. Novo: e2e/specs/model-picker-recovery.e2e.ts. Fora: release/updater e contas.

Owned areas delimitam responsabilidade, não autorizam editar toda a pasta. Um ticket por vez; preservar trabalho preexistente. Arquivos/símbolos de predecessores são previstos, existentes somente após sua implementação.

## Contrato técnico

Entradas: 480 px lógicos, nomes longos, compacto/expandido, DPI 100/150/200%, mouse/teclado. Saídas: botão com retângulo positivo e menu acessível sem corte. Escape fecha menu e devolve foco, mantém Overlay. Sem diminuir mínimo nativo para fugir da prova; foco/drag region precisam ser observados na janela real.

Erros, unidades, limites, efeitos e compatibilidade seguem spec/plan e, na web, contracts.md. Nenhuma credencial em UI/config/log. Nenhum estado verificado sem execução.

## Exemplos de aceite

- **AC-008**: cada DPI + 480 px + nomes longos → botão/seta acionáveis, última opção alcançável dentro da área útil.
- **AC-009**: Tab/Enter escolher/Escape → foco no botão e Overlay aberto; pt-BR/en com paridade.

Oráculos: literais da spec/fixtures sintéticas do tdd.md; não recompor algoritmo nem gerar esperado da implementação.

## Dependências e sequência de execução

Depende de: TK-002 em done; comportamento/código necessários para esta fatia.

- [ ] TK-003.1 Executar `python .hybrid/hybrid.py package --project . --effort 038-restaurar-seletores-modelo-raciocinio --ticket TK-003 --json`; exigir ready:true e insumos atuais antes de implementar.
- [ ] TK-003.2 Confirmar seams; primeiro caso do tdd.md → red comportamental, não falta de ambiente.
- [ ] TK-003.3 Implementar mínimo para green, um caso por vez. Caso já verde registra cobertura sem fabricar red.
- [ ] TK-003.4 Executar regressões/integração abaixo, resultados e limitações.
- [ ] TK-003.5 `evidence add` após execução real, `ticket update` para estado, checkpoint, revisão antes de done; projeções por render.

## Validação

Diretório: raiz. Comandos futuros:

```powershell
pnpm -C apps/desktop tauri build --no-bundle --features demo
pnpm -C apps/desktop/e2e test -- --spec ./specs/model-picker-recovery.e2e.ts
pnpm -C apps/desktop/e2e test -- --spec ./specs/composer.e2e.ts --spec ./specs/model-effort.e2e.ts --spec ./specs/model-efforts.e2e.ts --spec ./specs/profile-model.e2e.ts --spec ./specs/resize.e2e.ts
pnpm -C apps/desktop test
pnpm -C apps/desktop typecheck
```

Esperado: exemplos/efeitos comprovados, regressões verdes. Comandos planejados abaixo; resultados atuais somente nas evidências geradas pelo runner. Targets novos serão criados na implementação; ausência atual não é red.

Ambiente: toolchain do projeto; Windows/MSVC/WebView2/WebDriver para E2E; AURA_CODEX_BIN aponta para fixado verificado nos testes reais. Falta de ferramenta/conta/rede é impedimento/not_run, não aprovação. Dourado, app-server e benchmark seguem plan/tdd, sem dumps de conteúdo/chaves.

## Condição de retorno à planejadora

Contrato real incompatível, predecessor não done, insumo mudou, necessidade de pago/JS/upgrade ou mudança de comportamento/autoridade → retornar com caminho/símbolo/resultado. Preservar progresso; não ampliar escopo nem reduzir oráculo.

## Relatório de saída

Arquivos/símbolos, AC, comandos executados, EV, revisão testada, limites, desvios e próxima ação. Done só com evidência passada e revisão requerida. Pacote atual sem implementação/evidência de aceites.
