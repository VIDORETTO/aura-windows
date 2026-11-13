# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — failed

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `Antes da correção: cargo test -p aura-capture --lib retention (novo caso continuous_segments_follow_age_and_size_limits)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:796faf0651d9d42a74a84b6d671bc96dcac734dafc0d9cce2e188679c206e62b`
- Timestamp: `2026-11-13T23:56:15+00:00`
- Observations: Red comportamental: plan devolveu [] para 10 Segmentos continuous de 2 GB (esperado d0-d2).
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk001-red.log`
- Limitations: none recorded

## EV-002 — passed

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `cargo test -p aura-capture (lib + testes de integração)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:796faf0651d9d42a74a84b6d671bc96dcac734dafc0d9cce2e188679c206e62b`
- Timestamp: `2026-11-13T23:56:15+00:00`
- Observations: 11 testes de unidade (3 novos: continuous_segments_follow_age_and_size_limits, manual_recordings_stay_outside_the_continuous_budget, continuous_budget_is_shared_by_all_sources) e 4 de integração passaram.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk001-green.log`
- Limitations: none recorded
