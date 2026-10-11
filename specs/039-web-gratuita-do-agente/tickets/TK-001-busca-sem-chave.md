---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 039-web-gratuita-do-agente
type: delivery
status: done
ticket_revision: 55
requires: []
requirement_refs: ["FR-001", "FR-002", "FR-007"]
acceptance_refs: ["AC-001", "AC-002", "AC-003", "AC-012"]
spec_revision: 1
plan_revision: 11
owned_areas: ["Cargo.toml", "Cargo.lock", "crates/aura-web"]
verification_status: passed
last_update: Git LF normalization explicitly revalidated; exact original fingerprints recoverable without behavioral edits. Canonical evidence/current0.3 release documented in041.
---






















































# TK-001 — Buscar fontes gratuitamente com alternativa limitada

## Objetivo e limites

Primeira fatia pública de busca: WebService::search, Parallel anônimo primário/DDG alternativo e fontes estruturadas. Capturar contrato publicado/fixture e provar rota gratuita; não inclui extração de páginas, UI, APIs pagas ou instalação global.

Autorização: o usuário pediu executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Não inclui itens fora da fatia acima.

## Leitura em ordem

1. `AGENTS.md`, `docs/HANDOFF.md`, `CONTEXT.md` e ADRs do plano.
2. `specs/039-web-gratuita-do-agente/spec.md` r1 e `plan.md` r8.
3. `specs/039-web-gratuita-do-agente/tdd.md` → casos desta fatia, oráculos e seams propostas.
4. `specs/039-web-gratuita-do-agente/research.md` → gratuidade/fornecedores/fontes.
5. `specs/039-web-gratuita-do-agente/contracts.md` → identidade, busca e erros.
6. `Cargo.toml` → workspace/dependências.
7. `crates/aura-mcp/src/lib.rs` → tipos e padrão JSON-RPC para consumidor.

## Decisões já resolvidas

Abordagem/contratos e limites do plano r8 são os desta fatia. Seams públicas do plano adotadas no pedido de execução; TDD somente nessas interfaces, sem mock de módulos internos. Não alterar comportamento para fazer fixture passar.

Liberdade local: nomes privados, organização interna e mensagens redigidas dentro do contrato. Alternativas descartadas no plano. Dependência/contrato material novo retorna à planejadora.

## Mapa de alterações

Novo: crates/aura-web Cargo.toml, lib.rs, search.rs, transport.rs, tests/search.rs e fixtures mínimas. Existente: workspace/lockfile. Transporte seguro inicial comum conforme plano. Fora: APIs Windows e app-server.

Owned areas delimitam responsabilidade, não autorizam editar toda a pasta. Um ticket por vez; preservar trabalho preexistente. Arquivos/símbolos de predecessores são previstos, existentes somente após sua implementação.

## Contrato técnico

Entradas e saídas em contracts.md, limites da spec. Sem Authorization/chave/model_name/histórico; sessionId aleatório por Conversa, confiável e estável. Dedupe mantém ordem/parâmetros semânticos; sucesso vazio distinto de erro. Fallback limitado, sem cota evadida, pago ou retry em cancel/input/disabled. Rede/DNS/clock variam via adapter; parser/registry/budget reais.

Erros, unidades, limites, efeitos e compatibilidade seguem spec/plan e, na web, contracts.md. Nenhuma credencial em UI/config/log. Nenhum estado verificado sem execução.

## Exemplos de aceite

- **AC-001**: fixture Manual público /manual → W1 com campos literais, sem conta/chave/header sensível.
- **AC-002**: 429 primário → origem alternativa/degraded; todos falham → unavailable; CAPTCHA bloqueado e sessão preservada.
- **AC-003**: manual#top/manual?utm_source=x → uma fonte; javascript removido; outro?q=1 e ?q=2 distintos; publishedAt null.
- **AC-012**: entradas acima dos limites → invalid_input sem rede; dois concorrentes permitidos/terceiro limitado; erros redigidos. Demais partes deste AC são completadas pelos TK-002/TK-004.

Oráculos: literais da spec/fixtures sintéticas do tdd.md; não recompor algoritmo nem gerar esperado da implementação.

## Dependências e sequência de execução

Depende de: nenhum.

- [x] TK-001.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-001 --json`; exigir ready:true e insumos atuais antes de implementar.
- [x] TK-001.2 Confirmar seams; primeiro caso do tdd.md → red comportamental, não falta de ambiente.
- [x] TK-001.3 Implementar mínimo para green, um caso por vez. Caso já verde registra cobertura sem fabricar red.
- [x] TK-001.4 Executar regressões/integração abaixo, resultados e limitações.
- [ ] TK-001.5 `evidence add` após execução real, `ticket update` para estado, checkpoint, revisão antes de done; projeções por render.

## Validação

Diretório: raiz. Comandos de validação:

```powershell
cargo test -p aura-web --test search
cargo clippy -p aura-web --all-targets -- -D warnings
cargo deny check licenses
```

Resultados executados ficam nas evidências do runner. Target search existente; testes públicos atravessam parsing, registry e orçamento. Live smoke é contrato pontual, sem aprovar o gate de qualidade do TK-005.

Ambiente: toolchain do projeto; Windows/MSVC/WebView2/WebDriver para E2E; AURA_CODEX_BIN aponta para fixado verificado nos testes reais. Falta de ferramenta/conta/rede é impedimento/not_run, não aprovação. Dourado, app-server e benchmark seguem plan/tdd, sem dumps de conteúdo/chaves.

## Condição de retorno à planejadora

Contrato real incompatível, predecessor não done, insumo mudou, necessidade de pago/JS/upgrade ou mudança de comportamento/autoridade → retornar com caminho/símbolo/resultado. Preservar progresso; não ampliar escopo nem reduzir oráculo.

## Relatório de saída

Arquivos/símbolos, AC, comandos executados, EV, revisão testada, limites, desvios e próxima ação. Done só com evidência passada e revisão requerida. Implementação de busca existente; integração Host, leitura, UI/cache e qualidade pertencem aos sucessores.
