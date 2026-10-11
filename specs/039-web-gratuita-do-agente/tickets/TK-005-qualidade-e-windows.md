---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 039-web-gratuita-do-agente
type: delivery
status: done
ticket_revision: 22
requires: ["TK-004", "TK-006"]
requirement_refs: ["FR-001", "FR-006", "FR-008"]
acceptance_refs: ["AC-001", "AC-009", "AC-013", "AC-014"]
spec_revision: 1
plan_revision: 11
owned_areas: ["crates/aura-app/tests/web_research.rs", "crates/aura-app/tests/real_app_server.rs", "apps/desktop/e2e/specs/web-research.e2e.ts", "apps/desktop/src-tauri/src/main.rs", "apps/desktop/src-tauri/src/e2e_web.rs", "apps/desktop/src-tauri/Cargo.toml", "Cargo.lock", "specs/039-web-gratuita-do-agente/evidence"]
verification_status: passed
last_update: Git LF normalization explicitly revalidated; exact original fingerprints recoverable without behavioral edits. Canonical evidence/current0.3 release documented in041.
---





















# TK-005 — Provar pesquisa integrada e qualidade da rota gratuita

## Objetivo e limites

Provar fluxo completo determinístico e Windows; executar três jornadas de síntese ao vivo e incorporar o gate backend de relevância/extração do TK-006. Não inclui reduzir metas após resultado, exigir conta paga ou publicar instalador.

Autorização: o usuário pediu executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Não inclui itens fora da fatia.

## Leitura em ordem

1. `AGENTS.md`, `docs/HANDOFF.md`, `CONTEXT.md` e ADRs do plano.
2. `specs/039-web-gratuita-do-agente/spec.md` r1, `plan.md` r10, `contracts.md`, `tdd.md` → contratos e casos.
3. `specs/039-web-gratuita-do-agente/tdd.md` → TK-005/consultas/metas/limites.
4. `crates/aura-app/tests/real_app_server.rs` → app-server fixado.
5. `apps/desktop/e2e/wdio.conf.ts` → driver/demo/provedor local.
6. `apps/desktop/e2e/provider.ts` → upstream controlado.
7. `docs/HANDOFF.md` → contrato real e limites Windows.

## Decisões já resolvidas

Abordagem/contratos/limites do plano r10. Seams confirmadas: Host real, processo app-server fixado, gateway e MCP reais; somente provedor externo, rede/DNS e SO são fixtures. Composição nativa no e2e_web.rs compila apenas com feature e2e e opt-in AURA_E2E_WEB=1; requer AURA_CODEX_BIN fixado e aplica HostConfig.web. Não alterar comportamento para fixture passar.

Liberdade local: nomes privados/organização e mensagens redigidas dentro do contrato. Alternativas no plano. Dependência/contrato material novo retorna à planejadora.

## Mapa de alterações

Novos tests/web_research.rs/E2E web-research.e2e.ts e src-tauri/src/e2e_web.rs; complemento real_app_server. main.rs apenas aplica composição sob cfg e2e; Cargo.toml ativa aura-web/futures opcionais do workspace nessa feature, sem biblioteca nova. Evidence somente runner após execução. Correção nova de comportamento retorna ticket dono/spec, sem maquiar gate com fixture alterada.

Owned areas delimitam responsabilidade; um ticket por vez, preservando trabalho preexistente. Arquivos/símbolos dos predecessores são previstos; existentes após sua implementação.

## Contrato técnico

Fixtures sintéticas 42/12/30 no fluxo completo sem internet; live12 consultas fixas, relevância top5 >=10/12, extração alvo >=8/10 HTML estáticos. 12buscas10leituras principais/60requestsHTTP/30min. Não instalar contas/credenciais/pago. Windows activity/citations/cancel e fixado transports cobertos; restrições ambiente registradas.

Erros/unidades/limites/efeitos/compatibilidade seguem spec/plan/contracts. Segredos fora de UI/config/log. Nenhuma evidência inventada.

## Exemplos de aceite

- **AC-001**: sem credencial de busca, fluxo integração usa apenas endpoints gratuitos.
- **AC-009**: janela Windows mostra busca/abertura/fontes, clique/cancel; idioma/efemeridade regressão.
- **AC-013**: literais do TDD passam; benchmark12consultas e10páginas atinge metas ou registra falha/partial; nunca aprovação inferida.
- **AC-014**: fixado com upstreams scripted/protocolos/modos funciona; smoke SIWC real só se conta disponível, sem inferir acesso pelo demo.

