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

## EV-003 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `cargo test -p aura-capture (após rustfmt, HEAD 67110c3)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:be88c1cd031cb5dfa3a632ddafbe89f3069495ea8726756593b59825c971131c`
- Timestamp: `2026-11-13T23:57:02+00:00`
- Observations: 11 testes de unidade e 4 de integração passaram, incluindo os 3 novos de 041.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk001-green2.log`
- Limitations: Evidence invalidated because an input changed.

## EV-004 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`
- Procedure: `Assinatura nova com o comportamento antigo (política clonada no new): cargo test -p aura-audio --test recording`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:cd4d45829c03eaeb509c16883896daab0d0cb0ac34430e93263d416c47640f64`
- Timestamp: `2026-11-13T23:59:00+00:00`
- Observations: Red comportamental: com a política viva em 30 min, a cópia de 5 min apagou 14 Segmentos de tela (left 6, right 20).
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk002-red.log`
- Limitations: Evidence invalidated because an input changed.

## EV-005 — stale

- Ticket: `TK-002`
- Acceptance: `AC-004`
- Procedure: `cargo test -p aura-audio; cargo test -p aura-app`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:0ac08bb1dcbbbe6a202d474c7fa6c30e9600636e36afab0ba43053bcc7664391`
- Timestamp: `2026-11-13T23:59:00+00:00`
- Observations: aura-audio 11 + 2 (live_retention_change_reaches_audio verde: 20 telas mantidas com 30 min; com 5 min sobram as que terminam em 16 a 20 min). aura-app: todas as suítes passaram.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk002-green.log`
- Limitations: Evidence invalidated because an input changed.

## EV-006 — stale

- Ticket: `TK-003`
- Acceptance: `AC-008`
- Procedure: `sweep_orphans extraído mas ainda chamado por apply_retention: cargo test -p aura-capture --test retention_store orphans`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D; NVMe Kingston NV3
- Tested revision: `local:cfa04481a4cf612f099c613c2661bbe6207a0670db55aeb51e35a78717b53676`
- Timestamp: `2026-11-14T00:04:49+00:00`
- Observations: Red comportamental: apply_retention devolveu 3 (removeu .seg órfão, .seg.tmp e linha sem arquivo) em vez de 0.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-red.log`
- Limitations: Evidence invalidated because an input changed.

## EV-007 — stale

- Ticket: `TK-003`
- Acceptance: `AC-007`, `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-capture; cargo test -p aura-audio; cargo clippy -p aura-capture -p aura-audio --all-targets -- -D warnings`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D; NVMe Kingston NV3
- Tested revision: `local:3f3d042e604e258004499fd7811c6ac99f4ea22d8f97ae3c1a6fa4eba9130863`
- Timestamp: `2026-11-14T00:04:49+00:00`
- Observations: Equivalência com plan em 3 sementes x 4 políticas x 2.000 Segmentos; órfãos só na varredura; range com literais 20/30/40/50 s; regressões de pipeline e áudio verdes; clippy limpo.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-green.log`
- Limitations: Evidence invalidated because an input changed.

## EV-008 — stale

