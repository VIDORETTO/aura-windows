---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 038-restaurar-seletores-modelo-raciocinio
type: delivery
status: done
ticket_revision: 18
requires: []
requirement_refs: ["FR-001", "FR-002", "FR-003"]
acceptance_refs: ["AC-001", "AC-002", "AC-003", "AC-004"]
spec_revision: 1
plan_revision: 3
owned_areas: ["apps/desktop/src/overlay/Header.tsx", "apps/desktop/src/overlay/ModelPicker.tsx", "apps/desktop/src/overlay/session.ts", "apps/desktop/src/overlay/Composer.test.tsx", "apps/desktop/src/overlay/OverlayApp.test.tsx", "apps/desktop/src/i18n", "crates/aura-app/src/host.rs", "crates/aura-app/tests/host.rs"]
verification_status: passed
last_update: "Current253UI/74Host/typecheck,StandardsSpec reviewed; nativeTK003 pending"
---


















# TK-001 — Disponibilizar os seletores e recuperar falhas do catálogo

## Objetivo e limites

Restaurar acesso antes da primeira mensagem e durante Conversa, distinguindo catálogo carregando/vazio/erro, preservando opções válidas e oferecendo repetição. Iniciar pelo diagnóstico delimitado em discovery; componente já existe. Não inclui persistência nova, sidecar, web ou redesenho global.

Autorização: o usuário posteriormente pediu executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Não inclui itens fora da fatia acima.

## Leitura em ordem

1. `AGENTS.md`, `docs/HANDOFF.md`, `CONTEXT.md` e ADRs do plano.
2. `specs/038-restaurar-seletores-modelo-raciocinio/spec.md` r1 e `plan.md` r2.
3. `specs/038-restaurar-seletores-modelo-raciocinio/tdd.md` → casos desta fatia, oráculos e seams propostas.
4. `apps/desktop/src/overlay/Header.tsx` → Header e controles compactos.
5. `apps/desktop/src/overlay/ModelPicker.tsx` → ModelPicker e modelos/esforços.
6. `apps/desktop/src/overlay/session.ts` → loadCatalog/catalogRequest.
7. `crates/aura-app/src/host.rs` → Host::models.
8. `apps/desktop/src/overlay/Composer.test.tsx` → describe model picker.

## Decisões já resolvidas

Abordagem/contratos e limites do plano r2 são os desta fatia. Seams documentadas adotadas no pedido de execução dos esforços/tickets; aplicar TDD na seam pública. Não alterar comportamento para fazer fixture passar.

Liberdade local: nomes privados, organização interna e mensagens redigidas dentro do contrato. Alternativas descartadas no plano. Dependência/contrato material novo retorna à planejadora.

## Mapa de alterações

Existentes: Header/ModelPicker/loadCatalog e testes UI; novos estados de carregamento/erro/retry previstos. Host::models só se reprodução exigir classificação de erro. Catálogo curado e IPC existentes preservados.

Owned areas delimitam responsabilidade, não autorizam editar toda a pasta. Um ticket por vez; preservar trabalho preexistente. Arquivos/símbolos de predecessores são previstos, existentes somente após sua implementação.

## Contrato técnico

Entradas: provedor atual, catálogo ModelInfo[], sucesso/erro/pendência IPC, Overlay compacto/expandido. Saídas: controle acionável, estados explícitos, últimas opções válidas e retry. Invariantes: provedor de Conversa fixo, nenhum modelo fictício confirmado, nenhuma resposta obsoleta. Efeitos: consulta ao catálogo apenas; preserve escolhas ao falhar. Concorrência: request novo vence antigo; modelos/provedores atualizam independentemente.

Erros, unidades, limites, efeitos e compatibilidade seguem spec/plan e, na web, contracts.md. Nenhuma credencial em UI/config/log. Nenhum estado verificado sem execução.

## Exemplos de aceite

- **AC-001**: QA/qa-reasoner, compacto sem Conversa → clique abre diálogo antes de conversation_start.
- **AC-002**: Conversa expandida → modelo/esforço disponíveis, provedor bloqueado.
- **AC-003**: pendente/erro/sucesso [] → três estados distintos e botão sempre presente.
- **AC-004**: A válido + refresh rejeitado → A/escolha conservados; retry B → diálogo atualiza; B antes de A → B permanece; erro ChatGPT não apaga BYOK.

Oráculos: literais da spec/fixtures sintéticas do tdd.md; não recompor algoritmo nem gerar esperado da implementação.

## Dependências e sequência de execução

Depende de: nenhum.

- [ ] TK-001.1 Executar `python .hybrid/hybrid.py package --project . --effort 038-restaurar-seletores-modelo-raciocinio --ticket TK-001 --json`; exigir ready:true e insumos atuais antes de implementar.
- [ ] TK-001.2 Confirmar seams; primeiro caso do tdd.md → red comportamental, não falta de ambiente.
- [ ] TK-001.3 Implementar mínimo para green, um caso por vez. Caso já verde registra cobertura sem fabricar red.
- [ ] TK-001.4 Executar regressões/integração abaixo, resultados e limitações.
- [ ] TK-001.5 `evidence add` após execução real, `ticket update` para estado, checkpoint, revisão antes de done; projeções por render.

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
