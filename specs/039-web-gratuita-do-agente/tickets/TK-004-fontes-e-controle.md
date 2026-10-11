---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 039-web-gratuita-do-agente
type: delivery
status: done
ticket_revision: 33
requires: ["TK-003", "TK-007"]
requirement_refs: ["FR-005", "FR-006", "FR-007"]
acceptance_refs: ["AC-008", "AC-009", "AC-010", "AC-011", "AC-012"]
spec_revision: 1
plan_revision: 11
owned_areas: ["crates/aura-core/src/settings.rs", "crates/aura-app/src/events.rs", "crates/aura-app/src/host.rs", "crates/aura-app/src/web.rs", "crates/aura-app/src/web_history.rs", "crates/aura-app/src/web_history_tests.rs", "crates/aura-app/src/lib.rs", "crates/aura-codex/src/service.rs", "crates/aura-app/tests/real_app_server.rs", "crates/aura-app/tests/ipc_contract.rs", "crates/aura-web", "apps/desktop/src", "apps/desktop/src-tauri/src/commands.rs"]
verification_status: passed
last_update: Plan11 scoped UI/source/toggle proof renewed. No new UI implementation required.
---
































# TK-004 — Exibir fontes e dar controle da web ao usuário

## Objetivo e limites

Atividade/fontes/citações acessíveis, webEnabled, cancelamento ao desativar, cache isolado e histórico/efemeridade. Não inclui páginas completas persistidas, credenciais de busca ou editor de MCP duplicado.

Autorização: o usuário pediu executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Não inclui itens fora da fatia.

## Leitura em ordem

1. `AGENTS.md`, `docs/HANDOFF.md`, `CONTEXT.md` e ADRs do plano.
2. `specs/039-web-gratuita-do-agente/spec.md` r1, `plan.md` r9, `contracts.md`, `tdd.md` → contratos e casos.
3. `apps/desktop/src/overlay/Messages.tsx` → itens/markdown.
4. `apps/desktop/src/state/conversation.ts` → blocos e redutor.
5. `apps/desktop/src/settings/General.tsx` → GeneralSection/updateSettings.
6. `crates/aura-core/src/settings.rs` → Settings/Patch/default.
7. `crates/aura-app/src/events.rs` → HostEvent.
8. `crates/aura-app/tests/ipc_contract.rs` → dourado.
9. `apps/desktop/src/ipc/contract.test.ts` → contrato TS.

## Decisões já resolvidas

Abordagem/contratos/limites do plano r9. Seams públicas confirmadas para controles/fontes e histórico. Testes de comportamento já executados; preservar os oráculos. A entrega transitória de outputs/argumentos/raciocínio foi concluída em TK007 com EV068, sem dispensa de aceite. Renovar os aceites de interface deste ticket antes de done.

Liberdade local: nomes privados/organização e mensagens redigidas dentro do contrato. Alternativas no plano. Dependência/contrato material novo retorna à planejadora.

## Mapa de alterações

Novo WebSources.tsx, evento webSource/DTOs, pref webEnabled. Settings General/i18n/state/IPC/mock/dourado atualizados juntos; aura-web cache/disabled/budget integra toggle. Novo web_history.rs persiste metadados citados em Store, sem snippet; CodexService fornece IDs internos para correlação, Host expõe sources opcional e shell encaminha o DTO. Persistência seletiva do Aura é complementada pela entrega transitória TK007/ADR0010; os gates falhos anteriores permanecem históricos, sem alteração de seus oráculos.

Owned areas delimitam responsabilidade; um ticket por vez, preservando trabalho preexistente. Arquivos/símbolos dos predecessores são previstos; existentes após sua implementação.

## Contrato técnico

Prefs antigas default true; false cancela e bloqueia rede/cache/nativo antes de resultado. Fontes validadas em registry, URLs HTTP(S) antes de opener; HTML renderizado como texto. Cache15min32entradas8MiB por Conversa/32MiBglobal; logs sem conteúdo; Conversa efêmera sem persistência. IDs de Turno não aceitos da ferramenta.

