---
schema: hybrid/change
schema_version: 1.0
effort_id: 035-habitos-em-sugestoes
revision: 1
status: draft
profile: compact
---

# Change: Hábitos viram sugestões

## Objetivo e limites

Candidato CAND-048. O Aura conta, só localmente e só pelo nome do comando, quais comandos rápidos o usuário roda em cada app; a partir de três usos o comando aparece primeiro nas sugestões da tela vazia daquele app. Há um botão para esquecer tudo.

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — Um comando MUST virar hábito após 3 usos no mesmo app (sem diferenciar maiúsculas), guardar só o nome do comando e aparecer primeiro, sem repetir, com no máximo quatro sugestões.
- **AC-001** — Dado três usos de /traduzir no chrome.exe, "/traduzir" é a primeira sugestão ali e não em outro app; a lista nunca passa de quatro; sem app conhecido nada é gravado. **Feito e testado (05/10).**
- **FR-002** — O usuário MUST poder esquecer os hábitos.
- **AC-002** — Dado hábitos gravados, "Esquecer hábitos" os apaga. **Feito e testado (05/10).**

## Leitura e mapa de alterações

- `apps/desktop/src/overlay/habits.ts` → `recordHabit/topHabits/clearHabits` + `suggestionsFor(..., habits)`; new.

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
