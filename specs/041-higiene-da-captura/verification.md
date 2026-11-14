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

## EV-006 — failed

- Ticket: `TK-003`
- Acceptance: `AC-008`
- Procedure: `sweep_orphans extraído mas ainda chamado por apply_retention: cargo test -p aura-capture --test retention_store orphans`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D; NVMe Kingston NV3
- Tested revision: `local:cfa04481a4cf612f099c613c2661bbe6207a0670db55aeb51e35a78717b53676`
- Timestamp: `2026-11-14T00:04:49+00:00`
- Observations: Red comportamental: apply_retention devolveu 3 (removeu .seg órfão, .seg.tmp e linha sem arquivo) em vez de 0.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-red.log`
- Limitations: none recorded

## EV-007 — passed

- Ticket: `TK-003`
- Acceptance: `AC-007`, `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-capture; cargo test -p aura-audio; cargo clippy -p aura-capture -p aura-audio --all-targets -- -D warnings`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D; NVMe Kingston NV3
- Tested revision: `local:3f3d042e604e258004499fd7811c6ac99f4ea22d8f97ae3c1a6fa4eba9130863`
- Timestamp: `2026-11-14T00:04:49+00:00`
- Observations: Equivalência com plan em 3 sementes x 4 políticas x 2.000 Segmentos; órfãos só na varredura; range com literais 20/30/40/50 s; regressões de pipeline e áudio verdes; clippy limpo.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-green.log`
- Limitations: none recorded

## EV-008 — passed

- Ticket: `TK-003`
- Acceptance: `AC-007`
- Procedure: `cargo test -p aura-capture --release --test retention_cost -- --ignored --nocapture, no código anterior (dd37ee5 + teste) e no novo`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D; NVMe Kingston NV3
- Tested revision: `local:042bce66c991881d741fa08e202bf801c383168b0b6f16076d9eb913630ce1c3`
- Timestamp: `2026-11-14T00:04:49+00:00`
- Observations: SC-002: 60.000 Segmentos continuous em disco (SQLite em arquivo). Retenção após um Segmento: antes p50 1.034,5 ms (1.026-1.268); depois p50 15,5 ms (14,2-16,2); 7 execuções cada.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-cost-antes.log`, `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-cost-depois.log`
- Limitations: none recorded

## EV-009 — passed

- Ticket: `TK-003`
- Acceptance: `AC-007`, `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-capture (inclui recorder_sweeps_orphans_at_start_and_hourly); cargo clippy -p aura-capture --all-targets -- -D warnings`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:003ea6ae2e29cb5a9b5afcd181472da78668014d4743aad5ca3a2673e44548e8`
- Timestamp: `2026-11-14T00:05:44+00:00`
- Observations: Gravador varre órfãos ao iniciar, não varre antes de 1 h e varre depois de 1 h (relógio injetado); equivalência, órfãos e range verdes; clippy limpo.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-green3.log`
- Limitations: none recorded
