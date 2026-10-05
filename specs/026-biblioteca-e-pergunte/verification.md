# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-app (search_lists_meetings..., agent_searches_and_reads_saved_meetings)`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:dec38fcf8d36c4821f2081cb15c7d6b803de0420f32f188fff33f504e8332c0a`
- Timestamp: `2026-10-05T17:32:32+00:00`
- Observations: Busca sem acento por palavras, por reunião; ferramentas meeting_search/meeting_get devolvem reunião e minuto
- Evidence refs: none
- Limitations: Sem filtro por pessoa/Receita, sem UI de biblioteca e sem verificação com o agente real

## EV-002 — partial

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-app (projects_*, meeting search); vitest MeetingPanel (projetos)`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:22415b2b1050a8ddcd9239dc6e98a008c9ea34c3cbfb2ab3dc5432d43e360330`
- Timestamp: `2026-10-05T18:04:45+00:00`
- Observations: Projetos (nome único + instruções), reunião movida ao projeto, busca limitada ao projeto, meeting_get devolve as instruções do projeto; painel cria projeto e move reunião
- Evidence refs: none
- Limitations: Sem filtro por pessoa (depende de diarização); memória restrita ao projeto e arquivos do projeto não implementados
