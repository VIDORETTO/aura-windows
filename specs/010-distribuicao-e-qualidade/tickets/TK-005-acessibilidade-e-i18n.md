---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 010-distribuicao-e-qualidade
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-005"]
acceptance_refs: ["AC-009", "AC-010"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src/i18n", "docs/qa/acessibilidade.md", "apps/desktop/src/a11y"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-005 — Acessibilidade e idiomas (pt-BR/en)

## Objetivo e limites

Entrega auditoria e correções de acessibilidade (nomes acessíveis, foco, `aria-live` por frase, contraste nos três temas, navegação só por teclado), teste de cobertura de chaves i18n, detecção do idioma do Windows e escolha manual, e a Persona no idioma da interface.

Não inclui: outros idiomas além de pt-BR/en.

## Leitura em ordem

1. `docs/design/ui-ux.md` → "Acessibilidade" e mapa de teclado.
2. `apps/desktop/src/i18n/` (criado desde 001) e `apps/desktop/src-tauri/resources/persona/` (002).
3. WCAG 2.1 AA; axe-core.

## Decisões já resolvidas

- Anúncio da resposta: `aria-live="polite"` num nó oculto que recebe frases completas (buffer até `.`, `?`, `!` ou 2 s).
- Alto contraste do Windows (`forced-colors`) desliga translucidez.
- Chaves ausentes quebram o teste de cobertura.
- Liberdade local: correções pontuais de componentes.

## Mapa de alterações

- Novo: `apps/desktop/src/a11y/{LiveAnnouncer.tsx,focusRing.css}`; `docs/qa/acessibilidade.md`.
- Existente: componentes com nomes acessíveis; `apps/desktop/src/i18n/{pt-BR.json,en.json}` completos.

## Contrato técnico

- Entradas: UI existente.
- Saídas: 0 violações sérias/críticas do axe; roteiro Narrador aprovado.

## Exemplos de aceite

- **AC-009**: axe nas telas Overlay, Configurações, Onboarding → 0 violações sérias; roteiro com Narrador: abrir, perguntar, `/screen`, aprovar (`A`), copiar — tudo por teclado, resposta anunciada por frase; contraste AA medido sobre fundo claro e escuro.
- **AC-010**: teste de cobertura: todas as chaves de `pt-BR.json` existem em `en.json` e vice-versa; E2E com locale `en-US` → UI em inglês; `thread/start.baseInstructions` = `persona/en.md`.

## Dependências e sequência de execução

Depende de: interface principal pronta (Marco 4); nenhum ticket interno.

- [ ] TK-005.1 Teste de cobertura i18n red→green.
- [ ] TK-005.2 LiveAnnouncer (Vitest) red→green.
- [ ] TK-005.3 Auditoria axe + correções; roteiro Narrador; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `pnpm -C apps/desktop test -- a11y i18n`; `pnpm -C apps/desktop e2e -- --spec e2e/a11y.spec.ts`; roteiro manual.
- Estado esperado: verdes; roteiro aprovado.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Nenhuma prevista.

## Relatório de saída

Relatar violações corrigidas, resultados, EV refs.
