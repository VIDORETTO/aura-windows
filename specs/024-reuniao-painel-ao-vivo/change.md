---
schema: hybrid/change
schema_version: 1.0
effort_id: 024-reuniao-painel-ao-vivo
revision: 1
status: draft
profile: compact
---

# Change: Reunião: Preparo e painel ao vivo

## Objetivo e limites

Candidatos: CAND-036. Origem: plano `docs/project/plano-reuniao-e-configuracao-assistida.md`. Janela própria da Reunião com Preparo, ações sob demanda, notas do usuário e marcadores.

Nota: Depende de 022 e 023.

## Contrato de comportamento

- Entradas: ver requisitos e critérios abaixo.
- Saída: ver critérios de aceite.
- Erros/invariantes: segredos nunca expostos; captura só pela Política de privacidade.
- Compatibilidade: comportamento existente do Overlay, Comandos rápidos e Configurações.

## Requisitos e aceite

- **FR-001** — A Reunião MUST abrir com o Preparo de três velocidades.
- **AC-001** — Dado "1:1 com a Ana sobre promoção", então o cartão "Entendi assim" aparece com no máximo 2 perguntas e "Começar" disponível.
- **FR-002** — O painel ao vivo MUST oferecer Perdi o fio, Resumo, O que respondo?, Termo e Marcar.
- **AC-002** — Dada uma reunião em andamento, quando o usuário aciona "Perdi o fio", então o resumo do último minuto chega em até 5 s com trechos citáveis.
- **FR-003** — O painel MUST ficar fora de capturas e respeitar a opção de ocultar.
- **AC-003** — Dado `hideFromCapture = true`, então a janela `meeting` tem `WDA_EXCLUDEFROMCAPTURE`.

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
