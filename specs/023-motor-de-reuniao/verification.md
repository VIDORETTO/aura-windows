# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-001`, `AC-003`
- Procedure: `cargo test -p aura-app (meeting::tests, meeting_is_opt_in_and_records_audio_only_while_it_runs); pnpm -C apps/desktop test`
- Execution: `executed`
- Environment: Linux x86_64 (fonte de falas falsa; áudio sintético)
- Tested revision: `local:4b8ad035e4f3c4fa4ad4949824a923a787dea24abf1269254c389aaa6f8a949a`
- Timestamp: `2026-10-05T17:32:32+00:00`
- Observations: Reunião só começa por chamada explícita, grava mic+sistema só enquanto roda e volta ao que a política diz; privacidade pausada bloqueia; falas guardadas uma vez com tempo e falante; pausa pula o intervalo; 'esqueci de iniciar' cria Reunião do buffer; reuniões interrompidas por queda são marcadas
- Evidence refs: none
- Limitations: AC-002 (transcrição contínua real Você/Eles) sem verificação: depende dos motores de voz locais ou da nuvem no Windows; apagar o áudio ao encerrar e retenção por reunião ainda não implementados; sem UI de iniciar/parar (esforço 024)
