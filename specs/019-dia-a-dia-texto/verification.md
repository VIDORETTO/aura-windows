# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `cargo test -p aura-extensions -p aura-app; pnpm -C apps/desktop test`
- Execution: `executed`
- Environment: Linux x86_64, Rust 1.98.1
- Tested revision: `local:f6bca256a851811b97e9a3826aef8a5567023f368c39d9617dcb92c56c1b1fc0`
- Timestamp: `2026-10-05T16:28:10+00:00`
- Observations: Comandos /formal /curto /amigavel /golpe /responder /parei expandem como especificado; 17+55 testes Rust e 191 de UI verdes
- Evidence refs: none
- Limitations: AC-002 e AC-003 pendentes

## EV-002 — partial

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `cargo test -p aura-app quick_command_result_replaces...; vitest Replace.test.tsx`
- Execution: `executed`
- Environment: Linux x86_64 (colar/desfazer falsos)
- Tested revision: `local:4a350fccef00b5f7b5e426e829139495c7f246014d91a729d0a0ef08304cabc3`
- Timestamp: `2026-10-05T17:24:10+00:00`
- Observations: Substituir seleção cola o texto novo, devolve o original, mostra antes→depois e desfaz uma vez (Ctrl+Z); botão some sem seleção
- Evidence refs: none
- Limitations: Mini-menu flutuante com atalho próprio (janela do shell) não implementado; colar e Ctrl+Z reais em apps do Windows não verificados; o alvo da substituição não expira ao trocar de conversa

## EV-003 — partial

- Ticket: `—`
- Acceptance: `AC-003`
- Procedure: `cargo test -p aura-extensions -p aura-app (colar); vitest speech.test.ts`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:36b4855b64c580a836c9d618bd95d21a84ad5646ed80f3806d33e5f2e4d45b97`
- Timestamp: `2026-10-05T17:24:10+00:00`
- Observations: /colar lê a área de transferência só nesse comando e explica quando está vazia; /ler fala a seleção lembrada
- Evidence refs: none
- Limitations: Voz offline, clipboard e colagem reais do Windows não verificados; 'Colar como' não cola sozinho (usa Inserir no app)
