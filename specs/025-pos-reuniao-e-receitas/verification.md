# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-002`, `AC-001`, `AC-003`
- Procedure: `cargo test -p aura-app (recipes, recipes_are_listed_created_by_the_agent...); pnpm -C apps/desktop test (MeetingPanel)`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:39bc1a781e27a7ea5333e68bc9f90222dcd40407b28f441b0d84726e19d17f6c`
- Timestamp: `2026-10-05T17:40:51+00:00`
- Observations: 8 Receitas embutidas válidas; o agente cria Receitas com recipe_save (aprovação, embutidas protegidas); meeting_get devolve a Receita da reunião; ações do pós-reunião (resumo e ações, e-mail, ata em modo Tarefa) exigem minuto [mm:ss] de origem; Skill aura-criar-receita
- Evidence refs: none
- Limitations: Citação do minuto é exigida no prompt mas não verificada nas respostas do modelo real; 'clicar abre o trecho' e 'sem fonte' visível não implementados; geração real de ata não verificada com o app-server; promessas, 'sem resposta' e debrief só como instrução de prompt
