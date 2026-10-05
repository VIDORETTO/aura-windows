---
schema: hybrid/change
schema_version: 1.0
effort_id: 027-compromissos
revision: 1
status: draft
profile: compact
---

# Change: Compromissos e promessas entre reuniões

## Objetivo e limites

Candidato CAND-047. O que o usuário deve e o que lhe prometeram, com dono, prazo e o minuto da reunião de origem; envelhecem e ficam atrasados. Fora do escopo: lembretes automáticos de compromissos atrasados (usar Lembretes) e sincronização com gerenciadores de tarefas (conectores).

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — O Aura MUST guardar compromissos com dono (eu / outra pessoa), prazo e minuto de origem, sem inventar nenhum.
- **AC-001** — Dado `action_save {text:"enviar a proposta", owner:"you", due:"2000-01-01", minute:"12:31"}`, então `action_list {owner:"you", status:"open"}` devolve 1 item com `overdue=true` e `minute="12:31"`; um prazo futuro devolve `overdue=false`; data inválida ("amanhã") e dono inválido devolvem erro. **Feito e testado (05/10).**
- **FR-002** — O usuário MUST ver e dar baixa nos compromissos abertos.
- **AC-002** — Dado dois compromissos abertos, o painel Reunião mostra "Eu: …" e "Prometido a mim: …", com "atrasado desde AAAA-MM-DD" quando vencido; marcar a caixa os tira da lista e grava `done`. **Feito e testado (05/10).**

## Leitura e mapa de alterações

- `crates/aura-app/src/actions.rs` → `ActionsRepo` — compromissos; new.

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
