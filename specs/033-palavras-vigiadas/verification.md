# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-core -p aura-app; pnpm -C apps/desktop test e typecheck; clippy -D warnings`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:0cd93b2b0012cf5e7f8be866b0120e95abc14effd7309c2d1843132e9efa1f79`
- Timestamp: `2026-10-05T18:55:49+00:00`
- Observations: watch_hits, validação de settings e UI testadas; contrato dourado regenerado; 225 testes de UI passam
- Evidence refs: none
- Limitations: Aviso via Notice do Overlay (sem teste de ponta a ponta do poll com ASR real); respeita o Modo transmissão só porque fica dentro do Overlay
