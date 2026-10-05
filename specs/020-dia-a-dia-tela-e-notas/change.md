---
schema: hybrid/change
schema_version: 1.0
effort_id: 020-dia-a-dia-tela-e-notas
revision: 1
status: draft
profile: compact
---

# Change: Dia a Dia: texto da tela, lembretes, notas e salvos

## Objetivo e limites

Candidatos: CAND-059, CAND-062, CAND-063, CAND-068. Origem: plano `docs/project/plano-reuniao-e-configuracao-assistida.md`. Copiar texto de uma região da tela, criar lembretes por conversa, guardar anotações rápidas e salvar respostas.

Nota: **Feito (05/10):** lembretes (`clock_now`, `reminder_*`, recorrência, timer de 15 s, evento → notificação) e notas/salvos (`note_save`, `note_search`), comandos `/lembrar`, `/anota`, `/notas`. **Também feito:** `/texto-traduzir` (OCR da região + /traduzir). **Falta:** validação do OCR e do toast no Windows, telas de lista/apagar e estrela em Salvos na UI, validação do toast no Windows.

## Contrato de comportamento

- Entradas: ver requisitos e critérios abaixo.
- Saída: ver critérios de aceite.
- Erros/invariantes: segredos nunca expostos; captura só pela Política de privacidade.
- Compatibilidade: comportamento existente do Overlay, Comandos rápidos e Configurações.

## Requisitos e aceite

- **FR-001** — O usuário MUST copiar o texto de uma região (OCR) com tradução opcional.
- **AC-001** — Dada uma região com texto em inglês, quando o usuário arrasta e escolhe "Copiar texto", então o texto vai para a área de transferência; com "Traduzir", a tradução vai no lugar.
- **FR-002** — O usuário MUST criar lembretes em linguagem natural, únicos ou recorrentes, com notificação nativa.
- **AC-002** — Dado "me lembra às 15h de ligar para o João", quando o usuário confirma o cartão, então às 15h uma notificação do Windows mostra "Ligar para o João"; o lembrete sobrevive a reiniciar o Aura.
- **FR-003** — O usuário MUST guardar anotações por voz ou texto numa caixa de entrada pesquisável e salvar respostas.
- **AC-003** — Dado "anota: renovar o seguro em março", então a nota aparece na caixa de entrada com a data e é achada pela busca "seguro"; a estrela numa resposta a põe em Salvos.

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
