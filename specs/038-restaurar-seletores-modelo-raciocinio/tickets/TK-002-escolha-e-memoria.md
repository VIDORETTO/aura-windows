---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 038-restaurar-seletores-modelo-raciocinio
type: delivery
status: done
ticket_revision: 15
requires: ["TK-001"]
requirement_refs: ["FR-004", "FR-005", "FR-006"]
acceptance_refs: ["AC-005", "AC-006", "AC-007"]
spec_revision: 1
plan_revision: 3
owned_areas: ["apps/desktop/src/overlay/session.ts", "apps/desktop/src/overlay/ModelPicker.tsx", "apps/desktop/src/overlay/OverlayApp.test.tsx", "apps/desktop/src/i18n", "crates/aura-app/tests/host.rs"]
verification_status: passed
last_update: "Current253UI/74Host/typecheck,StandardsSpec reviewed; nativeTK003 pending"
---















# TK-002 — Aplicar modelo e esforço válidos e restaurar preferências

## Objetivo e limites

Garantir esforço/modelo escolhidos no próximo Turno, preferências existentes por modo/modelo e reconciliação após capacidades confirmadas. Depende de catálogo recuperável do TK-001. Não inclui nova política de acesso, último modelo global ou web.

Autorização: o usuário posteriormente pediu executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Não inclui itens fora da fatia acima.

## Leitura em ordem

1. `AGENTS.md`, `docs/HANDOFF.md`, `CONTEXT.md` e ADRs do plano.
2. `specs/038-restaurar-seletores-modelo-raciocinio/spec.md` r1 e `plan.md` r2.
3. `specs/038-restaurar-seletores-modelo-raciocinio/tdd.md` → casos desta fatia, oráculos e seams propostas.
4. `apps/desktop/src/overlay/session.ts` → effectiveModel, modelCapabilities, setModel/setEffort/setMode, send.
5. `apps/desktop/src/overlay/ModelPicker.tsx` → opções e rótulos.
6. `apps/desktop/src/overlay/OverlayApp.test.tsx` → esforço por modelo/modo.
7. `specs/013-modelos-e-esforco/change.md` → contrato de memória.

## Decisões já resolvidas

Abordagem/contratos e limites do plano r2 são os desta fatia. Seams documentadas adotadas no pedido de execução dos esforços/tickets; aplicar TDD na seam pública. Não alterar comportamento para fazer fixture passar.

Liberdade local: nomes privados, organização interna e mensagens redigidas dentro do contrato. Alternativas descartadas no plano. Dependência/contrato material novo retorna à planejadora.

## Mapa de alterações

Existentes: session.ts/ModelPicker e testes UI/Host para restauração/revalidação. Sem nova migração ou mudança de wire. Fora: catálogo comercial, SIWC e versão do app-server.

Owned areas delimitam responsabilidade, não autorizam editar toda a pasta. Um ticket por vez; preservar trabalho preexistente. Arquivos/símbolos de predecessores são previstos, existentes somente após sua implementação.

## Contrato técnico

Entradas: qa-reasoner [low,high,max], default high, presets Chat low/Tarefa max; resultado bem-sucedido de catálogo. Saídas: UI compatível e conversation_send com modelo/esforço corretos. Erros de refresh não invalidam suporte. Escolha durante execução afeta apenas próximo Turno; guardar presets pelo IPC existente. Banco temporário real e APIs públicas para persistência.

Erros, unidades, limites, efeitos e compatibilidade seguem spec/plan e, na web, contracts.md. Nenhuma credencial em UI/config/log. Nenhum estado verificado sem execução.

## Exemplos de aceite

- **AC-005**: qa-reasoner/max + oi → model=qa-reasoner, effort=max; medium ausente; modelo sem esforços → explicação e esforço null/omitido.
- **AC-006**: Chat low/Tarefa max; troca/reinício/selecionar mesmo modelo → preferências restauradas.
- **AC-007**: sucesso remove max → aviso/default válido; refresh falha → mantém max; catálogo tardio restaura preferência compatível.

Oráculos: literais da spec/fixtures sintéticas do tdd.md; não recompor algoritmo nem gerar esperado da implementação.

## Dependências e sequência de execução

Depende de: TK-001 em done; comportamento/código necessários para esta fatia.

- [ ] TK-002.1 Executar `python .hybrid/hybrid.py package --project . --effort 038-restaurar-seletores-modelo-raciocinio --ticket TK-002 --json`; exigir ready:true e insumos atuais antes de implementar.
- [ ] TK-002.2 Confirmar seams; primeiro caso do tdd.md → red comportamental, não falta de ambiente.
- [ ] TK-002.3 Implementar mínimo para green, um caso por vez. Caso já verde registra cobertura sem fabricar red.
- [ ] TK-002.4 Executar regressões/integração abaixo, resultados e limitações.
- [ ] TK-002.5 `evidence add` após execução real, `ticket update` para estado, checkpoint, revisão antes de done; projeções por render.

## Validação

Diretório: raiz. Comandos futuros:

```powershell
pnpm -C apps/desktop test -- src/overlay/Composer.test.tsx src/overlay/OverlayApp.test.tsx
pnpm -C apps/desktop typecheck
cargo test -p aura-app --test host
```

Esperado: exemplos/efeitos comprovados, regressões verdes. Comandos planejados abaixo; resultados atuais somente nas evidências geradas pelo runner. Targets novos serão criados na implementação; ausência atual não é red.

Ambiente: toolchain do projeto; Windows/MSVC/WebView2/WebDriver para E2E; AURA_CODEX_BIN aponta para fixado verificado nos testes reais. Falta de ferramenta/conta/rede é impedimento/not_run, não aprovação. Dourado, app-server e benchmark seguem plan/tdd, sem dumps de conteúdo/chaves.

## Condição de retorno à planejadora

Contrato real incompatível, predecessor não done, insumo mudou, necessidade de pago/JS/upgrade ou mudança de comportamento/autoridade → retornar com caminho/símbolo/resultado. Preservar progresso; não ampliar escopo nem reduzir oráculo.

## Relatório de saída

Arquivos/símbolos, AC, comandos executados, EV, revisão testada, limites, desvios e próxima ação. Done só com evidência passada e revisão requerida. Pacote atual sem implementação/evidência de aceites.
