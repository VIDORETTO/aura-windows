---
schema: hybrid/change
schema_version: 1.0
effort_id: 026-biblioteca-e-pergunte
revision: 1
status: draft
profile: compact
---

# Change: Biblioteca de reuniões e Pergunte

## Objetivo e limites

Candidatos: CAND-039. Origem: plano `docs/project/plano-reuniao-e-configuracao-assistida.md`. Biblioteca local com busca e ferramenta `meeting_search` para perguntar sobre reuniões e projetos.

Nota: Depende de 025. **Feito (05/10):** busca por palavras e ferramentas `meeting_search`/`meeting_get`. **Falta:** filtro por pessoa/Receita, UI da biblioteca, Projetos.

## Contrato de comportamento

- Entradas: ver requisitos e critérios abaixo.
- Saída: ver critérios de aceite.
- Erros/invariantes: segredos nunca expostos; captura só pela Política de privacidade.
- Compatibilidade: comportamento existente do Overlay, Comandos rápidos e Configurações.

## Requisitos e aceite

- **FR-001** — As reuniões MUST ser pesquisáveis por texto, pessoa e Receita.
- **AC-001** — Dadas 3 reuniões, a busca "orçamento" lista só as que citam o termo.
- **FR-002** — O agente MUST responder sobre reuniões com chips de origem.
- **AC-002** — Dado "o que combinamos com a Ana em agosto?", então a resposta cita reunião e minuto.

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
