---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 039-web-gratuita-do-agente
type: delivery
status: done
ticket_revision: 49
requires: ["TK-001"]
requirement_refs: ["FR-003", "FR-004", "FR-007"]
acceptance_refs: ["AC-004", "AC-005", "AC-006", "AC-012"]
spec_revision: 1
plan_revision: 11
owned_areas: ["crates/aura-web", "Cargo.lock"]
verification_status: passed
last_update: Git LF normalization explicitly revalidated; exact original fingerprints recoverable without behavioral edits. Canonical evidence/current0.3 release documented in041.
---
















































# TK-002 — Ler conteúdo principal de páginas públicas com continuidade

## Objetivo e limites

WebService::fetch com conteúdo principal/estrutura, versão/continuação, DNS/redirect seguro e limites. Usa transporte/registry da busca. Não inclui JS/login/PDF/paywall, browser ou endpoint privado configurável.

Autorização: o usuário pediu executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Não inclui itens fora da fatia.

## Leitura em ordem

1. `AGENTS.md`, `docs/HANDOFF.md`, `CONTEXT.md` e ADRs do plano.
2. `specs/039-web-gratuita-do-agente/spec.md` r1, `plan.md` r8, `contracts.md`, `tdd.md` → contratos e casos.
3. `specs/039-web-gratuita-do-agente/contracts.md` → leitura/erros.
4. `specs/039-web-gratuita-do-agente/tdd.md` → TK-002.
5. `specs/039-web-gratuita-do-agente/plan.md` → transporte/extração/orçamento.
6. `crates/aura-ingest/src/text.rs` → html_to_text existente, limite de strip tags.

## Decisões já resolvidas

Abordagem/contratos/limites do plano r8. Confirmar seams propostas antes da escrita de testes na implementação, conforme skill TDD. Testes públicos de fetch executados e registrados no runner. Não alterar comportamento para fixture passar.

Liberdade local: nomes privados/organização e mensagens redigidas dentro do contrato. Alternativas no plano. Dependência/contrato material novo retorna à planejadora.

## Mapa de alterações

Novos fetch.rs/tests/fetch.rs/fixtures HTML em aura-web; transporte/registry da fatia anterior existentes após TK-001. Extração dom_smoothie e fallback DOM conservador. Sem tocar ingestor de anexos ou Windows.

Owned areas delimitam responsabilidade; um ticket por vez, preservando trabalho preexistente. Arquivos/símbolos dos predecessores são previstos; existentes após sua implementação.

## Contrato técnico

URL validada antes de DNS/conexão; pin IP público/TLS hostname, redirect manual até 3, sem proxy/cookie/creds. Stream até 2 MiB/30 s, documento100k/output20k Unicode. MIME/extração/JS bloqueados com erros; versão/nextStartChar impedem misturar leitura. Texto externo, HTML nunca executado.

Erros/unidades/limites/efeitos/compatibilidade seguem spec/plan/contracts. Segredos fora de UI/config/log. Nenhuma evidência inventada.

## Exemplos de aceite

- **AC-004**: relatório sintético → título e fatos 42/12/30 com lista/tabela, não script/nav dominante.
- **AC-005**: ABCDE max3 → ABC/[0,3)/next3; versão igual → DE/[3,5)/nextnull; emoji íntegro, URL final e limites explicitados.
- **AC-006**: IP privado/mapped IPv6/DNS misto/rebind/redirect privado → unsafe_url sem conexão proibida; público seguro funciona.
- **AC-012**: descomprimido 2 MiB+1 → too_large, atraso → timeout; texto >100k sinaliza documentTruncated. Restante AC nos outros tickets.

Oráculo independente: literais de spec e fixtures sintéticas de tdd.md; sem recompor algoritmo ou esperado da implementação.

## Dependências e sequência de execução

Depende de TK-001 em done: serviço/contrato do predecessor é necessário.

- [x] TK-002.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-002 --json`; ready:true e inputs atuais antes de implementar.
- [x] TK-002.2 Confirmar seams, primeiro caso → red comportamental, não ambiente.
- [x] TK-002.3 Green mínimo, um caso por vez; caso já verde é cobertura, não red artificial.
- [x] TK-002.4 Regressões/integração e resultado com limitações.
- [ ] TK-002.5 Evidence add só após execução real, ticket update/checkpoint, revisão antes de done, projeções por render.

## Validação

Diretório: raiz. Comandos de validação:

```powershell
cargo test -p aura-web --test fetch
cargo test -p aura-web --test search
cargo clippy -p aura-web --all-targets -- -D warnings
```

Resultados executados registrados pelo runner; public fetch e regressão search existentes. Live HTML prova um contrato pontual, não a meta de qualidade do TK-005.

Toolchain projeto; Windows/MSVC/WebView2/WebDriver para E2E; AURA_CODEX_BIN fixado verificado para real. Falta de ferramenta/conta/rede é not_run/impedimento, não aprovação. Dourado/real/benchmark seguem plano/TDD, sem dumps de conteúdo/chaves.

## Condição de retorno à planejadora

Contrato real incompatível, predecessor não done, input mudou, necessidade de pago/JS/upgrade, mudança de comportamento/autoridade → devolver caminho/símbolo/resultado. Preservar progresso; não ampliar escopo/reduzir oráculo.

## Relatório de saída

Arquivos/símbolos/AC/comandos executados/EV/revisão/limites/desvios/próxima ação. Done só com evidência passada/revisão. Leitura e paginação implementadas; ferramentas Host e UI pertencem aos sucessores.