Oráculo independente: literais de spec e fixtures sintéticas de tdd.md; sem recompor algoritmo ou esperado da implementação.

## Dependências e sequência de execução

Depende de TK-004 e TK-006 em done. TK-006 executa a avaliação backend de 12 buscas/dez leituras antes prevista aqui. Este ticket conserva síntese de três jornadas, cadeia determinística integrada, build/E2E Windows e convergência final; nenhum gate foi dispensado.

- [ ] TK-005.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-005 --json`; ready:true e inputs atuais antes de implementar.
- [ ] TK-005.2 Confirmar seams, primeiro caso → red comportamental, não ambiente.
- [ ] TK-005.3 Green mínimo, um caso por vez; caso já verde é cobertura, não red artificial.
- [ ] TK-005.4 Regressões/integração e resultado com limitações.
- [ ] TK-005.5 Evidence add só após execução real, ticket update/checkpoint, revisão antes de done, projeções por render.

## Validação

Diretório: raiz. Comandos futuros:

```powershell
cargo test -p aura-app --test web_research -- --ignored --test-threads=1
cargo test -p aura-app --test real_app_server -- --ignored --nocapture
cargo test --workspace --exclude aura-desktop
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check licenses
pnpm -C apps/desktop test
pnpm -C apps/desktop typecheck
pnpm -C apps/desktop tauri build --no-bundle --features demo,e2e
pnpm -C apps/desktop/e2e test -- --spec ./specs/web-research.e2e.ts
```

O target web_research exige AURA_CODEX_BIN apontando para o app-server fixado e AURA_E2E_DIR para TEMP próprio. Seus casos são ignorados por padrão porque exigem esse binário externo; a invocação explícita acima executa todos. Não usam conta/inferência ou internet. Ausência do ambiente não é red comportamental.

Toolchain projeto; Windows/MSVC/WebView2/WebDriver para E2E; AURA_CODEX_BIN fixado verificado para real. Falta de ferramenta/conta/rede é not_run/impedimento, não aprovação. Dourado/real/benchmark seguem plano/TDD, sem dumps de conteúdo/chaves.

## Condição de retorno à planejadora

Contrato real incompatível, predecessor não done, input mudou, necessidade de pago/JS/upgrade, mudança de comportamento/autoridade → devolver caminho/símbolo/resultado. Preservar progresso; não ampliar escopo/reduzir oráculo.

## Relatório de saída

10/10/2026: web_research.rs implementado e executado com o pin rust-v0.159.0. Cinco casos atravessam Host → processo → gateway → MCP → rede pública controlada → provedor → resposta/transcript/fontes: deduplicação/HTML/fatos42/12/30/dois domínios; primário429/fallback gratuito; primeira página403 com ressalva e só W2 citada; interrupção de stream pendente sem fonte lida nem próxima página; sétima leitura recusada mesmo com cache. Cada caso foi executado separadamente, já verde no comportamento existente; nenhum red artificial foi fabricado. Após rustfmt, suíte5/5 em3,57s; Clippy do target -Dwarnings, fmt/diffcheck passaram. Sem mudança produtiva nesta rodada.

Build demo nativo iniciado com `pnpm -C apps/desktop tauri build --no-bundle --features demo`, sessão23928; frontend/tsc/Vite passou, aguardar saída terminal do Rust antes de afirmar aprovação. E2E web-research.e2e.ts e três jornadas de síntese ao vivo continuam pendentes. Matriz real de protocolos/modos preexistente EV068 e gate backend EV073 não dispensam esses gates. A composição atual demo usa CodexRuntime::Fake/WebService::live; a fixture para pesquisa determinística na janela precisa ser definida na seam HostConfig.web já aprovada, sem permitir URL privada na produção ou reduzir a jornada a eventos sintéticos de UI. msedgedriver existente em target/qa-tools/edge-154 está fora do PATH padrão; conferir versão e configurar somente o processo QA.

Saída terminal da sessão23928 observada: build demo exit0,3m30s,EV075 parcial. Plano r10 detalha a composição nativa da seam já prevista; todos os gates anteriores foram preservados e evidências do plano anterior ficaram stale pelo runner, aguardando renovação real. Produção sem e2e não compila adapter sintético; demo sem opt-in mantém Fake. Native E2E não usará injeção de eventos UI.

Arquivos/símbolos/AC/comandos executados/EV/revisão/limites/desvios/próxima ação. Done só com todos os gates e revisão atuais. Estado permanece in_progress/partial.
