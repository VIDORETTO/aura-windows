---
schema: hybrid/change
schema_version: 1.0
effort_id: 031-sugestoes-na-tela-vazia
revision: 1
status: draft
profile: compact
---

# Change: Sugestões na tela vazia e detecção suave de reunião

## Objetivo e limites

Candidato CAND-028. O Overlay vazio sugere o que fazer conforme o app em primeiro plano (reunião, navegador, e-mail, editor, Office), incluindo "Preparar reunião" quando o app anterior é Teams, Zoom, Meet etc. A sugestão nunca inicia captura: só abre o painel ou envia um comando que o usuário vê.

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — As sugestões MUST depender do app anterior e nunca passar de quatro.
- **AC-001** — Dado `ms-teams.exe`, `Zoom.exe` ou uma página "Google Meet", a primeira sugestão é "Preparar reunião"; `OUTLOOK.EXE` começa por "Responder isto"; `Code.exe` por "Explicar o erro da tela"; navegador oferece resumir, golpe e documento; app desconhecido, as três genéricas. **Feito e testado (05/10).**
- **FR-002** — Escolher "Preparar reunião" MUST abrir o painel Reunião sem iniciar nada.
- **AC-002** — Dado o Overlay expandido e vazio, quando o usuário clica "Preparar reunião", então o painel "Nova reunião" aparece e nenhuma reunião começa. **Feito e testado (05/10).**

## Leitura e mapa de alterações

- `apps/desktop/src/overlay/suggestions.ts` → `suggestionsFor` e o grupo "Sugestões para este app" no estado vazio; new.

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