Erros/unidades/limites/efeitos/compatibilidade seguem spec/plan/contracts. Segredos fora de UI/config/log. Nenhuma evidência inventada.

## Exemplos de aceite

- **AC-008**: W999 nunca badge válido; texto externo não altera permissões.
- **AC-009**: atividade/painel pt-BR/en/teclado; clique URL segura; histórico sem rede e efêmera sem escrita.
- **AC-010**: configuração antiga true → false cancela/bloqueia tudo incluindo cache; true permite de novo.
- **AC-011**: mesma Conversa T+14 cached, T+16 rede; refresh/outro ID não reutiliza indevidamente; limites LRU.
- **AC-012**: quarto search/sétimo fetch → limite; novo Turno via Host reseta; logs/payload sem histórico/segredo.

Oráculo independente: literais de spec e fixtures sintéticas de tdd.md; sem recompor algoritmo ou esperado da implementação.

## Dependências e sequência de execução

Depende de TK-003 e TK-007 em done: serviço/contrato do predecessor e entrega transitória pelo gateway são necessários. A solução e seu protótipo estão registrados em `../persistence-research.md`; retornar ao gate integral deste ticket após TK-007, sem dispensar a retenção normal.

- [x] TK-004.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-004 --json`; ready:true e inputs atuais antes de implementar.
- [x] TK-004.2 Confirmar seams, primeiro caso → red comportamental, não ambiente.
- [x] TK-004.3 Green mínimo, um caso por vez; caso já verde é cobertura, não red artificial.
- [x] TK-004.4 Regressões/integração e resultado com limitações.
- [x] TK-004.5 Evidence add só após execução real, ticket update/checkpoint, revisão antes de done, projeções por render.

## Validação

Diretório: raiz. Comandos de verificação:

```powershell
cargo test -p aura-app --test web_tools
cargo test -p aura-app --test ipc_contract
pnpm -C apps/desktop test
pnpm -C apps/desktop typecheck
```

Renovação atual:504testes workspace (incluindo14web_tools,74Host,2IPC e WebHistory real),25integrações fixadas,253UI/33arquivos e typecheck passaram. Clippy workspace/all-targets sem desktop,fmt/diff passaram. Identidade entre Turnos/reinício,citação após toggle,exclusão/isolamento de metadata,limites do registry e efemeridade estão cobertos. Os gates de retenção anteriores EV023/024 são históricos; entrega transitória atual e revisão em EV068. Fontes/atividade/abertura por teclado foram verificadas em pt-BR/en na seam renderizada OverlayApp; dados inválidos/HTML/falsoW999 não criam fonte verificada. Detalhes atuais em `../progress-tk004.md`.

Toolchain projeto; Windows/MSVC/WebView2/WebDriver para E2E; AURA_CODEX_BIN fixado verificado para real. Falta de ferramenta/conta/rede é not_run/impedimento, não aprovação. Dourado/real/benchmark seguem plano/TDD, sem dumps de conteúdo/chaves.

## Condição de retorno à planejadora

Contrato real incompatível, predecessor não done, input mudou, necessidade de pago/JS/upgrade, mudança de comportamento/autoridade → devolver caminho/símbolo/resultado. Preservar progresso; não ampliar escopo/reduzir oráculo.

## Relatório de saída

Arquivos/símbolos/AC/comandos executados/EV/revisão/limites/desvios/próxima ação em `../progress-tk004.md`. Implementação e aceites deste ticket renovados, com gate de entrega transitória EV068 e UI/tipos atuais. Build/E2E nativo da pesquisa e síntese permanecem em TK005; live TK006 e DPI038 não são aprovados pela seam UI. Done exige a evidência atual e revisão separada Standards/Spec.
