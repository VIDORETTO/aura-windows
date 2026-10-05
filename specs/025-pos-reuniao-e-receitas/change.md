---
schema: hybrid/change
schema_version: 1.0
effort_id: 025-pos-reuniao-e-receitas
revision: 1
status: draft
profile: compact
---

# Change: Pós-reunião e Receitas

## Objetivo e limites

Candidatos: CAND-037. Origem: plano `docs/project/plano-reuniao-e-configuracao-assistida.md`. Notas melhoradas por Receita, ações com origem citada, e-mail e agente que executa o trabalho.

Nota: Depende de 024. **Feito (05/10):** 8 Receitas embutidas + Receitas do usuário (migração 0012), `recipe_list`/`recipe_save` (aprovação), Skill `aura-criar-receita`, seletor de Receita no painel, `meeting_get` com a Receita, ações Resumo e ações / E-mail / Gerar ata (agente em modo Tarefa) com minuto citado. **Falta:** verificar citações, clicar na citação para abrir o trecho, notas melhoradas combinando as notas do usuário, debrief de 30 s, painel de promessas.

## Contrato de comportamento

- Entradas: ver requisitos e critérios abaixo.
- Saída: ver critérios de aceite.
- Erros/invariantes: segredos nunca expostos; captura só pela Política de privacidade.
- Compatibilidade: comportamento existente do Overlay, Comandos rápidos e Configurações.

## Requisitos e aceite

- **FR-001** — Toda nota e ação MUST citar o minuto da fala de origem.
- **AC-001** — Dada uma ação gerada, então ela traz o trecho de origem e clicar abre a fala.
- **FR-002** — O Aura MUST ter 8 Receitas embutidas e criar novas por IA.
- **AC-002** — Dado "Receita para reuniões com fornecedores", então a IA cria e salva a Receita após o Preparo.
- **FR-003** — O agente MUST gerar artefatos (ata, minuta, e-mail) em modo Tarefa com aprovação.
- **AC-003** — Dado "Gerar minuta", então o agente cria o arquivo no workspace após aprovação.

## Leitura e mapa de alterações

- `[path]` → `[symbol]` — [motivo]; [existing/new].

## Plano breve

Seam: [interface sob teste]. Abordagem: [decisão técnica local]. Dependências: [none ou refs].

## Sequência e tarefas

- [ ] C-001 Escrever caso relevante e observar red pelo motivo esperado.
- [ ] C-002 Implementar o mínimo e observar green.
- [ ] C-003 Executar regressão e registrar evidência.

## Validação e evidência

Comando/procedimento: `[exact command]`

Resultado executado: [pending]

Limitações: [o que não foi verificado]. Comando apenas identificado na configuração: [none ou registro].

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
