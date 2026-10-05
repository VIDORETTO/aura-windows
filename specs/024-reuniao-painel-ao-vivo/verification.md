# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `pnpm -C apps/desktop test (MeetingPanel.test.tsx, a11y); cargo test -p aura-app (agent_prepares_the_briefing_but_only_the_user_starts_the_meeting)`
- Execution: `executed`
- Environment: Linux x86_64, UI em jsdom com backend simulado
- Tested revision: `local:e2cce9e9ac6d0ad02301266bce9a44cbe0069ee7c671fc4fd8d140160886ccb0`
- Timestamp: `2026-10-05T17:36:47+00:00`
- Observations: Painel Reunião no Overlay: Preparo ⚡/🧭/▶, briefing do agente aguarda Começar, ao vivo com transcrição, pausa, encerrar, 'esqueci de iniciar', reuniões salvas; ações Perdi o fio/Resumo/O que respondo? abrem conversa com meeting_get; sem violações axe; agente nunca inicia a reunião
- Evidence refs: none
- Limitations: AC-003 (janela própria 'meeting' excluída de captura) não implementada: o painel vive dentro do Overlay; respostas em 5 s não medidas; notas do usuário e marcadores pendentes; transcrição real depende do ASR no Windows

## EV-002 — partial

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `cargo test -p aura-app (notes_and_markers_share_the_timeline...); vitest MeetingPanel (notas e ★)`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:e4f84f9b2c8f6f05920976530c0a081435133603180d25bab98be6ec260bedf1`
- Timestamp: `2026-10-05T17:47:41+00:00`
- Observations: Notas do usuário e marcadores ★ ficam na mesma linha do tempo da fala, aparecem no painel, são pesquisáveis e o agente as lê como 'Nota do usuário'
- Evidence refs: none
- Limitations: Termo/sigla, janela própria e orçamento de 5 s continuam pendentes
