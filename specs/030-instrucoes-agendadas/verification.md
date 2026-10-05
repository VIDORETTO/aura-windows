# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `cargo test -p aura-app (scheduled_instructions_open_a_chat_when_due, reminders)`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:4f16517fbedd3c73c9a8d8e9e0ba9cf43114ecb8e082771b38d0bfee2c033e9a`
- Timestamp: `2026-10-05T18:16:00+00:00`
- Observations: A instrução agendada abre Chat só quando vence; recorrência pula ocorrências perdidas; migração 0015
- Evidence refs: none
- Limitations: Rotina roda com o Aura aberto (bandeja); sem execução em modo Tarefa; sem lista de execuções; comportamento do agente real não verificado
