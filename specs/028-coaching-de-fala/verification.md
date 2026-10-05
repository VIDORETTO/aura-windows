# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `cargo test -p aura-app (speech_stats, speech_stats_come_from_the_saved_transcript); vitest MeetingPanel`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:b6412ea71fee808ceb5d03937883fa1486a2242aa38c332925d59eb5af545a33`
- Timestamp: `2026-10-05T18:12:43+00:00`
- Observations: Tempo de fala, ritmo, muletas (pt/en, palavras inteiras), maior turno e perguntas; painel e ferramenta meeting_stats
- Evidence refs: none
- Limitations: Ritmo depende da qualidade dos tempos do ASR; sem tom de voz nem comparação ao longo do tempo
