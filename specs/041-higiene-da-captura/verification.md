# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `Antes da correção: cargo test -p aura-capture --lib retention (novo caso continuous_segments_follow_age_and_size_limits)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:796faf0651d9d42a74a84b6d671bc96dcac734dafc0d9cce2e188679c206e62b`
- Timestamp: `2026-11-13T23:56:15+00:00`
- Observations: Red comportamental: plan devolveu [] para 10 Segmentos continuous de 2 GB (esperado d0-d2).
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk001-red.log`
- Limitations: Evidence invalidated because an input changed.

## EV-002 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `cargo test -p aura-capture (lib + testes de integração)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:796faf0651d9d42a74a84b6d671bc96dcac734dafc0d9cce2e188679c206e62b`
- Timestamp: `2026-11-13T23:56:15+00:00`
- Observations: 11 testes de unidade (3 novos: continuous_segments_follow_age_and_size_limits, manual_recordings_stay_outside_the_continuous_budget, continuous_budget_is_shared_by_all_sources) e 4 de integração passaram.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk001-green.log`
- Limitations: Evidence invalidated because an input changed.

## EV-003 — passed

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `cargo test -p aura-capture (após rustfmt, HEAD 67110c3)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:be88c1cd031cb5dfa3a632ddafbe89f3069495ea8726756593b59825c971131c`
- Timestamp: `2026-11-13T23:57:02+00:00`
- Observations: 11 testes de unidade e 4 de integração passaram, incluindo os 3 novos de 041.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk001-green2.log`
- Limitations: none recorded

## EV-004 — failed

- Ticket: `TK-002`
- Acceptance: `AC-004`
- Procedure: `Assinatura nova com o comportamento antigo (política clonada no new): cargo test -p aura-audio --test recording`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:cd4d45829c03eaeb509c16883896daab0d0cb0ac34430e93263d416c47640f64`
- Timestamp: `2026-11-13T23:59:00+00:00`
- Observations: Red comportamental: com a política viva em 30 min, a cópia de 5 min apagou 14 Segmentos de tela (left 6, right 20).
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk002-red.log`
- Limitations: none recorded

## EV-005 — passed

- Ticket: `TK-002`
- Acceptance: `AC-004`
- Procedure: `cargo test -p aura-audio; cargo test -p aura-app`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:0ac08bb1dcbbbe6a202d474c7fa6c30e9600636e36afab0ba43053bcc7664391`
- Timestamp: `2026-11-13T23:59:00+00:00`
- Observations: aura-audio 11 + 2 (live_retention_change_reaches_audio verde: 20 telas mantidas com 30 min; com 5 min sobram as que terminam em 16 a 20 min). aura-app: todas as suítes passaram.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk002-green.log`
- Limitations: none recorded
