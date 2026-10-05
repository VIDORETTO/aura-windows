# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-002`, `AC-003`
- Procedure: `cargo test -p aura-app (reminders, notes, reminders_and_notes_through_agent_tools); pnpm -C apps/desktop test`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:7131c2afdb472998fe4b16269483ba9b86fac48822e220e82bb94490eb491cb7`
- Timestamp: `2026-10-05T17:17:48+00:00`
- Observations: Lembretes únicos e recorrentes (diário, dias úteis, semanal) disparam uma vez e pulam ocorrências perdidas; evento chega à UI que chama notify; notas e salvos pesquisáveis sem acento; ferramentas MCP criadas
- Evidence refs: none
- Limitations: Toast real do Windows, persistência após reiniciar o Aura (só SQLite testado em memória), tela de lista/estrela em Salvos e ditado global para 'anota' não verificados; AC-001 (OCR de região) não iniciado
