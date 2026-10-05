---
schema: hybrid/change
schema_version: 1.0
effort_id: 030-instrucoes-agendadas
revision: 1
status: draft
profile: compact
---

# Change: Instruções agendadas (primeira fatia dos agentes agendados)

## Objetivo e limites

Candidato CAND-017, primeira fatia. Um lembrete pode carregar uma instrução (`agent_prompt`) que o agente executa, na hora marcada, numa conversa nova em modo Chat (somente leitura), com repetição diária, em dias úteis ou semanal. Fora do escopo: gatilhos por atividade, execução em modo Tarefa, histórico de execuções.

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — Um lembrete com `agent_prompt` MUST abrir uma conversa de Chat com essa instrução quando vencer, e só então.
- **AC-001** — Dado `reminder_create {text:"Resumo da manhã", delay_minutes:5, repeat:"daily", agent_prompt:"Resuma meus compromissos abertos com action_list"}`, quando ele vence, então chega `AgentTask{mode:"chat"}` com o texto; um lembrete sem `agent_prompt` não abre conversa; o diário volta no dia seguinte. **Feito e testado (05/10).**
- **FR-002** — O usuário MUST criar a rotina por conversa (`/agendar`) e aprovar a criação.
- **AC-002** — Dado `/agendar todo dia útil às 8h resuma meus compromissos`, então o agente chama `clock_now` e `reminder_create` com `agent_prompt` e `repeat=weekdays`, e o usuário aprova a chamada. **Comando e ferramenta prontos; comportamento real do agente não verificado.**

## Leitura e mapa de alterações

- `crates/aura-app/src/reminders.rs` → `Reminder.prompt`, `Host::tick_reminders_at` — instrução agendada; existing.

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