- Ticket: `TK-003`
- Acceptance: `AC-007`
- Procedure: `cargo test -p aura-capture --release --test retention_cost -- --ignored --nocapture, no código anterior (dd37ee5 + teste) e no novo`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D; NVMe Kingston NV3
- Tested revision: `local:042bce66c991881d741fa08e202bf801c383168b0b6f16076d9eb913630ce1c3`
- Timestamp: `2026-11-14T00:04:49+00:00`
- Observations: SC-002: 60.000 Segmentos continuous em disco (SQLite em arquivo). Retenção após um Segmento: antes p50 1.034,5 ms (1.026-1.268); depois p50 15,5 ms (14,2-16,2); 7 execuções cada.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-cost-antes.log`, `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-cost-depois.log`
- Limitations: Evidence invalidated because an input changed.

## EV-009 — stale

- Ticket: `TK-003`
- Acceptance: `AC-007`, `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-capture (inclui recorder_sweeps_orphans_at_start_and_hourly); cargo clippy -p aura-capture --all-targets -- -D warnings`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:003ea6ae2e29cb5a9b5afcd181472da78668014d4743aad5ca3a2673e44548e8`
- Timestamp: `2026-11-14T00:05:44+00:00`
- Observations: Gravador varre órfãos ao iniciar, não varre antes de 1 h e varre depois de 1 h (relógio injetado); equivalência, órfãos e range verdes; clippy limpo.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk003-green3.log`
- Limitations: Evidence invalidated because an input changed.

## EV-010 — stale

- Ticket: `TK-004`
- Acceptance: `AC-005`
- Procedure: `Antes da correção (só is_finished adicionado): cargo test -p aura-capture --test pipeline one_failed`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:c96c0a620cd9e61e13d8273aff6ce3003faa37f8c4d0804fb55984dba7bc0311`
- Timestamp: `2026-11-14T00:30:15+00:00`
- Observations: Red comportamental: a tarefa do gravador encerrou no 1º erro de selagem (is_finished verdadeiro).
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk004-red.log`
- Limitations: Evidence invalidated because an input changed.

## EV-011 — stale

- Ticket: `TK-004`
- Acceptance: `AC-006`
- Procedure: `Antes da correção (supervise sem comportamento): cargo test -p aura-app --test host dead_screen`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:4b2f4b6ea26df083d2ce9dc75f62929304906948c3701f94d421684b48598039`
- Timestamp: `2026-11-14T00:30:15+00:00`
- Observations: Red comportamental: active() continuou informando screen com o gravador morto.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk004-app-red.log`
- Limitations: Evidence invalidated because an input changed.

## EV-012 — stale

- Ticket: `TK-004`
- Acceptance: `AC-005`, `AC-006`
- Procedure: `cargo test -p aura-capture --test pipeline; cargo test -p aura-app --test host dead_screen screen_restart; cargo clippy -p aura-capture -p aura-audio -p aura-app --all-targets -- -D warnings; cargo fmt --all -- --check`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:23e87c65b3ba1ecd59fa27956322b51de095d811fb2c752c2bac459f8f1de596`
- Timestamp: `2026-11-14T00:30:15+00:00`
- Observations: 1 falha tolerada e gravação segue; 5 falhas seguidas encerram; active() honesto; supervise reinicia já, respeita o recuo de 15 s e informa só mudanças; recuo 15..900 s; clippy e fmt limpos.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-041/tk004-green-capture.log`
- Limitations: Evidence invalidated because an input changed.

## EV-013 — passed

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `cargo test -p aura-capture`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:4b720715ab00bd24cd8969d0d54b30bbafa4e5b4837f8dbca42715b68f617daf`
- Timestamp: `2026-11-14T00:32:26+00:00`
- Observations: 21 testes do aura-capture passaram, inclusive os 3 de retenção do Contínuo.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-045/capture.log`
- Limitations: Reexecução após renome do esforço 041→045 (IDs em comentários e artefatos; mudança editorial). A suíte completa do workspace (529 testes) rodou antes do renome; os 2 testes de Media Foundation do aura-win que travam neste PC continuam fora.

## EV-014 — passed

- Ticket: `TK-002`
- Acceptance: `AC-004`
- Procedure: `cargo test -p aura-audio; cargo test -p aura-app`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:28a8ffbbcc8290ba7dc91ea1e299506eafe40085d76b1c1772f70d00f8f9766c`
- Timestamp: `2026-11-14T00:32:27+00:00`
- Observations: 13 testes do aura-audio (live_retention_change_reaches_audio) e 149 do aura-app passaram.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-045/audio.log`
- Limitations: Reexecução após renome do esforço 041→045 (IDs em comentários e artefatos; mudança editorial). A suíte completa do workspace (529 testes) rodou antes do renome; os 2 testes de Media Foundation do aura-win que travam neste PC continuam fora.

## EV-015 — passed

- Ticket: `TK-003`
- Acceptance: `AC-007`, `AC-008`, `AC-009`
- Procedure: `cargo test -p aura-capture; cargo test -p aura-audio`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:9365c8614660c1f69fe3e0274ecf49fe92e5e4dc7ff6500ded8c7e776b2d1e76`
- Timestamp: `2026-11-14T00:32:27+00:00`
- Observations: Equivalência com plan, órfãos só na varredura (cadência de 1 h), range e varredura do áudio verdes.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-045/capture.log`
- Limitations: Reexecução após renome do esforço 041→045 (IDs em comentários e artefatos; mudança editorial). A suíte completa do workspace (529 testes) rodou antes do renome; os 2 testes de Media Foundation do aura-win que travam neste PC continuam fora.

## EV-016 — passed

- Ticket: `TK-004`
- Acceptance: `AC-005`, `AC-006`
- Procedure: `cargo test -p aura-capture; cargo test -p aura-app; cargo clippy -p aura-capture -p aura-audio -p aura-app --all-targets -- -D warnings; cargo fmt --all -- --check`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D
- Tested revision: `local:749dbea736ced5168bbf4a8d31c2ec28c920a59f2affab7c45eb2e5279cbc645`
- Timestamp: `2026-11-14T00:32:27+00:00`
- Observations: Falha tolerada, 5 seguidas encerram, active() honesto, supervise com recuo; clippy e fmt limpos.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-045/app.log`
- Limitations: Reexecução após renome do esforço 041→045 (IDs em comentários e artefatos; mudança editorial). A suíte completa do workspace (529 testes) rodou antes do renome; os 2 testes de Media Foundation do aura-win que travam neste PC continuam fora.
